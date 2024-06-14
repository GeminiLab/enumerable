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
    let mut peekable_names: Vec<Ident> = Vec::with_capacity(field_count);

    let enumerator_struct_ident = format_ident!("{}Enumerator", ident);
    let mut enumerator_struct_fields = TokenStream2::new();
    let mut enumerator_struct_initializer = TokenStream2::new();
    let mut enumerator_struct_peek_function_body = TokenStream2::new();
    let mut enumerator_struct_advance_function_body = TokenStream2::new();
    for field in fields.iter().enumerate() {
        let (i, field) = field;

        let field_name = field
            .ident
            .as_ref()
            .cloned()
            .unwrap_or(format_ident!("field_{}", i));
        let peekable_name = field
            .ident
            .as_ref()
            .map_or(format_ident!("peekable_{}", i), |ident| {
                format_ident!("peekable_{}", ident)
            });

        let typ = field.ty.clone();
        let enumerator_typ = quote!(<#typ as Enumerable>::Enumerator);
        let peekable_typ = quote!(std::iter::Peekable<#enumerator_typ>);

        enumerator_struct_fields.append_all(quote!(
            #peekable_name: #peekable_typ,
        ));

        enumerator_struct_initializer.append_all(quote!(
            #peekable_name: <#typ>::enumerator().peekable(),
        ));

        enumerator_struct_peek_function_body.append_all(quote!(
            let #field_name = self.#peekable_name.peek().copied()?;
        ));

        enumerator_struct_advance_function_body = if i == 0 {
            quote!(
                self.#peekable_name.next();
            )
        } else {
            quote!(
                self.#peekable_name.next();
                if self.#peekable_name.peek().is_some() {
                    return;
                }
                self.#peekable_name = <#typ>::enumerator().peekable();

                #enumerator_struct_advance_function_body
            )
        };

        field_names.push(field_name);
        peekable_names.push(peekable_name);
    }

    let enumerator_struct_peek_function_return = if is_fields_named {
        quote!(return Some(#ident{#(#field_names),*}))
    } else {
        quote!(return Some(#ident(#(#field_names),*)))
    };

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
            #enumerator_struct_fields
        }

        impl #enumerator_struct_ident {
            fn new() -> Self {
                Self {
                    #enumerator_struct_initializer
                }
            }

            fn peek(&mut self) -> Option<<Self as Iterator>::Item> {
                #enumerator_struct_peek_function_body
                #enumerator_struct_peek_function_return
            }

            fn advance(&mut self) {
                #enumerator_struct_advance_function_body
            }
        }

        #[automatically_derived]
        impl Iterator for #enumerator_struct_ident {
            type Item = #ident;

            fn next(&mut self) -> Option<<Self as Iterator>::Item> {
                let result = self.peek();
                self.advance();
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
