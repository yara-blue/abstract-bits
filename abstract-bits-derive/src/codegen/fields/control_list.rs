use proc_macro2::{Literal, TokenStream};
use quote::{quote, quote_spanned};
use syn::Ident;

use crate::codegen::{arbitrary_uint, is_primitive, list_len_ident};

pub fn read(controlled: &Ident, bits: usize, struct_name: &Literal) -> TokenStream {
    let list_name = Literal::string(&controlled.to_string());
    let len_ident = list_len_ident(controlled);
    if let Some(ty) = is_primitive(bits) {
        quote_spanned! {controlled.span()=>
            let #len_ident = #ty::read_abstract_bits(reader)
                .map_err(|cause| cause.read_list_length(#struct_name, #list_name))?;
        }
    } else {
        let utype = arbitrary_uint(bits, controlled.span());
        quote_spanned! {controlled.span()=>
            let #len_ident = #utype::read_abstract_bits(reader)
                .map_err(|cause| cause.read_list_length(#struct_name, #list_name))?;
            let #len_ident = #len_ident.value();
        }
    }
}

pub fn write(controlled: &Ident, bits: usize) -> TokenStream {
    let len_ident = list_len_ident(controlled);
    if let Some(ty) = is_primitive(bits) {
        quote_spanned! {controlled.span()=>
            let #len_ident: #ty = self.#controlled.len().try_into()
                .map_err(|_| ::abstract_bits::ToBytesError::ListTooLong {
                    max: #ty::MAX as usize,
                    got: self.#controlled.len(),
            })?;
            ::abstract_bits::AbstractBits::write_abstract_bits(&#len_ident, writer)?;
        }
    } else {
        let utype = arbitrary_uint(bits, controlled.span());
        quote_spanned! {controlled.span()=>
            let #len_ident = self.#controlled.len().try_into()
                .map_err(|_| ::abstract_bits::ToBytesError::ListTooLong {
                    max: 2usize.pow(#utype::BITS as u32) - 1,
                    got: self.#controlled.len(),
                })?;
            let #len_ident = #utype::try_new(#len_ident)
                .map_err(|_| ::abstract_bits::ToBytesError::ListTooLong {
                    max: 2usize.pow(#utype::BITS as u32) - 1,
                    got: self.#controlled.len(),
                })?;
            ::abstract_bits::AbstractBits::write_abstract_bits(&#len_ident, writer)?;
        }
    }
}

pub(crate) fn min_bits(n_bits: usize) -> TokenStream {
    let n_bits = proc_macro2::Literal::usize_unsuffixed(n_bits);
    quote! {
        #n_bits
    }
}

pub(crate) fn max_bits(n_bits: usize) -> TokenStream {
    let n_bits = proc_macro2::Literal::usize_unsuffixed(n_bits);
    quote! {
        #n_bits
    }
}
