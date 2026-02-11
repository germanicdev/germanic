//! # Schema Macro Implementation
//!
//! Uses `darling` for declarative attribute parsing.
//!
//! ## Architecture: Why darling?
//!
//! **Problem without darling:**
//! ```text
//! Manual parsing:
//!   for attr in &input.attrs {
//!       if attr.path().is_ident("germanic") {
//!           attr.parse_nested_meta(|meta| {
//!               if meta.path.is_ident("schema_id") {
//!                   // 50+ lines of boilerplate per attribute
//!               }
//!           })?;
//!       }
//!   }
//! ```
//!
//! **With darling:**
//! ```text
//! #[derive(FromDeriveInput)]
//! #[darling(attributes(germanic))]
//! struct SchemaOpts {
//!     schema_id: String,  // Automatically parsed!
//! }
//! # Schema Macro Implementation
//!
//! Uses `darling` for declarative attribute parsing.
//!
//! ## Generated Traits
//!
//! - `SchemaMetadata` → schema_id(), schema_version()
//! - `Validate` → validate()
//! - `Default` → default()

use darling::{FromDeriveInput, FromField, ast::Data, util::Flag};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Ident, Type};

// ============================================================================
// DATA STRUCTURES FOR ATTRIBUTE PARSING (darling)
// ============================================================================

/// Options at struct level.
///
/// ```rust,ignore
/// #[germanic(schema_id = "de.gesundheit.praxis.v1")]
/// ```
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(germanic), supports(struct_named))]
pub struct SchemaOptions {
    /// Struct name
    ident: Ident,
    /// Generics
    generics: syn::Generics,
    /// Struct fields
    data: Data<(), FieldOptions>,
    /// Unique schema ID (required)
    schema_id: String,
    /// Path to FlatBuffer type (optional, for later)
    #[darling(default)]
    flatbuffer: Option<String>,
}

/// Options at field level.
///
/// ```rust,ignore
/// #[germanic(required)]
/// pub name: String,
///
/// #[germanic(default = "DE")]
/// pub land: String,
/// ```
#[derive(Debug, FromField)]
#[darling(attributes(germanic))]
pub struct FieldOptions {
    /// Field name
    ident: Option<Ident>,
    /// Field type
    ty: Type,
    /// Required field flag
    #[darling(default)]
    required: Flag,
    /// Default value as string (e.g. "DE", "true", "false")
    #[darling(default)]
    default: Option<String>,
}

// ============================================================================
// MAIN IMPLEMENTATION
// ============================================================================

/// Entry point for macro expansion.
///
/// Generates three trait implementations:
/// 1. `SchemaMetadata` – Schema ID and version
/// 2. `Validate` – Required field validation
/// 3. `Default` – Default values for all fields
pub fn implement_germanic_schema(input: DeriveInput) -> Result<TokenStream, darling::Error> {
    // Parse attributes with darling
    let options = SchemaOptions::from_derive_input(&input)?;

    // Extract information
    let struct_name = &options.ident;
    let (impl_generics, ty_generics, where_clause) = options.generics.split_for_impl();
    let schema_id = &options.schema_id;

    // Extract fields
    let fields = match &options.data {
        Data::Struct(fields) => fields,
        _ => {
            return Err(darling::Error::custom(
                "GermanicSchema can only be applied to structs with named fields",
            ));
        }
    };

    // Generate code for the three traits
    let validations = generate_validations(&fields.fields);
    let default_fields = generate_default_fields(&fields.fields);

    // Combine everything
    let expanded = quote! {
        // ════════════════════════════════════════════════════════════════════
        // GENERATED CODE - DO NOT EDIT MANUALLY
        // ════════════════════════════════════════════════════════════════════

        impl #impl_generics ::germanic::schema::SchemaMetadata for #struct_name #ty_generics
        #where_clause
        {
            fn schema_id(&self) -> &'static str {
                #schema_id
            }

            fn schema_version(&self) -> u8 {
                1
            }
        }

        impl #impl_generics ::germanic::schema::Validate for #struct_name #ty_generics
        #where_clause
        {
            fn validate(&self) -> ::std::result::Result<(), ::germanic::error::ValidationError> {
                let mut errors = Vec::new();
                #validations
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(::germanic::error::ValidationError::RequiredFieldsMissing(errors))
                }
            }
        }

        impl #impl_generics ::std::default::Default for #struct_name #ty_generics
        #where_clause
        {
            fn default() -> Self {
                Self {
                    #default_fields
                }
            }
        }
    };

    Ok(expanded.into())
}

// ============================================================================
// CODE GENERATION: VALIDATION
// ============================================================================

