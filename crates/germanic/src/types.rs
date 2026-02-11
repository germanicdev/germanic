//! # .grm Format Definitions
//!
//! Defines the binary .grm format for GERMANIC schemas.
//!
//! ## Format Specification
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                        .grm FILE FORMAT                                     │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   Offset │ Size  │ Content                                                  │
//! │   ───────┼───────┼────────────────────────────────────────                  │
//! │   0x00   │ 3     │ Magic: "GRM" (0x47 0x52 0x4D)                            │
//! │   0x03   │ 1     │ Version (current: 0x01)                                  │
//! │   0x04   │ 2     │ Schema-ID length (little-endian u16)                     │
//! │   0x06   │ n     │ Schema-ID (UTF-8, e.g. "de.gesundheit.praxis.v1")        │
//! │   0x06+n │ 64    │ Ed25519 signature (optional, 0x00 if unsigned)           │
//! │   ...    │ ...   │ FlatBuffer Payload                                       │
//! │                                                                             │
//! │   EXAMPLE (praxis.grm):                                                     │
//! │   47 52 4D 01              → "GRM" + Version 1                              │
//! │   19 00                    → Schema-ID length: 25 bytes                     │
//! │   64 65 2E 67 65 ...       → "de.gesundheit.praxis.v1"                      │
//! │   00 00 00 ... (64 bytes)  → No signature                                   │
//! │   <flatbuffer bytes>       → Actual data                                    │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Architectural Decisions
//!
//! 1. **Magic Bytes**: Enable fast identification without parsing
//! 2. **Schema-ID in header**: AI systems can identify the schema
//! 3. **Optional signature**: For trusted sources
//! 4. **FlatBuffer payload**: Zero-copy deserialization

/// Magic bytes at the beginning of every .grm file.
///
/// - Bytes 0-2: "GRM" as ASCII
/// - Byte 3: Format version (current: 0x01)
pub const GRM_MAGIC: [u8; 4] = [0x47, 0x52, 0x4D, 0x01]; // "GRM" + Version 1

/// Current .grm format version.
pub const GRM_VERSION: u8 = 0x01;

/// Size of the Ed25519 signature in bytes.
pub const SIGNATURE_SIZE: usize = 64;

/// Header structure for .grm files.
///
/// ## Usage
///
/// ```rust,ignore
/// let header = GrmHeader {
///     schema_id: "de.gesundheit.praxis.v1".to_string(),
///     signature: None,
/// };
///
/// let bytes = header.to_bytes();
/// ```
#[derive(Debug, Clone)]
pub struct GrmHeader {
    /// Unique schema ID.
    ///
    /// Format: `"{namespace}.{domain}.{name}.v{version}"`
    /// Example: `"de.gesundheit.praxis.v1"`
    pub schema_id: String,

    /// Optional Ed25519 signature.
    ///
    /// If present: 64 bytes
    /// If not: None (written as 64 null bytes)
    pub signature: Option<[u8; SIGNATURE_SIZE]>,
}

impl GrmHeader {
    /// Creates a new header without signature.
    pub fn new(schema_id: impl Into<String>) -> Self {
        Self {
            schema_id: schema_id.into(),
            signature: None,
        }
    }

    /// Creates a new header with signature.
    pub fn signed(schema_id: impl Into<String>, signature: [u8; SIGNATURE_SIZE]) -> Self {
        Self {
            schema_id: schema_id.into(),
            signature: Some(signature),
        }
    }

    /// Serializes the header to bytes.
    ///
    /// ## Format
    ///
    /// ```text
    /// [Magic 4B][Schema-ID length 2B][Schema-ID nB][Signature 64B]
    /// ```
    pub fn to_bytes(&self) -> Vec<u8> {
        let schema_bytes = self.schema_id.as_bytes();
        let schema_len = schema_bytes.len() as u16;

        // Capacity: 4 (Magic) + 2 (Length) + n (Schema) + 64 (Signature)
        let capacity = 4 + 2 + schema_bytes.len() + SIGNATURE_SIZE;
        let mut bytes = Vec::with_capacity(capacity);

        // 1. Magic bytes
        bytes.extend_from_slice(&GRM_MAGIC);

        // 2. Schema-ID length (little-endian u16)
        bytes.extend_from_slice(&schema_len.to_le_bytes());

        // 3. Schema-ID
        bytes.extend_from_slice(schema_bytes);

        // 4. Signature (64 bytes, or zeros)
        match &self.signature {
            Some(sig) => bytes.extend_from_slice(sig),
            None => bytes.extend_from_slice(&[0u8; SIGNATURE_SIZE]),
        }

        bytes
    }

