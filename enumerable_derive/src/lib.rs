use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote, quote_spanned, TokenStreamExt};
use syn::{spanned::Spanned, Field, Fields, Item, ItemEnum, ItemStruct};

fn get_default_enumerator_name(implemented: &Ident) -> Ident {
    format_ident!("{}Enumerator", implemented)
}

/// Implements the `Enumerable` trait for an empty type.
fn impl_enumerable_for_empty_type(ident: &Ident) -> TokenStream2 {
    quote!(
        impl Enumerable for #ident {
            type Enumerator = std::iter::Empty<Self>;

            fn enumerator() -> Self::Enumerator {
                std::iter::empty()
            }
        }
    )
}

/// Implements the `Enumerable` trait for a unit type.
fn impl_enumerable_for_unit_type(ident: &Ident, value: TokenStream2) -> TokenStream2 {
    quote!(
        impl Enumerable for #ident {
            type Enumerator = std::iter::Once<Self>;

            fn enumerator() -> Self::Enumerator {
                std::iter::once(#value)
            }
        }
    )
}

struct GeneratedFieldsNextCalculator {
    body: TokenStream2,
    binder: TokenStream2,
    field_refs: Vec<TokenStream2>,
    field_types: Vec<TokenStream2>,
    enumerator_refs: Vec<TokenStream2>,
    enumerator_types: Vec<TokenStream2>,
}

enum FieldNameOrIndex<'a> {
    Name(&'a Ident),
    Index(usize),
}

fn field_name_or_index(index: usize, field: &Field) -> FieldNameOrIndex {
    field
        .ident
        .as_ref()
        .map(FieldNameOrIndex::Name)
        .unwrap_or_else(move || FieldNameOrIndex::Index(index))
}

