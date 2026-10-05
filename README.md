# PikchrMirror
This is a simple editor with preview and export functionality around the [Pikchr language](https://pikchr.org) written in [Rust](https://www.rust-lang.org/).

# Functions
The program has a split main screen with some basic functionality.

## Editor
A main editor presented at the left side of the screen, with syntax highlighting and line numbers. As you type, the preview is updated in real-time.

## Preview
A preview area on the right side of the screen where the rendered SVG is shown.

## File Operations
- **New**: Clear the editor to start a new diagram.
- **Open**: Open a `.pikchr` file from your computer.
- **Save**: Save the current diagram source as a `.pikchr` or `.txt` file.
- **Export**: Export the rendered diagram. Click the Export button to reveal format options, then choose **SVG** or **PNG**. For PNG exports, select the desired resolution from the quality picker (**1x**, **2x**, **4x**) — `2x` is the default and produces a high-DPI image at twice the diagram's natural size.

## Theming
You can choose between different editor themes.

## Chat assistant (optional)
Built with `--features llm`, the toolbar gets a **Chat** button that opens the assistant in its **own window**, next to the editor. Describe a diagram, then use **Apply to editor** to take the generated code. The window can be closed and reopened independently; closing the editor window quits the app. The model runs locally and is selected in `models.toml`; it is downloaded at build time, not stored in git. See [WEIGHTS.md](WEIGHTS.md).

# MCP server (`pikchr-mcp`)
`pikchr-mcp` lets any [MCP](https://modelcontextprotocol.io)-capable LLM client (Claude Desktop, Claude Code, IDE agents, ...) render pikchr diagrams as a tool, so the model writes the diagram and no model needs to be embedded in the app. It speaks MCP over stdio.

Build and register it:
```
cargo build --release -p pikchrmirror-mcp
claude mcp add pikchr -- /path/to/target/release/pikchr-mcp
```
For clients configured through JSON, use `{"mcpServers": {"pikchr": {"command": "/path/to/pikchr-mcp"}}}`.

Tools:
- `render_pikchr(source, png?, scale?, include_svg?)` compiles the source and returns the SVG (plus a PNG image when `png` is true). On a syntax error it returns `isError` with `error: {message, line, col, context}` so the model can fix that spot and retry.
- `pikchr_syntax_reference()` returns a compact syntax cheat sheet. The same text is available as the resource `pikchr://syntax-reference`.

The rendering logic lives in the `pikchrmirror-core` crate, shared with the GUI.

# Libraries
Following libraries are used to enable the main functionalities.

## iced
[iced](https://github.com/iced-rs/iced/) is a cross-platform GUI library for Rust.

## Pikchr
This is a simple wrapper around the official Pikchr library. Documented under https://docs.rs/pikchr/latest/pikchr/.
