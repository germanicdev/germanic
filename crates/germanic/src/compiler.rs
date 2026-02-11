//! # JSON → .grm Compiler
//!
//! Kompiliert JSON-Daten in das binäre .grm Format.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                    KOMPILIERUNGS-PIPELINE                                   │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   INPUT                         VERARBEITUNG                    OUTPUT      │
//! │   ┌─────────┐                   ┌─────────────┐                ┌─────────┐  │
//! │   │ praxis  │                   │             │                │         │  │
//! │   │  .json  │ ──→ Parse ──→     │ PraxisSchema│ ──→ Serialize  │ .grm    │  │
//! │   │         │                   │             │                │         │  │
//! │   └─────────┘                   └─────────────┘                └─────────┘  │
//! │        │                              │                             │       │
//! │        ▼                              ▼                             ▼       │
//! │   serde_json::from_str          1. validiere()              GrmHeader +     │
//! │                                 2. zu_bytes()               FlatBuffer      │
//! │                                                                             │
//! │   FEHLER-PUNKTE:                                                            │
//! │   1. JSON-Syntax ungültig        → JsonFehler                               │
//! │   2. Schema-Struktur falsch      → DeserializeFehler                        │
//! │   3. Pflichtfelder fehlen        → ValidierungsFehler                       │
//! │   4. IO-Fehler beim Schreiben    → IoFehler                                 │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```

use crate::fehler::{GermanicFehler, GermanicResult};
use crate::schema::{GermanicSerialisieren, SchemaMetadaten, Validieren};
use crate::types::GrmHeader;
use serde::de::DeserializeOwned;
use std::path::Path;

// ============================================================================
// KOMPILIERUNG
// ============================================================================

/// Kompiliert ein Schema-Objekt zu .grm Bytes.
///
/// ## Pipeline
///
/// ```text
/// Schema ──► validiere() ──► zu_bytes() ──► Header + Payload
/// ```
///
/// ## Architektonische Leitfragen:
///
/// 1. **Wer validiert?** Der Compiler, bevor Bytes geschrieben werden.
/// 2. **Was passiert bei Fehlern?** Fail-Fast mit aussagekräftiger Meldung.
/// 3. **Wer besitzt die Daten?** Unveränderliche Leihe (`&schema`).
///
/// ## Beispiel
///
/// ```rust,ignore
/// use germanic::compiler::kompiliere;
/// use germanic::schemas::PraxisSchema;
///
/// let praxis = PraxisSchema {
///     name: "Dr. Maria Sonnenschein".to_string(),
///     bezeichnung: "Zahnärztin".to_string(),
///     // ...
/// };
///
/// let bytes = kompiliere(&praxis)?;
/// std::fs::write("praxis.grm", bytes)?;
/// ```
pub fn kompiliere<S>(schema: &S) -> GermanicResult<Vec<u8>>
where
    S: SchemaMetadaten + Validieren + GermanicSerialisieren,
{
    // 1. Validiere Pflichtfelder
    schema.validiere().map_err(GermanicFehler::Validierung)?;

    // 2. Erstelle Header
    let header = GrmHeader::neu(schema.schema_id());
    let header_bytes = header.zu_bytes();

    // 3. Serialisiere Schema zu FlatBuffer
    let payload_bytes = schema.zu_bytes();

    // 4. Kombiniere Header + Payload
    let mut ausgabe = Vec::with_capacity(header_bytes.len() + payload_bytes.len());
    ausgabe.extend_from_slice(&header_bytes);
    ausgabe.extend_from_slice(&payload_bytes);

    Ok(ausgabe)
}

/// Kompiliert JSON-String zu .grm Bytes.
///
/// Dies ist die Hauptfunktion für den Concierge-Workflow:
/// 1. Plugin exportiert JSON
/// 2. CLI ruft diese Funktion auf
/// 3. .grm wird generiert
///
/// ## Beispiel
///
/// ```rust,ignore
/// use germanic::compiler::kompiliere_json;
/// use germanic::schemas::PraxisSchema;
///
/// let json = std::fs::read_to_string("praxis.json")?;
/// let bytes = kompiliere_json::<PraxisSchema>(&json)?;
/// std::fs::write("praxis.grm", bytes)?;
/// ```
pub fn kompiliere_json<S>(json: &str) -> GermanicResult<Vec<u8>>
where
    S: DeserializeOwned + SchemaMetadaten + Validieren + GermanicSerialisieren,
{
    // 1. Parse JSON zu Rust-Struct
    let schema: S = serde_json::from_str(json)?;

    // 2. Delegiere an kompiliere()
    kompiliere(&schema)
}

