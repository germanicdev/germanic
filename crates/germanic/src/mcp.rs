//! # GERMANIC MCP Server
//!
//! Exposes GERMANIC functionality as MCP tools over stdio.
//!
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │                 MCP Client (Agent)                    │
//! │   Claude / ChatGPT / OpenClaw / Cursor / etc.        │
//! └───────────────────┬──────────────────────────────────┘
//!                     │ JSON-RPC over stdio
//! ┌───────────────────▼──────────────────────────────────┐
//! │              GermanicServer                           │
//! │  ┌─────────┬──────────┬─────────┬────────┬────────┐  │
//! │  │ compile │ validate │ inspect │schemas │  init  │  │
//! │  └────┬────┴────┬─────┴────┬────┴───┬────┴───┬────┘  │
//! │       │         │          │        │        │        │
//! │       ▼         ▼          ▼        ▼        ▼        │
//! │   [existing germanic library code]                    │
//! └──────────────────────────────────────────────────────┘
//! ```

use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::*,
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Parameter structs
// ---------------------------------------------------------------------------

/// Parameters for the `germanic_compile` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompileParams {
    /// Path to .schema.json or JSON Schema Draft 7 file
    pub schema: String,
    /// Path to JSON data file
    pub data: String,
    /// Output path for .grm (default: data path with .grm extension)
    pub output: Option<String>,
}

/// Parameters for the `germanic_validate` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FileParams {
    /// Path to .grm file
    pub file: String,
}

/// Parameters for the `germanic_inspect` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct InspectParams {
    /// Path to .grm file
    pub file: String,
    /// Include hex dump of first 64 bytes
    pub hex: Option<bool>,
}

/// Parameters for the `germanic_schemas` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SchemasParams {
    /// Schema name for details (e.g. 'practice')
    pub name: Option<String>,
}

/// Parameters for the `germanic_init` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct InitParams {
    /// Path to example JSON file
    pub from: String,
    /// Schema ID (e.g. 'de.dining.restaurant.v1')
    pub schema_id: String,
    /// Output path for .schema.json
    pub output: Option<String>,
}

/// Parameters for the `germanic_convert` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ConvertParams {
    /// Path to JSON Schema Draft 7 file
    pub input: String,
    /// Output path for .schema.json
    pub output: Option<String>,
}

// ---------------------------------------------------------------------------
// Server struct
// ---------------------------------------------------------------------------

/// MCP server exposing GERMANIC tools over stdio.
#[derive(Debug, Clone)]
pub struct GermanicServer {
    tool_router: ToolRouter<Self>,
}

