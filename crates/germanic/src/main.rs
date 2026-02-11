//! # GERMANIC CLI
//!
//! Kommandozeilenwerkzeug für den Concierge MVP.
//!
//! ## Haupt-Workflow
//!
//! ```bash
//! # Kompiliere Praxis-JSON zu .grm
//! germanic compile --schema praxis --input praxis.json --output praxis.grm
//!
//! # Validiere eine .grm Datei
//! germanic validate praxis.grm
//!
//! # Inspiziere Header einer .grm Datei
//! germanic inspect praxis.grm
//! ```

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// GERMANIC - Maschinenlesbare Schemas für Websites
#[derive(Parser)]
#[command(name = "germanic")]
#[command(author = "GERMANIC Project")]
#[command(version)]
#[command(about = "Kompiliert und validiert GERMANIC-Schemas")]
#[command(long_about = r#"
GERMANIC macht Websites maschinenlesbar für KI-Systeme.

Concierge Workflow:
  1. Plugin exportiert JSON    → praxis.json
  2. CLI kompiliert zu .grm    → germanic compile --schema praxis ...
  3. .grm wird hochgeladen     → /germanic/data.grm

Beispiel:
  germanic compile --schema praxis --input dr-sonnenschein.json
"#)]
struct Cli {
    #[command(subcommand)]
    kommando: Kommandos,
}

