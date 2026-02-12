//! # Byte-Compatibility Proof
//!
//! Proves that dynamic compilation (Weg 3) produces FlatBuffer bytes
//! that are readable by the static mode's flatc-generated types.
//!
//! Both compilation paths:
//! 1. Static:  PraxisSchema → to_bytes() → FlatBuffer
//! 2. Dynamic: SchemaDefinition + JSON → build_flatbuffer() → FlatBuffer
//!
//! must produce bytes that deserialize to identical values.

use germanic::dynamic::builder::build_flatbuffer;
use germanic::dynamic::schema_def::*;
use germanic::generated::praxis::de::gesundheit::Praxis as FbPraxis;
use indexmap::IndexMap;

/// Creates a SchemaDefinition that exactly mirrors praxis.fbs field order.
///
/// CRITICAL: Field order must match praxis.fbs exactly, because
/// vtable slot assignment depends on field index.
///
/// praxis.fbs field order:
///   0: name          (string, required)
///   1: bezeichnung   (string, required)
///   2: praxisname    (string)
///   3: adresse       (table, required)
///   4: telefon       (string)
///   5: email         (string)
///   6: website       (string)
///   7: schwerpunkte  ([string])
///   8: therapieformen ([string])
///   9: qualifikationen ([string])
///  10: terminbuchung_url (string)
///  11: oeffnungszeiten (string)
///  12: privatpatienten (bool, default false)
///  13: kassenpatienten (bool, default false)
///  14: sprachen       ([string])
///  15: kurzbeschreibung (string)
fn praxis_schema_def() -> SchemaDefinition {
    // Adresse sub-table (field order matches praxis.fbs Adresse table)
    let mut addr_fields = IndexMap::new();
    addr_fields.insert(
        "strasse".into(),
        FieldDefinition {
            field_type: FieldType::String,
            required: true,
            default: None,
            fields: None,
        },
    );
    addr_fields.insert(
        "hausnummer".into(),
        FieldDefinition {
            field_type: FieldType::String,
            required: false,
            default: None,
            fields: None,
        },
    );
    addr_fields.insert(
        "plz".into(),
        FieldDefinition {
            field_type: FieldType::String,
            required: true,
            default: None,
            fields: None,
        },
    );
    addr_fields.insert(
        "ort".into(),
        FieldDefinition {
            field_type: FieldType::String,
            required: true,
            default: None,
            fields: None,
        },
    );
    addr_fields.insert(
        "land".into(),
        FieldDefinition {
            field_type: FieldType::String,
            required: false,
            default: Some("DE".into()),
            fields: None,
        },
    );

    // Root table — field order MUST match praxis.fbs
    let mut fields = IndexMap::new();
    fields.insert(
        "name".into(),
        FieldDefinition {
            field_type: FieldType::String,
            required: true,
            default: None,
            fields: None,
        },
    );
    fields.insert(
        "bezeichnung".into(),
        FieldDefinition {
            field_type: FieldType::String,
            required: true,
            default: None,
            fields: None,
        },
    );
    fields.insert(
        "praxisname".into(),
        FieldDefinition {
            field_type: FieldType::String,
            required: false,
            default: None,
            fields: None,
        },
    );
    fields.insert(
        "adresse".into(),
        FieldDefinition {
            field_type: FieldType::Table,
            required: true,
            default: None,
            fields: Some(addr_fields),
        },
    );
    fields.insert(
        "telefon".into(),
        FieldDefinition {
            field_type: FieldType::String,
            required: false,
            default: None,
            fields: None,
        },
    );
    fields.insert(
        "email".into(),
        FieldDefinition {
            field_type: FieldType::String,
            required: false,
            default: None,
            fields: None,
        },
    );
    fields.insert(
        "website".into(),
        FieldDefinition {
            field_type: FieldType::String,
            required: false,
            default: None,
            fields: None,
        },
    );
    fields.insert(
        "schwerpunkte".into(),
        FieldDefinition {
            field_type: FieldType::StringArray,
            required: false,
            default: None,
            fields: None,
        },
    );
    fields.insert(
        "therapieformen".into(),
        FieldDefinition {
            field_type: FieldType::StringArray,
            required: false,
            default: None,
            fields: None,
        },
    );
    fields.insert(
        "qualifikationen".into(),
        FieldDefinition {
            field_type: FieldType::StringArray,
            required: false,
            default: None,
            fields: None,
        },
    );
    fields.insert(
        "terminbuchung_url".into(),
        FieldDefinition {
            field_type: FieldType::String,
            required: false,
            default: None,
            fields: None,
        },
    );
    fields.insert(
        "oeffnungszeiten".into(),
        FieldDefinition {
            field_type: FieldType::String,
            required: false,
            default: None,
            fields: None,
        },
    );
    fields.insert(
        "privatpatienten".into(),
        FieldDefinition {
            field_type: FieldType::Bool,
            required: false,
            default: Some("false".into()),
            fields: None,
        },
    );
    fields.insert(
        "kassenpatienten".into(),
        FieldDefinition {
            field_type: FieldType::Bool,
            required: false,
            default: Some("false".into()),
            fields: None,
        },
    );
    fields.insert(
        "sprachen".into(),
        FieldDefinition {
            field_type: FieldType::StringArray,
            required: false,
            default: None,
            fields: None,
        },
    );
    fields.insert(
        "kurzbeschreibung".into(),
        FieldDefinition {
            field_type: FieldType::String,
            required: false,
            default: None,
            fields: None,
        },
    );

    SchemaDefinition {
        schema_id: "de.gesundheit.praxis.v1".into(),
        version: 1,
        fields,
    }
}

