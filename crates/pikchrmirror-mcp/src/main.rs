//! `pikchr-mcp`: a stdio MCP server that renders pikchr diagrams.
//!
//! The protocol uses newline-delimited JSON-RPC on stdin/stdout, so nothing but
//! protocol messages may ever be written to stdout; diagnostics go to stderr.

mod server;

use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(response) = server::handle_message(&line) {
            writeln!(stdout, "{response}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}
