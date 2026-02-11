//! # .grm Format-Definitionen
//!
//! Definiert das binäre .grm Format für GERMANIC-Schemas.
//!
//! ## Format-Spezifikation
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                        .grm DATEIFORMAT                                     │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   Offset │ Größe │ Inhalt                                                   │
//! │   ───────┼───────┼────────────────────────────────────────                  │
//! │   0x00   │ 3     │ Magic: "GRM" (0x47 0x52 0x4D)                             │
//! │   0x03   │ 1     │ Version (aktuell: 0x01)                                   │
//! │   0x04   │ 2     │ Schema-ID Länge (little-endian u16)                       │
//! │   0x06   │ n     │ Schema-ID (UTF-8, z.B. "de.gesundheit.praxis.v1")         │
//! │   0x06+n │ 64    │ Ed25519 Signatur (optional, 0x00 wenn nicht signiert)     │
//! │   ...    │ ...   │ FlatBuffer Payload                                        │
//! │                                                                             │
//! │   BEISPIEL (praxis.grm):                                                    │
//! │   47 52 4D 01              → "GRM" + Version 1                               │
//! │   19 00                    → Schema-ID Länge: 25 Bytes                       │
//! │   64 65 2E 67 65 ...       → "de.gesundheit.praxis.v1"                       │
//! │   00 00 00 ... (64 Bytes)  → Keine Signatur                                  │
//! │   <flatbuffer bytes>       → Eigentliche Daten                               │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Architektonische Entscheidungen
//!
//! 1. **Magic Bytes**: Ermöglichen schnelle Identifikation ohne Parsing
//! 2. **Schema-ID im Header**: KI-Systeme können das Schema identifizieren
//! 3. **Optionale Signatur**: Für vertrauenswürdige Quellen
//! 4. **FlatBuffer Payload**: Zero-Copy Deserialisierung

/// Magische Bytes am Anfang jeder .grm Datei.
///
/// - Bytes 0-2: "GRM" als ASCII
/// - Byte 3: Formatversion (aktuell: 0x01)
pub const GRM_MAGIC: [u8; 4] = [0x47, 0x52, 0x4D, 0x01]; // "GRM" + Version 1

/// Aktuelle .grm Format-Version.
pub const GRM_VERSION: u8 = 0x01;

/// Größe der Ed25519-Signatur in Bytes.
pub const SIGNATUR_GROESSE: usize = 64;

/// Header-Struktur für .grm Dateien.
///
/// ## Verwendung
///
/// ```rust,ignore
/// let header = GrmHeader {
///     schema_id: "de.gesundheit.praxis.v1".to_string(),
///     signatur: None,
/// };
///
/// let bytes = header.zu_bytes();
/// ```
#[derive(Debug, Clone)]
pub struct GrmHeader {
    /// Eindeutige Schema-ID.
    ///
    /// Format: `"{namespace}.{domain}.{name}.v{version}"`
    /// Beispiel: `"de.gesundheit.praxis.v1"`
    pub schema_id: String,

    /// Optionale Ed25519-Signatur.
    ///
    /// Wenn vorhanden: 64 Bytes
    /// Wenn nicht: None (wird als 64 Null-Bytes geschrieben)
    pub signatur: Option<[u8; SIGNATUR_GROESSE]>,
}

impl GrmHeader {
    /// Erstellt einen neuen Header ohne Signatur.
    pub fn neu(schema_id: impl Into<String>) -> Self {
        Self {
            schema_id: schema_id.into(),
            signatur: None,
        }
    }

    /// Erstellt einen neuen Header mit Signatur.
    pub fn signiert(schema_id: impl Into<String>, signatur: [u8; SIGNATUR_GROESSE]) -> Self {
        Self {
            schema_id: schema_id.into(),
            signatur: Some(signatur),
        }
    }

    /// Serialisiert den Header in Bytes.
    ///
    /// ## Format
    ///
    /// ```text
    /// [Magic 4B][Schema-ID Länge 2B][Schema-ID nB][Signatur 64B]
    /// ```
    pub fn zu_bytes(&self) -> Vec<u8> {
        let schema_bytes = self.schema_id.as_bytes();
        let schema_len = schema_bytes.len() as u16;

        // Kapazität: 4 (Magic) + 2 (Länge) + n (Schema) + 64 (Signatur)
        let kapazitaet = 4 + 2 + schema_bytes.len() + SIGNATUR_GROESSE;
        let mut bytes = Vec::with_capacity(kapazitaet);

        // 1. Magic Bytes
        bytes.extend_from_slice(&GRM_MAGIC);

        // 2. Schema-ID Länge (little-endian u16)
        bytes.extend_from_slice(&schema_len.to_le_bytes());

        // 3. Schema-ID
        bytes.extend_from_slice(schema_bytes);

        // 4. Signatur (64 Bytes, oder Nullen)
        match &self.signatur {
            Some(sig) => bytes.extend_from_slice(sig),
            None => bytes.extend_from_slice(&[0u8; SIGNATUR_GROESSE]),
        }

        bytes
    }