    /// Parses a header from bytes.
    ///
    /// # Errors
    ///
    /// - Too few bytes
    /// - Invalid magic bytes
    /// - Invalid UTF-8 schema ID
    pub fn from_bytes(data: &[u8]) -> Result<(Self, usize), HeaderParseError> {
        // Minimum size: 4 (Magic) + 2 (Length) + 64 (Signature)
        const MIN_SIZE: usize = 4 + 2 + SIGNATURE_SIZE;

        if data.len() < MIN_SIZE {
            return Err(HeaderParseError::InsufficientData {
                expected: MIN_SIZE,
                received: data.len(),
            });
        }

        // 1. Check magic bytes
        if &data[0..4] != &GRM_MAGIC {
            return Err(HeaderParseError::InvalidMagicBytes {
                received: [data[0], data[1], data[2], data[3]],
            });
        }

        // 2. Read schema-ID length
        let schema_len = u16::from_le_bytes([data[4], data[5]]) as usize;

        // 3. Check if enough data for schema-ID
        let total_header_len = 4 + 2 + schema_len + SIGNATURE_SIZE;
        if data.len() < total_header_len {
            return Err(HeaderParseError::InsufficientData {
                expected: total_header_len,
                received: data.len(),
            });
        }

        // 4. Parse schema-ID
        let schema_start = 6;
        let schema_end = schema_start + schema_len;
        let schema_id = std::str::from_utf8(&data[schema_start..schema_end])
            .map_err(|_| HeaderParseError::InvalidSchemaId)?
            .to_string();

        // 5. Read signature
        let sig_start = schema_end;
        let sig_end = sig_start + SIGNATURE_SIZE;
        let sig_bytes: [u8; SIGNATURE_SIZE] = data[sig_start..sig_end]
            .try_into()
            .expect("Signature slice has wrong length");

        // Check if signature is all zeros
        let signature = if sig_bytes.iter().all(|&b| b == 0) {
            None
        } else {
            Some(sig_bytes)
        };

        let header = GrmHeader {
            schema_id,
            signature,
        };

        Ok((header, total_header_len))
    }

    /// Calculates the header size in bytes.
    pub fn size(&self) -> usize {
        4 + 2 + self.schema_id.len() + SIGNATURE_SIZE
    }
}

/// Error when parsing a .grm header.
#[derive(Debug, Clone, thiserror::Error)]
pub enum HeaderParseError {
    #[error("Insufficient data: expected {expected}, received {received}")]
    InsufficientData { expected: usize, received: usize },

    #[error("Invalid magic bytes: received {:02X?}", received)]
    InvalidMagicBytes { received: [u8; 4] },

    #[error("Invalid schema ID (not valid UTF-8)")]
    InvalidSchemaId,
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
        let original = GrmHeader::new("de.gesundheit.praxis.v1");
        let bytes = original.to_bytes();
        let (parsed, length) = GrmHeader::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.schema_id, original.schema_id);
        assert_eq!(parsed.signature, None);
        assert_eq!(length, bytes.len());
    }

    #[test]
    fn test_header_with_signature() {
        let signature = [0xAB; SIGNATURE_SIZE];
        let original = GrmHeader::signed("test.v1", signature);
        let bytes = original.to_bytes();
        let (parsed, _) = GrmHeader::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.signature, Some(signature));
    }

    #[test]
    fn test_invalid_magic_bytes() {
        let data = [0x00; 100];
        let result = GrmHeader::from_bytes(&data);

        assert!(matches!(
            result,
            Err(HeaderParseError::InvalidMagicBytes { .. })
        ));
    }
}
