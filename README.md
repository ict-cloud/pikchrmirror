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

# Libraries
Following libraries are used to enable the main functionalities.

## iced
[iced](https://github.com/iced-rs/iced/) is a cross-platform GUI library for Rust.

## Pikchr
This is a simple wrapper around the official Pikchr library. Documented under https://docs.rs/pikchr/latest/pikchr/.
