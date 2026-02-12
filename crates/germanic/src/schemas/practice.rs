//! # Practice Schema
//!
//! Schema for healthcare practitioners, doctors and therapists.
//!
//! ## Data Flow
//!
//! ```text
//! WordPress Plugin
//!       │
//!       ▼
//!   praxis.json
//!       │
//!       ▼
//!   serde_json::from_str::<PraxisSchema>()
//!       │
//!       ▼
//!   PraxisSchema (Rust struct)
//!       │
//!       ├── validate() → Ok(())
//!       │
//!       ▼
//!   to_bytes() → FlatBuffer Bytes
//!       │
//!       ▼
//!   .grm file (Header + Payload)
//! ```

use crate::GermanicSchema;
use crate::schema::GermanicSerialize;
use flatbuffers::FlatBufferBuilder;
use serde::{Deserialize, Serialize};

// Import of generated FlatBuffer types
use crate::generated::praxis::de::gesundheit::{
    Adresse as FbAdresse, AdresseArgs as FbAdresseArgs, Praxis as FbPraxis,
    PraxisArgs as FbPraxisArgs,
};

// ============================================================================
// ADRESSE
// ============================================================================

/// Address of a practice.
///
/// ## Fields
///
/// | Field       | Type           | Required | Default |
/// |-------------|----------------|----------|---------|
/// | strasse     | String         | ✅       | -       |
/// | hausnummer  | `Option<String>` | ❌       | None    |
/// | plz         | String         | ✅       | -       |
/// | ort         | String         | ✅       | -       |
/// | land        | String         | ❌       | "DE"    |
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, GermanicSchema)]
#[germanic(schema_id = "de.gesundheit.adresse.v1")]
pub struct AdresseSchema {
    /// Street name (without house number)
    #[germanic(required)]
    pub strasse: String,

    /// House number (optional)
    #[serde(default)]
    pub hausnummer: Option<String>,

    /// Postal code
    #[germanic(required)]
    pub plz: String,

    /// City name
    #[germanic(required)]
    pub ort: String,

    /// Country code (ISO 3166-1 alpha-2)
    #[serde(default = "default_land")]
    #[germanic(default = "DE")]
    pub land: String,
}

fn default_land() -> String {
    "DE".to_string()
}

impl GermanicSerialize for AdresseSchema {
    /// Serializes the address to FlatBuffer bytes.
    ///
    /// **Note:** AdresseSchema alone is not a valid root type.
    /// This method is mainly used for tests.
    /// Normally address is serialized as part of PraxisSchema.
    fn to_bytes(&self) -> Vec<u8> {
        let mut builder = FlatBufferBuilder::with_capacity(256);

        // Create strings (leaves first)
        let strasse = builder.create_string(&self.strasse);
        let hausnummer = self.hausnummer.as_ref().map(|h| builder.create_string(h));
        let plz = builder.create_string(&self.plz);
        let ort = builder.create_string(&self.ort);
        let land = builder.create_string(&self.land);

        // Create address table
        let adresse = FbAdresse::create(
            &mut builder,
            &FbAdresseArgs {
                strasse: Some(strasse),
                hausnummer,
                plz: Some(plz),
                ort: Some(ort),
                land: Some(land),
            },
        );

        // Note: Address is not a root type in the FlatBuffer schema,
        // so we use finish_minimal instead of finish
        builder.finish_minimal(adresse);
        builder.finished_data().to_vec()
    }
}

// ============================================================================
// PRAXIS
// ============================================================================

