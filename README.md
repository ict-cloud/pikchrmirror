# PikchrMirror
This is a simple editor with preview and export functionality around the [Pikchr language](https://pikchr.org) written in [Rust](https://www.rust-lang.org/).

# Functions
The program has a split main screen with some basic functionality.

## Editor
A main editor presented at the left side of the screen. Showing line numbers.

## Preview
A preview area where the rendered SVG is shown as soon as the button for rendering is clicked.

## Render and export
A button to render the graphic is at the bottom. And the result can be exported as a PNG.

# Libraries
Following libraries are used to enable the main functionalities.

## Floem

[Floem](https://github.com/lapce/floem) is an UI library written in Rust originating from the [Lapce Editor](https://lapce.dev/).
Although the library is under heavy development I decided to use it, because it offers seamless integratione accross different platforms and operations modes. So far one of the best libraries I found.

## Pikchr
This is a simple wrapper around the official Pikchr library. Documented under https://docs.rs/pikchr/latest/pikchr/.