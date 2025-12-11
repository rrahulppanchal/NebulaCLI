# NebulaCLI

**NebulaCLI** is a modern, high-performance terminal emulator built in Rust. Designed to provide a premium command-line experience, it combines the speed of a TUI with advanced features like tabbed browsing, modal editing, and seamless Windows integration.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/built_with-Rust-d66761.svg)

## 🚀 Features

*   **Multi-Tab Support**: Effortlessly manage multiple terminal sessions within a single window.
*   **Modal Editing**: Switch between **Insert** (typing) and **Normal** (navigation/selection) modes, similar to Vim.
*   **Rich User Interface**:
    *   Custom status indicators.
    *   Visual tab management.
    *   Dynamic prompt with user and directory information.
*   **Clipboard Integration**: Seamless copy/paste operations (`Ctrl+C`, `Ctrl+V`, `y`).
*   **Windows Integration**: 
    *   Context menu support ("Open NebulaCLI here").
    *   Start Menu and Desktop shortcuts.
    *   PATH integration.
*   **High Performance**: Built on `ratatui` and `crossterm` for low latency rendering.

## 📥 Download

**Experience the future of command line interfaces today.**

[![Download Installer](https://img.shields.io/badge/Download-Installer-blue.svg?style=for-the-badge&logo=windows)](https://github.com/rrahulppanchal/NebulaCLI/blob/development/installer.exe)

> **Note**: This installer will automatically configure your environment, shortcuts, and context menus.

### Portable Version
If you prefer a portable version, you can download the [Standalone Executable](https://github.com/rrahulppanchal/NebulaCLI/blob/development/installer.exe).

## 🛠️ Installation

### Using the Installer (Windows)

1.  Download the latest release or build the installer from source.
2.  Run `installer.exe`.
3.  Follow the setup wizard to configure your username and installation path.
4.  NebulaCLI will be added to your desktop, start menu, and context menus.

### Building from Source

Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed.

1.  **Build the Terminal**:
    ```bash
    cargo build --release --bin terminal
    ```

2.  **Build the Installer (Optional)**:
    *Note: The installer requires the terminal binary to be built first.*
    ```bash
    cargo build --release --bin installer
    ```

3.  **Run**:
    ```bash
    cargo run --release --bin terminal
    ```

## ⌨️ Shortcuts & Usage

| Action | Shortcut / Command | Mode |
| :--- | :--- | :--- |
| **General** | | |
| Quit | `Ctrl + q` | All |
| New Tab | `Ctrl + t` | All |
| Close Tab | `Ctrl + w` | All |
| Next Tab | `Ctrl + Tab` | All |
| Previous Tab | `Ctrl + Shift + Tab` | All |
| Clear Output | `Ctrl + l` | All |
| **Insert Mode** | | |
| Type Command | (Typing) | Insert |
| Execute | `Enter` | Insert |
| Paste | `Ctrl + v` | Insert |
| Switch to Normal Mode | `Esc` | Insert |
| **Normal Mode** | | |
| Switch to Insert Mode | `i` | Normal |
| Visual Selection | `v` | Normal |
| Copy Selection | `y` or `Ctrl + c` | Normal |
| Scroll Up/Down | `Up`/`Down` or `k`/`j` | Normal |
| Clear Selection | `Esc` | Normal |

### Tab Renaming
You can rename a tab by clicking on its title in the tab bar. Type the new name and press `Enter`.

## 🤝 Contributing

Contributions are welcome! Please read our [Contribution Guidelines](CONTRIBUTING.md) for details on how to submit pull requests, report issues, and suggest improvements.

## 📄 License

This project is open-sourced under the [MIT License](LICENSE).
