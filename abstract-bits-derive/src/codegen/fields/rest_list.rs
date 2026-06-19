use proc_macro2::{Literal, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

use crate::codegen::generics_to_fully_qualified;
use crate::model::NormalField;

pub(crate) fn write(inner_type: &NormalField) -> TokenStream {
    let field_ident = &inner_type.ident;
    quote_spanned! {field_ident.span()=>
        for element in &self.#field_ident {
            ::abstract_bits::AbstractBits::write_abstract_bits(element, writer)?;
        }
    }
}

pub(crate) fn read(inner_type: &NormalField, struct_name: &Literal) -> TokenStream {
    let field_name = Literal::string(&inner_type.ident.to_string());
    let field_ident = &inner_type.ident;
    let inner_ty = generics_to_fully_qualified(inner_type.out_ty.clone());
    quote_spanned! {inner_ty.span()=>
        let mut #field_ident = ::std::vec::Vec::new();
        while reader.remaining_bits() > 0 {
            let element =
                <#inner_ty as ::abstract_bits::AbstractBits>::read_abstract_bits(reader)
                    .map_err(|cause| cause.read_field(#struct_name, #field_name))?;
            #field_ident.push(element);
        }
    }
}

pub(crate) fn min_bits() -> TokenStream {
    quote! { 0 }
}

pub(crate) fn max_bits(max_bytes: usize) -> TokenStream {
    let bits = Literal::usize_unsuffixed(max_bytes * 8);
    quote! { #bits }
}
