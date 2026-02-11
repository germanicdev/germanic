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
/// Generiert drei Trait-Implementierungen:
/// 1. `SchemaMetadaten` – Schema-ID und Version
/// 2. `Validieren` – Pflichtfeld-Prüfung
/// 3. `Default` – Standardwerte für alle Felder
pub fn implementiere_germanic_schema(eingabe: DeriveInput) -> Result<TokenStream, darling::Error> {
    // Parse Attribute mit darling
    let optionen = SchemaOptionen::from_derive_input(&eingabe)?;

    // Extrahiere Informationen
    let struct_name = &optionen.ident;
    let (impl_generics, ty_generics, where_clause) = optionen.generics.split_for_impl();
    let schema_id = &optionen.schema_id;

    // Extrahiere Felder
    let felder = match &optionen.data {
        Data::Struct(felder) => felder,
        _ => {
            return Err(darling::Error::custom(
                "GermanicSchema kann nur auf Structs mit benannten Feldern angewendet werden",
            ));
        }
    };

    // Generiere Code für die drei Traits
    let validierungen = generiere_validierungen(&felder.fields);
    let default_felder = generiere_default_felder(&felder.fields);

    // Kombiniere alles
    let expandiert = quote! {
        // ════════════════════════════════════════════════════════════════════
        // GENERIERTER CODE - NICHT MANUELL BEARBEITEN
        // ════════════════════════════════════════════════════════════════════

        impl #impl_generics ::germanic::schema::SchemaMetadaten for #struct_name #ty_generics
        #where_clause
        {
            fn schema_id(&self) -> &'static str {
                #schema_id
            }

            fn schema_version(&self) -> u8 {
                1
            }
        }

        impl #impl_generics ::germanic::schema::Validieren for #struct_name #ty_generics
        #where_clause
        {
            fn validiere(&self) -> ::std::result::Result<(), ::germanic::fehler::ValidierungsFehler> {
                let mut fehler = Vec::new();
                #validierungen
                if fehler.is_empty() {
                    Ok(())
                } else {
                    Err(::germanic::fehler::ValidierungsFehler::PflichtfelderFehlen(fehler))
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

/// Generiert Validierungscode für alle Felder.
///
/// Logik:
/// - required String/Vec/Option → prüfe auf leer/None
/// - Nested Structs (Andere) → rufe rekursiv validiere() auf
fn generiere_validierungen(felder: &[FeldOptionen]) -> TokenStream2 {
    let mut validierungen = Vec::new();

    for feld in felder {
        let Some(feld_name) = feld.ident.as_ref() else {
            continue;
        };
        let feld_name_str = feld_name.to_string();
        let typ = typ_kategorie(&feld.ty);

        // 1. Required-Validierung für primitive Typen
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
                // Bool hat immer einen Wert
                TypKategorie::Bool => None,
                // Nested Structs werden separat behandelt
                TypKategorie::Andere => None,
            };

            if let Some(v) = validierung {
                validierungen.push(v);
            }
        }

        // 2. Rekursive Validierung für Nested Structs
        //    (unabhängig von required - der Nested Struct hat eigene required-Felder)
        if typ == TypKategorie::Andere {
            validierungen.push(quote! {
                // Rekursive Validierung des Nested Structs
                if let Err(nested_fehler) = self.#feld_name.validiere() {
                    // Präfix hinzufügen für bessere Fehlermeldungen
                    if let ::germanic::fehler::ValidierungsFehler::PflichtfelderFehlen(nested_felder) = nested_fehler {
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

/// Generiert Default-Werte für alle Felder.
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

/// Generiert den Default-Wert für ein einzelnes Feld.
///
/// Logik:
/// 1. Wenn `#[germanic(default = "...")]` gesetzt → parse und verwende
/// 2. Sonst → typ-spezifischer Default
fn generiere_default_wert(feld: &FeldOptionen) -> TokenStream2 {
    let typ = typ_kategorie(&feld.ty);

    match (&feld.default, typ) {
        // Expliziter Default für String: #[germanic(default = "DE")]
        (Some(wert), TypKategorie::String) => {
            quote! { #wert.to_string() }
        }

        // Expliziter Default für bool: #[germanic(default = "true")] oder "false"
        (Some(wert), TypKategorie::Bool) => {
            let bool_wert: bool = wert.parse().unwrap_or(false);
            quote! { #bool_wert }
        }

        // Expliziter Default für Option: #[germanic(default = "wert")]
        (Some(wert), TypKategorie::Option) => {
            quote! { Some(#wert.to_string()) }
        }

        // Expliziter Default für Vec: nicht unterstützt, verwende leer
        (Some(_), TypKategorie::Vec) => {
            quote! { Vec::new() }
        }

        // Expliziter Default für andere Typen: versuche Default::default()
        (Some(_), TypKategorie::Andere) => {
            quote! { Default::default() }
        }

        // Kein expliziter Default → typ-spezifische Defaults
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

/// Kategorien für Rust-Typen zur Validierungs- und Default-Logik.
#[derive(Debug, Clone, Copy, PartialEq)]
enum TypKategorie {
    String,
    Bool,
    Option,
    Vec,
    Andere,
}

/// Analysiert einen Typ und bestimmt seine Kategorie.
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