/// Main schema for a healthcare practice.
///
/// ## Fields
///
/// | Field             | Type           | Required | Description                      |
/// |-------------------|----------------|----------|----------------------------------|
/// | name              | String         | ✅       | Name of practitioner             |
/// | bezeichnung       | String         | ✅       | "Heilpraktikerin", "Arzt", etc.  |
/// | adresse           | AdresseSchema  | ✅       | Complete address                 |
/// | praxisname        | `Option<String>` | ❌       | Name of practice                 |
/// | telefon           | `Option<String>` | ❌       | Phone number                     |
/// | ...               | ...            | ...      | additional optional fields       |
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, GermanicSchema)]
#[germanic(schema_id = "de.gesundheit.praxis.v1")]
pub struct PraxisSchema {
    // ────────────────────────────────────────────────────────────────────────
    // REQUIRED FIELDS
    // ────────────────────────────────────────────────────────────────────────
    /// Name of practitioner
    #[germanic(required)]
    pub name: String,

    /// Professional title
    #[germanic(required)]
    pub bezeichnung: String,

    /// Complete practice address
    pub adresse: AdresseSchema,

    // ────────────────────────────────────────────────────────────────────────
    // OPTIONAL FIELDS
    // ────────────────────────────────────────────────────────────────────────
    /// Name of practice
    #[serde(default)]
    pub praxisname: Option<String>,

    /// Phone number
    #[serde(default)]
    pub telefon: Option<String>,

    /// Email address
    #[serde(default)]
    pub email: Option<String>,

    /// Website URL
    #[serde(default)]
    pub website: Option<String>,

    /// Online appointment booking URL
    #[serde(default)]
    pub terminbuchung_url: Option<String>,

    /// Opening hours as free text
    #[serde(default)]
    pub oeffnungszeiten: Option<String>,

    /// Brief self-description
    #[serde(default)]
    pub kurzbeschreibung: Option<String>,

    // ────────────────────────────────────────────────────────────────────────
    // LISTS
    // ────────────────────────────────────────────────────────────────────────
    /// Medical specializations
    #[serde(default)]
    pub schwerpunkte: Vec<String>,

    /// Offered therapy forms
    #[serde(default)]
    pub therapieformen: Vec<String>,

    /// Qualifications and certificates
    #[serde(default)]
    pub qualifikationen: Vec<String>,

    /// Spoken languages
    #[serde(default)]
    pub sprachen: Vec<String>,

    // ────────────────────────────────────────────────────────────────────────
    // BOOLEANS
    // ────────────────────────────────────────────────────────────────────────
    /// Treats private patients?
    #[serde(default)]
    #[germanic(default = "false")]
    pub privatpatienten: bool,

    /// Treats public insurance patients?
    #[serde(default)]
    #[germanic(default = "false")]
    pub kassenpatienten: bool,
}

