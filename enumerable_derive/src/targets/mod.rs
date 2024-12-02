use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{Field, Generics, ItemEnum, ItemStruct, Visibility};

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
    /// The generic parameters of the target type, with bounds and defaults stripped.
    generic_params_simple: TokenStream,
    /// The generic parameters of the target type, with bounds retained and defaults stripped.
    generic_params_full: TokenStream,
    /// The where clause of the target type, with extra bounds `F: Enumerable` for each field type.
    where_clause: TokenStream,
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

    /// Gets the generic parameters of the target type, with bounds and defaults stripped.
    pub fn generic_params_simple(&self) -> &TokenStream {
        &self.generic_params_simple
    }

    /// Gets the generic parameters of the target type, with bounds retained and defaults stripped.
    pub fn generic_params_full(&self) -> &TokenStream {
        &self.generic_params_full
    }

    /// Gets the where clause of the target type, with extra bounds `F: Enumerable` for each field type.
    pub fn where_clause(&self) -> &TokenStream {
        &self.where_clause
    }
}

impl Target {
    /// Generates the generic parameters, and the where clause from the target type's generics and fields.
    fn get_generic_info<'a>(
        generics: &'a Generics,
        fields: impl Iterator<Item = &'a Field>,
        enumerable_trait_path: &TokenStream,
    ) -> Result<(TokenStream, TokenStream, TokenStream), TokenStream> {
        let mut where_clause_for_fields = TokenStream::new();

        for field in fields {
            let ty = &field.ty;
            where_clause_for_fields.extend(quote!(#ty: #enumerable_trait_path,));
        }

        let where_clause = match &generics.where_clause {
            Some(wc) => {
                if wc.predicates.trailing_punct() {
                    quote!(#wc #where_clause_for_fields)
                } else {
                    quote!(#wc, #where_clause_for_fields)
                }
            }
            None => {
                if where_clause_for_fields.is_empty() {
                    quote!()
                } else {
                    quote!(where #where_clause_for_fields)
                }
            }
        };

        if generics.params.is_empty() {
            return Ok((quote!(), quote!(), where_clause));
        }

        if let Some(lifetime) = generics.lifetimes().next() {
            return Err(
                quote_spanned!(lifetime.lifetime.span() => compile_error!("Lifetime parameters are not supported.")),
            );
        }

        if let Some(const_param) = generics.const_params().next() {
            return Err(
                quote_spanned!(const_param.ident.span() => compile_error!("Const parameters are not supported.")),
            );
        }

        let mut params_simple = quote!(<);
        let mut params_full = quote!(<);

        for param in generics.type_params() {
            let ident = &param.ident;
            let colon_token = &param.colon_token;
            let bounds = &param.bounds;

            params_simple.extend(quote!(#ident,));
            params_full.extend(quote!(#ident #colon_token #bounds,));
        }

        params_simple.extend(quote!(>));
        params_full.extend(quote!(>));

        Ok((params_simple, params_full, where_clause))
    }

    /// Creates a new [`Target`] for a struct.
    pub fn new_for_struct(target: &ItemStruct) -> Result<Self, TokenStream> {
        let enumerable_trait_path = get_enumerable_trait_path()?;
        let enumerator_type = enumerator_naming::get_enumerator_name(&target.ident, &target.attrs)?;
        let target_type = target.ident.to_token_stream();
        let vis = target.vis.clone();

        let (generic_params_simple, generic_params_full, where_clause) = Self::get_generic_info(
            &target.generics,
            target.fields.iter(),
            &enumerable_trait_path,
        )?;

        Ok(Self {
            enumerable_trait_path,
            enumerator_type: quote!(#enumerator_type),
            target_type,
            vis,
            generic_params_simple,
            generic_params_full,
            where_clause,
        })
    }

    /// Creates a new [`Target`] for an enum.
    pub fn new_for_enum(target: &ItemEnum) -> Result<Self, TokenStream> {
        let enumerable_trait_path = get_enumerable_trait_path()?;
        let enumerator_type = enumerator_naming::get_enumerator_name(&target.ident, &target.attrs)?;
        let target_type = target.ident.to_token_stream();
        let vis = target.vis.clone();

        let (generic_params_simple, generic_params_full, where_clause) = Self::get_generic_info(
            &target.generics,
            target.variants.iter().flat_map(|v| v.fields.iter()),
            &enumerable_trait_path,
        )?;

        Ok(Self {
            enumerable_trait_path,
            enumerator_type: quote!(#enumerator_type),
            target_type,
            vis,
            generic_params_simple,
            generic_params_full,
            where_clause,
        })
    }

    #[allow(dead_code)]
    /// Creates a new [`Target`] for any type.
    pub fn new_for_any(
        target: impl Into<TokenStream>,
        enumerator_type: TokenStream,
        vis: Visibility,
        generic_params_simple: TokenStream,
        generic_params_full: TokenStream,
        where_clause: TokenStream,
    ) -> Result<Self, TokenStream> {
        let enumerable_trait_path = get_enumerable_trait_path()?;
        let target_type = target.into();

        Ok(Self {
            enumerable_trait_path,
            enumerator_type,
            target_type,
            vis,
            generic_params_simple,
            generic_params_full,
            where_clause,
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
