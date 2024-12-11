use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Field, Fields, Ident};

/// An identifier or an index.
///
/// Used to represent a field in a field list.
pub enum IdentOrIndex<'a> {
    Name(&'a Ident),
    Index(usize),
}

/// Returns the name of a field or its index if it's from a list of unnamed fields.
fn field_name_or_index(index: usize, field: &Field) -> IdentOrIndex {
    field
        .ident
        .as_ref()
        .map(IdentOrIndex::Name)
        .unwrap_or_else(move || IdentOrIndex::Index(index))
}

/// A field in a field list that needs to be enumerated.
pub struct FieldToEnumerate {
    pub field_ref: Ident,
    pub field_type: TokenStream,
    pub enumerator_ref: Ident,
}

/// A list of fields that need to be enumerated.
pub struct FieldsToEnumerate {
    pub fields: Vec<FieldToEnumerate>,
    pub binder: TokenStream,
}

impl FieldsToEnumerate {
    /// Create a new `FieldsToEnumerate` from a [`Fields`] instance.
    pub fn from_fields(
        fields: &Fields,
        mut field_ref_naming: impl FnMut(IdentOrIndex) -> Ident,
        mut enumerator_ref_naming: impl FnMut(IdentOrIndex) -> Ident,
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

    /// Create a new `FieldsToEnumerate` from a list of unnamed fields constructed manually.
    ///
    /// The fields are provided as an iterator of tuples, where the first element is the field's
    /// reference, the second element is the field's type, and the third element is the enumerator's
    /// reference.
    pub fn new_unnamed(fields: impl Iterator<Item = (String, TokenStream, String)>) -> Self {
        let fields_to_enumerate: Vec<_> = fields
            .map(|(field_ref, field_type, enumerator_ref)| FieldToEnumerate {
                field_ref: Ident::new(&field_ref, Span::call_site()),
                field_type,
                enumerator_ref: Ident::new(&enumerator_ref, Span::call_site()),
            })
            .collect();

        let field_refs = fields_to_enumerate.iter().map(|field| &field.field_ref);

        let binder = quote!(( #(#field_refs),* ));

        Self {
            fields: fields_to_enumerate,
            binder,
        }
    }

    /// Return an iterator over the fields that need to be enumerated.
    pub fn fields_iter(&self) -> impl Iterator<Item = &FieldToEnumerate> {
        self.fields.iter()
    }

    /// Return an iterator over the field references.
    pub fn field_refs(&self) -> impl Iterator<Item = &Ident> {
        self.fields.iter().map(|field| &field.field_ref)
    }

    /// Return an iterator over the field types.
    pub fn field_types(&self) -> impl Iterator<Item = &TokenStream> {
        self.fields.iter().map(|field| &field.field_type)
    }

    /// Return an iterator over the enumerator references.
    pub fn enumerator_refs(&self) -> impl Iterator<Item = &Ident> {
        self.fields.iter().map(|field| &field.enumerator_ref)
    }
}