    /// Parst einen Header aus Bytes.
    ///
    /// # Fehler
    ///
    /// - Zu wenige Bytes
    /// - Falsche Magic Bytes
    /// - Ungültige UTF-8 Schema-ID
    pub fn von_bytes(daten: &[u8]) -> Result<(Self, usize), HeaderParseFehler> {
        // Mindestgröße: 4 (Magic) + 2 (Länge) + 64 (Signatur)
        const MIN_GROESSE: usize = 4 + 2 + SIGNATUR_GROESSE;

        if daten.len() < MIN_GROESSE {
            return Err(HeaderParseFehler::ZuWenigDaten {
                erwartet: MIN_GROESSE,
                erhalten: daten.len(),
            });
        }

        // 1. Magic prüfen
        if &daten[0..4] != &GRM_MAGIC {
            return Err(HeaderParseFehler::FalscheMagicBytes {
                erhalten: [daten[0], daten[1], daten[2], daten[3]],
            });
        }

        // 2. Schema-ID Länge lesen
        let schema_len = u16::from_le_bytes([daten[4], daten[5]]) as usize;

        // 3. Prüfen ob genug Daten für Schema-ID
        let total_header_len = 4 + 2 + schema_len + SIGNATUR_GROESSE;
        if daten.len() < total_header_len {
            return Err(HeaderParseFehler::ZuWenigDaten {
                erwartet: total_header_len,
                erhalten: daten.len(),
            });
        }

        // 4. Schema-ID parsen
        let schema_start = 6;
        let schema_end = schema_start + schema_len;
        let schema_id = std::str::from_utf8(&daten[schema_start..schema_end])
            .map_err(|_| HeaderParseFehler::UngueltigeSchemaId)?
            .to_string();

        // 5. Signatur lesen
        let sig_start = schema_end;
        let sig_end = sig_start + SIGNATUR_GROESSE;
        let sig_bytes: [u8; SIGNATUR_GROESSE] = daten[sig_start..sig_end]
            .try_into()
            .expect("Signatur-Slice hat falsche Länge");

        // Prüfen ob Signatur alle Nullen ist
        let signatur = if sig_bytes.iter().all(|&b| b == 0) {
            None
        } else {
            Some(sig_bytes)
        };

        let header = GrmHeader { schema_id, signatur };

        Ok((header, total_header_len))
    }

    /// Berechnet die Header-Größe in Bytes.
    pub fn groesse(&self) -> usize {
        4 + 2 + self.schema_id.len() + SIGNATUR_GROESSE
    }
}

/// Fehler beim Parsen eines .grm Headers.
#[derive(Debug, Clone, thiserror::Error)]
pub enum HeaderParseFehler {
    #[error("Zu wenige Daten: erwartet {erwartet}, erhalten {erhalten}")]
    ZuWenigDaten { erwartet: usize, erhalten: usize },

    #[error("Falsche Magic Bytes: erhalten {:02X?}", erhalten)]
    FalscheMagicBytes { erhalten: [u8; 4] },

    #[error("Ungültige Schema-ID (kein gültiges UTF-8)")]
    UngueltigeSchemaId,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_bytes() {
        assert_eq!(&GRM_MAGIC[0..3], b"GRM");
        assert_eq!(GRM_MAGIC[3], GRM_VERSION);
    }

    #[test]
    fn test_header_roundtrip() {
        let original = GrmHeader::neu("de.gesundheit.praxis.v1");
        let bytes = original.zu_bytes();
        let (geparst, laenge) = GrmHeader::von_bytes(&bytes).unwrap();

        assert_eq!(geparst.schema_id, original.schema_id);
        assert_eq!(geparst.signatur, None);
        assert_eq!(laenge, bytes.len());
    }

    #[test]
    fn test_header_mit_signatur() {
        let signatur = [0xAB; SIGNATUR_GROESSE];
        let original = GrmHeader::signiert("test.v1", signatur);
        let bytes = original.zu_bytes();
        let (geparst, _) = GrmHeader::von_bytes(&bytes).unwrap();

        assert_eq!(geparst.signatur, Some(signatur));
    }

    #[test]
    fn test_falsche_magic_bytes() {
        let daten = [0x00; 100];
        let ergebnis = GrmHeader::von_bytes(&daten);

        assert!(matches!(
            ergebnis,
            Err(HeaderParseFehler::FalscheMagicBytes { .. })
        ));
    }
}
