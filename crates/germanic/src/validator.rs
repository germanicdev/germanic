//! # Schema-Validierung
//!
//! Validiert .grm Dateien und JSON-Daten gegen Schemas.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                    VALIDIERUNGS-EBENEN                                      │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   Ebene 1: SYNTAX                                                           │
//! │   ┌─────────────────────────────────────────┐                               │
//! │   │ • Ist das JSON syntaktisch korrekt?     │                               │
//! │   │ • Hat die .grm Datei gültige Magic Bytes?│                               │
//! │   └─────────────────────────────────────────┘                               │
//! │                      │                                                      │
//! │                      ▼                                                      │
//! │   Ebene 2: STRUKTUR                                                         │
//! │   ┌─────────────────────────────────────────┐                               │
//! │   │ • Entspricht JSON dem Rust-Struct?      │                               │
//! │   │ • Ist der .grm Header vollständig?       │                               │
//! │   └─────────────────────────────────────────┘                               │
//! │                      │                                                      │
//! │                      ▼                                                      │
//! │   Ebene 3: SEMANTIK                                                         │
//! │   ┌─────────────────────────────────────────┐                               │
//! │   │ • Sind alle Pflichtfelder ausgefüllt?   │                               │
//! │   │ • Erfüllen Werte Business-Constraints?  │                               │
//! │   └─────────────────────────────────────────┘                               │
//! │                                                                             │
//! │   FAIL-FAST: Jede Ebene bricht bei Fehler sofort ab.                        │
//! │   Kein Sinn, semantische Prüfung bei Syntaxfehler zu machen.                │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```

use crate::error::GermanicResult;
use crate::types::{GrmHeader, GRM_MAGIC};

// ============================================================================
// .GRM VALIDIERUNG
// ============================================================================

/// Validiert eine .grm Datei auf strukturelle Korrektheit.
///
/// ## Prüfungen
///
/// 1. Magic Bytes vorhanden und korrekt
/// 2. Header vollständig und parsbar
/// 3. Schema-ID ist gültiges UTF-8
/// 4. Genug Daten für den angegebenen Payload
///
/// ## Beispiel
///
/// ```rust,ignore
/// let bytes = std::fs::read("praxis.grm")?;
/// let validierung = validiere_grm(&bytes)?;
/// println!("Schema-ID: {}", validierung.schema_id);
/// ```
pub fn validiere_grm(daten: &[u8]) -> GermanicResult<GrmValidierung> {
    // 1. Mindestgröße prüfen
    if daten.len() < 4 {
        return Ok(GrmValidierung {
            gueltig: false,
            schema_id: None,
            fehler: Some("Datei zu kurz für Magic Bytes".to_string()),
        });
    }

    // 2. Magic Bytes prüfen
    if &daten[0..4] != &GRM_MAGIC {
        return Ok(GrmValidierung {
            gueltig: false,
            schema_id: None,
            fehler: Some(format!(
                "Ungültige Magic Bytes: {:02X?} (erwartet: {:02X?})",
                &daten[0..4],
                &GRM_MAGIC
            )),
        });
    }

    // 3. Header parsen
    match GrmHeader::from_bytes(daten) {
        Ok((header, _laenge)) => Ok(GrmValidierung {
            gueltig: true,
            schema_id: Some(header.schema_id),
            fehler: None,
        }),
        Err(e) => Ok(GrmValidierung {
            gueltig: false,
            schema_id: None,
            fehler: Some(format!("Header-Fehler: {}", e)),
        }),
    }
}

/// Ergebnis der .grm Validierung.
#[derive(Debug, Clone)]
pub struct GrmValidierung {
    /// Ist die Datei strukturell gültig?
    pub gueltig: bool,

    /// Extrahierte Schema-ID (wenn Header parsbar)
    pub schema_id: Option<String>,

    /// Fehlermeldung (wenn ungültig)
    pub fehler: Option<String>,
}

// ============================================================================
// JSON-SCHEMA VALIDIERUNG
// ============================================================================

/// Validiert JSON gegen ein bekanntes Schema.
///
/// Diese Funktion ist ein Wrapper für die Schema-spezifische Validierung.
/// Die eigentliche Validierungslogik wird vom `Validieren` Trait bereitgestellt,
/// der durch das Macro generiert wird.
///
/// ## Beispiel
///
/// ```rust,ignore
/// let json = r#"{"name": "", "bezeichnung": "Heilpraktiker"}"#;
/// let result = validiere_json::<PraxisSchema>(json);
/// // → Err: "name" ist leer aber required
/// ```
pub fn validiere_json<S>(json: &str) -> GermanicResult<S>
where
    S: serde::de::DeserializeOwned + crate::schema::Validieren,
{
    // 1. Parse JSON zu Struct
    let schema: S = serde_json::from_str(json)?;

    // 2. Validiere Pflichtfelder
    schema.validiere()?;

    Ok(schema)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validiere_grm_zu_kurz() {
        let daten = [0x47, 0x52, 0x4D]; // Nur 3 Bytes
        let ergebnis = validiere_grm(&daten).unwrap();

        assert!(!ergebnis.gueltig);
        assert!(ergebnis.fehler.unwrap().contains("zu kurz"));
    }

    #[test]
    fn test_validiere_grm_falsche_magic() {
        let daten = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let ergebnis = validiere_grm(&daten).unwrap();

        assert!(!ergebnis.gueltig);
        assert!(ergebnis.fehler.unwrap().contains("Magic"));
    }

    #[test]
    fn test_validiere_grm_gueltig() {
        let header = GrmHeader::new("test.v1");
        let bytes = header.to_bytes();
        let ergebnis = validiere_grm(&bytes).unwrap();

        assert!(ergebnis.gueltig);
        assert_eq!(ergebnis.schema_id, Some("test.v1".to_string()));
    }
}