impl GermanicServer {
    /// Creates a new server instance with all tools registered.
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

impl Default for GermanicServer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tool implementations
// ---------------------------------------------------------------------------

#[tool_router(router = tool_router)]
impl GermanicServer {
    /// Compile JSON data against a GERMANIC schema into binary .grm.
    #[tool(
        name = "germanic_compile",
        description = "Compile JSON data against a GERMANIC schema into binary .grm"
    )]
    async fn germanic_compile(
        &self,
        Parameters(params): Parameters<CompileParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let schema_path = std::path::Path::new(&params.schema);
        let input_path = PathBuf::from(&params.data);

        match crate::dynamic::compile_dynamic(schema_path, &input_path) {
            Ok(grm_bytes) => {
                let output_path = params
                    .output
                    .map(PathBuf::from)
                    .unwrap_or_else(|| input_path.with_extension("grm"));

                match std::fs::write(&output_path, &grm_bytes) {
                    Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
                        "Compiled successfully\n  Output: {}\n  Size: {} bytes",
                        output_path.display(),
                        grm_bytes.len()
                    ))])),
                    Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                        "Write failed: {e}"
                    ))])),
                }
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Compilation failed: {e}"
            ))])),
        }
    }

    /// Validate a .grm binary file.
    #[tool(
        name = "germanic_validate",
        description = "Validate a .grm binary file — checks magic bytes, header, and structure"
    )]
    async fn germanic_validate(
        &self,
        Parameters(params): Parameters<FileParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let data = std::fs::read(&params.file).map_err(|e| {
            ErrorData::internal_error(format!("Read failed: {e}"), None)
        })?;

        match crate::validator::validate_grm(&data) {
            Ok(result) if result.valid => {
                let schema_info = result
                    .schema_id
                    .map(|id| format!("\n  Schema-ID: {id}"))
                    .unwrap_or_default();
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Valid .grm file{schema_info}"
                ))]))
            }
            Ok(result) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Invalid: {}",
                result.error.unwrap_or_else(|| "Unknown error".into())
            ))])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Validation error: {e}"
            ))])),
        }
    }

    /// Inspect a .grm file header and metadata.
    #[tool(
        name = "germanic_inspect",
        description = "Inspect a .grm file and show its header metadata"
    )]
    async fn germanic_inspect(
        &self,
        Parameters(params): Parameters<InspectParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let data = std::fs::read(&params.file).map_err(|e| {
            ErrorData::internal_error(format!("Read failed: {e}"), None)
        })?;

        match crate::types::GrmHeader::from_bytes(&data) {
            Ok((header, header_len)) => {
                let mut info = format!(
                    "Schema-ID: {}\nSigned: {}\nHeader: {} bytes\nPayload: {} bytes",
                    header.schema_id,
                    if header.signature.is_some() {
                        "Yes"
                    } else {
                        "No"
                    },
                    header_len,
                    data.len() - header_len
                );

                if params.hex.unwrap_or(false) {
                    info.push_str("\n\nHex dump (first 64 bytes):\n");
                    let show_len = std::cmp::min(64, data.len());
                    for (i, chunk) in data[..show_len].chunks(16).enumerate() {
                        info.push_str(&format!("  {:04X}: ", i * 16));
                        for byte in chunk {
                            info.push_str(&format!("{byte:02X} "));
                        }
                        info.push('\n');
                    }
                }

                Ok(CallToolResult::success(vec![Content::text(info)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Header error: {e}"
            ))])),
        }
    }

    /// List available GERMANIC schemas.
    #[tool(
        name = "germanic_schemas",
        description = "List all available GERMANIC schemas or show details for a specific one"
    )]
    async fn germanic_schemas(
        &self,
        Parameters(params): Parameters<SchemasParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let text = match params.name.as_deref() {
            Some("practice" | "praxis") => {
                "Schema: practice (praxis)\n\
                 ID: de.gesundheit.praxis.v1\n\
                 Type: Healthcare practitioners\n\n\
                 Required: name, bezeichnung, adresse (strasse, plz, ort)\n\
                 Optional: telefon, email, website, schwerpunkte, ..."
                    .to_string()
            }
            Some(name) => format!("Unknown schema: '{name}'\nAvailable: practice"),
            None => {
                "Available schemas:\n\n\
                 Built-in:\n  practice -- Healthcare practitioners\n\n\
                 Dynamic: Any .schema.json file can be used"
                    .to_string()
            }
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    /// Infer a GERMANIC schema from example JSON.
    #[tool(
        name = "germanic_init",
        description = "Infer a GERMANIC schema from an example JSON file"
    )]
    async fn germanic_init(
        &self,
        Parameters(params): Parameters<InitParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let json_str = std::fs::read_to_string(&params.from).map_err(|e| {
            ErrorData::internal_error(format!("Read failed: {e}"), None)
        })?;
        let data: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
            ErrorData::internal_error(format!("Invalid JSON: {e}"), None)
        })?;

        let schema =
            crate::dynamic::infer::infer_schema(&data, &params.schema_id).ok_or_else(|| {
                ErrorData::internal_error(
                    "Could not infer -- input must be JSON object",
                    None,
                )
            })?;

        let output_path = params
            .output
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(format!(
                    "{}.schema.json",
                    params.schema_id.replace('.', "_")
                ))
            });

        schema.to_file(&output_path).map_err(|e| {
            ErrorData::internal_error(format!("Write failed: {e}"), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Schema inferred\n  Output: {}\n  Fields: {}",
            output_path.display(),
            schema.field_count()
        ))]))
    }

    /// Convert JSON Schema Draft 7 to GERMANIC .schema.json format.
    #[tool(
        name = "germanic_convert",
        description = "Convert a JSON Schema Draft 7 file to GERMANIC .schema.json format"
    )]
    async fn germanic_convert(
        &self,
        Parameters(params): Parameters<ConvertParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let input_str = std::fs::read_to_string(&params.input).map_err(|e| {
            ErrorData::internal_error(format!("Read failed: {e}"), None)
        })?;

        match crate::dynamic::json_schema::convert_json_schema(&input_str) {
            Ok((schema, warnings)) => {
                let output_path = params
                    .output
                    .map(PathBuf::from)
                    .unwrap_or_else(|| {
                        PathBuf::from(&params.input).with_extension("schema.json")
                    });

                schema.to_file(&output_path).map_err(|e| {
                    ErrorData::internal_error(format!("Write failed: {e}"), None)
                })?;

                let mut result = format!(
                    "Converted successfully\n  Output: {}\n  Fields: {}",
                    output_path.display(),
                    schema.field_count()
                );

                if !warnings.is_empty() {
                    result.push_str("\n\n  Warnings:");
                    for w in &warnings {
                        result.push_str(&format!("\n  - {w}"));
                    }
                }

                Ok(CallToolResult::success(vec![Content::text(result)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Conversion failed: {e}"
            ))])),
        }
    }
}

