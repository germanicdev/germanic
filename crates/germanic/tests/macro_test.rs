//! Integration-Tests für das GermanicSchema Macro
//!
//! Testet:
//! - Validieren Trait (required-Felder)
//! - Default Trait (Standardwerte)
//! - SchemaMetadaten Trait (schema_id)

use germanic::GermanicSchema;
use germanic::schema::{SchemaMetadata, Validate};

// ============================================================================
// TEST 1: Validierung von Pflichtfeldern
// ============================================================================

#[derive(GermanicSchema)]
#[germanic(schema_id = "test.validierung.v1")]
pub struct ValidierungTestSchema {
    #[germanic(required)]
    pub name: String,

    pub optional: Option<String>,
}

#[test]
fn test_validierung_name_leer() {
    let schema = ValidierungTestSchema {
        name: "".to_string(),
        optional: None,
    };

    // Leerer required String sollte Fehler werfen
    let ergebnis = schema.validate();
    assert!(ergebnis.is_err());
}

#[test]
fn test_validierung_ok() {
    let schema = ValidierungTestSchema {
        name: "Test".to_string(),
        optional: None,
    };

    assert!(schema.validate().is_ok());
}

// ============================================================================
// TEST 2: Default Trait
// ============================================================================

#[derive(GermanicSchema)]
#[germanic(schema_id = "test.default.v1")]
pub struct DefaultTestSchema {
    #[germanic(default = "Deutschland")]
    pub land: String,

    #[germanic(default = "true")]
    pub aktiv: bool,

    pub name: String, // Kein Default → String::new()

    pub optional: Option<String>, // Kein Default → None

    pub liste: Vec<String>, // Kein Default → Vec::new()
}

#[test]
fn test_default_trait() {
    let schema = DefaultTestSchema::default();

    assert_eq!(schema.land, "Deutschland");
    assert_eq!(schema.aktiv, true);
    assert_eq!(schema.name, "");
    assert!(schema.optional.is_none());
    assert!(schema.liste.is_empty());
}

#[test]
fn test_default_bool_false() {
    #[derive(GermanicSchema)]
    #[germanic(schema_id = "test.bool.v1")]
    pub struct BoolTestSchema {
        #[germanic(default = "false")]
        pub deaktiviert: bool,

        pub ohne_default: bool, // → false
    }

    let schema = BoolTestSchema::default();
    assert_eq!(schema.deaktiviert, false);
    assert_eq!(schema.ohne_default, false);
}

// ============================================================================
// TEST 3: SchemaMetadaten Trait
// ============================================================================

#[test]
fn test_schema_metadaten() {
    let schema = ValidierungTestSchema {
        name: "Test".to_string(),
        optional: None,
    };

    assert_eq!(schema.schema_id(), "test.validierung.v1");
    assert_eq!(schema.schema_version(), 1);
}

// ============================================================================
// TEST 4: Kombinierte Validierung und Default
// ============================================================================

#[derive(GermanicSchema)]
#[germanic(schema_id = "test.kombiniert.v1")]
pub struct KombiniertTestSchema {
    #[germanic(required)]
    pub pflicht: String,

    #[germanic(default = "Standard")]
    pub mit_default: String,

    #[germanic(required)]
    pub pflicht_vec: Vec<String>,
}

#[test]
fn test_default_erfuellt_nicht_required() {
    // Default erzeugt leere Strings/Vecs, die bei required fehlschlagen sollten
    let schema = KombiniertTestSchema::default();

    let ergebnis = schema.validate();
    assert!(ergebnis.is_err());

    // Prüfe welche Felder fehlen
    if let Err(germanic::error::ValidationError::RequiredFieldsMissing(felder)) = ergebnis {
        assert!(felder.contains(&"pflicht".to_string()));
        assert!(felder.contains(&"pflicht_vec".to_string()));
        // mit_default hat einen Wert, sollte NICHT in der Fehlerliste sein
        assert!(!felder.contains(&"mit_default".to_string()));
    }
}

#[test]
fn test_kombiniert_valide() {
    let schema = KombiniertTestSchema {
        pflicht: "Ausgefüllt".to_string(),
        mit_default: "Überschrieben".to_string(),
        pflicht_vec: vec!["Eintrag".to_string()],
    };

    assert!(schema.validate().is_ok());
}

// ============================================================================
// TEST 5: Nested Structs
// ============================================================================

#[derive(GermanicSchema)]
#[germanic(schema_id = "test.adresse.v1")]
pub struct AdresseTestSchema {
    #[germanic(required)]
    pub strasse: String,

    #[germanic(required)]
    pub plz: String,

    #[germanic(required)]
    pub ort: String,

    #[germanic(default = "DE")]
    pub land: String,
}

#[derive(GermanicSchema)]
#[germanic(schema_id = "test.praxis.v1")]
pub struct PraxisTestSchema {
    #[germanic(required)]
    pub name: String,

    pub adresse: AdresseTestSchema, // Nested Struct
}

#[test]
fn test_nested_default() {
    let schema = PraxisTestSchema::default();

    assert_eq!(schema.name, "");
    assert_eq!(schema.adresse.strasse, "");
    assert_eq!(schema.adresse.land, "DE");
}

#[test]
fn test_nested_validierung_fehler() {
    let schema = PraxisTestSchema::default();

    let ergebnis = schema.validate();
    assert!(ergebnis.is_err());

    if let Err(germanic::error::ValidationError::RequiredFieldsMissing(felder)) = ergebnis {
        // Hauptfeld
        assert!(felder.contains(&"name".to_string()));
        // Nested Felder mit Präfix
        assert!(felder.contains(&"adresse.strasse".to_string()));
        assert!(felder.contains(&"adresse.plz".to_string()));
        assert!(felder.contains(&"adresse.ort".to_string()));
    }
}

#[test]
fn test_nested_validierung_ok() {
    let schema = PraxisTestSchema {
        name: "Dr. Müller".to_string(),
        adresse: AdresseTestSchema {
            strasse: "Hauptstraße 1".to_string(),
            plz: "12345".to_string(),
            ort: "Berlin".to_string(),
            land: "DE".to_string(),
        },
    };

    assert!(schema.validate().is_ok());
}

#[test]
fn test_nested_partial_fehler() {
    // Nur das Nested Struct hat Fehler
    let schema = PraxisTestSchema {
        name: "Dr. Müller".to_string(), // OK
        adresse: AdresseTestSchema {
            strasse: "".to_string(), // FEHLER
            plz: "12345".to_string(),
            ort: "Berlin".to_string(),
            land: "DE".to_string(),
        },
    };

    let ergebnis = schema.validate();
    assert!(ergebnis.is_err());

    if let Err(germanic::error::ValidationError::RequiredFieldsMissing(felder)) = ergebnis {
        assert_eq!(felder.len(), 1);
        assert!(felder.contains(&"adresse.strasse".to_string()));
    }
}
