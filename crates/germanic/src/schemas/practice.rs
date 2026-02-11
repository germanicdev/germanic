//! # Praxis-Schema
//!
//! Schema für Heilpraktiker, Ärzte und Therapeuten.
//!
//! ## Datenfluss
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
//!   PraxisSchema (Rust-Struct)
//!       │
//!       ├── validiere() → Ok(())
//!       │
//!       ▼
//!   zu_bytes() → FlatBuffer Bytes
//!       │
//!       ▼
//!   .grm Datei (Header + Payload)
//! ```

use crate::GermanicSchema;
use crate::schema::GermanicSerialisieren;
use flatbuffers::FlatBufferBuilder;
use serde::{Deserialize, Serialize};

// Import der generierten FlatBuffer-Typen
use crate::generated::praxis::de::gesundheit::{
    Adresse as FbAdresse, AdresseArgs as FbAdresseArgs, Praxis as FbPraxis,
    PraxisArgs as FbPraxisArgs,
};

// ============================================================================
// ADRESSE
// ============================================================================

/// Adresse einer Praxis.
///
/// ## Felder
///
/// | Feld        | Typ            | Pflicht | Default |
/// |-------------|----------------|---------|---------|
/// | strasse     | String         | ✅      | -       |
/// | hausnummer  | Option<String> | ❌      | None    |
/// | plz         | String         | ✅      | -       |
/// | ort         | String         | ✅      | -       |
/// | land        | String         | ❌      | "DE"    |
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, GermanicSchema)]
#[germanic(schema_id = "de.gesundheit.adresse.v1")]
pub struct AdresseSchema {
    /// Straßenname (ohne Hausnummer)
    #[germanic(required)]
    pub strasse: String,

    /// Hausnummer (optional)
    #[serde(default)]
    pub hausnummer: Option<String>,

    /// Postleitzahl
    #[germanic(required)]
    pub plz: String,

    /// Ortsname
    #[germanic(required)]
    pub ort: String,

    /// Ländercode (ISO 3166-1 alpha-2)
    #[serde(default = "default_land")]
    #[germanic(default = "DE")]
    pub land: String,
}

fn default_land() -> String {
    "DE".to_string()
}

impl GermanicSerialisieren for AdresseSchema {
    /// Serialisiert die Adresse zu FlatBuffer-Bytes.
    ///
    /// **Hinweis:** AdresseSchema allein ist kein gültiger Root-Typ.
    /// Diese Methode wird hauptsächlich für Tests verwendet.
    /// Im Normalfall wird Adresse als Teil von PraxisSchema serialisiert.
    fn zu_bytes(&self) -> Vec<u8> {
        let mut builder = FlatBufferBuilder::with_capacity(256);

        // Strings erstellen (Blätter zuerst)
        let strasse = builder.create_string(&self.strasse);
        let hausnummer = self.hausnummer.as_ref().map(|h| builder.create_string(h));
        let plz = builder.create_string(&self.plz);
        let ort = builder.create_string(&self.ort);
        let land = builder.create_string(&self.land);

        // Adresse-Table erstellen
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

        // Hinweis: Adresse ist kein Root-Type im FlatBuffer-Schema,
        // daher verwenden wir finish_minimal statt finish
        builder.finish_minimal(adresse);
        builder.finished_data().to_vec()
    }
}

// ============================================================================
// PRAXIS
// ============================================================================

/// Hauptschema für eine Gesundheitspraxis.
///
/// ## Felder
///
/// | Feld              | Typ            | Pflicht | Beschreibung                    |
/// |-------------------|----------------|---------|----------------------------------|
/// | name              | String         | ✅      | Name des Behandlers              |
/// | bezeichnung       | String         | ✅      | "Heilpraktikerin", "Arzt", etc.  |
/// | adresse           | AdresseSchema  | ✅      | Vollständige Adresse             |
/// | praxisname        | Option<String> | ❌      | Name der Praxis                  |
/// | telefon           | Option<String> | ❌      | Telefonnummer                    |
/// | ...               | ...            | ...     | weitere optionale Felder         |
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, GermanicSchema)]
#[germanic(schema_id = "de.gesundheit.praxis.v1")]
pub struct PraxisSchema {
    // ────────────────────────────────────────────────────────────────────────
    // PFLICHTFELDER
    // ────────────────────────────────────────────────────────────────────────
    /// Name des Behandlers
    #[germanic(required)]
    pub name: String,

    /// Berufsbezeichnung
    #[germanic(required)]
    pub bezeichnung: String,

    /// Vollständige Adresse der Praxis
    pub adresse: AdresseSchema,

    // ────────────────────────────────────────────────────────────────────────
    // OPTIONALE FELDER
    // ────────────────────────────────────────────────────────────────────────
    /// Name der Praxis
    #[serde(default)]
    pub praxisname: Option<String>,

    /// Telefonnummer
    #[serde(default)]
    pub telefon: Option<String>,

    /// E-Mail-Adresse
    #[serde(default)]
    pub email: Option<String>,

    /// Website-URL
    #[serde(default)]
    pub website: Option<String>,

