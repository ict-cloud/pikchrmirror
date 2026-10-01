use std::panic::{catch_unwind, AssertUnwindSafe};

use base64::Engine;
use serde_json::{json, Value};

pub const SYNTAX_REFERENCE: &str = include_str!("syntax_reference.md");
const SYNTAX_URI: &str = "pikchr://syntax-reference";

const LATEST_PROTOCOL: &str = "2025-06-18";
const SUPPORTED_PROTOCOLS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];

/// Largest pikchr source accepted, to bound work done per call.
const MAX_SOURCE_BYTES: usize = 256 * 1024;

const INSTRUCTIONS: &str = "Pikchr diagram renderer. Write pikchr source, call render_pikchr to \
compile it, and if it reports an error (message, line, col) fix that spot and call it again. \
Call pikchr_syntax_reference (or read resource pikchr://syntax-reference) for a syntax cheat sheet.";

fn tool_definitions() -> Value {
    json!([
        {
            "name": "render_pikchr",
            "description": "Compile pikchr diagram source into SVG (and optionally PNG). \
    On a syntax error returns isError with message, line and col so the source can be corrected and retried.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "source": {
                        "type": "string",
                        "description": "Complete pikchr source for the diagram."
                    },
                    "png": {
                        "type": "boolean",
                        "description": "Also return a PNG (as an image and png_base64). Default false."
                    },
                    "scale": {
                        "type": "number",
                        "description": "PNG scale factor, default 2.0. Large values are rejected."
                    },
                    "include_svg": {
                        "type": "boolean",
                        "description": "Include the SVG markup in the result. Default true."
                    }
                },
                "required": ["source"]
            }
        },
        {
            "name": "pikchr_syntax_reference",
            "description": "Compact pikchr syntax cheat sheet with verified examples.",
            "inputSchema": { "type": "object", "properties": {} }
        }
    ])
}

fn rpc_result(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn rpc_error(id: Value, code: i64, message: impl Into<String>) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message.into() } })
}

fn text_result(text: impl Into<String>, is_error: bool) -> Value {
    json!({ "content": [{ "type": "text", "text": text.into() }], "isError": is_error })
}

/// Handles one raw JSON-RPC line. Returns the response to write, or `None` for notifications.
pub fn handle_message(line: &str) -> Option<Value> {
    let msg: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => return Some(rpc_error(Value::Null, -32700, format!("Parse error: {e}"))),
    };
    if !msg.is_object() {
        return Some(rpc_error(Value::Null, -32600, "Invalid Request"));
    }

    let id = msg.get("id").cloned();
    let method = msg.get("method").and_then(Value::as_str);
    let (Some(id), Some(method)) = (id, method) else {
        // Notifications (no id) and stray responses (no method) get no reply.
        return None;
    };
    let params = msg.get("params").cloned().unwrap_or(Value::Null);

    Some(match method {
        "initialize" => rpc_result(id, initialize(&params)),
        "ping" => rpc_result(id, json!({})),
        "tools/list" => rpc_result(id, json!({ "tools": tool_definitions() })),
        "tools/call" => rpc_result(id, call_tool(&params)),
        "resources/list" => rpc_result(
            id,
            json!({ "resources": [{
                "uri": SYNTAX_URI,
                "name": "Pikchr syntax reference",
                "description": "Compact pikchr syntax cheat sheet",
                "mimeType": "text/markdown"
            }] }),
        ),
        "resources/read" => match params.get("uri").and_then(Value::as_str) {
            Some(SYNTAX_URI) => rpc_result(
                id,
                json!({ "contents": [{
                    "uri": SYNTAX_URI,
                    "mimeType": "text/markdown",
                    "text": SYNTAX_REFERENCE
                }] }),
            ),
            Some(other) => rpc_error(id, -32002, format!("Resource not found: {other}")),
            None => rpc_error(id, -32602, "Missing 'uri'"),
        },
        other => rpc_error(id, -32601, format!("Method not found: {other}")),
    })
}

fn initialize(params: &Value) -> Value {
    let requested = params.get("protocolVersion").and_then(Value::as_str);
    let version = match requested {
        Some(v) if SUPPORTED_PROTOCOLS.contains(&v) => v,
        _ => LATEST_PROTOCOL,
    };
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": {}, "resources": {} },
        "serverInfo": { "name": "pikchr-mcp", "version": env!("CARGO_PKG_VERSION") },
        "instructions": INSTRUCTIONS
    })
}