#[derive(Subcommand)]
enum Kommandos {
    /// Kompiliert JSON zu .grm
    ///
    /// Liest eine JSON-Datei, validiert sie gegen das Schema,
    /// und erstellt eine .grm Binärdatei.
    Compile {
        /// Name des Schemas (aktuell: "praxis")
        #[arg(short, long)]
        schema: String,

        /// Pfad zur JSON-Eingabedatei
        #[arg(short, long)]
        input: PathBuf,

        /// Pfad zur .grm Ausgabedatei
        /// Standard: gleicher Name wie Input mit .grm Endung
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Zeigt verfügbare Schemas
    Schemas {
        /// Zeigt Details zu einem spezifischen Schema
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Validiert eine .grm Datei
    Validate {
        /// Pfad zur .grm Datei
        datei: PathBuf,
    },

    /// Zeigt Header und Metadaten einer .grm Datei
    Inspect {
        /// Pfad zur .grm Datei
        datei: PathBuf,

        /// Zeige auch Hex-Dump des Headers
        #[arg(long)]
        hex: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.kommando {
        Kommandos::Compile {
            schema,
            input,
            output,
        } => cmd_compile(&schema, &input, output.as_deref()),

        Kommandos::Schemas { name } => cmd_schemas(name.as_deref()),

        Kommandos::Validate { datei } => cmd_validate(&datei),

        Kommandos::Inspect { datei, hex } => cmd_inspect(&datei, hex),
    }
}

/// Kompiliert JSON zu .grm
fn cmd_compile(schema_name: &str, input: &PathBuf, output: Option<&std::path::Path>) -> Result<()> {
    use germanic::compiler::{SchemaTyp, kompiliere_json};
    use germanic::schemas::PraxisSchema;

    println!("┌─────────────────────────────────────────");
    println!("│ GERMANIC Compiler");
    println!("├─────────────────────────────────────────");
    println!("│ Schema: {}", schema_name);
    println!("│ Input:  {}", input.display());

    // 1. Schema-Typ validieren
    let schema_typ = SchemaTyp::von_str(schema_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Unbekanntes Schema: '{}'\n\
             Verfügbare Schemas: praxis",
            schema_name
        )
    })?;

    // 2. JSON einlesen
    let json = std::fs::read_to_string(input).context("JSON-Datei konnte nicht gelesen werden")?;

    // 3. Schema-spezifisch kompilieren
    let grm_bytes = match schema_typ {
        SchemaTyp::Praxis => {
            kompiliere_json::<PraxisSchema>(&json).context("Kompilierung fehlgeschlagen")?
        }
    };

    // 4. Ausgabepfad bestimmen
    let ausgabe_pfad = output
        .map(PathBuf::from)
        .unwrap_or_else(|| input.with_extension("grm"));

    // 5. Schreiben
    std::fs::write(&ausgabe_pfad, &grm_bytes).context("Schreiben fehlgeschlagen")?;

    println!("│ Output: {}", ausgabe_pfad.display());
    println!("│ Größe:  {} Bytes", grm_bytes.len());
    println!("├─────────────────────────────────────────");
    println!("│ ✓ Kompilierung erfolgreich");
    println!("└─────────────────────────────────────────");

    Ok(())
}

/// Zeigt verfügbare Schemas
fn cmd_schemas(name: Option<&str>) -> Result<()> {
    println!("┌─────────────────────────────────────────");
    println!("│ GERMANIC Schemas");
    println!("├─────────────────────────────────────────");

    match name {
        Some("praxis") => {
            println!("│");
            println!("│ Schema: praxis");
            println!("│ ID:     de.gesundheit.praxis.v1");
            println!("│ Typ:    Heilpraktiker, Ärzte, Therapeuten");
            println!("│");
            println!("│ Pflichtfelder:");
            println!("│   - name         : String");
            println!("│   - bezeichnung  : String");
            println!("│   - adresse      : Adresse");
            println!("│     - strasse    : String");
            println!("│     - plz        : String");
            println!("│     - ort        : String");
            println!("│");
            println!("│ Optionale Felder:");
            println!("│   - praxisname, telefon, email, website");
            println!("│   - schwerpunkte, therapieformen, qualifikationen");
            println!("│   - terminbuchung_url, oeffnungszeiten");
            println!("│   - privatpatienten, kassenpatienten");
            println!("│   - sprachen, kurzbeschreibung");
        }
        Some(unknown) => {
            println!("│ ✗ Unbekanntes Schema: '{}'", unknown);
            println!("│");
            println!("│ Verfügbar: praxis");
        }
        None => {
            println!("│");
            println!("│ Verfügbare Schemas:");
            println!("│");
            println!("│   praxis    Heilpraktiker, Ärzte, Therapeuten");
            println!("│             → germanic compile --schema praxis ...");
        }
    }

    println!("└─────────────────────────────────────────");
    Ok(())
}

/// Validiert eine .grm Datei
fn cmd_validate(datei: &PathBuf) -> Result<()> {
    use germanic::validator::validiere_grm;

    println!("Validiere {}...", datei.display());

    let daten = std::fs::read(datei).context("Datei konnte nicht gelesen werden")?;

    let ergebnis = validiere_grm(&daten)?;

    if ergebnis.gueltig {
        println!("✓ Datei ist gültig");
        if let Some(id) = ergebnis.schema_id {
            println!("  Schema-ID: {}", id);
        }
    } else {
        println!("✗ Datei ist ungültig");
        if let Some(fehler) = ergebnis.fehler {
            println!("  Fehler: {}", fehler);
        }
    }

    Ok(())
}

/// Zeigt Header und Metadaten einer .grm Datei
fn cmd_inspect(datei: &PathBuf, hex: bool) -> Result<()> {
    use germanic::types::GrmHeader;

    println!("┌─────────────────────────────────────────");
    println!("│ GERMANIC Inspector");
    println!("├─────────────────────────────────────────");
    println!("│ Datei: {}", datei.display());

    let daten = std::fs::read(datei).context("Datei konnte nicht gelesen werden")?;

    println!("│ Größe: {} Bytes", daten.len());
    println!("│");

    // Header parsen
    match GrmHeader::von_bytes(&daten) {
        Ok((header, header_len)) => {
            println!("│ Header:");
            println!("│   Schema-ID: {}", header.schema_id);
            println!(
                "│   Signiert:  {}",
                if header.signatur.is_some() {
                    "Ja"
                } else {
                    "Nein"
                }
            );
            println!("│   Header-Länge: {} Bytes", header_len);
            println!("│   Payload-Länge: {} Bytes", daten.len() - header_len);

            if hex {
                println!("│");
                println!("│ Hex-Dump (erste 64 Bytes):");
                let show_len = std::cmp::min(64, daten.len());
                for (i, chunk) in daten[..show_len].chunks(16).enumerate() {
                    print!("│   {:04X}:  ", i * 16);
                    for byte in chunk {
                        print!("{:02X} ", byte);
                    }
                    println!();
                }
            }
        }
        Err(e) => {
            println!("│ ✗ Header-Fehler: {}", e);
        }
    }

    println!("└─────────────────────────────────────────");
    Ok(())
}
