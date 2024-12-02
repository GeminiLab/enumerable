use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{size_option::SizeOption, targets::Target};

pub struct EnumerableImpl<'a> {
    target: &'a Target,
    size_option: SizeOption,
    enumerator_creator: Option<&'a TokenStream>,
    enumerator_type: Option<&'a TokenStream>,
}

impl<'a> EnumerableImpl<'a> {
    pub fn new(target: &'a Target, size_option: SizeOption) -> Self {
        Self {
            target,
            size_option,
            enumerator_creator: None,
            enumerator_type: None,
        }
    }

    pub fn override_enumerator_creator(mut self, enumerator_creator: &'a TokenStream) -> Self {
        self.enumerator_creator = Some(enumerator_creator);
        self
    }

    pub fn override_enumerator_type(mut self, enumerator_type: &'a TokenStream) -> Self {
        self.enumerator_type = Some(enumerator_type);
        self
    }

    pub fn generate(&self) -> TokenStream {
        let enumerable_trait_path = self.target.enumerable_trait_path();
        let impl_generics = self.target.generic_params_full();
        let target_type = self.target.target_type();
        let target_generics = self.target.generic_params_simple();
        let where_clause = self.target.where_clause();
        let enumerator_type = self
            .enumerator_type
            .unwrap_or_else(|| self.target.enumerator_type());
        let enumerator_creator = self
            .enumerator_creator
            .cloned()
            .unwrap_or_else(|| quote!(<#enumerator_type #target_generics>::new()));
        let size_option = &self.size_option;

        quote!(
            #[automatically_derived]
            impl #impl_generics #enumerable_trait_path for #target_type #target_generics #where_clause {
                type Enumerator = #enumerator_type #target_generics;

                fn enumerator() -> Self::Enumerator {
                    #enumerator_creator
                }

                const ENUMERABLE_SIZE_OPTION: Option<usize> = #size_option;
            }
        )
    }
}

/// Generates the implementation of the `Enumerable` trait for the target type.
pub fn enumerable_impl<'a>(target: &'a Target, size_option: SizeOption) -> EnumerableImpl<'a> {
    EnumerableImpl::new(target, size_option)
}

pub enum EnumeratorKeyword {
    Struct,
    Enum,
}

impl ToTokens for EnumeratorKeyword {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Struct => quote!(struct),
            Self::Enum => quote!(enum),
        }
        .to_tokens(tokens)
    }
}

pub struct EnumeratorInfo {
    pub keyword: EnumeratorKeyword,
    pub body: TokenStream,
    pub new_fn_body: TokenStream,
    pub step_fn_body: TokenStream,
    pub next_to_yield_fn_body: TokenStream,
}

pub struct EnumerableImplWithEnumerator<'a> {
    enumerable_impl: EnumerableImpl<'a>,
    enumerator_info: EnumeratorInfo,
}

pub fn enumerable_impl_with_enumerator<'a>(
    target: &'a Target,
    size_option: SizeOption,
    enumerator_info: EnumeratorInfo,
) -> EnumerableImplWithEnumerator<'a> {
    EnumerableImplWithEnumerator {
        enumerable_impl: EnumerableImpl::new(target, size_option),
        enumerator_info,
    }
}

impl<'a> EnumerableImplWithEnumerator<'a> {
    #[allow(dead_code)]
    pub fn with_enumerable_impl<F: FnOnce(EnumerableImpl<'a>) -> EnumerableImpl<'a>>(
        mut self,
        f: F,
    ) -> Self {
        self.enumerable_impl = f(self.enumerable_impl);
        self
    }

    pub fn target(&self) -> &Target {
        self.enumerable_impl.target
    }

    pub fn generate(&self) -> TokenStream {
        let enumerable_impl = self.enumerable_impl.generate();

        let vis = self.target().visibility();
        let target_type = self.target().target_type();
        let enumerator_type = self.target().enumerator_type();
        let enumerator_def_generics = self.target().generic_params_full();
        let where_clause = self.target().where_clause();
        let impl_generics = self.target().generic_params_full();
        let enumerator_generics = self.target().generic_params_simple();
        let enumerator_keyword = &self.enumerator_info.keyword;
        let enumerator_body = &self.enumerator_info.body;
        let enumerator_new_fn_body = &self.enumerator_info.new_fn_body;
        let enumerator_step_fn_body = &self.enumerator_info.step_fn_body;
        let enumerator_next_to_yield_fn_body = &self.enumerator_info.next_to_yield_fn_body;

        quote!(
            #enumerable_impl

            #[doc(hidden)]
            #vis #enumerator_keyword #enumerator_type #enumerator_def_generics #where_clause {
                #enumerator_body
            }

            impl #impl_generics #enumerator_type #enumerator_generics #where_clause {
                fn new() -> Self {
                    #enumerator_new_fn_body
                }

                fn step(&mut self) {
                    #enumerator_step_fn_body
                }

                fn next_to_yield(&self) -> Option<#target_type #enumerator_generics> {
                    #enumerator_next_to_yield_fn_body
                }
            }

            #[automatically_derived]
            impl #impl_generics ::core::iter::Iterator for #enumerator_type #enumerator_generics #where_clause {
                type Item = #target_type #enumerator_generics;

                fn next(&mut self) -> Option<Self::Item> {
                    // `Option::inspect` is not available until Rust 1.76.0.
                    self.next_to_yield().map(|item| {
                        self.step();
                        item
                    })
                }
            }
        )
    }
}
