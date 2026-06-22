use proc_macro2::{Literal, TokenStream};
use quote::quote_spanned;
use syn::Expr;
use syn::spanned::Spanned;

use crate::codegen::generics_to_fully_qualified;
use crate::model::NormalField;

pub fn read_field_code(
    NormalField {
        ident,
        out_ty,
        bits,
        ..
    }: &NormalField,
    struct_name: &Literal,
) -> TokenStream {
    let field_name = proc_macro2::Literal::string(&ident.to_string());
    if let Some(bits) = bits {
        let utype: syn::Type = syn::parse_str(&format!("::abstract_bits::u{bits}"))
            .expect("should be valid type path");
        quote_spanned! {out_ty.span()=>
            let #ident = #utype::read_abstract_bits(reader)
                .map_err(|cause| cause.read_option(#struct_name, #field_name))?;
            let #ident = #ident.value();
        }
    } else {
        let out_ty = generics_to_fully_qualified(out_ty.clone());
        quote_spanned! {out_ty.span()=>
            let #ident = #out_ty::read_abstract_bits(reader)
                .map_err(|cause| cause.read_option(#struct_name, #field_name))?;
        }
    }
}

pub fn read(
    field: &NormalField,
    controller: &Expr,
    struct_name: &Literal,
) -> TokenStream {
    let field_ident = &field.ident;
    let field_read_code = read_field_code(field, struct_name);

    quote_spanned! {field.ident.span()=>
        let #field_ident = if #controller {
            #field_read_code
            Some(#field_ident)
        } else {
            None
        };
    }
}

pub fn write(field: &NormalField) -> TokenStream {
    let field_ident = &field.ident;
    let write_code = if let Some(bits) = field.bits {
        let utype: syn::Type = syn::parse_str(&format!("::abstract_bits::u{bits}"))
            .expect("should be valid type path");
        quote_spanned! {field.out_ty.span()=>
            let #field_ident = #utype::new(#field_ident);
            #field_ident.write_abstract_bits(writer)?;
        }
    } else {
        quote_spanned! {field.out_ty.span()=>
            #field_ident.write_abstract_bits(writer)?;
        }
    };

    quote_spanned!(field_ident.span()=>
        if let Some(ref #field_ident) = self.#field_ident {
            #write_code
        }
    )
}

pub(crate) fn min_bits(inner_type: &NormalField) -> TokenStream {
    let ty = &inner_type.out_ty;
    quote_spanned! {inner_type.ident.span()=>
        #ty::MIN_BITS
    }
}

pub(crate) fn max_bits(inner_type: &NormalField) -> TokenStream {
    let ty = &inner_type.out_ty;
    quote_spanned! {inner_type.ident.span()=>
        #ty::MAX_BITS
    }
}
