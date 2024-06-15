use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote, quote_spanned, TokenStreamExt};
use syn::{spanned::Spanned, Fields, Item, ItemEnum, ItemStruct};

/// Implements the `Enumerable` trait for a empty type.
fn impl_enumerable_for_empty_type(ident: &Ident) -> TokenStream {
    quote!(
        impl Enumerable for #ident {
            type Enumerator = std::iter::Empty<Self>;

            fn enumerator() -> Self::Enumerator {
                std::iter::empty()
            }
        }
    )
    .into()
}

/// Implements the `Enumerable` trait for a unit type.
fn impl_enumerable_for_unit_type(ident: &Ident, value: TokenStream2) -> TokenStream {
    quote!(
        impl Enumerable for #ident {
            type Enumerator = std::iter::Once<Self>;

            fn enumerator() -> Self::Enumerator {
                std::iter::once(#value)
            }
        }
    )
    .into()
}

struct EnumerableField {
    mut_ref_name: Ident,
    field_type: TokenStream2,
    enumerator_expr: TokenStream2,
}

fn impl_calculate_next_for_field_list(
    fields: impl Iterator<Item = EnumerableField>,
    none_setter_breaker: TokenStream2,
) -> TokenStream2 {
    let mut result = quote!();
    let mut iter = fields;
    let first_field = iter.next().unwrap();

    let mut_ref_name = &first_field.mut_ref_name;
    let enumerator_expr = &first_field.enumerator_expr;
    result.append_all(quote!(
        *#mut_ref_name = match #enumerator_expr.next() {
            Some(value) => value,
            None => {
                #none_setter_breaker;
            },
        };
    ));

    while let Some(field) = iter.next() {
        let mut_ref_name = &field.mut_ref_name;
        let field_type = &field.field_type;
        let enumerator_expr = &field.enumerator_expr;
        result = quote!(
            *#mut_ref_name = match #enumerator_expr.next() {
                Some(value) => value,
                None => {
                    #result

                    #enumerator_expr = <#field_type as Enumerable>::enumerator();
                    #enumerator_expr.next().unwrap()
                },
            };
        );
    }

    result
}

// TODO: should we keep using a const ref to a static array or replace it with a state-machine?
/// Implements the `Enumerable` trait for an enum.
fn impl_enumerable_for_enum(e: ItemEnum) -> TokenStream {
    let ident = &e.ident;
    let variants = &e.variants;

    if let Some(v) = variants.iter().find(|v| !v.fields.is_empty()) {
        return quote_spanned!(v.ident.span() => compile_error!("no fields expected")).into();
    }

    let variants_count = variants.iter().count();
    let variants_iter = variants.iter().map(|v| &v.ident);

    if variants_count == 0 {
        return impl_enumerable_for_empty_type(ident);
    }

    quote!(
        #[automatically_derived]
        impl Enumerable for #ident {
            type Enumerator = std::iter::Copied<std::slice::Iter<'static, Self>>;

            fn enumerator() -> Self::Enumerator {
                const ALL_VARIANTS: &[#ident; #variants_count] = &[#(#ident::#variants_iter),*];

                return ALL_VARIANTS.iter().copied()
            }
        }
    )
    .into()
}

