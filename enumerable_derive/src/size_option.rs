//! Calculations for the size of enumerable types.

use proc_macro2::{Span, TokenStream};
use syn::LitInt;
use quote::{quote, ToTokens};

/// An constant expression with type `Option<usize>`.
/// 
/// This type is used to represent the size of an enumerable type.
pub struct SizeOption {
    token_stream: TokenStream,
}

impl SizeOption {
    pub fn from_raw(token_stream: TokenStream) -> Self {
        Self {
            token_stream
        }
    }

    pub fn from_type(type_name: TokenStream, enumerable_trait_path: &TokenStream) -> Self {
        Self::from_raw(quote!(
            <#type_name as #enumerable_trait_path>::ENUMERABLE_SIZE_OPTION
        ))
    }

    pub fn from_usize(size: usize) -> Self {
        let size_lit = LitInt::new(&format!("{}usize", size), Span::call_site());

        Self::from_raw(quote!(Some(#size_lit)))
    }

    pub fn from_product(sizes: impl Iterator<Item = impl ToTokens>) -> Self {
        Self::from_raw(quote!(
            {
                let size: Option<usize> = Some(1usize);
                #(
                    let size: Option<usize> = match (size, #sizes) {
                        (Some(0), _) | (_, Some(0)) => Some(0),
                        (Some(size), Some(size_field)) => size.checked_mul(size_field),
                        _ => None,
                    };
                )*
                size
            }
        ))
    }

    pub fn from_sum(sizes: impl Iterator<Item = impl ToTokens>) -> Self {
        Self::from_raw(quote!(
            {
                let size: Option<usize> = Some(0usize);
                #(
                    let size: Option<usize> = match (size, #sizes) {
                        (Some(size), Some(size_field)) => size.checked_add(size_field),
                        _ => None,
                    };
                )*
                size
            }
        ))
    }
}

impl ToTokens for SizeOption {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.token_stream.to_tokens(tokens);
    }
}
