//! # GERMANIC CLI
//!
//! Command-line tool for the Concierge MVP.
//!
//! ## Main Workflow
//!
//! ```bash
//! # Compile practice JSON to .grm (static mode)
//! germanic compile --schema practice --input practice.json --output practice.grm
//!
//! # Infer schema from example JSON (dynamic mode)
//! germanic init --from example.json --schema-id de.dining.restaurant.v1
//!
//! # Compile with dynamic schema
//! germanic compile --schema restaurant.schema.json --input data.json
//!
//! # Validate a .grm file
//! germanic validate practice.grm
//!
//! # Inspect header of a .grm file
//! germanic inspect practice.grm
//! ```

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// GERMANIC - Machine-readable schemas for websites
#[derive(Parser)]
#[command(name = "germanic")]
#[command(author = "GERMANIC Project")]
#[command(version)]
#[command(about = "Compiles and validates GERMANIC schemas")]
#[command(long_about = r#"
GERMANIC makes websites machine-readable for AI systems.

Concierge Workflow:
  1. Plugin exports JSON       → practice.json
  2. CLI compiles to .grm      → germanic compile --schema practice ...
  3. .grm is uploaded          → /germanic/data.grm

Dynamic Workflow (Weg 3):
  1. Provide example JSON      → germanic init --from example.json --schema-id ...
  2. Edit .schema.json          → mark required fields
  3. Compile dynamically       → germanic compile --schema my.schema.json --input data.json

Example:
  germanic compile --schema practice --input dr-sonnenschein.json
  germanic init --from restaurant.json --schema-id de.dining.restaurant.v1
"#)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compiles JSON to .grm
    ///
    /// Reads a JSON file, validates it against the schema,
    /// and creates a .grm binary file.
    ///
    /// Static mode: --schema practice (or praxis)
    /// Dynamic mode: --schema path/to/schema.json
    Compile {
        /// Schema name (e.g. "practice") or path to .schema.json
        #[arg(short, long)]
        schema: String,

        /// Path to JSON input file
        #[arg(short, long)]
        input: PathBuf,

        /// Path to .grm output file
        /// Default: same name as input with .grm extension
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Infers a schema from example JSON
    Init {
        /// Path to example JSON file
        #[arg(long)]
        from: PathBuf,

        /// Schema ID (e.g. "de.dining.restaurant.v1")
        #[arg(long)]
        schema_id: String,

        /// Output path for .schema.json
        /// Default: same directory, schema_id as filename
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Shows available schemas
    Schemas {
        /// Show details for a specific schema
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Validates a .grm file
    Validate {
        /// Path to .grm file
        file: PathBuf,
    },

    /// Shows header and metadata of a .grm file
    Inspect {
        /// Path to .grm file
        file: PathBuf,

        /// Also show hex dump of header
        #[arg(long)]
        hex: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile {
            schema,
            input,
            output,
        } => {
            let schema_path = std::path::Path::new(&schema);
            if schema_path.extension().is_some_and(|ext| ext == "json") && schema_path.exists() {
                // Dynamic mode (Weg 3)
                cmd_compile_dynamic(schema_path, &input, output.as_deref())
            } else {
                // Static mode (existing)
                cmd_compile(&schema, &input, output.as_deref())
            }
        }

        Commands::Init {
            from,
            schema_id,
            output,
        } => cmd_init(&from, &schema_id, output.as_deref()),

        Commands::Schemas { name } => cmd_schemas(name.as_deref()),

        Commands::Validate { file } => cmd_validate(&file),

        Commands::Inspect { file, hex } => cmd_inspect(&file, hex),
    }
}

/// Compiles JSON to .grm (static mode)
fn cmd_compile(
    schema_name: &str,
    input: &PathBuf,
    output: Option<&std::path::Path>,
) -> Result<()> {
    use germanic::compiler::{compile_json, SchemaType};
    use germanic::schemas::PraxisSchema;

    println!("┌─────────────────────────────────────────");
    println!("│ GERMANIC Compiler");
    println!("├─────────────────────────────────────────");
    println!("│ Schema: {}", schema_name);
    println!("│ Input:  {}", input.display());

    // 1. Validate schema type
    let schema_type = SchemaType::from_str(schema_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown schema: '{}'\n\
             Available schemas: practice, praxis\n\
             Or provide a .schema.json path for dynamic mode",
            schema_name
        )
    })?;

    // 2. Read JSON
    let json = std::fs::read_to_string(input).context("Could not read JSON file")?;

    // 3. Compile schema-specifically
    let grm_bytes = match schema_type {
        SchemaType::Practice => compile_json::<PraxisSchema>(&json).context("Compilation failed")?,
    };

    // 4. Determine output path
    let output_path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| input.with_extension("grm"));

    // 5. Write
    std::fs::write(&output_path, &grm_bytes).context("Write failed")?;

    println!("│ Output: {}", output_path.display());
    println!("│ Size:   {} bytes", grm_bytes.len());
    println!("├─────────────────────────────────────────");
    println!("│ ✓ Compilation successful");
    println!("└─────────────────────────────────────────");

    Ok(())
}

