# AGENTS.md - AI Agent Integration Guide

This document provides detailed information for AI agents and agent frameworks on how to integrate with and use the PikchrMirror project.

## Overview

PikchrMirror is an open source project that aims to provide a graphical user interfact to develop graphics with the pikchr language (https://pikchr.org).
The main development language is Rust. The interface is using the iced framework.

## Main features

The user interface is a split view with a left panel for editing the pikchr code and a right panel for previewing the resulting graphics as SVG. The preview is automatically updated whenever the code changes.
The project also contains a wrapper around the pikchr library to export PNG images at a dedicated size.
This leaves following basic functions:

- Create new files to create pikchr code
- Save pikchr files
- Save SVG images
- Export PNG images
- Load existing pikchr files

## Main User Interface

The main user interface consists of an editor panel on the left side and a preview panel on the right side. A tab bar is on the top accross both panels.

### Tab bar functions

- Create new file
- Save SVG image
- Load existing file
- Switch theme of the main window