impl GermanicSerialize for PraxisSchema {
    /// Serializes the practice schema to FlatBuffer bytes.
    ///
    /// ## Algorithm (Inside-Out)
    ///
    /// ```text
    /// 1. Create strings             → Offsets
    /// 2. Create string vectors      → Offsets
    /// 3. Create address             → Offset (needs string offsets)
    /// 4. Create practice            → Offset (needs all others)
    /// 5. finish()                   → Bytes
    /// ```
    fn to_bytes(&self) -> Vec<u8> {
        // Estimate capacity: ~100 bytes base + strings
        let capacity = 256 + self.name.len() + self.bezeichnung.len();
        let mut builder = FlatBufferBuilder::with_capacity(capacity);

        // ════════════════════════════════════════════════════════════════════
        // STEP 1: Create all strings (leaves first)
        // ════════════════════════════════════════════════════════════════════

        // Required strings
        let name = builder.create_string(&self.name);
        let bezeichnung = builder.create_string(&self.bezeichnung);

        // Optional strings (only if present)
        let praxisname = self.praxisname.as_ref().map(|s| builder.create_string(s));
        let telefon = self.telefon.as_ref().map(|s| builder.create_string(s));
        let email = self.email.as_ref().map(|s| builder.create_string(s));
        let website = self.website.as_ref().map(|s| builder.create_string(s));
        let terminbuchung_url = self
            .terminbuchung_url
            .as_ref()
            .map(|s| builder.create_string(s));
        let oeffnungszeiten = self
            .oeffnungszeiten
            .as_ref()
            .map(|s| builder.create_string(s));
        let kurzbeschreibung = self
            .kurzbeschreibung
            .as_ref()
            .map(|s| builder.create_string(s));

        // ════════════════════════════════════════════════════════════════════
        // STEP 2: Create string vectors
        // ════════════════════════════════════════════════════════════════════
        //
        // FlatBuffer expects: Vector<WIPOffset<&str>>
        // We must first create all strings in the vector, then the vector

        let schwerpunkte = if !self.schwerpunkte.is_empty() {
            let offsets: Vec<_> = self
                .schwerpunkte
                .iter()
                .map(|s| builder.create_string(s))
                .collect();
            Some(builder.create_vector(&offsets))
        } else {
            None
        };

        let therapieformen = if !self.therapieformen.is_empty() {
            let offsets: Vec<_> = self
                .therapieformen
                .iter()
                .map(|s| builder.create_string(s))
                .collect();
            Some(builder.create_vector(&offsets))
        } else {
            None
        };

        let qualifikationen = if !self.qualifikationen.is_empty() {
            let offsets: Vec<_> = self
                .qualifikationen
                .iter()
                .map(|s| builder.create_string(s))
                .collect();
            Some(builder.create_vector(&offsets))
        } else {
            None
        };

        let sprachen = if !self.sprachen.is_empty() {
            let offsets: Vec<_> = self
                .sprachen
                .iter()
                .map(|s| builder.create_string(s))
                .collect();
            Some(builder.create_vector(&offsets))
        } else {
            None
        };

        // ════════════════════════════════════════════════════════════════════
        // STEP 3: Create address (Nested Table)
        // ════════════════════════════════════════════════════════════════════

        let adresse = {
            let strasse = builder.create_string(&self.adresse.strasse);
            let hausnummer = self
                .adresse
                .hausnummer
                .as_ref()
                .map(|h| builder.create_string(h));
            let plz = builder.create_string(&self.adresse.plz);
            let ort = builder.create_string(&self.adresse.ort);
            let land = builder.create_string(&self.adresse.land);

            FbAdresse::create(
                &mut builder,
                &FbAdresseArgs {
                    strasse: Some(strasse),
                    hausnummer,
                    plz: Some(plz),
                    ort: Some(ort),
                    land: Some(land),
                },
            )
        };

        // ════════════════════════════════════════════════════════════════════
        // STEP 4: Create practice (Root)
        // ════════════════════════════════════════════════════════════════════

        let praxis = FbPraxis::create(
            &mut builder,
            &FbPraxisArgs {
                // Required
                name: Some(name),
                bezeichnung: Some(bezeichnung),
                adresse: Some(adresse),
                // Optional
                praxisname,
                telefon,
                email,
                website,
                terminbuchung_url,
                oeffnungszeiten,
                kurzbeschreibung,
                // Vektoren
                schwerpunkte,
                therapieformen,
                qualifikationen,
                sprachen,
                // Booleans
                privatpatienten: self.privatpatienten,
                kassenpatienten: self.kassenpatienten,
            },
        );

        // ════════════════════════════════════════════════════════════════════
        // STEP 5: Finalize
        // ════════════════════════════════════════════════════════════════════

        builder.finish(praxis, None);
        builder.finished_data().to_vec()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{SchemaMetadata, Validate};

    // ────────────────────────────────────────────────────────────────────────
    // EXISTING TESTS
    // ────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_praxis_schema_id() {
        let praxis = PraxisSchema::default();
        assert_eq!(praxis.schema_id(), "de.gesundheit.praxis.v1");
    }

    #[test]
    fn test_adresse_schema_id() {
        let adresse = AdresseSchema::default();
        assert_eq!(adresse.schema_id(), "de.gesundheit.adresse.v1");
    }

    #[test]
    fn test_adresse_default_land() {
        let adresse = AdresseSchema::default();
        assert_eq!(adresse.land, "DE");
    }

