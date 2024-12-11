//! Calculations for the size of enumerable types.

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::LitInt;

/// An constant expression with type `Option<usize>`.
///
/// This type is used to represent the size of an enumerable type.
pub struct SizeOption {
    token_stream: TokenStream,
}

impl SizeOption {
    /// Creates a new `SizeOption` from a raw token stream.
    ///
    /// ## Safety
    ///
    /// The token stream must be a valid Rust expression of type `Option<usize>`.
    pub unsafe fn from_raw(token_stream: TokenStream) -> Self {
        Self { token_stream }
    }

    /// Creates a new `SizeOption` from the `ENUMERABLE_SIZE_OPTION` constant of
    /// a type implementing the `Enumerable` trait.
    pub fn from_type(type_name: impl ToTokens, enumerable_trait_path: impl ToTokens) -> Self {
        // SAFETY: It's a `ENUMERABLE_SIZE_OPTION` constant of a type implementing the `Enumerable` trait.
        unsafe {
            Self::from_raw(quote!(
                <#type_name as #enumerable_trait_path>::ENUMERABLE_SIZE_OPTION
            ))
        }
    }

    pub fn from_usize(size: usize) -> Self {
        let size_lit = LitInt::new(&format!("{}usize", size), Span::call_site());

        // SAFETY: It's a literal expression of type `Option<usize>`.
        unsafe { Self::from_raw(quote!(Some(#size_lit))) }
    }

    /// Creates a new `SizeOption` from the product of a list of `SizeOption`s.
    pub fn from_product(sizes: impl Iterator<Item = SizeOption>) -> Self {
        let mut sizes = sizes.peekable();

        // Optimization: if the iterator is empty, the product is 1.
        let size_first = match sizes.next() {
            Some(size) => size,
            None => {
                return Self::from_usize(1);
            }
        };

        match sizes.peek() {
            Some(_) => {
                // there are at least two sizes

                // SAFETY: `size` is always an `Option<usize>`.
                unsafe {
                    Self::from_raw(quote!(
                        {
                            let size: Option<usize> = #size_first;
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
            }
            None => {
                // there is only one size
                size_first
            }
        }
    }

    /// Creates a new `SizeOption` from the sum of a list of `SizeOption`s.
    pub fn from_sum(sizes: impl Iterator<Item = SizeOption>) -> Self {
        let mut sizes = sizes.peekable();

        // Optimization: if the iterator is empty, the sum is 0.
        let size_first = match sizes.next() {
            Some(size) => size,
            None => {
                return Self::from_usize(0);
            }
        };

        match sizes.peek() {
            Some(_) => {
                // there are at least two sizes

                // SAFETY: `size` is always an `Option<usize>`.
                unsafe {
                    Self::from_raw(quote!(
                        {
                            let size: Option<usize> = #size_first;
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
            None => {
                // there is only one size
                size_first
            }
        }
    }
}

impl ToTokens for SizeOption {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.token_stream.to_tokens(tokens);
    }
}