/// Generate the code fragment which calculates the next value of the fields from a list of enumerators.
///
/// ## Parameters
/// - `fields`: The list of fields. The order of the fields is important, as the generated code will enumerate them in the lexicographic order.
/// - `breaker`: The code to execute when the enumeration of the fields is done.
/// - `field_ref_factory`: A function that generates the code to a mutable reference to a field.
/// - `enumerator_ref_factory`: A function that generates the code to a mutable reference to the enumerator of a field.
fn generate_next_calculator_for_fields(
    fields: &Fields,
    breaker: TokenStream2,
    mut field_ref_factory: impl FnMut(FieldNameOrIndex) -> TokenStream2,
    mut enumerator_ref_factory: impl FnMut(FieldNameOrIndex) -> TokenStream2,
) -> GeneratedFieldsNextCalculator {
    if fields.is_empty() {
        let empty_binder = if let Fields::Unnamed(_) = fields {
            quote!(())
        } else {
            quote!({})
        };

        return GeneratedFieldsNextCalculator {
            body: breaker,
            binder: empty_binder,
            field_refs: vec![],
            field_types: vec![],
            enumerator_refs: vec![],
            enumerator_types: vec![],
        };
    }

    let is_named = if let Fields::Named(_) = fields {
        true
    } else {
        false
    };

    let iter = fields.iter().enumerate();
    let mut field_refs = vec![];
    let mut field_types = vec![];
    let mut enumerator_refs = vec![];
    let mut enumerator_types = vec![];
    let mut binder_items: Vec<TokenStream2> = vec![];

    let mut calculator_body = quote!();

    for (index, field) in iter {
        let field_ref = field_ref_factory(field_name_or_index(index, field));
        let enumerator_ref = enumerator_ref_factory(field_name_or_index(index, field));
        let field_type = &field.ty;

        calculator_body = if index == 0 {
            quote!(
                *#field_ref = match #enumerator_ref.next() {
                    Some(value) => value,
                    None => {
                        #breaker
                    },
                };
            )
        } else {
            quote!(
                *#field_ref = match #enumerator_ref.next() {
                    Some(value) => value,
                    None => {
                        #calculator_body

                        *#enumerator_ref = <#field_type as Enumerable>::enumerator();
                        #enumerator_ref.next().unwrap()
                    },
                };
            )
        };

        binder_items.push(if is_named {
            let field_name = field.ident.as_ref().unwrap();
            quote!(#field_name: #field_ref)
        } else {
            quote!(#field_ref)
        });

        field_refs.push(field_ref);
        field_types.push(quote!(#field_type));
        enumerator_refs.push(enumerator_ref);
        enumerator_types.push(quote!(<#field_type as Enumerable>::Enumerator));
    }

    return GeneratedFieldsNextCalculator {
        body: calculator_body,
        binder: if is_named {
            quote!({ #(#binder_items),* })
        } else {
            quote!(( #(#binder_items),* ))
        },
        field_refs,
        field_types,
        enumerator_refs,
        enumerator_types,
    };
}

// TODO: should we keep using a const ref to a static array or replace it with a state-machine?
fn impl_enumerable_for_plain_enum<'a>(
    ident: &Ident,
    vars: impl Iterator<Item = &'a Ident>,
) -> TokenStream2 {
    let vars: Vec<_> = vars.collect();
    let vars_count = vars.len();

    if vars_count == 0 {
        return impl_enumerable_for_empty_type(ident);
    }

    quote!(
        #[automatically_derived]
        impl Enumerable for #ident {
            type Enumerator = std::iter::Copied<std::slice::Iter<'static, Self>>;

            fn enumerator() -> Self::Enumerator {
                const ALL_VARIANTS: &[#ident; #vars_count] = &[#(#ident::#vars),*];

                return ALL_VARIANTS.iter().copied()
            }
        }
    )
}

/// Implements the `Enumerable` trait for an enum.
fn impl_enumerable_for_enum(e: ItemEnum) -> TokenStream2 {
    let vis = &e.vis;
    let ident = &e.ident;
    let variants = &e.variants;

    if variants.iter().all(|v| v.fields.is_empty()) {
        return impl_enumerable_for_plain_enum(ident, variants.iter().map(|v| &v.ident));
    }

    if !e.generics.params.is_empty() {
        return quote_spanned!(e.generics.span() => compile_error!("generic types not supported"))
            .into();
    }

    let enumerator_ident = get_default_enumerator_name(ident);
    let mut enumerator_variants = TokenStream2::new();
    let mut calculate_next_match_branches = TokenStream2::new();
    let mut get_calculated_next_match_branches = TokenStream2::new();

    let enumerator_variant_name_before = |variant: &Ident| format_ident!("Before{}", variant);
    let enumerator_variant_name_in = |variant: &Ident| format_ident!("In{}", variant);
    let enumerator_variant_name_done = format_ident!("Done");

    let variant_idents = variants.iter().map(|v| v.ident.clone()).collect::<Vec<_>>();
    let enumerator_variant_names_before = variant_idents
        .iter()
        .map(enumerator_variant_name_before)
        .collect::<Vec<_>>();
    let enumerator_variant_names_in = variant_idents
        .iter()
        .map(enumerator_variant_name_in)
        .collect::<Vec<_>>();
    let variant_count = variant_idents.len();
    let first_enumerator_variant = enumerator_variant_name_before(&variant_idents[0]);

    for (index, var) in variants.iter().enumerate() {
        let var_ident = &variant_idents[index];
        let enumerator_variant_before = &enumerator_variant_names_before[index];
        let enumerator_variant_in = &enumerator_variant_names_in[index];

        let next_enumerator_variant_before = if index < variant_count - 1 {
            &enumerator_variant_names_before[index + 1]
        } else {
            &enumerator_variant_name_done
        };

        let GeneratedFieldsNextCalculator {
            body,
            binder,
            field_refs,
            field_types,
            enumerator_refs,
            enumerator_types,
        } = generate_next_calculator_for_fields(
            &var.fields,
            quote!(*self = Self::#next_enumerator_variant_before; continue;),
            |field_name_or_index| {
                let ident = match field_name_or_index {
                    FieldNameOrIndex::Name(field_name) => format_ident!("calculated_{}", field_name),
                    FieldNameOrIndex::Index(index) => format_ident!("calculated_field_{}", index),
                };
                quote!(#ident)
            },
            |field_name_or_index| {
                let ident = match field_name_or_index {
                    FieldNameOrIndex::Name(field_name) => format_ident!("enumerator_{}", field_name),
                    FieldNameOrIndex::Index(index) => format_ident!("enumerator_field_{}", index),
                };
                quote!(#ident)
            },
        );

        enumerator_variants.append_all(quote!(
            #enumerator_variant_before,
            #enumerator_variant_in{#(#enumerator_refs:#enumerator_types,)* #(#field_refs:#field_types,)*},
        ));

        calculate_next_match_branches.append_all(quote!(
            Self::#enumerator_variant_before => {
                #(
                    let mut #enumerator_refs = <#field_types as Enumerable>::enumerator();
                    let #field_refs = #enumerator_refs.next();
                )*

                if false #(|| #field_refs.is_none())* {
                    *self = Self::#next_enumerator_variant_before;
                    continue;
                } else {
                    #(
                        let #field_refs = #field_refs.unwrap();
                    )*
                    *self = Self::#enumerator_variant_in{#(#enumerator_refs,)* #(#field_refs,)*};
                }
            },
            Self::#enumerator_variant_in{#(#enumerator_refs,)* #(#field_refs,)*} => {
                #body
            },
        ));

        get_calculated_next_match_branches.append_all(quote!(
            Self::#enumerator_variant_in{#(#field_refs,)* ..} => {
                #(
                    let #field_refs = *#field_refs;
                )*
                Some(#ident::#var_ident #binder)
            },
        ));
    }

    quote!(
        #[automatically_derived]
        impl Enumerable for #ident {
            type Enumerator = #enumerator_ident;

            fn enumerator() -> Self::Enumerator {
                #enumerator_ident::new()
            }
        }

        #[doc(hidden)]
        #vis enum #enumerator_ident {
            #enumerator_variants
            #enumerator_variant_name_done,
        }

        #[automatically_derived]
        impl Iterator for #enumerator_ident {
            type Item = #ident;

            fn next(&mut self) -> Option<<Self as Iterator>::Item> {
                let result = self.get_calculated_next();
                self.calculate_next();
                result
            }
        }

        impl #enumerator_ident {
            fn new() -> Self {
                let mut result = #enumerator_ident::#first_enumerator_variant;
                result.calculate_next();
                result
            }

            fn calculate_next(&mut self) {
                loop {
                    match self {
                        #calculate_next_match_branches
                        _ => *self = Self::#enumerator_variant_name_done,
                    }

                    break;
                }
            }

            fn get_calculated_next(&mut self) -> Option<#ident> {
                match self {
                    #get_calculated_next_match_branches
                    _ => None,
                }
            }
        }
    )
}