/// Implements the `Enumerable` trait for a struct.
fn impl_enumerable_for_struct(s: ItemStruct) -> TokenStream {
    let vis = &s.vis;
    let ident = &s.ident;
    let fields = &s.fields;

    if !s.generics.params.is_empty() {
        return quote_spanned!(s.generics.span() => compile_error!("generic types not supported"))
            .into();
    }

    let field_count = fields.iter().count();
    let is_fields_named = match fields {
        Fields::Named(_) if field_count > 0 => true,
        Fields::Unnamed(_) if field_count > 0 => false,
        Fields::Unnamed(_) => {
            // Fields::Unnamed with no fields
            return impl_enumerable_for_unit_type(ident, quote!(#ident()));
        }
        _ => {
            // Fields::Unit or Fields::Named with no fields or Fields::Unnamed with no fields
            return impl_enumerable_for_unit_type(ident, quote!(#ident{}));
        }
    };

    let mut field_names: Vec<Ident> = Vec::with_capacity(field_count);
    let mut field_types: Vec<TokenStream2> = Vec::with_capacity(field_count);
    let mut field_enumerator_names: Vec<Ident> = Vec::with_capacity(field_count);
    let mut field_enumerator_types: Vec<TokenStream2> = Vec::with_capacity(field_count);
    for (index, field) in fields.iter().enumerate() {
        let field_name = field
            .ident
            .as_ref()
            .cloned()
            .unwrap_or(format_ident!("field_{}", index));
        let field_type = &field.ty;
        let enumerator_name = format_ident!("enumerator_{}", field_name);
        let enumerator_type = quote!(<#field_type as Enumerable>::Enumerator);

        field_names.push(field_name);
        field_types.push(quote!(#field_type));
        field_enumerator_names.push(enumerator_name);
        field_enumerator_types.push(enumerator_type);
    }

    let enumerator_struct_ident = format_ident!("{}Enumerator", ident);
    let field_enumerators = quote!(#(#field_enumerator_names: #field_enumerator_types,)*);
    let calculated_next_binder = if is_fields_named {
        quote!(#ident{#(#field_names),*})
    } else {
        quote!(#ident(#(#field_names),*))
    };
    let enumerator_struct_creator = quote!(
        #(
            let mut #field_enumerator_names = <#field_types as Enumerable>::enumerator();
            let #field_names = #field_enumerator_names.next();
        )*

        let calculated_next = if false #(|| #field_names.is_none())* {
            None
        } else {
            #(let #field_names = #field_names.unwrap();)*
            Some(#calculated_next_binder)
        };

        Self {
            #(#field_enumerator_names,)*
            calculated_next,
        }
    );
    let calculate_next_body = impl_calculate_next_for_field_list(
        field_names
            .iter()
            .zip(field_enumerator_names.iter().zip(field_types.iter()))
            .map(
                |(field_name, (enumerator_name, field_type))| EnumerableField {
                    mut_ref_name: field_name.clone(),
                    field_type: field_type.clone(),
                    enumerator_expr: quote!(self.#enumerator_name),
                },
            ),
        quote!(self.calculated_next = None; return;),
    );

    let result = quote!(
        #[automatically_derived]
        impl Enumerable for #ident {
            type Enumerator = #enumerator_struct_ident;

            fn enumerator() -> Self::Enumerator {
                #enumerator_struct_ident::new()
            }
        }

        #[doc(hidden)]
        #vis struct #enumerator_struct_ident {
            #field_enumerators
            calculated_next: Option<#ident>,
        }

        impl #enumerator_struct_ident {
            fn new() -> Self {
                #enumerator_struct_creator
            }

            fn calculate_next(&mut self) {
                if let Some(#calculated_next_binder) = &mut self.calculated_next {
                    #calculate_next_body
                }
            }
        }

        #[automatically_derived]
        impl Iterator for #enumerator_struct_ident {
            type Item = #ident;

            fn next(&mut self) -> Option<<Self as Iterator>::Item> {
                let result = self.calculated_next;
                self.calculate_next();
                result
            }
        }
    );

    result.into()
}

/// Derives the `Enumerable` trait for an enum or struct.
#[proc_macro_derive(Enumerable)]
pub fn derive_enumerable(input: TokenStream) -> TokenStream {
    let target = syn::parse_macro_input!(input as Item);

    match target {
        Item::Enum(e) => impl_enumerable_for_enum(e),
        Item::Struct(s) => impl_enumerable_for_struct(s),
        _ => quote_spanned!(target.span() => compile_error!("expected enum or struct")).into(),
    }
}
