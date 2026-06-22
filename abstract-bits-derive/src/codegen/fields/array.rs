use proc_macro2::{Literal, TokenStream};
use quote::quote_spanned;
use syn::spanned::Spanned;

// TO-Do remove unsafe:
// generate:
// `return Ok([read()?, read()?, read()?, read()?])`
pub(crate) fn read(
    length: &syn::Expr,
    ty: &syn::Type,
    field: &syn::Field,
    struct_name: &Literal,
) -> TokenStream {
    let field_ident = &field.ident;
    let field_name = Literal::string(
        &field_ident
            .as_ref()
            .map(|i| i.to_string())
            .unwrap_or_default(),
    );

    quote_spanned! {field.ident.span()=>
        const LEN: usize = #length;

        // Any panic beyond this pint will leak memory
        let mut array: [::core::mem::MaybeUninit<#ty>; LEN] = unsafe {
            // # SAFETY
            // `MaybeUninit<T>` does not require initialization. It also does not
            // drop the `T`.
            ::core::mem::MaybeUninit::uninit().assume_init()
        };

        for i in 0..LEN {
            match <#ty as ::abstract_bits::AbstractBits>::read_abstract_bits(reader) {
                Ok(val) => {
                    array[i] = ::core::mem::MaybeUninit::new(val);
                }
                Err(e) => {
                    // # SAFETY
                    // `array[0..i]` are initialized, we need to drop those elements
                    unsafe {
                        for j in 0..i {
                            array[j].assume_init_drop();
                        }
                    }
                    return Err(e.read_array(#struct_name, #field_name, LEN));
                }
            } // match
        } // for

        // # SAFETY
        // The loop completed, every element is therefore initialized. In memory
        // arrays of `MaybeUninit<T>` look the same as `T`, therefore the transmute
        // is safe
        let res = unsafe { ::core::mem::transmute::<_, [#ty; LEN]>(array) };
        let #field_ident = res;
    }
}

pub(crate) fn write(field: &syn::Field) -> TokenStream {
    let field_ident = &field.ident;
    quote_spanned! {field_ident.span()=>
        for element in &self.#field_ident {
            ::abstract_bits::AbstractBits::write_abstract_bits(element, writer)?;
        }
    }
}

pub(crate) fn min_bits(inner_type: &syn::Type, length: &syn::Expr) -> TokenStream {
    quote_spanned! {inner_type.span()=>
        <#inner_type as ::abstract_bits::AbstractBits>::MIN_BITS * #length
    }
}

pub(crate) fn max_bits(inner_type: &syn::Type, length: &syn::Expr) -> TokenStream {
    quote_spanned! {inner_type.span()=>
        <#inner_type as ::abstract_bits::AbstractBits>::MAX_BITS * #length
    }
}
