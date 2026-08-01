use proc_macro2::Literal;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote_spanned};
use syn::spanned::Spanned;

use crate::codegen::{arbitrary_uint, generics_to_fully_qualified};
use crate::model::NormalField;

pub fn read(
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
        let utype = arbitrary_uint(bits, out_ty.span());
        quote_spanned! {out_ty.span()=>
            let #ident = #utype::read_abstract_bits(reader)
                .map_err(|cause| cause.read_field(#struct_name, #field_name))?;
            let #ident = #ident.value();
        }
    } else {
        let out_ty = generics_to_fully_qualified(out_ty.clone());
        quote_spanned! {out_ty.span()=>
            let #ident = #out_ty::read_abstract_bits(reader)
                .map_err(|cause| cause.read_field(#struct_name, #field_name))?;
        }
    }
}

pub fn write(
    NormalField {
        ident,
        out_ty,
        bits,
        ..
    }: &NormalField,
) -> TokenStream {
    if let Some(bits) = *bits {
        let utype = arbitrary_uint(bits, out_ty.span());
        quote_spanned! {out_ty.span()=>
            let #ident = #utype::new(self.#ident);
            #ident.write_abstract_bits(writer)?;
        }
    } else {
        quote_spanned! {out_ty.span()=>
            self.#ident.write_abstract_bits(writer)?;
        }
    }
}

pub(crate) fn min_bits(normal_field: &crate::model::NormalField) -> TokenStream {
    let ty = &normal_field.out_ty;
    if let Some(n) = normal_field.bits {
        proc_macro2::Literal::usize_unsuffixed(n as usize).to_token_stream()
    } else {
        quote_spanned! {normal_field.ident.span()=>
            #ty::MIN_BITS
        }
    }
}

pub(crate) fn max_bits(normal_field: &crate::model::NormalField) -> TokenStream {
    let ty = &normal_field.out_ty;
    if let Some(n) = normal_field.bits {
        proc_macro2::Literal::usize_unsuffixed(n as usize).to_token_stream()
    } else {
        quote_spanned! {normal_field.ident.span()=>
                #ty::MAX_BITS
        }
    }
}
