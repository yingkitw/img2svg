//! img2svg MCP (Model Context Protocol) server
//!
//! This is a standalone MCP server binary that exposes img2svg functionality
//! through the Model Context Protocol, allowing AI assistants to convert images
//! to SVG format.

use img2svg::{
    generate_svg, load_image, vectorize, vectorize_enhanced, write_enhanced_svg,
    EnhancedOptions,
};
use img2svg::image_processor::resize_if_needed;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::Path;

/// Send a JSON-RPC notification for progress updates.
fn send_progress_notification<W: Write>(
    writer: &mut W,
    progress_token: &str,
    progress: f64,
    total: f64,
) -> io::Result<()> {
    let msg = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/progress",
        "params": {
            "progressToken": progress_token,
            "progress": progress,
            "total": total
        }
    });
    writeln!(writer, "{}", msg)?;
    writer.flush()
}

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

struct ConversionInfo {
    width: u32,
    height: u32,
    colors_used: usize,
    pipeline: &'static str,
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
                                },
                                "preprocess": {
                                    "type": "boolean",
                                    "description": "Apply edge-preserving smoothing and color reduction before vectorization. Uses the enhanced Bézier pipeline when enabled.",
                                    "default": false
                                },
                                "max_size": {
                                    "type": "integer",
                                    "description": "Maximum image dimension in pixels. Images larger than this will be auto-resized (Lanczos3) to prevent OOM. Default: 4096.",
                                    "minimum": 64,
                                    "maximum": 16384,
                                    "default": 4096
                                },
                                "progress_token": {
                                    "type": "string",
                                    "description": "Token for receiving real-time progress notifications during conversion."
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
                let preprocess = args.get("preprocess").and_then(|v| v.as_bool()).unwrap_or(false);
                let max_size = args.get("max_size").and_then(|v| v.as_i64()).unwrap_or(4096) as u32;
                let progress_token = args.get("progress_token").and_then(|v| v.as_str());

                match (input_path, output_path) {
                    (Some(input), Some(output)) => {
                        let mut notify = |stage: f64, total: f64| {
                            if let Some(token) = progress_token {
                                let _ = send_progress_notification(
                                    &mut io::stdout(),
                                    token,
                                    stage,
                                    total,
                                );
                            }
                        };

                        let result = if preprocess {
                            notify(0.0, 100.0);
                            match load_image(Path::new(input)) {
                                Ok(image_data) => {
                                    notify(20.0, 100.0);
                                    let image_data = resize_if_needed(image_data, max_size);
                                    notify(40.0, 100.0);
                                    let opts = EnhancedOptions {
                                        num_colors,
                                        preprocess: true,
                                        ..Default::default()
                                    };
                                    match vectorize_enhanced(&image_data, &opts) {
                                        Ok(data) => {
                                            notify(70.0, 100.0);
                                            let info = ConversionInfo {
                                                width: data.width,
                                                height: data.height,
                                                colors_used: data.paths.len(),
                                                pipeline: "enhanced",
                                            };
                                            match write_enhanced_svg(&data, Path::new(output)) {
                                                Ok(()) => {
                                                    notify(100.0, 100.0);
                                                    Ok(info)
                                                }
                                                Err(e) => Err(format!("SVG write failed: {}", e)),
                                            }
                                        }
                                        Err(e) => Err(format!("Vectorization failed: {}", e)),
                                    }
                                }
                                Err(e) => Err(format!("Image load failed: {}", e)),
                            }
                        } else {
                            notify(0.0, 100.0);
                            match load_image(Path::new(input)) {
                                Ok(image_data) => {
                                    notify(25.0, 100.0);
                                    let image_data = resize_if_needed(image_data, max_size);
                                    notify(50.0, 100.0);
                                    match vectorize(&image_data, num_colors, threshold, smooth_level, false) {
                                        Ok(vectorized_data) => {
                                            notify(75.0, 100.0);
                                            match generate_svg(&vectorized_data, Path::new(output)) {
                                                Ok(()) => {
                                                    notify(100.0, 100.0);
                                                    Ok(ConversionInfo {
                                                        width: image_data.width,
                                                        height: image_data.height,
                                                        colors_used: vectorized_data.curves.len(),
                                                        pipeline: "legacy",
                                                    })
                                                }
                                                Err(e) => Err(format!("SVG write failed: {}", e)),
                                            }
                                        }
                                        Err(e) => Err(format!("Vectorization failed: {}", e)),
                                    }
                                }
                                Err(e) => Err(format!("Image load failed: {}", e)),
                            }
                        };

                        match result {
                            Ok(info) => {
                                let output_size = std::fs::metadata(output)
                                    .map(|m| m.len())
                                    .unwrap_or(0);
                                McpResponse {
                                    jsonrpc: "2.0".to_string(),
                                    id,
                                    result: Some(json!({
                                        "content": [
                                            {
                                                "type": "text",
                                                "text": format!("Successfully converted {} to {}", input, output)
                                            }
                                        ],
                                        "details": {
                                            "pipeline": info.pipeline,
                                            "output_size_bytes": output_size,
                                            "width": if info.width > 0 { Some(info.width) } else { None::<u32> },
                                            "height": if info.height > 0 { Some(info.height) } else { None::<u32> },
                                            "colors_used": info.colors_used
                                        }
                                    })),
                                    error: None,
                                }
                            }
                            Err(msg) => McpResponse {
                                jsonrpc: "2.0".to_string(),
                                id,
                                result: None,
                                error: Some(McpError {
                                    code: -32000,
                                    message: msg,
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

#[cfg(test)]
mod tests {
    use super::Img2SvgMcpServer;
    use serde_json::json;

    #[test]
    fn initialize_returns_server_info() {
        let s = Img2SvgMcpServer;
        let r = s.handle_initialize(json!({}), json!(1));
        assert!(r.error.is_none());
        let res = r.result.unwrap();
        assert_eq!(res["protocolVersion"], "2024-11-05");
        assert_eq!(res["serverInfo"]["name"], "img2svg");
    }

    #[test]
    fn tools_list_exposes_convert_tool() {
        let s = Img2SvgMcpServer;
        let r = s.handle_tools_list(json!(42));
        assert!(r.error.is_none());
        let body = r.result.unwrap();
        let tools = body["tools"].as_array().expect("tools array");
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "convert_image_to_svg");
        let required = tools[0]["inputSchema"]["required"]
            .as_array()
            .expect("required");
        assert!(required.contains(&json!("input_path")));
    }

    #[test]
    fn tools_call_rejects_non_object_arguments() {
        let s = Img2SvgMcpServer;
        let r = s.handle_tools_call(
            super::ToolCallParams {
                name: "convert_image_to_svg".to_string(),
                arguments: json!("not-an-object"),
            },
            json!(1),
        );
        assert!(r.result.is_none());
        let err = r.error.unwrap();
        assert_eq!(err.code, -32602);
    }

    #[test]
    fn tools_call_missing_paths_returns_error() {
        let s = Img2SvgMcpServer;
        let r = s.handle_tools_call(
            super::ToolCallParams {
                name: "convert_image_to_svg".to_string(),
                arguments: json!({ "num_colors": 8 }),
            },
            json!(2),
        );
        assert!(r.result.is_none());
        assert_eq!(r.error.unwrap().code, -32602);
    }

    #[test]
    fn tools_call_unknown_tool() {
        let s = Img2SvgMcpServer;
        let r = s.handle_tools_call(
            super::ToolCallParams {
                name: "nope".to_string(),
                arguments: json!({}),
            },
            json!(3),
        );
        assert_eq!(r.error.unwrap().code, -32601);
    }

    #[test]
    fn tools_call_convert_succeeds_for_example_png() {
        let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/input/simple.png");
        if !input.exists() {
            return;
        }
        let out = std::env::temp_dir().join("img2svg_mcp_convert_test.svg");
        let _ = std::fs::remove_file(&out);

        let s = Img2SvgMcpServer;
        let r = s.handle_tools_call(
            super::ToolCallParams {
                name: "convert_image_to_svg".to_string(),
                arguments: json!({
                    "input_path": input.to_str().unwrap(),
                    "output_path": out.to_str().unwrap(),
                    "num_colors": 8,
                }),
            },
            json!(99),
        );

        assert!(r.error.is_none(), "{:?}", r.error);
        assert!(out.exists());
        let text = std::fs::read_to_string(&out).expect("read output");
        assert!(text.contains("<svg"));
        // Check details field is present
        let res = r.result.unwrap();
        assert!(res["details"].is_object());
        assert_eq!(res["details"]["pipeline"], "legacy");
        assert!(res["details"]["output_size_bytes"].as_u64().unwrap_or(0) > 0);
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn tools_call_convert_with_preprocess_uses_enhanced_pipeline() {
        let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/input/simple.png");
        if !input.exists() {
            return;
        }
        let out = std::env::temp_dir().join("img2svg_mcp_preprocess_test.svg");
        let _ = std::fs::remove_file(&out);

        let s = Img2SvgMcpServer;
        let r = s.handle_tools_call(
            super::ToolCallParams {
                name: "convert_image_to_svg".to_string(),
                arguments: json!({
                    "input_path": input.to_str().unwrap(),
                    "output_path": out.to_str().unwrap(),
                    "num_colors": 8,
                    "preprocess": true,
                }),
            },
            json!(100),
        );

        assert!(r.error.is_none(), "{:?}", r.error);
        assert!(out.exists());
        let text = std::fs::read_to_string(&out).expect("read output");
        assert!(text.contains("<svg"));
        // Check enhanced pipeline was used
        let res = r.result.unwrap();
        assert_eq!(res["details"]["pipeline"], "enhanced");
        assert!(res["details"]["width"].is_number());
        assert!(res["details"]["height"].is_number());
        assert!(res["details"]["colors_used"].is_number());
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn send_progress_notification_format() {
        let mut buf = Vec::new();
        super::send_progress_notification(&mut buf, "token-123", 50.0, 100.0).unwrap();
        let text = String::from_utf8(buf).unwrap();
        let msg: serde_json::Value = serde_json::from_str(text.trim()).unwrap();
        assert_eq!(msg["jsonrpc"], "2.0");
        assert_eq!(msg["method"], "notifications/progress");
        assert_eq!(msg["params"]["progressToken"], "token-123");
        assert_eq!(msg["params"]["progress"], 50.0);
        assert_eq!(msg["params"]["total"], 100.0);
    }

    #[test]
    fn tools_call_convert_with_progress_token_succeeds() {
        let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/input/simple.png");
        if !input.exists() {
            return;
        }
        let out = std::env::temp_dir().join("img2svg_mcp_progress_test.svg");
        let _ = std::fs::remove_file(&out);

        let s = Img2SvgMcpServer;
        let r = s.handle_tools_call(
            super::ToolCallParams {
                name: "convert_image_to_svg".to_string(),
                arguments: json!({
                    "input_path": input.to_str().unwrap(),
                    "output_path": out.to_str().unwrap(),
                    "num_colors": 8,
                    "progress_token": "test-token-42",
                }),
            },
            json!(102),
        );

        assert!(r.error.is_none(), "{:?}", r.error);
        assert!(out.exists());
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn tools_call_convert_with_max_size_limits_dimensions() {
        let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/input/simple.png");
        if !input.exists() {
            return;
        }
        let out = std::env::temp_dir().join("img2svg_mcp_maxsize_test.svg");
        let _ = std::fs::remove_file(&out);

        let s = Img2SvgMcpServer;
        let r = s.handle_tools_call(
            super::ToolCallParams {
                name: "convert_image_to_svg".to_string(),
                arguments: json!({
                    "input_path": input.to_str().unwrap(),
                    "output_path": out.to_str().unwrap(),
                    "num_colors": 8,
                    "max_size": 32,
                }),
            },
            json!(101),
        );

        assert!(r.error.is_none(), "{:?}", r.error);
        assert!(out.exists());
        let res = r.result.unwrap();
        let width = res["details"]["width"].as_u64().unwrap_or(0);
        let height = res["details"]["height"].as_u64().unwrap_or(0);
        assert!(
            width <= 32 || height <= 32,
            "Expected dimensions <= 32 after resize, got {}x{}",
            width,
            height
        );
        let _ = std::fs::remove_file(&out);
    }

}
