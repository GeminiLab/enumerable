use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{Field, Generics, ItemEnum, ItemStruct, Visibility};

mod enumerator_naming;

/// One, two, or multiple token streams.
#[derive(Clone)]
pub enum TokenStreamRefs<'a> {
    None,
    Single(&'a TokenStream),
    Double(&'a TokenStream, &'a TokenStream),
    Multiple(Vec<&'a TokenStream>),
}

impl<'a> TokenStreamRefs<'a> {
    pub fn append_option(self, ts: Option<&'a TokenStream>) -> Self {
        match ts {
            Some(ts) => match self {
                TokenStreamRefs::None => TokenStreamRefs::Single(ts),
                TokenStreamRefs::Single(ts1) => TokenStreamRefs::Double(ts1, ts),
                TokenStreamRefs::Double(ts1, ts2) => TokenStreamRefs::Multiple(vec![ts1, ts2, ts]),
                TokenStreamRefs::Multiple(mut tss) => {
                    tss.push(ts);
                    TokenStreamRefs::Multiple(tss)
                }
            },
            None => self,
        }
    }
}

impl<'a> From<&'a TokenStream> for TokenStreamRefs<'a> {
    fn from(ts: &'a TokenStream) -> Self {
        TokenStreamRefs::Single(ts)
    }
}

impl<'a> From<Option<&'a TokenStream>> for TokenStreamRefs<'a> {
    fn from(opt: Option<&'a TokenStream>) -> Self {
        match opt {
            Some(ts) => TokenStreamRefs::Single(ts),
            None => TokenStreamRefs::None,
        }
    }
}

impl<'a> ToTokens for TokenStreamRefs<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            TokenStreamRefs::None => {}
            TokenStreamRefs::Single(ts) => ts.to_tokens(tokens),
            TokenStreamRefs::Double(ts1, ts2) => {
                ts1.to_tokens(tokens);
                ts2.to_tokens(tokens);
            }
            TokenStreamRefs::Multiple(tss) => {
                for ts in tss {
                    ts.to_tokens(tokens);
                }
            }
        }
    }
}

/// The information for a single type for which the `Enumerable` trait is being implemented.
#[derive(Clone)]
pub struct Target {
    /// The target type name. Generics are not included.
    target_type_name: TokenStream,
    /// The target type, with generics included but bounds and defaults stripped.
    ///
    /// By default, it's `#target_type_name #generic_params_simple`.
    target_type: Option<TokenStream>,
    /// The target type, with generics and bounds included but defaults stripped.
    ///
    /// By default, it's `#target_type_name #generic_params_full`.
    target_type_with_bound: Option<TokenStream>,
    /// The visibility of the enumerator to be generated.
    ///
    /// By default, it's the same as the target type's visibility. If not specified, it's `pub`.
    vis: Option<Visibility>,
    /// The type name of the enumerator to be generated for the current target type.
    enumerator_type_name: TokenStream,
    /// The type of the enumerator, with generics included but bounds and defaults stripped.
    ///
    /// By default, it's `#enumerator_type_name #generic_params_simple`.
    enumerator_type: Option<TokenStream>,
    /// The type of the enumerator, with generics and bounds included but defaults stripped.
    ///
    /// By default, it's `#enumerator_type_name #generic_params_full`.
    enumerator_type_with_bound: Option<TokenStream>,
    /// The generic parameters of the target type, with bounds and defaults stripped.
    ///
    /// This field is optional.
    generic_params_simple: Option<TokenStream>,
    /// The generic parameters of the target type, with bounds retained and defaults stripped.
    ///
    /// This field is optional.
    generic_params_full: Option<TokenStream>,
    /// The where clause of the target type.
    ///
    /// If the [`Target`] is created for a struct or an enum, this field is generated from the target type's where clause, with extra bounds `F: Enumerable` for each field type
    where_clause: Option<TokenStream>,
    /// The path to the `Enumerable` trait.
    enumerable_trait_path: TokenStream,
}

