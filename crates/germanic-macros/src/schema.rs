//! # Schema-Macro Implementierung
//!
//! Verwendet `darling` für deklaratives Attribut-Parsing.
//!
//! ## Architektur: Warum darling?
//!
//! **Problem ohne darling:**
//! ```text
//! Manuelles Parsing:
//!   for attr in &input.attrs {
//!       if attr.path().is_ident("germanic") {
//!           attr.parse_nested_meta(|meta| {
//!               if meta.path.is_ident("schema_id") {
//!                   // 50+ Zeilen Boilerplate pro Attribut
//!               }
//!           })?;
//!       }
//!   }
//! ```
//!
//! **Mit darling:**
//! ```text
//! #[derive(FromDeriveInput)]
//! #[darling(attributes(germanic))]
//! struct SchemaOpts {
//!     schema_id: String,  // Automatisch geparst!
//! }
//! # Schema-Macro Implementierung
//!
//! Verwendet `darling` für deklaratives Attribut-Parsing.
//!
//! ## Generierte Traits
//!
//! - `SchemaMetadaten` → schema_id(), schema_version()
//! - `Validieren` → validiere()
//! - `Default` → default()

use darling::{FromDeriveInput, FromField, ast::Data, util::Flag};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Ident, Type};

// ============================================================================
// DATENSTRUKTUREN FÜR ATTRIBUT-PARSING (darling)
// ============================================================================

/// Optionen auf Struct-Ebene.
///
/// ```rust,ignore
/// #[germanic(schema_id = "de.gesundheit.praxis.v1")]
/// ```
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(germanic), supports(struct_named))]
pub struct SchemaOptionen {
    /// Name des Structs
    ident: Ident,
    /// Generics
    generics: syn::Generics,
    /// Felder des Structs
    data: Data<(), FeldOptionen>,
    /// Eindeutige Schema-ID (Pflicht)
    schema_id: String,
    /// Pfad zum FlatBuffer-Typ (optional, für später)
    #[darling(default)]
    flatbuffer: Option<String>,
}

/// Optionen auf Feld-Ebene.
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
pub struct FeldOptionen {
    /// Feldname
    ident: Option<Ident>,
    /// Feldtyp
    ty: Type,
    /// Pflichtfeld-Flag
    #[darling(default)]
    required: Flag,
    /// Default-Wert als String (z.B. "DE", "true", "false")
    #[darling(default)]
    default: Option<String>,
}

// ============================================================================
// HAUPT-IMPLEMENTIERUNG
// ============================================================================

/// Einstiegspunkt für die Macro-Expansion.
///
/// Generates three trait implementations:
/// 1. `SchemaMetadata` – Schema ID and version
/// 2. `Validate` – Required field validation
/// 3. `Default` – Default values for all fields
pub fn implement_germanic_schema(input: DeriveInput) -> Result<TokenStream, darling::Error> {
    // Parse attributes with darling
    let optionen = SchemaOptionen::from_derive_input(&input)?;

    // Extract information
    let struct_name = &optionen.ident;
    let (impl_generics, ty_generics, where_clause) = optionen.generics.split_for_impl();
    let schema_id = &optionen.schema_id;

    // Extract fields
    let felder = match &optionen.data {
        Data::Struct(felder) => felder,
        _ => {
            return Err(darling::Error::custom(
                "GermanicSchema kann nur auf Structs mit benannten Feldern angewendet werden",
            ));
        }
    };

    // Generate code for the three traits
    let validierungen = generiere_validierungen(&felder.fields);
    let default_felder = generiere_default_felder(&felder.fields);

    // Combine everything
    let expandiert = quote! {
        // ════════════════════════════════════════════════════════════════════
        // GENERIERTER CODE - NICHT MANUELL BEARBEITEN
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
                let mut fehler = Vec::new();
                #validierungen
                if fehler.is_empty() {
                    Ok(())
                } else {
                    Err(::germanic::error::ValidationError::RequiredFieldsMissing(fehler))
                }
            }
        }

        impl #impl_generics ::std::default::Default for #struct_name #ty_generics
        #where_clause
        {
            fn default() -> Self {
                Self {
                    #default_felder
                }
            }
        }
    };

    Ok(expandiert.into())
}

// ============================================================================
// CODE-GENERIERUNG: VALIDIERUNG
// ============================================================================

