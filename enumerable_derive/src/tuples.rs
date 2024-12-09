use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, TokenStreamExt};
use syn::{parse::Parse, LitInt, Path, Type, TypePath};

use crate::{
    code_gen::{enumerable_impl_with_enumerator, EnumeratorInfo, EnumeratorKeyword},
    fields::FieldsToEnumerate,
    generate_init_for_fields, generate_step_for_fields,
    size_option::SizeOption,
    targets::Target,
};

/// Returns the i-th capital letter.
fn capital_letter(i: usize) -> char {
    char::from(b'A' + i as u8)
}

/// Returns the i-th lowercase letter.
fn lowercase_letter(i: usize) -> char {
    char::from(b'a' + i as u8)
}

/// Implements the `Enumerable` trait for a tuple with `n` elements.
fn impl_enumerable_for_tuple_n(n: usize) -> Result<TokenStream, TokenStream> {
    if n > 26 {
        let error = format!(
            "tuple with more than 26 elements are not supported, but got {}",
            n
        );
        return Err(quote_spanned!(Span::call_site() => compile_error!(#error);));
    }

    let enumerator_ident = format_ident!("Tuple{}Enumerator", n);
    // let enumerator_desc = format!("Enumerator for tuples with {} elements.", n);

    // names for types of the tuple elements
    let gen_names: Vec<_> = (0..n).map(capital_letter).collect();
    // identifiers for types of the tuple elements
    let gen_types: Vec<_> = gen_names
        .iter()
        .map(|c| format_ident!("{}", c))
        .map(|s| {
            Type::Path(TypePath {
                qself: None,
                path: Path::from(s),
            })
        })
        .collect();
    // the generic parameters for the enumerator
    let gen_params = quote!(<#( #gen_types ),*>);
    // the type of the tuple
    let tuple_type = quote!((#( #gen_types ),*));
    // where clause here
    let where_clause = quote!(
        where #( #gen_types: Enumerable ),*
    );

    let target = Target::new_for_any(tuple_type.clone(), quote!(#enumerator_ident))
        .with_generic_params(gen_params.clone(), gen_params.clone())
        .with_where_clause(where_clause.clone())
        .with_target_type(tuple_type.clone(), tuple_type.clone());
    let enumerable_trait_path = target.enumerable_trait_path();

    let fields = gen_types.iter().enumerate().map(|(i, ty)| {
        (
            format!("{}", lowercase_letter(i)),
            quote!(#ty),
            format!("enumerator_{}", lowercase_letter(i)),
        )
    });
    let fields = FieldsToEnumerate::new_unnamed(fields);
    let field_types: Vec<_> = fields.field_types().collect();
    let enumerator_refs: Vec<_> = fields.enumerator_refs().collect();
    let binder = &fields.binder;

    let step = generate_step_for_fields(
        fields.fields_iter(),
        quote!(self.next = None; return;),
        enumerable_trait_path.clone(),
    );

    let init = generate_init_for_fields(
        fields.fields_iter(),
        quote!(
            return Self {
                #( #enumerator_refs, )* next: Some(#binder),
            }
        ),
        quote!(
            return Self {
                #( #enumerator_refs, )* next: None,
            }
        ),
        enumerable_trait_path.clone(),
    );

    // the size option for the tuple
    let size_option = SizeOption::from_product(
        gen_types
            .iter()
            .map(|ty| SizeOption::from_type(ty, enumerable_trait_path.clone())),
    );

    let impl_ = enumerable_impl_with_enumerator(
        &target,
        size_option,
        EnumeratorInfo {
            keyword: EnumeratorKeyword::Struct,
            body: quote! {
                #( #enumerator_refs: <#field_types as #enumerable_trait_path>::Enumerator, )*
                next: Option<#tuple_type>,
            },
            new_fn_body: quote!(#init),
            step_fn_body: quote!({
                if let Some(#binder) = &mut self.next {
                    #(
                        let #enumerator_refs = &mut self.#enumerator_refs;
                    )*
                    {
                        #step
                    }
                }
            }),
            next_to_yield_fn_body: quote!(self.next),
        },
    );

    Ok(impl_.generate())
}

/// The input for the [`impl_enumerable_for_tuples`] function. An inclusive range of tuple sizes.
pub struct ImplEnumerableForTupleParams {
    from: usize,
    to: usize,
}

impl Parse for ImplEnumerableForTupleParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let from: LitInt = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let to: LitInt = input.parse()?;

        Ok(Self {
            from: from.base10_parse()?,
            to: to.base10_parse()?,
        })
    }
}

/// Implements the `Enumerable` trait for tuples with sizes in the given range.
pub fn impl_enumerable_for_tuples(
    params: ImplEnumerableForTupleParams,
) -> Result<TokenStream, TokenStream> {
    let mut result = TokenStream::new();
    for i in params.from..=params.to {
        result.append_all(impl_enumerable_for_tuple_n(i)?);
    }

    Ok(result)
}