/// Implements the `Enumerable` trait for a struct.
fn impl_enumerable_for_struct(s: ItemStruct) -> TokenStream2 {
    let vis = &s.vis;
    let ident = &s.ident;
    let fields = &s.fields;

    if !s.generics.params.is_empty() {
        return quote_spanned!(s.generics.span() => compile_error!("generic types not supported"))
            .into();
    }

    let GeneratedFieldsNextCalculator {
        body: calculate_next_inner,
        binder,
        field_refs: field_names,
        field_types,
        enumerator_refs: enumerator_names,
        enumerator_types,
    } = generate_next_calculator_for_fields(
        fields,
        quote!(self.calculated_next = None; return;),
        |field_name_or_index| {
            let ident = match field_name_or_index {
                FieldNameOrIndex::Name(field_name) => field_name.clone(),
                FieldNameOrIndex::Index(index) => format_ident!("field_{}", index),
            };
            quote!(#ident)
        },
        |field_name_or_index| {
            let ident = match field_name_or_index {
                FieldNameOrIndex::Name(field_name) => format_ident!("enumerator_{}", field_name),
                FieldNameOrIndex::Index(index) => format_ident!("enumerator_field_{}", index),
            };
            quote!(#ident)
        },
    );

    if field_names.is_empty() {
        return impl_enumerable_for_unit_type(ident, quote!(#ident #binder));
    }

    let enumerator_struct_ident = get_default_enumerator_name(ident);
    let field_enumerators = enumerator_names
        .iter()
        .zip(enumerator_types.iter())
        .map(|(name, ty)| quote!(#name: #ty,));
    let enumerator_struct_creator = quote!(
        #(
            let mut #enumerator_names = <#field_types as Enumerable>::enumerator();
            let #field_names = #enumerator_names.next();
        )*

        let calculated_next = if false #(|| #field_names.is_none())* {
            None
        } else {
            #(let #field_names = #field_names.unwrap();)*
            Some(#ident #binder)
        };

        Self {
            #(#enumerator_names,)*
            calculated_next,
        }
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
            #(#field_enumerators)*
            calculated_next: Option<#ident>,
        }

        impl #enumerator_struct_ident {
            fn new() -> Self {
                #enumerator_struct_creator
            }

            fn calculate_next(&mut self) {
                #(
                    let mut #enumerator_names = &mut self.#enumerator_names;
                )*

                if let Some(#ident #binder) = &mut self.calculated_next {
                    #calculate_next_inner
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

    result
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
    .into()
}
