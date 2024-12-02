use proc_macro2::TokenStream;
use quote::quote;

use crate::{context::Target, size_option::SizeOption};

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
        let target_type = self.target.target_type();
        let enumerator_type = self.enumerator_type.unwrap_or_else(|| self.target.enumerator_type());
        let enumerator_creator = self.enumerator_creator.cloned().unwrap_or_else(|| quote!(<#enumerator_type>::new()));
        let size_option = &self.size_option;

        quote!(
            #[automatically_derived]
            impl #enumerable_trait_path for #target_type {
                type Enumerator = #enumerator_type;

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
