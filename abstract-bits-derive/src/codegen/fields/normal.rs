use proc_macro2::Literal;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote_spanned};
use syn::spanned::Spanned;

use crate::codegen::{arbitrary_int_ty, generics_to_fully_qualified};
use crate::model::NormalField;

pub fn read(
    NormalField {
        ident,
        out_ty,
        arbitrary_int,
        ..
    }: &NormalField,
    struct_name: &Literal,
) -> TokenStream {
    let field_name = proc_macro2::Literal::string(&ident.to_string());
    if let Some(int) = arbitrary_int {
        let int_ty = arbitrary_int_ty(*int, out_ty.span());
        quote_spanned! {out_ty.span()=>
            let #ident = #int_ty::read_abstract_bits(reader)
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
        arbitrary_int,
        ..
    }: &NormalField,
) -> TokenStream {
    if let Some(int) = *arbitrary_int {
        let int_ty = arbitrary_int_ty(int, out_ty.span());
        quote_spanned! {out_ty.span()=>
            let #ident = #int_ty::new(self.#ident);
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
    if let Some(int) = normal_field.arbitrary_int {
        proc_macro2::Literal::usize_unsuffixed(int.bits).to_token_stream()
    } else {
        quote_spanned! {normal_field.ident.span()=>
            #ty::MIN_BITS
        }
    }
}

pub(crate) fn max_bits(normal_field: &crate::model::NormalField) -> TokenStream {
    let ty = &normal_field.out_ty;
    if let Some(int) = normal_field.arbitrary_int {
        proc_macro2::Literal::usize_unsuffixed(int.bits).to_token_stream()
    } else {
        quote_spanned! {normal_field.ident.span()=>
                #ty::MAX_BITS
        }
    }
}