    /// URL zur Online-Terminbuchung
    #[serde(default)]
    pub terminbuchung_url: Option<String>,

    /// Öffnungszeiten als Freitext
    #[serde(default)]
    pub oeffnungszeiten: Option<String>,

    /// Kurze Selbstbeschreibung
    #[serde(default)]
    pub kurzbeschreibung: Option<String>,

    // ────────────────────────────────────────────────────────────────────────
    // LISTEN
    // ────────────────────────────────────────────────────────────────────────
    /// Medizinische Schwerpunkte
    #[serde(default)]
    pub schwerpunkte: Vec<String>,

    /// Angebotene Therapieformen
    #[serde(default)]
    pub therapieformen: Vec<String>,

    /// Qualifikationen und Zertifikate
    #[serde(default)]
    pub qualifikationen: Vec<String>,

    /// Gesprochene Sprachen
    #[serde(default)]
    pub sprachen: Vec<String>,

    // ────────────────────────────────────────────────────────────────────────
    // BOOLEANS
    // ────────────────────────────────────────────────────────────────────────
    /// Behandelt Privatpatienten?
    #[serde(default)]
    #[germanic(default = "false")]
    pub privatpatienten: bool,

    /// Behandelt Kassenpatienten?
    #[serde(default)]
    #[germanic(default = "false")]
    pub kassenpatienten: bool,
}