/// Generates validation code for all fields.
///
/// Logic:
/// - required String/Vec/Option → check for empty/None
/// - Nested Structs (Other) → call validate() recursively
fn generiere_validierungen(felder: &[FeldOptionen]) -> TokenStream2 {
    let mut validierungen = Vec::new();

    for feld in felder {
        let Some(feld_name) = feld.ident.as_ref() else {
            continue;
        };
        let feld_name_str = feld_name.to_string();
        let typ = typ_kategorie(&feld.ty);

        // 1. Required validation for primitive types
        if feld.required.is_present() {
            let validierung = match typ {
                TypKategorie::String => Some(quote! {
                    if self.#feld_name.is_empty() {
                        fehler.push(#feld_name_str.to_string());
                    }
                }),
                TypKategorie::Option => Some(quote! {
                    if self.#feld_name.is_none() {
                        fehler.push(#feld_name_str.to_string());
                    }
                }),
                TypKategorie::Vec => Some(quote! {
                    if self.#feld_name.is_empty() {
                        fehler.push(#feld_name_str.to_string());
                    }
                }),
                // Bool always has a value
                TypKategorie::Bool => None,
                // Nested Structs are handled separately
                TypKategorie::Andere => None,
            };

            if let Some(v) = validierung {
                validierungen.push(v);
            }
        }

        // 2. Recursive validation for Nested Structs
        //    (independent of required - the nested struct has its own required fields)
        if typ == TypKategorie::Andere {
            validierungen.push(quote! {
                // Recursive validation of nested struct
                if let Err(nested_fehler) = self.#feld_name.validate() {
                    // Add prefix for better error messages
                    if let ::germanic::error::ValidationError::RequiredFieldsMissing(nested_felder) = nested_fehler {
                        for f in nested_felder {
                            fehler.push(format!("{}.{}", #feld_name_str, f));
                        }
                    }
                }
            });
        }
    }

    quote! { #(#validierungen)* }
}

// ============================================================================
// CODE-GENERIERUNG: DEFAULT
// ============================================================================

/// Generates default values for all fields.
fn generiere_default_felder(felder: &[FeldOptionen]) -> TokenStream2 {
    let default_zuweisungen: Vec<TokenStream2> = felder
        .iter()
        .filter_map(|feld| {
            let feld_name = feld.ident.as_ref()?;
            let default_wert = generiere_default_wert(feld);
            Some(quote! { #feld_name: #default_wert, })
        })
        .collect();

    quote! { #(#default_zuweisungen)* }
}

/// Generates the default value for a single field.
///
/// Logic:
/// 1. If `#[germanic(default = "...")]` is set → parse and use
/// 2. Otherwise → type-specific default
fn generiere_default_wert(feld: &FeldOptionen) -> TokenStream2 {
    let typ = typ_kategorie(&feld.ty);

    match (&feld.default, typ) {
        // Explicit default for String: #[germanic(default = "DE")]
        (Some(wert), TypKategorie::String) => {
            quote! { #wert.to_string() }
        }

        // Explicit default for bool: #[germanic(default = "true")] or "false"
        (Some(wert), TypKategorie::Bool) => {
            let bool_wert: bool = wert.parse().unwrap_or(false);
            quote! { #bool_wert }
        }

        // Explicit default for Option: #[germanic(default = "value")]
        (Some(wert), TypKategorie::Option) => {
            quote! { Some(#wert.to_string()) }
        }

        // Explicit default for Vec: not supported, use empty
        (Some(_), TypKategorie::Vec) => {
            quote! { Vec::new() }
        }

        // Explicit default for other types: try Default::default()
        (Some(_), TypKategorie::Andere) => {
            quote! { Default::default() }
        }

        // No explicit default → type-specific defaults
        (None, TypKategorie::String) => quote! { String::new() },
        (None, TypKategorie::Bool) => quote! { false },
        (None, TypKategorie::Option) => quote! { None },
        (None, TypKategorie::Vec) => quote! { Vec::new() },
        (None, TypKategorie::Andere) => quote! { Default::default() },
    }
}

// ============================================================================
// TYP-KATEGORISIERUNG
// ============================================================================

/// Categories for Rust types for validation and default logic.
#[derive(Debug, Clone, Copy, PartialEq)]
enum TypKategorie {
    String,
    Bool,
    Option,
    Vec,
    Andere,
}

/// Analyzes a type and determines its category.
fn typ_kategorie(ty: &Type) -> TypKategorie {
    let ty_string = quote!(#ty).to_string();

    if ty_string == "String" || ty_string.contains("& str") {
        TypKategorie::String
    } else if ty_string == "bool" {
        TypKategorie::Bool
    } else if ty_string.starts_with("Option <") || ty_string.starts_with("Option<") {
        TypKategorie::Option
    } else if ty_string.starts_with("Vec <") || ty_string.starts_with("Vec<") {
        TypKategorie::Vec
    } else {
        TypKategorie::Andere
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typ_kategorie_string() {
        let ty: Type = syn::parse_quote!(String);
        assert_eq!(typ_kategorie(&ty), TypKategorie::String);
    }

    #[test]
    fn test_typ_kategorie_bool() {
        let ty: Type = syn::parse_quote!(bool);
        assert_eq!(typ_kategorie(&ty), TypKategorie::Bool);
    }

    #[test]
    fn test_typ_kategorie_option() {
        let ty: Type = syn::parse_quote!(Option<String>);
        assert_eq!(typ_kategorie(&ty), TypKategorie::Option);
    }

    #[test]
    fn test_typ_kategorie_vec() {
        let ty: Type = syn::parse_quote!(Vec<String>);
        assert_eq!(typ_kategorie(&ty), TypKategorie::Vec);
    }

    #[test]
    fn test_typ_kategorie_i32() {
        let ty: Type = syn::parse_quote!(i32);
        assert_eq!(typ_kategorie(&ty), TypKategorie::Andere);
    }
}
