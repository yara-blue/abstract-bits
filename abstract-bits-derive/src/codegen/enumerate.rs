use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::Ident;

use crate::model::EmptyVariant;

use super::is_primitive;

pub(crate) fn write(repr: Ident, bits: usize) -> TokenStream {
    if is_primitive(bits).is_some() {
        quote_spanned! {repr.span()=>
            ::abstract_bits::AbstractBits::write_abstract_bits(&(*self as #repr), writer)
        }
    } else {
        let utype: syn::Type = syn::parse_str(&format!("::abstract_bits::u{bits}"))
            .expect("valid type path");
        quote_spanned! {repr.span()=>
            let discriminant = #utype::new(*self as #repr);
            ::abstract_bits::AbstractBits::write_abstract_bits(&discriminant, writer)
        }
    }
}

pub fn read(variants: &[EmptyVariant], repr: Ident, bits: usize) -> TokenStream {
    let variants_discriminants = variants
        .iter()
        .map(|v| v.discriminant)
        .map(proc_macro2::Literal::usize_unsuffixed);
    let variant_idents = variants.iter().map(|v| &v.ident);

    let read_discriminant = if is_primitive(bits).is_some() {
        quote_spanned! {repr.span()=>
            let discriminant = #repr::read_abstract_bits(reader)?;
        }
    } else {
        let utype: syn::Type = syn::parse_str(&format!("::abstract_bits::u{bits}"))
            .expect("valid type path");
        quote_spanned! {repr.span()=>
            let discriminant = #utype::read_abstract_bits(reader)?;
            let discriminant = discriminant.value();
        }
    };

    quote! {
        #read_discriminant;
        match discriminant {
            #(#variants_discriminants => Ok(Self::#variant_idents)),*,
            invalid => Err(::abstract_bits::FromBytesError::ReadEnum {
                enum_name: std::any::type_name::<Self>(),
                cause: ::abstract_bits::ReadErrorCause::InvalidDiscriminant {
                    ty: std::any::type_name::<Self>(),
                    got: discriminant as usize,
                }
            }),
        }
    }
}