/// Generates validation code for all fields.
///
/// Logic:
/// - required String/Vec/Option → check for empty/None
/// - Nested Structs (Other) → call validate() recursively
fn generate_validations(fields: &[FieldOptions]) -> TokenStream2 {
    let mut validations = Vec::new();

    for field in fields {
        let Some(field_name) = field.ident.as_ref() else {
            continue;
        };
        let field_name_str = field_name.to_string();
        let ty = type_category(&field.ty);

        // 1. Required validation for primitive types
        if field.required.is_present() {
            let validation = match ty {
                TypeCategory::String => Some(quote! {
                    if self.#field_name.is_empty() {
                        errors.push(#field_name_str.to_string());
                    }
                }),
                TypeCategory::Option => Some(quote! {
                    if self.#field_name.is_none() {
                        errors.push(#field_name_str.to_string());
                    }
                }),
                TypeCategory::Vec => Some(quote! {
                    if self.#field_name.is_empty() {
                        errors.push(#field_name_str.to_string());
                    }
                }),
                // Bool always has a value
                TypeCategory::Bool => None,
                // Nested Structs are handled separately
                TypeCategory::Other => None,
            };

            if let Some(v) = validation {
                validations.push(v);
            }
        }

        // 2. Recursive validation for Nested Structs
        //    (independent of required - the nested struct has its own required fields)
        if ty == TypeCategory::Other {
            validations.push(quote! {
                // Recursive validation of nested struct
                if let Err(nested_error) = self.#field_name.validate() {
                    // Add prefix for better error messages
                    if let ::germanic::error::ValidationError::RequiredFieldsMissing(nested_fields) = nested_error {
                        for f in nested_fields {
                            errors.push(format!("{}.{}", #field_name_str, f));
                        }
                    }
                }
            });
        }
    }

    quote! { #(#validations)* }
}

// ============================================================================
// CODE GENERATION: DEFAULT
// ============================================================================

/// Generates default values for all fields.
fn generate_default_fields(fields: &[FieldOptions]) -> TokenStream2 {
    let default_assignments: Vec<TokenStream2> = fields
        .iter()
        .filter_map(|field| {
            let field_name = field.ident.as_ref()?;
            let default_value = generate_default_value(field);
            Some(quote! { #field_name: #default_value, })
        })
        .collect();

    quote! { #(#default_assignments)* }
}

/// Generates the default value for a single field.
///
/// Logic:
/// 1. If `#[germanic(default = "...")]` is set → parse and use
/// 2. Otherwise → type-specific default
fn generate_default_value(field: &FieldOptions) -> TokenStream2 {
    let ty = type_category(&field.ty);

    match (&field.default, ty) {
        // Explicit default for String: #[germanic(default = "DE")]
        (Some(value), TypeCategory::String) => {
            quote! { #value.to_string() }
        }

        // Explicit default for bool: #[germanic(default = "true")] or "false"
        (Some(value), TypeCategory::Bool) => {
            let bool_value: bool = value.parse().unwrap_or(false);
            quote! { #bool_value }
        }

        // Explicit default for Option: #[germanic(default = "value")]
        (Some(value), TypeCategory::Option) => {
            quote! { Some(#value.to_string()) }
        }

        // Explicit default for Vec: not supported, use empty
        (Some(_), TypeCategory::Vec) => {
            quote! { Vec::new() }
        }

        // Explicit default for other types: try Default::default()
        (Some(_), TypeCategory::Other) => {
            quote! { Default::default() }
        }

        // No explicit default → type-specific defaults
        (None, TypeCategory::String) => quote! { String::new() },
        (None, TypeCategory::Bool) => quote! { false },
        (None, TypeCategory::Option) => quote! { None },
        (None, TypeCategory::Vec) => quote! { Vec::new() },
        (None, TypeCategory::Other) => quote! { Default::default() },
    }
}

// ============================================================================
// TYPE CATEGORIZATION
// ============================================================================

/// Categories for Rust types for validation and default logic.
#[derive(Debug, Clone, Copy, PartialEq)]
enum TypeCategory {
    String,
    Bool,
    Option,
    Vec,
    Other,
}

/// Analyzes a type and determines its category.
fn type_category(ty: &Type) -> TypeCategory {
    let ty_string = quote!(#ty).to_string();

    if ty_string == "String" || ty_string.contains("& str") {
        TypeCategory::String
    } else if ty_string == "bool" {
        TypeCategory::Bool
    } else if ty_string.starts_with("Option <") || ty_string.starts_with("Option<") {
        TypeCategory::Option
    } else if ty_string.starts_with("Vec <") || ty_string.starts_with("Vec<") {
        TypeCategory::Vec
    } else {
        TypeCategory::Other
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_category_string() {
        let ty: Type = syn::parse_quote!(String);
        assert_eq!(type_category(&ty), TypeCategory::String);
    }

    #[test]
    fn test_type_category_bool() {
        let ty: Type = syn::parse_quote!(bool);
        assert_eq!(type_category(&ty), TypeCategory::Bool);
    }

    #[test]
    fn test_type_category_option() {
        let ty: Type = syn::parse_quote!(Option<String>);
        assert_eq!(type_category(&ty), TypeCategory::Option);
    }

    #[test]
    fn test_type_category_vec() {
        let ty: Type = syn::parse_quote!(Vec<String>);
        assert_eq!(type_category(&ty), TypeCategory::Vec);
    }

    #[test]
    fn test_type_category_i32() {
        let ty: Type = syn::parse_quote!(i32);
        assert_eq!(type_category(&ty), TypeCategory::Other);
    }
}
