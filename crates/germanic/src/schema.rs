//! # Schema-Traits
//!
//! Definiert die Verträge (Traits), die das Macro implementiert.
//!
//! ## Architektur: Warum Traits?
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                    TRAIT-BASIERTE ABSTRAKTION                               │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   PROBLEM:                                                                  │
//! │   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
//! │   │  PraxisSchema   │  │  RestaurantSchema│  │   HotelSchema   │            │
//! │   └─────────────────┘  └─────────────────┘  └─────────────────┘            │
//! │          ↓                    ↓                    ↓                        │
//! │   Wie behandelt der Compiler all diese Typen einheitlich?                   │
//! │                                                                             │
//! │   LÖSUNG: Gemeinsamer Vertrag (Trait)                                       │
//! │   ┌─────────────────────────────────────────────────────────────┐           │
//! │   │                  trait Validieren                           │           │
//! │   │   fn validiere(&self) -> Result<(), ValidierungsFehler>     │           │
//! │   └─────────────────────────────────────────────────────────────┘           │
//! │          ↑                    ↑                    ↑                        │
//! │   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
//! │   │  PraxisSchema   │  │  RestaurantSchema│  │   HotelSchema   │            │
//! │   │ impl Validieren │  │ impl Validieren │  │ impl Validieren │            │
//! │   └─────────────────┘  └─────────────────┘  └─────────────────┘            │
//! │                                                                             │
//! │   Compiler kann jetzt mit `dyn Validieren` oder Generics arbeiten           │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```

use crate::error::ValidationError;

// ============================================================================
// SCHEMA-METADATEN
// ============================================================================

/// Trait für Schema-Metadaten.
///
/// Wird vom `#[derive(GermanicSchema)]` Macro automatisch implementiert.
///
/// ## Verwendung
///
/// ```rust,ignore
/// use germanic::schema::SchemaMetadaten;
///
/// let praxis = PraxisSchema { /* ... */ };
/// println!("Schema-ID: {}", praxis.schema_id());  // "de.gesundheit.praxis.v1"
/// ```
///
/// ## Architektonische Bedeutung
///
/// Die Schema-ID wird in den .grm Header geschrieben und ermöglicht:
/// - KI-Systeme können das Schema identifizieren
/// - Versionierung für Rückwärtskompatibilität
/// - Registry-Lookup für Schema-Definitionen
pub trait SchemaMetadaten {
    /// Die eindeutige Schema-ID.
    ///
    /// Format: `"{namespace}.{domain}.{name}.v{version}"`
    /// Beispiel: `"de.gesundheit.praxis.v1"`
    fn schema_id(&self) -> &'static str;

    /// Die Schema-Version (1-255).
    ///
    /// Wird für Migrations-Logik verwendet.
    fn schema_version(&self) -> u8;
}

// ============================================================================
// VALIDIERUNG
// ============================================================================

/// Trait für Schema-Validierung.
///
/// Prüft, ob alle Pflichtfelder (`#[germanic(required)]`) ausgefüllt sind.
///
/// ## Beispiel
///
/// ```rust,ignore
/// use germanic::schema::Validieren;
///
/// let praxis = PraxisSchema {
///     name: "".to_string(),  // LEER! → Fehler
///     bezeichnung: "Heilpraktiker".to_string(),
///     // ...
/// };
///
/// match praxis.validiere() {
///     Ok(()) => println!("Alles in Ordnung"),
///     Err(e) => eprintln!("Validierung fehlgeschlagen: {}", e),
/// }
/// ```
///
/// ## Architektonische Bedeutung
///
/// Validierung passiert **vor** der FlatBuffer-Serialisierung.
/// Das garantiert:
/// - Frühes Fehlschlagen (fail fast)
/// - Keine korrupten .grm Dateien
/// - Aussagekräftige Fehlermeldungen für den Nutzer
pub trait Validieren {
    /// Validiert das Schema.
    ///
    /// # Rückgabe
    ///
    /// - `Ok(())` wenn alle Pflichtfelder ausgefüllt sind
    /// - `Err(ValidationError)` mit Liste der fehlenden Felder
    fn validiere(&self) -> Result<(), ValidationError>;
}

// ============================================================================
// SERIALISIERUNG (Platzhalter für später)
// ============================================================================

/// Trait für FlatBuffer-Serialisierung.
///
/// **Noch nicht implementiert** – kommt in Phase 3 der Macro-Entwicklung.
///
/// ## Geplante Signatur
///
/// ```rust,ignore
/// pub trait GermanicSerialisieren {
///     /// Serialisiert das Schema in FlatBuffer-Bytes.
///     fn serialisiere(&self, builder: &mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset>;
/// }
/// ```
pub trait GermanicSerialisieren {
    /// Serialisiert das Schema in einen Byte-Vektor.
    fn zu_bytes(&self) -> Vec<u8>;
}

// ============================================================================
// KOMPOSITIONS-TRAIT
// ============================================================================

/// Marker-Trait für vollständige GERMANIC-Schemas.
///
/// Ein Typ implementiert `GermanicSchemaVollstaendig` wenn er alle
/// notwendigen Traits implementiert.
///
/// ## Automatische Implementierung
///
/// ```rust,ignore
/// // Automatisch für jeden Typ, der alle Traits implementiert:
/// impl<T> GermanicSchemaVollstaendig for T
/// where
///     T: SchemaMetadaten + Validieren + GermanicSerialisieren
/// {}
/// ```
pub trait GermanicSchemaVollstaendig: SchemaMetadaten + Validieren {}

// Blanket Implementation: Jeder Typ, der alle Traits hat, ist automatisch vollständig
impl<T> GermanicSchemaVollstaendig for T where T: SchemaMetadaten + Validieren {}