    #[test]
    fn test_praxis_default_booleans() {
        let praxis = PraxisSchema::default();
        assert!(!praxis.privatpatienten);
        assert!(!praxis.kassenpatienten);
    }

    #[test]
    fn test_practice_validation_missing() {
        let praxis = PraxisSchema::default();
        let result = praxis.validate();

        assert!(result.is_err());

        if let Err(crate::error::ValidationError::RequiredFieldsMissing(fields)) = result {
            assert!(fields.contains(&"name".to_string()));
            assert!(fields.contains(&"bezeichnung".to_string()));
            assert!(fields.contains(&"adresse.strasse".to_string()));
            assert!(fields.contains(&"adresse.plz".to_string()));
            assert!(fields.contains(&"adresse.ort".to_string()));
        }
    }

    #[test]
    fn test_practice_validation_ok() {
        let praxis = PraxisSchema {
            name: "Dr. Anna Schmidt".to_string(),
            bezeichnung: "Zahnärztin".to_string(),
            adresse: AdresseSchema {
                strasse: "Musterstraße".to_string(),
                hausnummer: Some("42".to_string()),
                plz: "12345".to_string(),
                ort: "Beispielstadt".to_string(),
                land: "DE".to_string(),
            },
            ..Default::default()
        };

        assert!(praxis.validate().is_ok());
    }

    #[test]
    fn test_json_deserialization() {
        let json = r#"{
            "name": "Dr. Müller",
            "bezeichnung": "Arzt",
            "adresse": {
                "strasse": "Hauptstraße",
                "plz": "12345",
                "ort": "Berlin"
            }
        }"#;

        let praxis: PraxisSchema = serde_json::from_str(json).unwrap();

