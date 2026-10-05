//! Pikchr MCP server logic, plus the optional local diagram-writing model.
//!
//! `server` is the protocol handler used by the `pikchr-mcp` binary. With the
//! `llm` feature, `llm` loads a local model that the `generate_pikchr` tool (and
//! the desktop app's chat) use to write diagrams.

pub mod server;

#[cfg(feature = "llm")]
pub mod llm;