// Constructors
impl Target {
    /// Creates a new [`Target`] for any type.
    pub fn new_for_any(
        target_type_name: impl Into<TokenStream>,
        enumerator_type_name: impl Into<TokenStream>,
    ) -> Self {
        let enumerable_trait_path = get_enumerable_trait_path().unwrap();
        let target_type_name = target_type_name.into();
        let enumerator_type_name = enumerator_type_name.into();

        Self {
            enumerable_trait_path,
            enumerator_type_name,
            target_type_name,
            vis: None,
            target_type: None,
            target_type_with_bound: None,
            enumerator_type: None,
            enumerator_type_with_bound: None,
            generic_params_simple: None,
            generic_params_full: None,
            where_clause: None,
        }
    }

    /// Creates a new [`Target`] for a struct.
    pub fn new_for_struct(target: &ItemStruct) -> Result<Self, TokenStream> {
        Self::new_for_any(
            target.ident.to_token_stream(),
            enumerator_naming::get_enumerator_name(&target.ident, &target.attrs)?.to_token_stream(),
        )
        .with_visibility(target.vis.clone())
        .with_where_clause_from_generics_and_fields(&target.generics, target.fields.iter())
        .with_generic_params_from_generics(&target.generics)
    }

    /// Creates a new [`Target`] for an enum.
    pub fn new_for_enum(target: &ItemEnum) -> Result<Self, TokenStream> {
        Self::new_for_any(
            target.ident.to_token_stream(),
            enumerator_naming::get_enumerator_name(&target.ident, &target.attrs)?
                .into_token_stream(),
        )
        .with_visibility(target.vis.clone())
        .with_where_clause_from_generics_and_fields(
            &target.generics,
            target.variants.iter().flat_map(|v| v.fields.iter()),
        )
        .with_generic_params_from_generics(&target.generics)
    }
}

// Mutators
#[allow(dead_code)]
impl Target {
    /// Sets the visibility of the enumerator to be generated.
    pub fn with_visibility(mut self, vis: Visibility) -> Self {
        self.vis = Some(vis);
        self
    }

    /// Sets the target type.
    pub fn with_target_type(
        mut self,
        target_type: impl Into<TokenStream>,
        target_type_with_bound: impl Into<TokenStream>,
    ) -> Self {
        self.target_type = Some(target_type.into());
        self.target_type_with_bound = Some(target_type_with_bound.into());
        self
    }

    /// Sets the enumerator type.
    pub fn with_enumerator_type(
        mut self,
        enumerator_type: impl Into<TokenStream>,
        enumerator_type_with_bound: impl Into<TokenStream>,
    ) -> Self {
        self.enumerator_type = Some(enumerator_type.into());
        self.enumerator_type_with_bound = Some(enumerator_type_with_bound.into());
        self
    }

    /// Sets the generic parameters of the target type.
    pub fn with_generic_params(
        mut self,
        generic_params_simple: impl Into<TokenStream>,
        generic_params_full: impl Into<TokenStream>,
    ) -> Self {
        self.generic_params_simple = Some(generic_params_simple.into());
        self.generic_params_full = Some(generic_params_full.into());
        self
    }

