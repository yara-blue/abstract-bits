use proc_macro2::{Literal, TokenStream};
use quote::quote_spanned;
use syn::Expr;

use crate::model::NormalField;

pub(crate) fn write(inner_type: &NormalField) -> TokenStream {
    let field_ident = &inner_type.ident;

    quote_spanned! {field_ident.span()=>
        for element in &self.#field_ident {
            ::abstract_bits::AbstractBits::write_abstract_bits(element, writer)?;
        }
    }
}

pub(crate) fn read(
    field: &NormalField,
    controller: &Expr,
    struct_name: &Literal,
) -> TokenStream {
    let field_name = Literal::string(&field.ident.to_string());
    let field_ident = &field.ident;

    quote_spanned! {field.ident.span()=>
        let res = (0..#controller).into_iter().map(|_|
            ::abstract_bits::AbstractBits::read_abstract_bits(reader)
        )
            .collect::<Result<_, ::abstract_bits::FromBytesError>>()
            .map_err(|cause| cause.read_list(#struct_name,
                    #field_name, #controller as usize));
        let #field_ident = res?;
    }
}

pub(crate) fn min_bits(inner_type: &NormalField) -> TokenStream {
    quote_spanned! {inner_type.ident.span()=>
        0
    }
}

pub(crate) fn max_bits(inner_type: &NormalField, max_len: usize) -> TokenStream {
    let ty = &inner_type.out_ty;
    quote_spanned! {inner_type.ident.span()=>
        #max_len * <#ty as ::abstract_bits::AbstractBits>::MAX_BITS
    }
}
