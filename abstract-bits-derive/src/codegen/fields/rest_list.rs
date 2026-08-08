use proc_macro2::{Literal, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

use crate::codegen::{arbitrary_int_ty, generics_to_fully_qualified};
use crate::model::NormalField;

pub(crate) fn write(inner_type: &NormalField) -> TokenStream {
    let field_ident = &inner_type.ident;
    if let Some(int) = inner_type.arbitrary_int {
        let int_ty = arbitrary_int_ty(int, field_ident.span());
        quote_spanned! {field_ident.span()=>
            for element in &self.#field_ident {
                #int_ty::new(*element).write_abstract_bits(writer)?;
            }
        }
    } else {
        quote_spanned! {field_ident.span()=>
            for element in &self.#field_ident {
                ::abstract_bits::AbstractBits::write_abstract_bits(element, writer)?;
            }
        }
    }
}

pub(crate) fn read(
    inner_type: &NormalField,
    struct_name: &Literal,
    max_bits: usize,
    following_min_bits: &[TokenStream],
) -> TokenStream {
    let field_name = Literal::string(&inner_type.ident.to_string());
    let field_ident = &inner_type.ident;
    let window = Literal::usize_unsuffixed(max_bits);

    // Bits the fields after us still need
    let reserved_bits = quote! { 0 #(+ (#following_min_bits))* };

    // Arbitrary integer fields are stored in a vec of _primitive_ integers
    if let Some(int) = inner_type.arbitrary_int {
        let int_ty = arbitrary_int_ty(int, field_ident.span());
        let element_bits = Literal::usize_unsuffixed(int.bits);

        quote_spanned! {field_ident.span()=>
            let mut #field_ident = ::std::vec::Vec::new();
            let rest_start = reader.bits_read();
            while reader.remaining_bits() >= #element_bits + (#reserved_bits)
                && (reader.bits_read() - rest_start) + #element_bits <= #window
            {
                let element = #int_ty::read_abstract_bits(reader)
                    .map_err(|cause| cause.read_field(#struct_name, #field_name))?
                    .value();
                #field_ident.push(element);
            }
        }
    } else {
        let inner_ty = generics_to_fully_qualified(inner_type.out_ty.clone());

        quote_spanned! {inner_ty.span()=>
            let mut #field_ident = ::std::vec::Vec::new();
            let rest_start = reader.bits_read();
            let required_bits = <#inner_ty as ::abstract_bits::AbstractBits>::MIN_BITS + (#reserved_bits);

            while reader.remaining_bits()
                >= required_bits
                && (reader.bits_read() - rest_start)
                    + <#inner_ty as ::abstract_bits::AbstractBits>::MIN_BITS <= #window
            {
                let element =
                    <#inner_ty as ::abstract_bits::AbstractBits>::read_abstract_bits(reader)
                        .map_err(|cause| cause.read_field(#struct_name, #field_name))?;
                #field_ident.push(element);
            }
        }
    }
}

pub(crate) fn min_bits() -> TokenStream {
    quote! { 0 }
}

pub(crate) fn max_bits(max_bits: usize) -> TokenStream {
    let bits = Literal::usize_unsuffixed(max_bits);
    quote! { #bits }
}