impl GermanicSerialisieren for PraxisSchema {
    /// Serialisiert das Praxis-Schema zu FlatBuffer-Bytes.
    ///
    /// ## Algorithmus (Inside-Out)
    ///
    /// ```text
    /// 1. Strings erstellen          → Offsets
    /// 2. String-Vektoren erstellen  → Offsets
    /// 3. Adresse erstellen          → Offset (braucht String-Offsets)
    /// 4. Praxis erstellen           → Offset (braucht alle anderen)
    /// 5. finish()                   → Bytes
    /// ```
    fn zu_bytes(&self) -> Vec<u8> {
        // Kapazität schätzen: ~100 Bytes Basis + Strings
        let kapazitaet = 256 + self.name.len() + self.bezeichnung.len();
        let mut builder = FlatBufferBuilder::with_capacity(kapazitaet);

        // ════════════════════════════════════════════════════════════════════
        // SCHRITT 1: Alle Strings erstellen (Blätter zuerst)
        // ════════════════════════════════════════════════════════════════════

        // Pflicht-Strings
        let name = builder.create_string(&self.name);
        let bezeichnung = builder.create_string(&self.bezeichnung);

        // Optionale Strings (nur wenn vorhanden)
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
        // SCHRITT 2: String-Vektoren erstellen
        // ════════════════════════════════════════════════════════════════════
        //
        // FlatBuffer erwartet: Vector<WIPOffset<&str>>
        // Wir müssen erst alle Strings im Vektor erstellen, dann den Vektor

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
        // SCHRITT 3: Adresse erstellen (Nested Table)
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
        // SCHRITT 4: Praxis (Root) erstellen
        // ════════════════════════════════════════════════════════════════════

        let praxis = FbPraxis::create(
            &mut builder,
            &FbPraxisArgs {
                // Pflicht
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
        // SCHRITT 5: Finalisieren
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
    use crate::schema::{SchemaMetadaten, Validieren};

    // ────────────────────────────────────────────────────────────────────────
    // BESTEHENDE TESTS
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
    fn test_praxis_validierung_fehlt() {
        let praxis = PraxisSchema::default();
        let ergebnis = praxis.validiere();

        assert!(ergebnis.is_err());

        if let Err(crate::error::ValidationError::RequiredFieldsMissing(felder)) = ergebnis {
            assert!(felder.contains(&"name".to_string()));
            assert!(felder.contains(&"bezeichnung".to_string()));
            assert!(felder.contains(&"adresse.strasse".to_string()));
            assert!(felder.contains(&"adresse.plz".to_string()));
            assert!(felder.contains(&"adresse.ort".to_string()));
        }
    }

    #[test]
    fn test_praxis_validierung_ok() {
        let praxis = PraxisSchema {
            name: "Dr. Maria Sonnenschein".to_string(),
            bezeichnung: "Zahnärztin".to_string(),
            adresse: AdresseSchema {
                strasse: "Lindenallee".to_string(),
                hausnummer: Some("26".to_string()),
                plz: "10115".to_string(),
                ort: "Berlin".to_string(),
                land: "DE".to_string(),
            },
            ..Default::default()
        };

        assert!(praxis.validiere().is_ok());
    }

    #[test]
    fn test_json_deserialisierung() {
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
        assert!(praxis.validiere().is_ok());
    }

    #[test]
    fn test_json_vollstaendig() {
        let json = r#"{
            "name": "Dr. Maria Sonnenschein",
            "bezeichnung": "Zahnärztin",
            "praxisname": "Praxis Sonnenschein",
            "adresse": {
                "strasse": "Lindenallee",
                "hausnummer": "26",
                "plz": "10115",
                "ort": "Berlin",
                "land": "DE"
            },
            "telefon": "+49 30 1234567",
            "email": "info@praxis-sonnenschein.example.de",
            "website": "https://praxis-sonnenschein.example.de",
            "schwerpunkte": ["Zahnerhaltung", "Prophylaxe"],
            "therapieformen": ["Wurzelbehandlung", "Bleaching"],
            "qualifikationen": ["Zahnärztin", "Implantologie-Zertifikat"],
            "terminbuchung_url": "https://praxis-sonnenschein.example.de/termin",
            "oeffnungszeiten": "Nach Vereinbarung",
            "privatpatienten": true,
            "kassenpatienten": false,
            "sprachen": ["Deutsch"],
            "kurzbeschreibung": "Ganzheitliche Medizin in Berlin"
        }"#;

        let praxis: PraxisSchema = serde_json::from_str(json).unwrap();

        assert_eq!(praxis.name, "Dr. Maria Sonnenschein");
        assert!(praxis.privatpatienten);
        assert!(!praxis.kassenpatienten);
        assert_eq!(praxis.schwerpunkte.len(), 2);
        assert!(praxis.validiere().is_ok());
    }

    // ────────────────────────────────────────────────────────────────────────
    // NEUE TESTS: FLATBUFFER-SERIALISIERUNG
    // ────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_praxis_serialisierung_minimal() {
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

        let bytes = praxis.zu_bytes();

        // FlatBuffer hat mindestens Header + Daten
        assert!(!bytes.is_empty());
        assert!(bytes.len() > 50); // Mindestgröße für die Daten
    }

    #[test]
    fn test_praxis_serialisierung_roundtrip() {
        let original = PraxisSchema {
            name: "Dr. Maria Sonnenschein".to_string(),
            bezeichnung: "Zahnärztin".to_string(),
            adresse: AdresseSchema {
                strasse: "Lindenallee".to_string(),
                hausnummer: Some("26".to_string()),
                plz: "10115".to_string(),
                ort: "Berlin".to_string(),
                land: "DE".to_string(),
            },
            praxisname: Some("Praxis Sonnenschein".to_string()),
            telefon: Some("+49 30 1234567".to_string()),
            schwerpunkte: vec!["Zahnerhaltung".to_string()],
            privatpatienten: true,
            ..Default::default()
        };

        // Serialisieren
        let bytes = original.zu_bytes();

        // Deserialisieren (Zero-Copy!)
        let praxis = flatbuffers::root::<FbPraxis>(&bytes).expect("FlatBuffer ungültig");

        // Vergleichen - required Felder geben &str zurück
        assert_eq!(praxis.name(), "Dr. Maria Sonnenschein");
        assert_eq!(praxis.bezeichnung(), "Zahnärztin");

        // Optionale Felder geben Option<&str> zurück
        assert_eq!(praxis.praxisname(), Some("Praxis Sonnenschein"));
        assert_eq!(praxis.telefon(), Some("+49 30 1234567"));
        assert!(praxis.privatpatienten());
        assert!(!praxis.kassenpatienten());

        // Adresse prüfen - required, gibt Adresse zurück (kein Option)
        let adresse = praxis.adresse();
        assert_eq!(adresse.strasse(), "Lindenallee");
        assert_eq!(adresse.hausnummer(), Some("26"));
        assert_eq!(adresse.plz(), "10115");
        assert_eq!(adresse.ort(), "Berlin");
        assert_eq!(adresse.land(), "DE");

        // Vektoren prüfen
        let schwerpunkte = praxis.schwerpunkte().expect("Schwerpunkte fehlen");
        assert_eq!(schwerpunkte.len(), 1);
        assert_eq!(schwerpunkte.get(0), "Zahnerhaltung");
    }

    #[test]
    fn test_praxis_serialisierung_alle_vektoren() {
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

        let bytes = praxis.zu_bytes();
        let fb = flatbuffers::root::<FbPraxis>(&bytes).unwrap();

        assert_eq!(fb.schwerpunkte().unwrap().len(), 2);
        assert_eq!(fb.therapieformen().unwrap().len(), 3);
        assert_eq!(fb.qualifikationen().unwrap().len(), 1);
        assert_eq!(fb.sprachen().unwrap().len(), 2);
    }

    #[test]
    fn test_adresse_serialisierung() {
        let adresse = AdresseSchema {
            strasse: "Hauptstraße".to_string(),
            hausnummer: Some("42".to_string()),
            plz: "10115".to_string(),
            ort: "Berlin".to_string(),
            land: "DE".to_string(),
        };

        let bytes = adresse.zu_bytes();

        // Adresse deserialisieren
        let fb = flatbuffers::root::<FbAdresse>(&bytes).expect("FlatBuffer ungültig");

        // required Felder: direkt &str
        assert_eq!(fb.strasse(), "Hauptstraße");
        assert_eq!(fb.plz(), "10115");
        assert_eq!(fb.ort(), "Berlin");

        // optionale Felder: Option<&str>
        assert_eq!(fb.hausnummer(), Some("42"));
        assert_eq!(fb.land(), "DE");
    }
}
