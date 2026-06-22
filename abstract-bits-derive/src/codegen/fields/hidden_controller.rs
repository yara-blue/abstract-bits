use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::Ident;

use crate::model::NormalField;

/// Derive the controller's value from the field it controls and write it: an
/// option's presence is its `is_some`, a list's length is its `len`.
pub fn write(field: &NormalField, controlled: &Ident, presence: bool) -> TokenStream {
    let ident = &field.ident;

    if presence {
        return quote_spanned! {ident.span()=>
            self.#controlled.is_some().write_abstract_bits(writer)?;
        };
    }

    if let Some(bits) = field.bits {
        let utype: syn::Type = syn::parse_str(&format!("::abstract_bits::u{bits}"))
            .expect("should be valid type path");
        quote_spanned! {ident.span()=>
            let len: #utype = self.#controlled.len().try_into().ok()
                .and_then(|len| #utype::try_new(len).ok())
                .ok_or_else(|| ::abstract_bits::ToBytesError::ListTooLong {
                    max: 2usize.pow(#utype::BITS as u32) - 1,
                    got: self.#controlled.len(),
                })?;
            len.write_abstract_bits(writer)?;
        }
    } else {
        let out_ty = &field.out_ty;
        quote_spanned! {ident.span()=>
            let len: #out_ty = self.#controlled.len().try_into()
                .map_err(|_| ::abstract_bits::ToBytesError::ListTooLong {
                    max: #out_ty::MAX as usize,
                    got: self.#controlled.len(),
                })?;
            len.write_abstract_bits(writer)?;
        }
    }
}
