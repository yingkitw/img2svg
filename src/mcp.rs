//! img2svg MCP (Model Context Protocol) server
//!
//! This is a standalone MCP server binary that exposes img2svg functionality
//! through the Model Context Protocol, allowing AI assistants to convert images
//! to SVG format.

use img2svg::{convert, ConversionOptions};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::Path;

/// MCP Request structure
#[derive(Debug, serde::Deserialize)]
struct McpRequest {
    #[serde(default)]
    #[allow(dead_code)]
    jsonrpc: String,
    #[serde(flatten)]
    kind: RequestKind,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum RequestKind {
    Initialize { id: Value, params: Value },
    ToolsList { id: Value },
    ToolsCall { id: Value, params: ToolCallParams },
}

#[derive(Debug, serde::Deserialize)]
struct ToolCallParams {
    name: String,
    #[serde(default)]
    arguments: Value,
}

/// MCP Response structure
#[derive(Debug, serde::Serialize)]
struct McpResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<McpError>,
}

#[derive(Debug, serde::Serialize)]
struct McpError {
    code: i32,
    message: String,
}

struct Img2SvgMcpServer;

impl Img2SvgMcpServer {
    fn handle_initialize(&self, _params: Value, id: Value) -> McpResponse {
        McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "img2svg",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "capabilities": {
                    "tools": {
                        "listChanged": false
                    }
                }
            })),
            error: None,
        }
    }

    fn handle_tools_list(&self, id: Value) -> McpResponse {
        McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "tools": [
                    {
                        "name": "convert_image_to_svg",
                        "description": "Convert a raster image (PNG, JPEG, etc.) to SVG format. This tool transforms pixel-based images into scalable vector graphics using color quantization, marching squares contour tracing, and path simplification algorithms.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "input_path": {
                                    "type": "string",
                                    "description": "Path to the input image file (PNG, JPEG, etc.)"
                                },
                                "output_path": {
                                    "type": "string",
                                    "description": "Path where the SVG output will be saved"
                                },
                                "num_colors": {
                                    "type": "integer",
                                    "description": "Number of colors for quantization (1-64, default: 16). More colors preserve more detail but increase file size.",
                                    "minimum": 1,
                                    "maximum": 64,
                                    "default": 16
                                },
                                "smooth_level": {
                                    "type": "integer",
                                    "description": "Path smoothing level (0-10, default: 5). Higher values create smoother curves but may lose sharp details.",
                                    "minimum": 0,
                                    "maximum": 10,
                                    "default": 5
                                },
                                "threshold": {
                                    "type": "number",
                                    "description": "Edge detection threshold (0.0-1.0, default: 0.1). Lower values detect more edges.",
                                    "minimum": 0.0,
                                    "maximum": 1.0,
                                    "default": 0.1
                                }
                            },
                            "required": ["input_path", "output_path"]
                        }
                    }
                ]
            })),
            error: None,
        }
    }

    fn handle_tools_call(&self, params: ToolCallParams, id: Value) -> McpResponse {
        match params.name.as_str() {
            "convert_image_to_svg" => {
                let args = if let Value::Object(map) = params.arguments {
                    map
                } else {
                    return McpResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(McpError {
                            code: -32602,
                            message: "Invalid arguments: expected object".to_string(),
                        }),
                    };
                };

                let input_path = args.get("input_path").and_then(|v| v.as_str());
                let output_path = args.get("output_path").and_then(|v| v.as_str());
                let num_colors = args.get("num_colors").and_then(|v| v.as_i64()).unwrap_or(16) as usize;
                let smooth_level = args.get("smooth_level").and_then(|v| v.as_i64()).unwrap_or(5) as u8;
                let threshold = args.get("threshold").and_then(|v| v.as_f64()).unwrap_or(0.1);

                match (input_path, output_path) {
                    (Some(input), Some(output)) => {
                        let options = ConversionOptions {
                            num_colors,
                            smooth_level,
                            threshold,
                            hierarchical: false,
                            advanced: false,
                        };

                        match convert(Path::new(input), Path::new(output), &options) {
                            Ok(()) => McpResponse {
                                jsonrpc: "2.0".to_string(),
                                id,
                                result: Some(json!({
                                    "content": [
                                        {
                                            "type": "text",
                                            "text": format!("Successfully converted {} to {}", input, output)
                                        }
                                    ]
                                })),
                                error: None,
                            },
                            Err(e) => McpResponse {
                                jsonrpc: "2.0".to_string(),
                                id,
                                result: None,
                                error: Some(McpError {
                                    code: -32000,
                                    message: format!("Conversion failed: {}", e),
                                }),
                            },
                        }
                    }
                    _ => McpResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(McpError {
                            code: -32602,
                            message: "Missing required parameters: input_path and output_path".to_string(),
                        }),
                    },
                }
            }
            _ => McpResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: format!("Unknown tool: {}", params.name),
                }),
            },
        }
    }

    fn run(&self) {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut stdout_lock = stdout.lock();

        for line in stdin.lock().lines() {
            if let Ok(json_str) = line {
                if let Ok(req) = serde_json::from_str::<McpRequest>(&json_str) {
                    let response = match req.kind {
                        RequestKind::Initialize { id, params } => self.handle_initialize(params, id),
                        RequestKind::ToolsList { id } => self.handle_tools_list(id),
                        RequestKind::ToolsCall { id, params } => self.handle_tools_call(params, id),
                    };

                    if let Ok(response_json) = serde_json::to_string(&response) {
                        writeln!(stdout_lock, "{}", response_json).ok();
                        stdout_lock.flush().ok();
                    }
                }
            }
        }
    }
}

fn main() {
    let server = Img2SvgMcpServer;
    server.run();
}