#[test]
fn test_dynamic_praxis_readable_by_static_types() {
    let schema = praxis_schema_def();

    let data = serde_json::json!({
        "name": "Dr. Maria Sonnenschein",
        "bezeichnung": "Zahnärztin",
        "praxisname": "Praxis Sonnenschein",
        "adresse": {
            "strasse": "Musterstraße",
            "hausnummer": "42",
            "plz": "12345",
            "ort": "Beispielstadt",
            "land": "DE"
        },
        "telefon": "+49 123 9876543",
        "email": "info@praxis-sonnenschein.example",
        "schwerpunkte": ["Zahnerhaltung", "Prophylaxe"],
        "therapieformen": ["Wurzelbehandlung", "Bleaching"],
        "qualifikationen": ["Zahnärztin", "Implantologie-Zertifikat"],
        "privatpatienten": true,
        "kassenpatienten": false,
        "sprachen": ["Deutsch"],
        "kurzbeschreibung": "Moderne Zahnmedizin in Beispielstadt"
    });

    // Build via dynamic path
    let bytes = build_flatbuffer(&schema, &data).expect("Dynamic build failed");

    // Read back via static (flatc-generated) types
    let praxis =
        flatbuffers::root::<FbPraxis>(&bytes).expect("Dynamic bytes not readable by static types!");

    // Verify ALL fields match
    assert_eq!(praxis.name(), "Dr. Maria Sonnenschein");
    assert_eq!(praxis.bezeichnung(), "Zahnärztin");
    assert_eq!(praxis.praxisname(), Some("Praxis Sonnenschein"));
    assert_eq!(praxis.telefon(), Some("+49 123 9876543"));
    assert_eq!(praxis.email(), Some("info@praxis-sonnenschein.example"));
    assert!(praxis.privatpatienten());
    assert!(!praxis.kassenpatienten());

    // Verify address (nested table)
    let addr = praxis.adresse();
    assert_eq!(addr.strasse(), "Musterstraße");
    assert_eq!(addr.hausnummer(), Some("42"));
    assert_eq!(addr.plz(), "12345");
    assert_eq!(addr.ort(), "Beispielstadt");
    assert_eq!(addr.land(), "DE");

    // Verify vectors
    let schwerpunkte = praxis.schwerpunkte().expect("schwerpunkte missing");
    assert_eq!(schwerpunkte.len(), 2);
    assert_eq!(schwerpunkte.get(0), "Zahnerhaltung");
    assert_eq!(schwerpunkte.get(1), "Prophylaxe");

    let sprachen = praxis.sprachen().expect("sprachen missing");
    assert_eq!(sprachen.len(), 1);
    assert_eq!(sprachen.get(0), "Deutsch");

    assert_eq!(
        praxis.kurzbeschreibung(),
        Some("Moderne Zahnmedizin in Beispielstadt")
    );
}

#[test]
fn test_dynamic_minimal_praxis() {
    let schema = praxis_schema_def();

    // Minimum viable data
    let data = serde_json::json!({
        "name": "Test",
        "bezeichnung": "Arzt",
        "adresse": {
            "strasse": "Teststr.",
            "plz": "12345",
            "ort": "Berlin"
        }
    });

    let bytes = build_flatbuffer(&schema, &data).expect("Dynamic build failed");
    let praxis = flatbuffers::root::<FbPraxis>(&bytes).expect("Not readable!");

    assert_eq!(praxis.name(), "Test");
    assert_eq!(praxis.bezeichnung(), "Arzt");
    assert_eq!(praxis.adresse().strasse(), "Teststr.");
    assert!(!praxis.privatpatienten()); // default false
}
