//! # Fehlertypen
//!
//! Definiert alle Fehler, die in GERMANIC auftreten können.
//!
//! ## Architektur: Fehler als Typen
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                    FEHLER-HIERARCHIE                                        │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │                      GermanicFehler                                         │
//! │                           │                                                 │
//! │       ┌───────────────────┼───────────────────┐                             │
//! │       │                   │                   │                             │
//! │       ▼                   ▼                   ▼                             │
//! │  Validierung        Serialisierung       Kompilierung                       │
//! │       │                   │                   │                             │
//! │       ▼                   ▼                   ▼                             │
//! │  PflichtfelderFehlen  FlatBufferFehler  DateiFehler                         │
//! │  TypFehler            SignaturFehler    SchemaFehler                        │
//! │                                                                             │
//! │  PRINZIP: Jeder Fehler hat einen eigenen Typ mit spezifischen Daten         │
//! │           Keine String-basierten Fehlermeldungen!                           │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Warum `thiserror`?
//!
//! `thiserror` generiert automatisch:
//! - `std::error::Error` Implementation
//! - `Display` Implementation (für Fehlermeldungen)
//! - `From` Implementations (für `?` Operator)

use thiserror::Error;

// ============================================================================
// HAUPT-FEHLERTYP
// ============================================================================

/// Hauptfehlertyp für alle GERMANIC-Operationen.
///
/// ## Verwendung
///
/// ```rust,ignore
/// use germanic::fehler::GermanicFehler;
///
/// fn kompiliere(json: &str) -> Result<Vec<u8>, GermanicFehler> {
///     let praxis: PraxisSchema = serde_json::from_str(json)?;  // → JsonFehler
///     praxis.validiere()?;  // → Validierung
///     // ...
/// }
/// ```
#[derive(Error, Debug)]
pub enum GermanicFehler {
    /// Validierungsfehler (Pflichtfelder, Typen)
    #[error("Validierung fehlgeschlagen: {0}")]
    Validierung(#[from] ValidierungsFehler),

    /// JSON-Parsing-Fehler
    #[error("JSON-Fehler: {0}")]
    Json(#[from] serde_json::Error),

    /// Dateisystem-Fehler
    #[error("IO-Fehler: {0}")]
    Io(#[from] std::io::Error),

    /// Schema nicht gefunden
    #[error("Unbekanntes Schema: {0}")]
    UnbekanntesSchema(String),

    /// Allgemeiner Fehler mit Nachricht
    #[error("{0}")]
    Allgemein(String),
}

// ============================================================================
// VALIDIERUNGS-FEHLER
// ============================================================================

/// Fehler bei der Schema-Validierung.
///
/// ## Beispiel
///
/// ```rust,ignore
/// match praxis.validiere() {
///     Err(ValidierungsFehler::PflichtfelderFehlen(felder)) => {
///         eprintln!("Fehlende Felder: {:?}", felder);
///         // → "Fehlende Felder: ["name", "adresse"]"
///     }
///     _ => {}
/// }
/// ```
#[derive(Error, Debug, Clone)]
pub enum ValidierungsFehler {
    /// Pflichtfelder sind leer oder nicht vorhanden.
    #[error("Pflichtfelder fehlen: {}", felder_liste(.0))]
    PflichtfelderFehlen(Vec<String>),

    /// Feldwert hat falschen Typ.
    #[error("Typfehler in Feld '{feld}': erwartet {erwartet}, gefunden {gefunden}")]
    TypFehler {
        feld: String,
        erwartet: String,
        gefunden: String,
    },

    /// Feldwert verletzt Constraints.
    #[error("Constraint-Verletzung in Feld '{feld}': {nachricht}")]
    ConstraintVerletzung { feld: String, nachricht: String },
}

/// Hilfsfunktion: Formatiert Feldliste als komma-separierte Zeichenkette.
fn felder_liste(felder: &[String]) -> String {
    if felder.is_empty() {
        "(keine)".to_string()
    } else {
        felder.join(", ")
    }
}

// ============================================================================
// KOMPILIERUNGS-FEHLER
// ============================================================================

/// Fehler bei der Kompilierung zu .grm.
#[derive(Error, Debug)]
pub enum KompilierungsFehler {
    /// Input-Datei nicht gefunden.
    #[error("Eingabedatei nicht gefunden: {pfad}")]
    DateiNichtGefunden { pfad: String },

    /// Ausgabe konnte nicht geschrieben werden.
    #[error("Ausgabe-Fehler: {nachricht}")]
    AusgabeFehler { nachricht: String },

    /// FlatBuffer-Serialisierung fehlgeschlagen.
    #[error("Serialisierung fehlgeschlagen: {nachricht}")]
    SerialisierungsFehler { nachricht: String },
}

// ============================================================================
// RESULT TYPE ALIAS
// ============================================================================

/// Praktischer Alias für GERMANIC-Operationen.
///
/// ```rust,ignore
/// fn meine_funktion() -> GermanicResult<Vec<u8>> {
///     // ...
/// }
/// ```
pub type GermanicResult<T> = Result<T, GermanicFehler>;

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pflichtfelder_fehlen_anzeige() {
        let fehler =
            ValidierungsFehler::PflichtfelderFehlen(vec!["name".into(), "adresse".into()]);

        assert_eq!(
            fehler.to_string(),
            "Pflichtfelder fehlen: name, adresse"
        );
    }

    #[test]
    fn test_leere_pflichtfelder() {
        let fehler = ValidierungsFehler::PflichtfelderFehlen(vec![]);

        assert_eq!(fehler.to_string(), "Pflichtfelder fehlen: (keine)");
    }

    #[test]
    fn test_fehler_konvertierung() {
        let validierung_fehler =
            ValidierungsFehler::PflichtfelderFehlen(vec!["name".into()]);

        let germanic_fehler: GermanicFehler = validierung_fehler.into();

        assert!(matches!(germanic_fehler, GermanicFehler::Validierung(_)));
    }
}
