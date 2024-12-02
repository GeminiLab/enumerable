use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote_spanned};
use syn::{spanned::Spanned, Attribute, Expr, ExprLit, Lit, Meta, MetaNameValue};

/// Generates the default name for the enumerator of a type by its name.
pub(super) fn default_enumerator_name(implemented: &Ident) -> Ident {
    format_ident!("{}Enumerator", implemented)
}

/// Gets the name of the custom enumerator from the attributes.
///
/// We accept two forms of `enumerator` attribute:
/// - `#[enumerator = "CustomEnumerator"]`
/// - `#[enumerator(CustomEnumerator)]`
pub(super) fn get_custom_enumerator_name_from_attrs(
    attrs: &Vec<Attribute>,
) -> Result<Option<Ident>, (Span, String)> {
    let mut already_found = None;

    for attr in attrs {
        if attr.path().is_ident("enumerator") {
            if already_found.is_some() {
                return Err((
                    attr.span(),
                    "multiple enumerator names specified".to_string(),
                ));
            }

            already_found = Some(match &attr.meta {
                Meta::List(list) => {
                    list.parse_args::<Ident>().map_err(|e| (list.span(), format!("failed while parsing expected enumerator name (a single identifier): {}", e)))?
                }
                Meta::NameValue(MetaNameValue { value: Expr::Lit(ExprLit { lit: Lit::Str(str), .. }), .. }) => {
                    Ident::new(&str.value(), str.span())
                }
                _ => return Err((attr.span(), "expected enumerator name not specified".to_string()))
            });
        }
    }

    Ok(already_found)
}

/// Gets the name of the enumerator to be generated for a type.
///
/// If the `enumerator` attribute is not specified, it returns the default name (`<TypeName>Enumerator`).
pub(super) fn get_enumerator_name(
    ident: &Ident,
    attrs: &Vec<Attribute>,
) -> Result<Ident, TokenStream> {
    match get_custom_enumerator_name_from_attrs(attrs) {
        Ok(Some(ident)) => Ok(ident),
        Ok(None) => Ok(default_enumerator_name(ident)),
        Err((span, e)) => Err(quote_spanned!(span => compile_error!(#e);)),
    }
}