// ---------------------------------------------------------------------------
// Server handler
// ---------------------------------------------------------------------------

#[tool_handler(router = self.tool_router)]
impl ServerHandler for GermanicServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "GERMANIC -- Schema-driven compilation framework. \
                 Compiles JSON data into binary .grm files for AI-readable websites. \
                 Supports both GERMANIC .schema.json and JSON Schema Draft 7 formats."
                    .into(),
            ),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability::default()),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Start the MCP server on stdio.
pub async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    // Logs go to stderr (stdout is reserved for MCP protocol)
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("GERMANIC MCP Server starting");

    let server = GermanicServer::new();
    let service = server.serve(rmcp::transport::stdio()).await?;

    tracing::info!("Server running, waiting for requests");
    service.waiting().await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_params_deserialize() {
        let json = r#"{"schema": "test.schema.json", "data": "input.json"}"#;
        let params: CompileParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.schema, "test.schema.json");
        assert_eq!(params.data, "input.json");
        assert!(params.output.is_none());
    }

    #[test]
    fn test_compile_params_with_output() {
        let json =
            r#"{"schema": "test.schema.json", "data": "input.json", "output": "out.grm"}"#;
        let params: CompileParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.output, Some("out.grm".into()));
    }

    #[test]
    fn test_server_info() {
        let server = GermanicServer::new();
        let info = server.get_info();
        assert!(info.instructions.is_some());
        assert!(info.capabilities.tools.is_some());
    }

    #[test]
    fn test_server_has_six_tools() {
        let server = GermanicServer::new();
        let router = &server.tool_router;
        let tools = router.list_all();
        assert_eq!(
            tools.len(),
            6,
            "Expected 6 tools, got {}: {:?}",
            tools.len(),
            tools.iter().map(|t| &t.name).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_tool_names() {
        let server = GermanicServer::new();
        let tools = server.tool_router.list_all();
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
        assert!(names.contains(&"germanic_compile"));
        assert!(names.contains(&"germanic_validate"));
        assert!(names.contains(&"germanic_inspect"));
        assert!(names.contains(&"germanic_schemas"));
        assert!(names.contains(&"germanic_init"));
        assert!(names.contains(&"germanic_convert"));
    }

    #[test]
    fn test_inspect_params_deserialize() {
        let json = r#"{"file": "test.grm"}"#;
        let params: InspectParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.file, "test.grm");
        assert!(params.hex.is_none());
    }

    #[test]
    fn test_init_params_deserialize() {
        let json = r#"{"from": "example.json", "schema_id": "de.test.v1"}"#;
        let params: InitParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.from, "example.json");
        assert_eq!(params.schema_id, "de.test.v1");
        assert!(params.output.is_none());
    }

    #[test]
    fn test_convert_params_deserialize() {
        let json = r#"{"input": "schema.json", "output": "out.schema.json"}"#;
        let params: ConvertParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.input, "schema.json");
        assert_eq!(params.output, Some("out.schema.json".into()));
    }
}