    /// Sets the generic parameters of the target type from [`Generics`].
    pub fn with_generic_params_from_generics<'a>(
        self,
        generics: &'a Generics,
    ) -> Result<Self, TokenStream> {
        if generics.params.is_empty() {
            return self.as_ok();
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

        self.with_generic_params(params_simple, params_full).as_ok()
    }

    /// Sets the where clause of the target type.
    pub fn with_where_clause(mut self, where_clause: impl Into<TokenStream>) -> Self {
        self.where_clause = Some(where_clause.into());
        self
    }

    /// Sets the where clause of the target type from [`Generics`] and iterator of [`Field`]s.
    pub fn with_where_clause_from_generics_and_fields<'a>(
        self,
        generics: &'a Generics,
        fields: impl Iterator<Item = &'a Field>,
    ) -> Self {
        let enumerable_trait_path = &self.enumerable_trait_path;
        let mut where_clause_for_fields = TokenStream::new();

        for field in fields {
            let ty = &field.ty;
            where_clause_for_fields.extend(quote!(#ty: #enumerable_trait_path,));
        }

        // Add an extra bound `T: ::core::marker::Copy` for each generic parameter `T`.
        //
        // See here for more information: https://github.com/GeminiLab/enumerable/issues/51.
        for param in generics.type_params() {
            let ident = &param.ident;
            where_clause_for_fields.extend(quote!(#ident: ::core::marker::Copy,));
        }

        let where_clause = match &generics.where_clause {
            Some(wc) => {
                let predicates = wc.predicates.iter();
                quote!(where #where_clause_for_fields #(#predicates,)*)
            }
            None => quote!(where #where_clause_for_fields),
        };

        self.with_where_clause(where_clause)
    }

    /// Converts the current [`Target`] into a [`Result`] with the current [`Target`] as the `Ok` variant.
    pub fn as_ok<T>(self) -> Result<Self, T> {
        Ok(self)
    }
}

// Getters
#[allow(dead_code)]
impl Target {
    /// Gets the target type name. Generics are not included.
    pub fn target_type_name(&self) -> TokenStreamRefs {
        (&self.target_type_name).into()
    }

    /// Gets the target type, with generics included but bounds and defaults stripped.
    pub fn target_type(&self) -> TokenStreamRefs {
        self.target_type
            .as_ref()
            .map(Into::into)
            .unwrap_or_else(|| {
                self.target_type_name()
                    .append_option(self.generic_params_simple.as_ref())
            })
    }

    /// Gets the target type, with generics and bounds included but defaults stripped.
    pub fn target_type_bounded(&self) -> TokenStreamRefs {
        self.target_type_with_bound
            .as_ref()
            .map(Into::into)
            .unwrap_or_else(|| {
                self.target_type_name()
                    .append_option(self.generic_params_full.as_ref())
            })
    }

    /// Gets the visibility of the enumerator to be generated.
    pub fn visibility(&self) -> Visibility {
        self.vis.clone().unwrap_or_else(|| {
            Visibility::Public(syn::token::Pub {
                span: Span::call_site(),
            })
        })
    }

    /// Gets the type of the enumerator to be generated for the current target type.
    pub fn enumerator_type_name(&self) -> TokenStreamRefs {
        (&self.enumerator_type_name).into()
    }

    /// Gets the type of the enumerator, with generics included but bounds and defaults stripped.
    pub fn enumerator_type(&self) -> TokenStreamRefs {
        self.enumerator_type
            .as_ref()
            .map(Into::into)
            .unwrap_or_else(|| {
                self.enumerator_type_name()
                    .append_option(self.generic_params_simple.as_ref())
            })
    }

    /// Gets the type of the enumerator, with generics and bounds included but defaults stripped.
    pub fn enumerator_type_bounded(&self) -> TokenStreamRefs {
        self.enumerator_type_with_bound
            .as_ref()
            .map(Into::into)
            .unwrap_or_else(|| {
                self.enumerator_type_name()
                    .append_option(self.generic_params_full.as_ref())
            })
    }

    /// Gets the generic parameters of the target type, with bounds and defaults stripped.
    pub fn generic_params_simple(&self) -> TokenStreamRefs {
        self.generic_params_simple.as_ref().into()
    }

    /// Gets the generic parameters of the target type, with bounds retained and defaults stripped.
    pub fn generic_params_full(&self) -> TokenStreamRefs {
        self.generic_params_full.as_ref().into()
    }

    /// Gets the where clause of the target type, with extra bounds `F: Enumerable` for each field type.
    pub fn where_clause(&self) -> TokenStreamRefs {
        self.where_clause.as_ref().into()
    }

    /// Gets the path to the `Enumerable` trait. If the `enumerable` crate is not found, it emits a compile error.
    pub fn enumerable_trait_path(&self) -> TokenStreamRefs {
        (&self.enumerable_trait_path).into()
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