/// Compiles JSON to .grm (dynamic mode — Weg 3)
///
/// Supports both GERMANIC native `.schema.json` and JSON Schema Draft 7 input.
/// Format is auto-detected transparently.
fn cmd_compile_dynamic(
    schema_path: &std::path::Path,
    input: &PathBuf,
    output: Option<&std::path::Path>,
) -> Result<()> {
    use germanic::dynamic::{compile_dynamic, load_schema_auto};

    println!("┌─────────────────────────────────────────");
    println!("│ GERMANIC Dynamic Compiler");
    println!("├─────────────────────────────────────────");
    println!("│ Schema: {}", schema_path.display());
    println!("│ Input:  {}", input.display());

    // Check for JSON Schema warnings (auto-detection happens inside compile_dynamic too,
    // but we run detection separately here to surface warnings to the user)
    if let Ok((_, warnings)) = load_schema_auto(schema_path) {
        for warning in &warnings {
            println!("│ ⚠ {}", warning);
        }
    }

    let grm_bytes =
        compile_dynamic(schema_path, input).context("Dynamic compilation failed")?;

    let output_path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| input.with_extension("grm"));

    std::fs::write(&output_path, &grm_bytes).context("Write failed")?;

    println!("│ Output: {}", output_path.display());
    println!("│ Size:   {} bytes", grm_bytes.len());
    println!("├─────────────────────────────────────────");
    println!("│ ✓ Dynamic compilation successful");
    println!("└─────────────────────────────────────────");

    Ok(())
}

/// Infers a schema from example JSON
fn cmd_init(
    from: &PathBuf,
    schema_id: &str,
    output: Option<&std::path::Path>,
) -> Result<()> {
    use germanic::dynamic::infer::infer_schema;

    println!("┌─────────────────────────────────────────");
    println!("│ GERMANIC Schema Inference");
    println!("├─────────────────────────────────────────");
    println!("│ Input: {}", from.display());
    println!("│ Schema-ID: {}", schema_id);

    let json_str = std::fs::read_to_string(from).context("Could not read JSON file")?;
    let data: serde_json::Value = serde_json::from_str(&json_str).context("Invalid JSON")?;

    let schema = infer_schema(&data, schema_id)
        .ok_or_else(|| anyhow::anyhow!("Could not infer schema — input must be a JSON object"))?;

    let output_path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let name = schema_id.replace('.', "_");
            PathBuf::from(format!("{}.schema.json", name))
        });

    schema
        .to_file(&output_path)
        .context("Could not write schema file")?;

    println!("│ Output: {}", output_path.display());
    println!("│ Fields: {}", schema.field_count());
    println!("├─────────────────────────────────────────");
    println!(
        "│ ✓ Schema inferred — edit {} to mark required fields",
        output_path.display()
    );
    println!("└─────────────────────────────────────────");

    Ok(())
}