/// Kompiliert eine JSON-Datei zu .grm Bytes.
///
/// ## Beispiel
///
/// ```rust,ignore
/// use germanic::compiler::kompiliere_datei;
/// use germanic::schemas::PraxisSchema;
///
/// let bytes = kompiliere_datei::<PraxisSchema>(Path::new("praxis.json"))?;
/// ```
pub fn kompiliere_datei<S>(pfad: &Path) -> GermanicResult<Vec<u8>>
where
    S: DeserializeOwned + SchemaMetadaten + Validieren + GermanicSerialisieren,
{
    let json = std::fs::read_to_string(pfad)?;
    kompiliere_json::<S>(&json)
}

/// Schreibt .grm Bytes in eine Datei.
///
/// ## Beispiel
///
/// ```rust,ignore
/// let bytes = kompiliere(&praxis)?;
/// schreibe_grm(&bytes, Path::new("praxis.grm"))?;
/// ```
pub fn schreibe_grm(daten: &[u8], pfad: &Path) -> GermanicResult<()> {
    std::fs::write(pfad, daten)?;
    Ok(())
}

// ============================================================================
// SCHEMA-REGISTRY (für CLI)
// ============================================================================

/// Bekannte Schema-Typen für die CLI.
///
/// Der CLI-Befehl `germanic compile --schema praxis` braucht
/// eine Zuordnung von String-Namen zu konkreten Typen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaTyp {
    /// Praxis-Schema für Heilpraktiker/Ärzte
    Praxis,
}

impl SchemaTyp {
    /// Parst einen Schema-Namen aus einem String.
    pub fn von_str(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "praxis" => Some(Self::Praxis),
            _ => None,
        }
    }

    /// Gibt den Schema-Namen zurück.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Praxis => "praxis",
        }
    }

    /// Gibt die Schema-ID zurück.
    pub fn schema_id(&self) -> &'static str {
        match self {
            Self::Praxis => "de.gesundheit.praxis.v1",
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::{AdresseSchema, PraxisSchema};

    #[test]
    fn test_schema_typ_parsing() {
        assert_eq!(SchemaTyp::von_str("praxis"), Some(SchemaTyp::Praxis));
        assert_eq!(SchemaTyp::von_str("PRAXIS"), Some(SchemaTyp::Praxis));
        assert_eq!(SchemaTyp::von_str("unknown"), None);
    }

    #[test]
    fn test_kompiliere_praxis() {
        let praxis = PraxisSchema {
            name: "Test".to_string(),
            bezeichnung: "Arzt".to_string(),
            adresse: AdresseSchema {
                strasse: "Teststr.".to_string(),
                hausnummer: None,
                plz: "12345".to_string(),
                ort: "Berlin".to_string(),
                land: "DE".to_string(),
            },
            ..Default::default()
        };

        let bytes = kompiliere(&praxis).expect("Kompilierung sollte funktionieren");

        // Header prüfen (Magic Bytes)
        assert_eq!(&bytes[0..3], b"GRM");

        // Schema-ID im Header prüfen
        let schema_id_len = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
        let schema_id = std::str::from_utf8(&bytes[6..6 + schema_id_len]).unwrap();
        assert_eq!(schema_id, "de.gesundheit.praxis.v1");
    }

    #[test]
    fn test_kompiliere_json_praxis() {
        let json = r#"{
            "name": "Dr. Müller",
            "bezeichnung": "Arzt",
            "adresse": {
                "strasse": "Hauptstraße",
                "plz": "12345",
                "ort": "Berlin"
            }
        }"#;

        let bytes =
            kompiliere_json::<PraxisSchema>(json).expect("Kompilierung sollte funktionieren");

        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..3], b"GRM");
    }

    #[test]
    fn test_kompiliere_validierung_fehler() {
        let praxis = PraxisSchema::default(); // Alle Pflichtfelder leer

        let ergebnis = kompiliere(&praxis);

        assert!(ergebnis.is_err());
        assert!(matches!(ergebnis, Err(GermanicFehler::Validierung(_))));
    }
}
