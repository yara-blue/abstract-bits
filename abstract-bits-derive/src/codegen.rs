use proc_macro2::{Punct, Spacing, TokenStream};
use quote::{ToTokens, TokenStreamExt, quote};
use syn::spanned::Spanned;
use syn::{Attribute, Ident, Visibility};

use crate::model::{EmptyVariant, Field, Model};

mod enumerate;
mod fields;

pub fn codegen(model: Model) -> TokenStream {
    match model.ty {
        crate::model::Type::NormalStruct(fields) => {
            normal_struct(model.vis, model.ident, model.attrs, fields)
        }
        crate::model::Type::UnitStruct(field) => {
            unit_struct(model.vis, model.ident, model.attrs, field)
        }
        crate::model::Type::Enum {
            variants,
            repr_type: repr,
            bits,
        } => normal_enum(model.vis, model.ident, model.attrs, variants, repr, bits),
    }
}

fn normal_enum(
    vis: Visibility,
    ident: Ident,
    attrs: Vec<Attribute>,
    variants: Vec<EmptyVariant>,
    repr: Ident,
    bits: usize,
) -> TokenStream {
    let write_code = enumerate::write(repr.clone(), bits);
    let read_code = enumerate::read(&variants, repr, bits);

    quote! {
        #(#attrs)*
        #vis enum #ident {
            #(#variants),*
        }

        #[automatically_derived]
        impl ::abstract_bits::AbstractBits for #ident {
            const MAX_BITS: usize = #bits;
            const MIN_BITS: usize = #bits;

            fn write_abstract_bits(&self, writer: &mut ::abstract_bits::BitWriter)
            -> Result<(), ::abstract_bits::ToBytesError> {
                #write_code
            }
            fn read_abstract_bits(reader: &mut ::abstract_bits::BitReader)
            -> Result<Self, ::abstract_bits::FromBytesError>
            where
                Self: Sized
            {
                #read_code
            }
        }
    }
}

fn unit_struct(
    vis: Visibility,
    ident: Ident,
    attrs: Vec<Attribute>,
    field: syn::Field,
) -> TokenStream {
    let field_ty = &field.ty;
    quote! {
        #(#attrs)*
        #vis struct #ident(#field);

        #[automatically_derived]
        impl ::abstract_bits::AbstractBits for #ident {
            const MAX_BITS: usize = <#field_ty as abstract_bits::AbstractBits>::MAX_BITS;
            const MIN_BITS: usize = <#field_ty as abstract_bits::AbstractBits>::MIN_BITS;

            fn write_abstract_bits(&self, writer: &mut ::abstract_bits::BitWriter)
            -> Result<(), ::abstract_bits::ToBytesError> {
                self.0.write_abstract_bits(writer)
            }
            fn read_abstract_bits(reader: &mut ::abstract_bits::BitReader)
            -> Result<Self, ::abstract_bits::FromBytesError>
            where
                Self: Sized
            {
                Ok(Self(<#field_ty>::read_abstract_bits(reader)?))
            }
        }
    }
}

fn normal_struct(
    vis: Visibility,
    ident: Ident,
    attrs: Vec<Attribute>,
    fields: Vec<Field>,
) -> TokenStream {
    let struct_fields: Vec<_> = fields
        .iter()
        .filter_map(Field::needed_in_struct_def)
        .collect();
    let write_code: Vec<_> = fields.iter().map(|f| f.write_code(&ident)).collect();
    let read_code: Vec<_> = fields.iter().map(|f| f.read_code(&ident)).collect();
    let min_bits_code: Vec<_> = fields.iter().map(Field::min_bits_code).collect();
    let max_bits_code: Vec<_> = fields.iter().map(Field::max_bits_code).collect();
    let out_struct_idents: Vec<_> = fields
        .iter()
        .filter_map(Field::needed_in_struct_def)
        .map(|f| f.ident)
        .collect();

    quote! {
        #(#attrs)*
        #vis struct #ident {
            #(#struct_fields),*
        }

        #[automatically_derived]
        impl ::abstract_bits::AbstractBits for #ident {
            const MIN_BITS: usize = const {
                let mut min = 0;
                #(min += #min_bits_code;)*
                min
            };
            const MAX_BITS: usize = const {
                let mut max = 0;
                #(max += #max_bits_code;)*
                max
            };

            fn write_abstract_bits(&self, writer: &mut ::abstract_bits::BitWriter)
            -> Result<(), ::abstract_bits::ToBytesError> {
                #(#write_code)*
                Ok(())
            }
            fn read_abstract_bits(reader: &mut ::abstract_bits::BitReader)
            -> Result<Self, ::abstract_bits::FromBytesError>
            where
                Self: Sized
            {
                #(#read_code)*
                Ok(Self {
                    #(#out_struct_idents),*
                })
            }
        }
    }
}

impl ToTokens for super::model::NormalField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for attr in &self.attrs {
            attr.to_tokens(tokens);
        }
        self.vis.to_tokens(tokens);

        self.ident.to_tokens(tokens);
        tokens.append(Punct::new(':', Spacing::Joint));

        self.out_ty.to_tokens(tokens);
    }
}

impl ToTokens for super::model::EmptyVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for attr in &self.attrs {
            attr.to_tokens(tokens);
        }
        self.ident.to_tokens(tokens);
        tokens.append(Punct::new('=', Spacing::Joint));

        proc_macro2::Literal::usize_unsuffixed(self.discriminant).to_tokens(tokens)
    }
}

pub fn is_primitive(bits: usize) -> Option<TokenStream> {
    match bits {
        8 => Some(quote! {u8}),
        16 => Some(quote! {u16}),
        32 => Some(quote! {u32}),
        64 => Some(quote! {u64}),
        _ => None,
    }
}

/// Turns `Type<T>` into `Type::<T>` which is needed for
/// `Type::<T>::read_abstract_bits(reader)`
pub fn generics_to_fully_qualified(mut ty: syn::Type) -> syn::Type {
    if let syn::Type::Path(typath) = &mut ty {
        let syn::Path { segments, .. } = &mut typath.path;
        let first_seg = segments
            .first_mut()
            .expect("type path always has at least one segment");
        let syn::PathArguments::AngleBracketed(args) = &mut first_seg.arguments else {
            return ty;
        };

        args.colon2_token = Some(syn::Token![::](args.span()));
    }
    ty
}
