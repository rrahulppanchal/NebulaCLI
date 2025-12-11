# Contributing to NebulaCLI

First off, thanks for taking the time to contribute! 🎉

The following is a set of guidelines for contributing to NebulaCLI. These were inspired by the open-source community and are meant to make the contribution process as smooth as possible.

## 🛠️ Development Setup

### 1. Prerequisites
*   [Rust and Cargo](https://rustup.rs/) (latest stable version).
*   **Windows OS** (required for building the installer and full feature testing).

### 2. Initial Setup
Clone the repository:
```bash
git clone https://github.com/your-username/nebula-cli.git
cd nebula-cli
```

### 3. Build Instructions (Important!) ⚠️
NebulaCLI consists of two parts: the **Terminal** (the actual app) and the **Installer** (the setup wizard). 

**You must build the Terminal first**, as the Installer embeds the compiled terminal binary.

1.  **Build the Terminal (Release Mode)**:
    ```bash
    cargo build --release --bin terminal
    ```
    *This creates `target/release/terminal.exe`.*

2.  **Build the Installer**:
    ```bash
    cargo build --release --bin installer
    ```
    *This embeds the `terminal.exe` into the installer.*

### 4. Running Locally
To just run the terminal during development:
```bash
cargo run --bin terminal
```

## 🎨 Assets & Icons
*   **logo.ico**: Used for the executable icon (Taskbar/Explorer). This is required for `build.rs` to embed the icon on Windows.
*   **src/bin/logo.png**: Used by the Installer wizard UI.

If you are changing the logo, ensure both files are updated. `logo.ico` should be in the project root.

## 🐛 Reporting Bugs

Bugs are tracked as GitHub issues. When filing an issue:
*   Use a clear and descriptive title.
*   Describe the exact steps which reproduce the problem.
*   Provide specific examples to demonstrate the steps.
*   Describe the behavior you observed after following the steps and explain the behavior you expected to see.

## 💡 Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues.
*   Use a clear and descriptive title for the issue to identify the suggestion.
*   Provide a step-by-step description of the suggested enhancement.
*   Explain why this enhancement would be useful to most NebulaCLI users.

## 📥 Pull Requests

1.  Fork the repo and create your branch from `main`.
2.  If you've added code that should be tested, add tests.
3.  Ensure the test suite passes.
4.  Make sure your code lints.
5.  Issue that pull request!

## 🔖 Style Guide

*   Run `cargo fmt` before committing to ensure your code follows standard Rust formatting.
*   Run `cargo clippy` to catch common mistakes and improve code quality.

## 📜 License

By contributing, you agree that your contributions will be licensed under its MIT License.
