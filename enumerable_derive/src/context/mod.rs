
use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, ToTokens};
use syn::{ItemEnum, ItemStruct, Visibility};

mod enumerator_naming;

/// The information for a single type for which the `Enumerable` trait is being implemented.
#[derive(Clone)]
pub struct Target {
    /// The path to the `Enumerable` trait.
    enumerable_trait_path: TokenStream,
    /// The target type.
    target_type: TokenStream,
    /// The type of the enumerator to be generated for the current target type.
    enumerator_type: TokenStream,
    /// The visibility of the enumerator to be generated. By default, it's the same as the target type's visibility.
    vis: Visibility,
}

impl Target {
    /// Gets the path to the `Enumerable` trait. If the `enumerable` crate is not found, it emits a compile error.
    pub fn enumerable_trait_path(&self) -> &TokenStream {
        &self.enumerable_trait_path
    }

    /// Gets the type of the enumerator to be generated for the current target type.
    pub fn enumerator_type(&self) -> &TokenStream {
        &self.enumerator_type
    }

    /// Gets the target type.
    pub fn target_type(&self) -> &TokenStream {
        &self.target_type
    }

    /// Gets the visibility of the enumerator to be generated.
    pub fn visibility(&self) -> &Visibility {
        &self.vis
    }
}

impl Target {
    /// Creates a new [`Target`] for a struct.
    pub fn new_for_struct(target: &ItemStruct) -> Result<Self, TokenStream> {
        let enumerable_trait_path = get_enumerable_trait_path()?;
        let enumerator_type = enumerator_naming::get_enumerator_name(&target.ident, &target.attrs)?;
        let target_type = target.ident.to_token_stream();
        let vis = target.vis.clone();

        Ok(Self {
            enumerable_trait_path,
            enumerator_type: quote!(#enumerator_type),
            target_type,
            vis,
        })
    }

    /// Creates a new [`Target`] for an enum.
    pub fn new_for_enum(target: &ItemEnum) -> Result<Self, TokenStream> {
        let enumerable_trait_path = get_enumerable_trait_path()?;
        let enumerator_type = enumerator_naming::get_enumerator_name(&target.ident, &target.attrs)?;
        let target_type = target.ident.to_token_stream();
        let vis = target.vis.clone();

        Ok(Self {
            enumerable_trait_path,
            enumerator_type: quote!(#enumerator_type),
            target_type,
            vis,
        })
    }

    /// Creates a new [`Target`] for any type.
    pub fn new_for_any(target: impl Into<TokenStream>, enumerator_type: TokenStream, vis: Visibility) -> Result<Self, TokenStream> {
        let enumerable_trait_path = get_enumerable_trait_path()?;
        let target_type = target.into();

        Ok(Self {
            enumerable_trait_path,
            enumerator_type,
            target_type,
            vis,
        })
    }
}

/// Gets the path to the `Enumerable` trait. Used when initializing a new [`Target`].
fn get_enumerable_trait_path() -> Result<TokenStream, TokenStream> {
    match crate_name("enumerable") {
        Ok(FoundCrate::Itself) => {
            // In tests and examples, we always use `enumerable::Enumerable` explicitly.
            Ok(quote!(Enumerable))
        }
        Ok(FoundCrate::Name(name)) => {
            let crate_name = format_ident!("{}", name);
            Ok(quote!(::#crate_name::Enumerable))
        }
        Err(e) => {
            let e = format!("failed to find crate `enumerable`: {}", e);
            Err(quote!(compile_error!(#e);))
        }
    }
}