/// Shows available schemas
fn cmd_schemas(name: Option<&str>) -> Result<()> {
    println!("┌─────────────────────────────────────────");
    println!("│ GERMANIC Schemas");
    println!("├─────────────────────────────────────────");

    match name {
        Some("praxis") | Some("practice") => {
            println!("│");
            println!("│ Schema: practice (praxis)");
            println!("│ ID:     de.gesundheit.praxis.v1");
            println!("│ Type:   Healthcare practitioners, doctors, therapists");
            println!("│");
            println!("│ Required fields:");
            println!("│   - name         : String");
            println!("│   - bezeichnung  : String");
            println!("│   - adresse      : Address");
            println!("│     - strasse    : String");
            println!("│     - plz        : String");
            println!("│     - ort        : String");
            println!("│");
            println!("│ Optional fields:");
            println!("│   - praxisname, telefon, email, website");
            println!("│   - schwerpunkte, therapieformen, qualifikationen");
            println!("│   - terminbuchung_url, oeffnungszeiten");
            println!("│   - privatpatienten, kassenpatienten");
            println!("│   - sprachen, kurzbeschreibung");
        }
        Some(unknown) => {
            println!("│ ✗ Unknown schema: '{}'", unknown);
            println!("│");
            println!("│ Available: practice, praxis");
        }
        None => {
            println!("│");
            println!("│ Available schemas:");
            println!("│");
            println!("│   practice   Healthcare practitioners, doctors, therapists");
            println!("│   (praxis)   → germanic compile --schema practice ...");
            println!("│");
            println!("│ Dynamic schemas:");
            println!("│   Any .schema.json file can be used with:");
            println!("│   germanic compile --schema my.schema.json --input data.json");
        }
    }

    println!("└─────────────────────────────────────────");
    Ok(())
}

/// Validates a .grm file
fn cmd_validate(file: &PathBuf) -> Result<()> {
    use germanic::validator::validate_grm;

    println!("Validating {}...", file.display());

    let data = std::fs::read(file).context("Could not read file")?;

    let result = validate_grm(&data)?;

    if result.valid {
        println!("✓ File is valid");
        if let Some(id) = result.schema_id {
            println!("  Schema-ID: {}", id);
        }
    } else {
        println!("✗ File is invalid");
        if let Some(error) = result.error {
            println!("  Error: {}", error);
        }
    }

    Ok(())
}

/// Shows header and metadata of a .grm file
fn cmd_inspect(file: &PathBuf, hex: bool) -> Result<()> {
    use germanic::types::GrmHeader;

    println!("┌─────────────────────────────────────────");
    println!("│ GERMANIC Inspector");
    println!("├─────────────────────────────────────────");
    println!("│ File: {}", file.display());

    let data = std::fs::read(file).context("Could not read file")?;

    println!("│ Size: {} bytes", data.len());
    println!("│");

    // Parse header
    match GrmHeader::from_bytes(&data) {
        Ok((header, header_len)) => {
            println!("│ Header:");
            println!("│   Schema-ID: {}", header.schema_id);
            println!(
                "│   Signed:    {}",
                if header.signature.is_some() {
                    "Yes"
                } else {
                    "No"
                }
            );
            println!("│   Header length:  {} bytes", header_len);
            println!("│   Payload length: {} bytes", data.len() - header_len);

            if hex {
                println!("│");
                println!("│ Hex dump (first 64 bytes):");
                let show_len = std::cmp::min(64, data.len());
                for (i, chunk) in data[..show_len].chunks(16).enumerate() {
                    print!("│   {:04X}:  ", i * 16);
                    for byte in chunk {
                        print!("{:02X} ", byte);
                    }
                    println!();
                }
            }
        }
        Err(e) => {
            println!("│ ✗ Header error: {}", e);
        }
    }

    println!("└─────────────────────────────────────────");
    Ok(())
}