fn call_tool(params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let args = params.get("arguments").cloned().unwrap_or(Value::Null);
    match name {
        "render_pikchr" => {
            // A rendering panic must not take down the whole server.
            catch_unwind(AssertUnwindSafe(|| render_pikchr(&args))).unwrap_or_else(|_| {
                text_result("Internal error while rendering the diagram.", true)
            })
        }
        "pikchr_syntax_reference" => text_result(SYNTAX_REFERENCE, false),
        other => text_result(format!("Unknown tool: {other}"), true),
    }
}

fn render_pikchr(args: &Value) -> Value {
    let Some(source) = args.get("source").and_then(Value::as_str) else {
        return text_result("Missing required string argument 'source'.", true);
    };
    if source.len() > MAX_SOURCE_BYTES {
        return text_result(
            format!(
                "Source is {} bytes; the limit is {MAX_SOURCE_BYTES}.",
                source.len()
            ),
            true,
        );
    }
    let want_png = args.get("png").and_then(Value::as_bool).unwrap_or(false);
    let include_svg = args
        .get("include_svg")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let scale = args.get("scale").and_then(Value::as_f64).map(|s| s as f32);

    let rendered = match pikchrmirror_core::render_svg(source) {
        Ok(r) => r,
        Err(e) => return compile_error_result(&e),
    };

    let mut structured = json!({ "width": rendered.width, "height": rendered.height });
    let mut content = Vec::new();

    if include_svg {
        structured["svg"] = json!(rendered.svg);
        content.push(json!({ "type": "text", "text": rendered.svg }));
    } else {
        content.push(json!({
            "type": "text",
            "text": format!("Rendered OK ({}x{}).", rendered.width, rendered.height)
        }));
    }

    if want_png {
        match pikchrmirror_core::try_svg_to_png(&rendered.svg, scale) {
            Ok(png) => {
                let b64 = base64::engine::general_purpose::STANDARD.encode(png);
                content.push(json!({ "type": "image", "data": b64, "mimeType": "image/png" }));
                structured["png_base64"] = json!(b64);
            }
            Err(e) => return text_result(format!("PNG rendering failed: {e}"), true),
        }
    }

    json!({ "content": content, "structuredContent": structured, "isError": false })
}