        assert_eq!(praxis.name, "Dr. Müller");
        assert_eq!(praxis.bezeichnung, "Arzt");
        assert_eq!(praxis.adresse.land, "DE"); // Default
        assert!(praxis.validate().is_ok());
    }

    #[test]
    fn test_json_complete() {
        let json = r#"{
            "name": "Dr. Anna Schmidt",
            "bezeichnung": "Zahnärztin",
            "praxisname": "Praxis Schmidt",
            "adresse": {
                "strasse": "Musterstraße",
                "hausnummer": "42",
                "plz": "12345",
                "ort": "Beispielstadt",
                "land": "DE"
            },
            "telefon": "+49 123 9876543",
            "email": "info@praxis-schmidt.example",
            "website": "https://praxis-schmidt.example",
            "schwerpunkte": ["Zahnerhaltung", "Prophylaxe"],
            "therapieformen": ["Wurzelbehandlung", "Bleaching"],
            "qualifikationen": ["Zahnärztin", "Implantologie-Zertifikat"],
            "terminbuchung_url": "https://praxis-schmidt.example/termin",
            "oeffnungszeiten": "Nach Vereinbarung",
            "privatpatienten": true,
            "kassenpatienten": false,
            "sprachen": ["Deutsch"],
            "kurzbeschreibung": "Ganzheitliche Medizin in Beispielstadt"
        }"#;

        let praxis: PraxisSchema = serde_json::from_str(json).unwrap();

        assert_eq!(praxis.name, "Dr. Anna Schmidt");
        assert!(praxis.privatpatienten);
        assert!(!praxis.kassenpatienten);
        assert_eq!(praxis.schwerpunkte.len(), 2);
        assert!(praxis.validate().is_ok());
    }

    // ────────────────────────────────────────────────────────────────────────
    // NEW TESTS: FLATBUFFER SERIALIZATION
    // ────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_practice_serialization_minimal() {
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

        let bytes = praxis.to_bytes();

        // FlatBuffer has at least header + data
        assert!(!bytes.is_empty());
        assert!(bytes.len() > 50); // Minimum size for the data
    }

    #[test]
    fn test_practice_serialization_roundtrip() {
        let original = PraxisSchema {
            name: "Dr. Anna Schmidt".to_string(),
            bezeichnung: "Zahnärztin".to_string(),
            adresse: AdresseSchema {
                strasse: "Musterstraße".to_string(),
                hausnummer: Some("42".to_string()),
                plz: "12345".to_string(),
                ort: "Beispielstadt".to_string(),
                land: "DE".to_string(),
            },
            praxisname: Some("Praxis Schmidt".to_string()),
            telefon: Some("+49 123 9876543".to_string()),
            schwerpunkte: vec!["Zahnerhaltung".to_string()],
            privatpatienten: true,
            ..Default::default()
        };

        // Serialize
        let bytes = original.to_bytes();

        // Deserialize (Zero-Copy!)
        let praxis = flatbuffers::root::<FbPraxis>(&bytes).expect("Invalid FlatBuffer");

        // Compare - required fields return &str
        assert_eq!(praxis.name(), "Dr. Anna Schmidt");
        assert_eq!(praxis.bezeichnung(), "Zahnärztin");

        // Optional fields return Option<&str>
        assert_eq!(praxis.praxisname(), Some("Praxis Schmidt"));
        assert_eq!(praxis.telefon(), Some("+49 123 9876543"));
        assert!(praxis.privatpatienten());
        assert!(!praxis.kassenpatienten());

        // Check address - required, returns Address (not Option)
        let adresse = praxis.adresse();
        assert_eq!(adresse.strasse(), "Musterstraße");
        assert_eq!(adresse.hausnummer(), Some("42"));
        assert_eq!(adresse.plz(), "12345");
        assert_eq!(adresse.ort(), "Beispielstadt");
        assert_eq!(adresse.land(), "DE");

        // Check vectors
        let schwerpunkte = praxis.schwerpunkte().expect("Specializations missing");
        assert_eq!(schwerpunkte.len(), 1);
        assert_eq!(schwerpunkte.get(0), "Zahnerhaltung");
    }

    #[test]
    fn test_practice_serialization_all_vectors() {
        let praxis = PraxisSchema {
            name: "Test".to_string(),
            bezeichnung: "Test".to_string(),
            adresse: AdresseSchema {
                strasse: "Test".to_string(),
                hausnummer: None,
                plz: "12345".to_string(),
                ort: "Test".to_string(),
                land: "DE".to_string(),
            },
            schwerpunkte: vec!["A".to_string(), "B".to_string()],
            therapieformen: vec!["X".to_string(), "Y".to_string(), "Z".to_string()],
            qualifikationen: vec!["Q1".to_string()],
            sprachen: vec!["Deutsch".to_string(), "Englisch".to_string()],
            ..Default::default()
        };

        let bytes = praxis.to_bytes();
        let fb = flatbuffers::root::<FbPraxis>(&bytes).unwrap();

        assert_eq!(fb.schwerpunkte().unwrap().len(), 2);
        assert_eq!(fb.therapieformen().unwrap().len(), 3);
        assert_eq!(fb.qualifikationen().unwrap().len(), 1);
        assert_eq!(fb.sprachen().unwrap().len(), 2);
    }

    #[test]
    fn test_address_serialization() {
        let adresse = AdresseSchema {
            strasse: "Hauptstraße".to_string(),
            hausnummer: Some("42".to_string()),
            plz: "12345".to_string(),
            ort: "Teststadt".to_string(),
            land: "DE".to_string(),
        };

        let bytes = adresse.to_bytes();

        // Deserialize address
        let fb = flatbuffers::root::<FbAdresse>(&bytes).expect("Invalid FlatBuffer");

        // required fields: direct &str
        assert_eq!(fb.strasse(), "Hauptstraße");
        assert_eq!(fb.plz(), "12345");
        assert_eq!(fb.ort(), "Teststadt");

        // optional fields: Option<&str>
        assert_eq!(fb.hausnummer(), Some("42"));
        assert_eq!(fb.land(), "DE");
    }
}
