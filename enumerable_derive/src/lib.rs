#![doc = include_str!("../IMPL_DETAIL.md")]

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned, TokenStreamExt};
use syn::{spanned::Spanned, Field, Fields, Item, ItemEnum, ItemStruct};

mod code_gen;
mod size_option;
mod targets;

use code_gen::{
    enumerable_impl, enumerable_impl_with_enumerator, EnumeratorInfo, EnumeratorKeyword,
};
use size_option::SizeOption;
use targets::Target;

/// Implements the `Enumerable` trait for an empty type.
fn impl_enumerable_for_empty_type(target: &Target) -> TokenStream {
    enumerable_impl(target, SizeOption::from_usize(0))
        .override_enumerator_type(&quote!(core::iter::Empty<Self>))
        .override_enumerator_creator(&quote!(core::iter::empty()))
        .generate()
}

/// Implements the `Enumerable` trait for a unit type.
fn impl_enumerable_for_unit_type(target: &Target, value: TokenStream) -> TokenStream {
    enumerable_impl(target, SizeOption::from_usize(1))
        .override_enumerator_type(&quote!(core::iter::Once<Self>))
        .override_enumerator_creator(&quote!(core::iter::once(#value)))
        .generate()
}

/// Implements the `Enumerable` trait for an enum without fields.
///
/// It calls `impl_enumerable_for_empty_type` if the enum has no variants.
// TODO: should we keep using a const ref to a static array or replace it with a state-machine?
fn impl_enumerable_for_plain_enum<'a>(
    target: &'a Target,
    vars: impl Iterator<Item = &'a Ident>,
) -> TokenStream {
    let target_type = target.target_type();
    let vars: Vec<_> = vars.collect();
    let vars_count = vars.len();

    if vars_count == 0 {
        return impl_enumerable_for_empty_type(target);
    }

    enumerable_impl(target, SizeOption::from_usize(vars_count))
        .override_enumerator_type(&quote!(
            core::iter::Copied<core::slice::Iter<'static, Self>>
        ))
        .override_enumerator_creator(&quote!(
            {
                const ALL_VARIANTS: &[#target_type; #vars_count] = &[#(#target_type::#vars),*];
                ALL_VARIANTS.iter().copied()
            }
        ))
        .generate()
}

/// The name of a field or its index if it's from a list of unnamed fields.
enum FieldNameOrIndex<'a> {
    Name(&'a Ident),
    Index(usize),
}

/// Returns the name of a field or its index if it's from a list of unnamed fields.
fn field_name_or_index(index: usize, field: &Field) -> FieldNameOrIndex {
    field
        .ident
        .as_ref()
        .map(FieldNameOrIndex::Name)
        .unwrap_or_else(move || FieldNameOrIndex::Index(index))
}

struct FieldToEnumerate {
    field_ref: Ident,
    field_type: TokenStream,
    enumerator_ref: Ident,
}

struct FieldsToEnumerate {
    fields: Vec<FieldToEnumerate>,
    binder: TokenStream,
}

impl FieldsToEnumerate {
    fn from_fields(
        fields: &Fields,
        mut field_ref_naming: impl FnMut(FieldNameOrIndex) -> Ident,
        mut enumerator_ref_naming: impl FnMut(FieldNameOrIndex) -> Ident,
    ) -> Self {
        let fields_to_enumerate: Vec<_> = fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let field_ref = field_ref_naming(field_name_or_index(index, field));
                let enumerator_ref = enumerator_ref_naming(field_name_or_index(index, field));
                let field_type = &field.ty;

                FieldToEnumerate {
                    field_ref,
                    field_type: quote!(#field_type),
                    enumerator_ref,
                }
            })
            .collect();

        let field_refs = fields_to_enumerate.iter().map(|field| &field.field_ref);

        let binder = if let Fields::Unnamed(_) = fields {
            quote!(( #(#field_refs),* ))
        } else {
            quote!({ #(#field_refs),* })
        };

        Self {
            fields: fields_to_enumerate,
            binder,
        }
    }

    fn fields_iter(&self) -> impl Iterator<Item = &FieldToEnumerate> {
        self.fields.iter()
    }

    fn field_refs(&self) -> impl Iterator<Item = &Ident> {
        self.fields.iter().map(|field| &field.field_ref)
    }

    fn field_types(&self) -> impl Iterator<Item = &TokenStream> {
        self.fields.iter().map(|field| &field.field_type)
    }

    fn enumerator_refs(&self) -> impl Iterator<Item = &Ident> {
        self.fields.iter().map(|field| &field.enumerator_ref)
    }
}

/// Generate the code fragment which move the generator enumerating the fields to the next state, and store the next values of the fields to yield.
fn generate_step_for_fields<'a>(
    fields: impl Iterator<Item = &'a FieldToEnumerate>,
    on_finished: TokenStream,
    enumerable_trait_path: &'a TokenStream,
) -> TokenStream {
    let mut result = on_finished;

    for (
        index,
        FieldToEnumerate {
            field_ref,
            field_type,
            enumerator_ref,
        },
    ) in fields.enumerate()
    {
        if index > 0 {
            result.append_all(quote!(
                *#enumerator_ref = <#field_type as #enumerable_trait_path>::enumerator();
                #enumerator_ref.next().unwrap()
            ));
        }

        result = quote!(
            *#field_ref = match #enumerator_ref.next() {
                Some(value) => value,
                None => {
                    #result
                },
            };
        );
    }

    quote!(
        // unreachable_patterns and unreachable_code will be triggered on uninhabited fields
        #[allow(unreachable_patterns, unreachable_code)]
        {
            #result
        }
    )
}

/// Generate the code fragment which initializes the enumerators of the fields to be able to start the enumeration, and store the first values of the fields to yield.
fn generate_init_for_fields<'a>(
    fields: impl Iterator<Item = &'a FieldToEnumerate>,
    on_non_empty: TokenStream,
    on_empty: TokenStream,
    enumerable_trait_path: &'a TokenStream,
) -> TokenStream {
    let mut field_refs = vec![];
    let mut field_types = vec![];
    let mut enumerator_refs = vec![];

    for FieldToEnumerate {
        field_ref,
        field_type,
        enumerator_ref,
    } in fields
    {
        field_refs.push(field_ref);
        field_types.push(field_type);
        enumerator_refs.push(enumerator_ref);
    }

    quote!(
        #(
            let mut #enumerator_refs = <#field_types as #enumerable_trait_path>::enumerator();
            let #field_refs = #enumerator_refs.next();
        )*

        // unreachable_patterns will be triggered on uninhabited fields
        #[allow(unreachable_patterns)]
        // unused_parens will be triggered if there is only one field
        #[allow(unused_parens)]
        match (#( #field_refs ),*) {
            ( #(Some(#field_refs)),* ) => {
                #on_non_empty
            }
            _ => {
                #on_empty
            }
        }
    )
}

fn field_ref_naming(field: FieldNameOrIndex) -> Ident {
    match field {
        FieldNameOrIndex::Name(field_name) => field_name.clone(),
        FieldNameOrIndex::Index(index) => format_ident!("field_{}", index),
    }
}

fn enumerator_ref_naming(field: FieldNameOrIndex) -> Ident {
    match field {
        FieldNameOrIndex::Name(field_name) => format_ident!("enumerator_{}", field_name),
        FieldNameOrIndex::Index(index) => format_ident!("enumerator_field_{}", index),
    }
}

/// Implements the `Enumerable` trait for an enum.
fn impl_enumerable_for_enum(e: ItemEnum) -> Result<TokenStream, TokenStream> {
    let target = Target::new_for_enum(&e)?;
    let ident = &e.ident;
    let variants = &e.variants;

    let enumerable_trait_path = target.enumerable_trait_path();

    // Call `impl_enumerable_for_empty_type` if the enum has no fields.
    //
    // This if covers empty enums also.
    if variants.iter().all(|v| v.fields.is_empty()) {
        return Ok(impl_enumerable_for_plain_enum(
            &target,
            variants.iter().map(|v| &v.ident),
        ));
    }

    // if !e.generics.params.is_empty() {
    //     return Err(
    //         quote_spanned!(e.generics.span() => compile_error!("generic types not supported yet");),
    //     );
    // }

    let mut enumerator_variants = TokenStream::new();
    let mut step_match_branches = TokenStream::new();
    let mut current_match_branches = TokenStream::new();

    let enumerator_variant_name_before = |variant: &Ident| format_ident!("Before{}", variant);
    let enumerator_variant_name_in = |variant: &Ident| format_ident!("In{}", variant);
    let enumerator_variant_name_done = format_ident!("Done");

    let variant_idents = variants.iter().map(|v| v.ident.clone()).collect::<Vec<_>>();
    let enumerator_variant_names_before: Vec<_> = variant_idents
        .iter()
        .map(enumerator_variant_name_before)
        .collect();
    let enumerator_variant_names_in: Vec<_> = variant_idents
        .iter()
        .map(enumerator_variant_name_in)
        .collect();
    let variant_count = variant_idents.len();
    let first_enumerator_variant = enumerator_variant_name_before(&variant_idents[0]);
    let mut size_options = vec![];

    for (index, var) in variants.iter().enumerate() {
        let var_ident = &variant_idents[index];
        let enumerator_variant_before = &enumerator_variant_names_before[index];
        let enumerator_variant_in = &enumerator_variant_names_in[index];

        let next_enumerator_variant_before = if index < variant_count - 1 {
            &enumerator_variant_names_before[index + 1]
        } else {
            &enumerator_variant_name_done
        };

        let fields_to_enumerate =
            FieldsToEnumerate::from_fields(&var.fields, field_ref_naming, enumerator_ref_naming);
        let binder = &fields_to_enumerate.binder;
        let enumerator_refs: Vec<_> = fields_to_enumerate.enumerator_refs().collect();
        let field_refs: Vec<_> = fields_to_enumerate.field_refs().collect();
        let field_types: Vec<_> = fields_to_enumerate.field_types().collect();

        let field_sizes = var.fields.iter().map(|f| {
            let ty = &f.ty;
            SizeOption::from_type(quote!(#ty), enumerable_trait_path)
        });
        size_options.push(SizeOption::from_product(field_sizes));

        let step = generate_step_for_fields(
            fields_to_enumerate.fields_iter(),
            quote!(*self = Self::#next_enumerator_variant_before; continue;),
            enumerable_trait_path,
        );
        let init = generate_init_for_fields(
            fields_to_enumerate.fields_iter(),
            quote!(
                *self = Self::#enumerator_variant_in{#(#enumerator_refs,)* #(#field_refs,)*};
            ),
            quote!(
                *self = Self::#next_enumerator_variant_before;
                continue;
            ),
            enumerable_trait_path,
        );

        enumerator_variants.append_all(quote!(
            #enumerator_variant_before,
            #enumerator_variant_in{
                #(#enumerator_refs: <#field_types as #enumerable_trait_path>::Enumerator,)*
                #(#field_refs: #field_types,)*
            },
        ));

        step_match_branches.append_all(quote!(
            Self::#enumerator_variant_before => {
                #init
            },
            Self::#enumerator_variant_in{#(#enumerator_refs,)* #(#field_refs,)*} => {
                #step
            },
        ));

        current_match_branches.append_all(quote!(
            Self::#enumerator_variant_in{#(#field_refs,)* ..} => {
                #(
                    let #field_refs = *#field_refs;
                )*
                Some(#ident::#var_ident #binder)
            },
        ));
    }

    enumerator_variants.append_all(quote!(#enumerator_variant_name_done,));

    let enumerable_size_option = SizeOption::from_sum(size_options.into_iter());
    let impl_ = enumerable_impl_with_enumerator(
        &target,
        enumerable_size_option,
        EnumeratorInfo {
            keyword: EnumeratorKeyword::Enum,
            body: enumerator_variants,
            new_fn_body: quote!({
                let mut result = Self::#first_enumerator_variant;
                result.step();
                result
            }),
            step_fn_body: quote!({
                loop {
                    match self {
                        #step_match_branches
                        Self::#enumerator_variant_name_done => {},
                    }

                    break;
                }
            }),
            next_to_yield_fn_body: quote!({
                match self {
                    #current_match_branches
                    _ => None,
                }
            }),
        },
    );

    Ok(impl_.generate())
}

/// Implements the `Enumerable` trait for a struct.
fn impl_enumerable_for_struct(s: ItemStruct) -> Result<TokenStream, TokenStream> {
    let target = Target::new_for_struct(&s)?;
    let ident = &s.ident;
    let fields = &s.fields;
    let enumerable_trait_path = target.enumerable_trait_path();

    // if !s.generics.params.is_empty() {
    //     return Err(
    //         quote_spanned!(s.generics.span() => compile_error!("generic types not supported yet");),
    //     );
    // }

    let fields_to_enumerate =
        FieldsToEnumerate::from_fields(fields, field_ref_naming, enumerator_ref_naming);
    let binder = &fields_to_enumerate.binder;
    let enumerator_refs: Vec<_> = fields_to_enumerate.enumerator_refs().collect();
    let field_types: Vec<_> = fields_to_enumerate.field_types().collect();
    let target_generics = target.generic_params_simple();

    if fields.is_empty() {
        return Ok(impl_enumerable_for_unit_type(
            &target,
            quote!(#ident #binder),
        ));
    }

    let field_sizes = fields.iter().map(|f| {
        let ty = &f.ty;
        SizeOption::from_type(quote!(#ty), enumerable_trait_path)
    });
    let enumerable_size_option = SizeOption::from_product(field_sizes);

    let step = generate_step_for_fields(
        fields_to_enumerate.fields_iter(),
        quote!(self.next = None; return;),
        enumerable_trait_path,
    );

    let init = generate_init_for_fields(
        fields_to_enumerate.fields_iter(),
        quote!(
            return Self {
                #( #enumerator_refs, )* next: Some(#ident #binder),
            }
        ),
        quote!(
            return Self {
                #( #enumerator_refs, )* next: None,
            }
        ),
        enumerable_trait_path,
    );

    let impl_ = enumerable_impl_with_enumerator(
        &target,
        enumerable_size_option,
        EnumeratorInfo {
            keyword: EnumeratorKeyword::Struct,
            body: quote! {
                #( #enumerator_refs: <#field_types as #enumerable_trait_path>::Enumerator, )*
                next: Option<#ident #target_generics>,
            },
            new_fn_body: quote!(#init),
            step_fn_body: quote!({
                if let Some(#ident #binder) = &mut self.next {
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

/// Derives the `Enumerable` trait for an enum or struct.
#[proc_macro_derive(Enumerable, attributes(enumerator))]
pub fn derive_enumerable(input: TokenStream1) -> TokenStream1 {
    let target = syn::parse_macro_input!(input as Item);

    let result = match target {
        Item::Enum(e) => impl_enumerable_for_enum(e),
        Item::Struct(s) => impl_enumerable_for_struct(s),
        _ => Err(quote_spanned!(target.span() => compile_error!("only enums and structs are supported");).into()),
    };

    match result {
        Ok(tokens) => tokens,
        Err(tokens) => tokens,
    }
    .into()
}