fn compile_error_result(e: &pikchrmirror_core::CompileError) -> Value {
    let position = match (e.line, e.col) {
        (Some(l), Some(c)) => format!(" at line {l}, col {c}"),
        (Some(l), None) => format!(" at line {l}"),
        _ => String::new(),
    };
    json!({
        "content": [{
            "type": "text",
            "text": format!("Pikchr error{position}: {}\n\n{}", e.message, e.context)
        }],
        "structuredContent": {
            "error": {
                "message": e.message,
                "line": e.line,
                "col": e.col,
                "context": e.context
            }
        },
        "isError": true
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(line: &str) -> Value {
        handle_message(line).expect("expected a response")
    }

    fn tool_call(args: Value) -> Value {
        let req = json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": { "name": "render_pikchr", "arguments": args }
        });
        call(&req.to_string())["result"].clone()
    }

    #[test]
    fn initialize_negotiates_version_and_advertises_capabilities() {
        let r = call(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}"#,
        );
        assert_eq!(r["id"], 1);
        assert_eq!(r["result"]["protocolVersion"], "2024-11-05");
        assert_eq!(r["result"]["serverInfo"]["name"], "pikchr-mcp");
        assert!(r["result"]["capabilities"]["tools"].is_object());
        assert!(r["result"]["capabilities"]["resources"].is_object());

        let r = call(
            r#"{"jsonrpc":"2.0","id":2,"method":"initialize","params":{"protocolVersion":"1999-01-01"}}"#,
        );
        assert_eq!(r["result"]["protocolVersion"], LATEST_PROTOCOL);
    }

    #[test]
    fn notifications_get_no_reply() {
        assert!(
            handle_message(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).is_none()
        );
    }

    #[test]
    fn malformed_json_is_a_parse_error() {
        let r = call("{not json");
        assert_eq!(r["error"]["code"], -32700);
        assert!(r["id"].is_null());
    }

    #[test]
    fn unknown_method_is_method_not_found() {
        let r = call(r#"{"jsonrpc":"2.0","id":7,"method":"nope"}"#);
        assert_eq!(r["error"]["code"], -32601);
        assert_eq!(r["id"], 7);
    }

    #[test]
    fn tools_list_has_both_tools_with_schemas() {
        let r = call(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#);
        let tools = r["result"]["tools"].as_array().unwrap();
        let names: Vec<_> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert_eq!(names, ["render_pikchr", "pikchr_syntax_reference"]);
        assert_eq!(tools[0]["inputSchema"]["required"][0], "source");
    }

    #[test]
    fn render_success_returns_svg_and_dimensions() {
        let r = tool_call(json!({ "source": "box \"hi\" fit" }));
        assert_eq!(r["isError"], false);
        let svg = r["structuredContent"]["svg"].as_str().unwrap();
        assert!(svg.contains("<svg"));
        assert!(r["structuredContent"]["width"].as_i64().unwrap() > 0);
        assert!(r["structuredContent"].get("png_base64").is_none());
        assert_eq!(r["content"][0]["text"], svg);
    }

    #[test]
    fn render_png_returns_valid_base64_png() {
        let r = tool_call(json!({ "source": "box \"hi\" fit", "png": true, "scale": 1.0 }));
        assert_eq!(r["isError"], false);
        let b64 = r["structuredContent"]["png_base64"].as_str().unwrap();
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .unwrap();
        assert_eq!(
            &bytes[..8],
            &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
        );
        assert_eq!(r["content"][1]["type"], "image");
        assert_eq!(r["content"][1]["mimeType"], "image/png");
    }

    #[test]
    fn include_svg_false_omits_markup() {
        let r = tool_call(json!({ "source": "box", "include_svg": false }));
        assert_eq!(r["isError"], false);
        assert!(r["structuredContent"].get("svg").is_none());
    }

    #[test]
    fn compile_error_reports_position() {
        let r = tool_call(json!({ "source": "box\narrow\nbogus_token_here\n" }));
        assert_eq!(r["isError"], true);
        let err = &r["structuredContent"]["error"];
        assert_eq!(err["line"], 3);
        assert!(err["col"].is_u64());
        assert!(!err["message"].as_str().unwrap().is_empty());
        assert!(r["content"][0]["text"].as_str().unwrap().contains("line 3"));
    }

    #[test]
    fn bad_arguments_are_tool_errors_not_protocol_errors() {
        let r = tool_call(json!({}));
        assert_eq!(r["isError"], true);
        let r = tool_call(json!({ "source": "x".repeat(MAX_SOURCE_BYTES + 1) }));
        assert_eq!(r["isError"], true);
        let r = tool_call(json!({ "source": "box", "png": true, "scale": 1.0e9 }));
        assert_eq!(r["isError"], true);
    }

    #[test]
    fn unknown_tool_is_a_tool_error() {
        let r = call(r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"nope"}}"#);
        assert_eq!(r["result"]["isError"], true);
    }

    #[test]
    fn syntax_reference_available_as_tool_and_resource() {
        let r = call(
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"pikchr_syntax_reference"}}"#,
        );
        assert!(r["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Pikchr syntax"));

        let r = call(r#"{"jsonrpc":"2.0","id":2,"method":"resources/list"}"#);
        assert_eq!(r["result"]["resources"][0]["uri"], SYNTAX_URI);

        let r = call(
            r#"{"jsonrpc":"2.0","id":3,"method":"resources/read","params":{"uri":"pikchr://syntax-reference"}}"#,
        );
        assert!(r["result"]["contents"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Pikchr syntax"));

        let r = call(
            r#"{"jsonrpc":"2.0","id":4,"method":"resources/read","params":{"uri":"pikchr://other"}}"#,
        );
        assert_eq!(r["error"]["code"], -32002);
    }

    #[test]
    fn every_example_in_the_syntax_reference_renders() {
        let mut in_block = false;
        let mut block = String::new();
        let mut count = 0;
        for line in SYNTAX_REFERENCE.lines() {
            if line.starts_with("```pikchr") {
                in_block = true;
                block.clear();
            } else if line.starts_with("```") && in_block {
                in_block = false;
                count += 1;
                if let Err(e) = pikchrmirror_core::render_svg(&block) {
                    panic!(
                        "reference example #{count} failed: {}\n{}",
                        e.message, e.context
                    );
                }
            } else if in_block {
                block.push_str(line);
                block.push('\n');
            }
        }
        assert!(count >= 5, "expected several examples, found {count}");
    }
}
