# Autonomous Agent and Developer Guidelines (AGENTS.md)

Welcome! This repository, `pikchrmirror`, uses a structured development workflow. Whether you are a human contributor or an autonomous AI development agent, you **must** strictly adhere to the rules, workflow, and guidelines outlined in this document.

---

## 🤖 AI Agent Persona & Scope
- **Context:** `pikchrmirror` is a mirror/utility tool related to Pikchr (a PIC-like diagram language).
- **Primary Goal:** Maintain codebase hygiene, adhere strictly to Git workflows, and write clean, documented, and tested code.
- **Validation:** Always verify existing code patterns and test suites before making structural changes.

---

## Repository Structure
  src/
    main.rs          — app entry point
    editor.rs        — code editor panel
    preview.rs       — SVG preview panel
    export.rs        — PNG export logic
    pikchr/          — FFI wrapper around pikchr C library
    
---

  ## Build & Run
  cargo build
  cargo run
  
---

  ## Testing
  cargo test
  cargo clippy --all-targets -- -D warnings
  cargo fmt --check
  
---

  ## Key Dependencies
  - iced: GUI framework (reactive, Elm-like architecture)
  - pikchr (C library): wrapped via FFI in src/pikchr/
  
---

  ## Architecture Notes
  The app uses iced's update/view pattern. State lives in the top-level
  App struct. Editor and preview panels communicate through Messages.

## 🔄 Git Workflow & Branching Strategy

We follow a modified Git Flow model where `develop` is the central integration branch. 

### Core Rules:
1. **Never** commit directly to the `main` or `develop` branches.
2. **Development Entry Point:** All development and feature work **must** start by branching off from the latest `develop` branch.
3. **Integration Point:** Once a feature, bug fix, or chore is complete, it must be integrated back into the `develop` branch via a Pull Request (PR).

### Branch Naming Convention
When creating a new branch from `develop`, use the following prefixes:
- `feature/` for new features or enhancements (e.g., `feature/add-view-box`)
- `bugfix/` for fixing issues (e.g., `bugfix/resolve-parser-crash`)
- `chore/` for maintenance, CI/CD, or documentation updates (e.g., `chore/update-dependencies`)

### Step-by-Step Workflow Example for Agents:
```text
[develop] ───► Create branch (feature/my-feature) ───► Make commits ───► Pull Request ───► [develop]
```

---

  ## Agent Guidelines
  - Always run `cargo clippy` and `cargo fmt` before considering a change done
  - The pikchr FFI wrapper must uphold memory safety — avoid raw pointer changes without careful review
  - SVG rendering is done inline; do not introduce external HTTP calls

---
