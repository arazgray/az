<p align="center">
  <img src="assets/logo.svg" alt="az Logo" height="64"/>
  <h1 align="center">az</h1>
</p>

<p align="center">
  <strong>A fast, lightweight terminal text editor built in Rust.<br>
  Keyboard-first, zero-configuration, and designed to stay out of your way.</strong><br>
  <a href="USER_MANUAL.md">User Manual</a> |
  <a href="CHANGELOG.md">Changelog</a> |
  <a href="AGENTS.md">AI Agent Guide</a>
</p>

<p align="center">
  <a href="https://github.com/arazgray/az/releases">
    <img src="https://img.shields.io/github/v/release/arazgray/az?label=version&color=brightgreen" alt="Version">
  </a>
  <img src="https://img.shields.io/badge/license-WTFPL-blue.svg" alt="License: WTFPL">
  <img src="https://img.shields.io/github/last-commit/arazgray/az" alt="Last Commit">
  <img src="https://img.shields.io/github/languages/top/arazgray/az" alt="Top Language">
</p>

<p align="center">
  <img src="screenshot.png" alt="az screenshot">
</p>

---

## Installation

### One-line installer

The easiest way to install `az` is with the official installer:

```sh
curl -fsSL https://raw.githubusercontent.com/arazgray/az/refs/heads/main/install.sh | sh
```

The installer automatically detects your operating system and installs the appropriate prebuilt package.

No Rust toolchain or compilation is required.

After installation:

```sh
az
```

> **Update checks:** `az` automatically checks the main repository for newer releases when it starts. Checks are skipped when offline. Set `AZ_NO_UPDATE_CHECK=1` to disable update checks.

---

## Packages

Prebuilt packages are available for Linux and Windows.

| Operating System | Package                                                                                                           | Architecture |    Size | SHA-256                                                            |
| ---------------- | ----------------------------------------------------------------------------------------------------------------- | ------------ | ------: | ------------------------------------------------------------------ |
| Linux            | [**az-4.0.0-linux-amd64**](https://github.com/arazgray/az/releases/download/4.0/az-4.0.0-linux-amd64)             | x86_64       | 1.23 MB | `eda877d465593f0501c41e1aa5b916dccf22e1a989af95db99bd8601c061cf73` |
| Windows          | [**az-4.0.0-windows-amd64.exe**](https://github.com/arazgray/az/releases/download/4.0/az-4.0.0-windows-amd64.exe) | x86_64       | 1.18 MB | `93588bfc89ac07221732958c9d73626952490a5b6e1099f1cc82ae8e8543d310` |
| Debian / Ubuntu  | [**az_4.0.0_amd64.deb**](https://github.com/arazgray/az/releases/download/4.0/az_4.0.0_amd64.deb)                 | x86_64       |  475 KB | `227756b78fc8ba523f7a5b4cfde95eed57a2cb5e7dee7e21145da51d0816d2df` |

### Debian / Ubuntu

To install the `.deb` package manually:

```sh
sudo dpkg -i az_4.0.0_amd64.deb
```

---

## Quick Start

Open a file:

```sh
az file.php
```

Open a project:

```sh
az project/
```

Open a file at a specific line:

```sh
az file.php:20
```

Create a new file:

```sh
az newfile.txt
```

Show help:

```sh
az --help
```

---

## Why az?

`az` is designed for quick edits, small projects, server work, focused writing, and terminal-based code changes.

No configuration maze. No plugin setup required for everyday editing. Just open a file and start working.

### Core capabilities

* Written entirely in Rust
* Fast startup and lightweight runtime
* Keyboard-first workflow
* Full mouse support
* Files and folders
* Project tree sidebar
* Tabs
* Quick Open
* Find and replace
* Find in files
* Command palette
* Syntax highlighting
* Context-aware autocomplete
* UTF-8 support
* RTL editing mode
* Word wrapping
* Large-file support
* Recovery files
* Session restoration
* Atomic saves
* Terminal-native clipboard support

---

## Features

### Editing

* 400-step undo / redo
* Automatic indentation
* Automatic bracket and tag closing
* Selection wrapping with brackets and quotes
* Whole-line deletion
* UTF-8 input
* Large-file editing
* Atomic saves
* Recovery files for unsaved work
* Session restoration per project
* Clean terminal state restoration on exit

### Navigation

* Project tree sidebar
* Fuzzy Quick Open
* Symbol navigation
* Find in files
* Go to line
* File and folder opening
* `file:line` syntax
* Tabs with `Alt+1` through `Alt+9`
* `Ctrl+Tab` to cycle through all open tabs

### Search & Replace

* Case-insensitive search by default
* Case-sensitive search with `%term`
* Find and replace in the current file
* Workspace-wide find and replace
* Live result counts
* Dedicated Find in Files modal

### Terminal

* Truecolor support
* SGR mouse integration
* Click, drag, double-click, and scroll support
* Bracketed paste up to 5 MB
* OSC52 clipboard fallback
* Wide-character and tab-aware rendering
* Horizontal and vertical scrollbars
* Terminal state restoration

### Interface

* Tokyo Night-inspired interface
* File-type colors in the project tree
* Interactive tabs
* Real-time clock
* Visible line numbers
* Status bar
* Command palette
* Searchable keyboard-shortcuts dialog
* Context-aware mouse menus

---

## Keyboard Shortcuts

| Action             | Shortcut             | Action           | Shortcut       |
| ------------------ | -------------------- | ---------------- | -------------- |
| Save File          | `Ctrl+S`             | Command Palette  | `Ctrl+P`       |
| Quick Open         | `Ctrl+O`             | Shortcuts        | `Ctrl+K`       |
| Find               | `Ctrl+F`             | Find in Files    | `Ctrl+Shift+O` |
| Replace            | `Ctrl+R`             | Replace in Files | `Ctrl+Shift+H` |
| Find Next          | `Ctrl+L`             | Go to Line       | `Ctrl+G`       |
| Undo / Redo        | `Ctrl+Z` / `Ctrl+Y`  | End of Line      | `Ctrl+E`       |
| Copy / Cut / Paste | `Ctrl+C` / `X` / `V` | Select All       | `Ctrl+A`       |
| New File           | `Ctrl+N`             | Close Tab        | `Ctrl+D`       |
| Switch Tabs        | `Alt+1–9`            | Quit             | `Ctrl+Q`       |
| Focus Project Tree | `Ctrl+T`             | Toggle Tree      | `Ctrl+H`       |
| Word Wrap          | `Alt+Z`              | RTL Mode         | `Alt+R`        |
| Adjust Tree Width  | `+` / `-`            |                  |                |

---

## Language Support

`az` currently includes syntax highlighting and language-aware features for **42 languages and formats**.

| Language    | Extensions / Files                |
| ----------- | --------------------------------- |
| PHP         | `*.php`, `*.phtml`                |
| Blade       | `*.blade.php`                     |
| HTML        | `*.html`, `*.htm`                 |
| CSS         | `*.css`                           |
| JavaScript  | `*.js`, `*.mjs`, `*.cjs`, `*.jsx` |
| TypeScript  | `*.ts`, `*.tsx`, `*.mts`, `*.cts` |
| XML         | `*.xml`, `*.svg`                  |
| Markdown    | `*.md`                            |
| JSON        | `*.json`, `*.jsonc`               |
| TOML        | `*.toml`                          |
| YAML        | `*.yaml`, `*.yml`                 |
| Bash        | `*.sh`, `*.bash`, `*.zsh`         |
| Dotenv      | `.env`, `*.env`                   |
| INI         | `*.ini`, `*.conf`, `*.cfg`        |
| Logs        | `*.log`                           |
| Rust        | `*.rs`                            |
| Nginx       | `nginx.conf`                      |
| Apache      | `.htaccess`, `httpd.conf`         |
| Dockerfile  | `Dockerfile*`                     |
| systemd     | `*.service`, `*.timer`            |
| SQL         | `*.sql`                           |
| Python      | `*.py`, `*.pyw`, `*.pyi`          |
| Java        | `*.java`                          |
| C#          | `*.cs`, `*.csx`                   |
| C / C++     | `*.c`, `*.h`, `*.cpp`, `*.hpp`    |
| Go          | `*.go`                            |
| Kotlin      | `*.kt`, `*.kts`                   |
| Swift       | `*.swift`                         |
| Ruby        | `*.rb`, `Gemfile`, `Rakefile`     |
| Dart        | `*.dart`                          |
| Scala       | `*.scala`, `*.sc`                 |
| R           | `*.r`                             |
| Lua         | `*.lua`                           |
| Perl        | `*.pl`, `*.pm`, `*.t`             |
| Haskell     | `*.hs`, `*.lhs`                   |
| Elixir      | `*.ex`, `*.exs`                   |
| Clojure     | `*.clj`, `*.cljs`, `*.edn`        |
| Zig         | `*.zig`                           |
| Julia       | `*.jl`                            |
| Objective-C | `*.m`, `*.mm`                     |
| Plain Text  | `*.txt` + fallback                |

Language-aware completion is available for many supported languages, including variables, members, tags, attributes, keywords, directives, and structural symbols.

Change the active language through the command palette:

```text
Ctrl+P → set [Language]
```

---

## What's New in 4.0

### Word Wrap

Toggle soft word wrapping with:

```text
Alt+Z
```

Word wrap can also be enabled through the command palette, editor context menu, or status bar.

### RTL Mode

`az` includes a dedicated right-to-left editing mode:

```text
Alt+R
```

RTL mode right-aligns text and provides an RTL-aware caret and click mapping.

### Improved Tabs

* Up to nine tabs are displayed at once.
* Tabs beyond the ninth remain accessible.
* `Alt+1–9` switches between visible tabs.
* `Ctrl+Tab` cycles through every open tab.
* `+` and `Ctrl+N` create new empty tabs.

### Better Search & Replace

Find, replace, and project-wide replacement use unified dialogs with:

* Path filtering
* Find and replace fields
* Live result counts
* Replace and cancel actions

### Clipboard & Privileged Saves

`az` supports system clipboard integration with OSC52 fallback for remote sessions.

When a file is not writable, `az` can request the root password and save it using `sudo`.

### Terminal Improvements

* Improved wheel navigation
* Horizontal scrolling
* Better caret positioning
* Better wide-character handling
* Faster paste operations
* Improved terminal cleanup

**4.0 test suite:** 56 tests, with 55 executed and 1 Wayland roundtrip test ignored.

See the [Changelog](CHANGELOG.md) for previous releases.

---

## Building from Source

If you prefer to compile `az` yourself:

```sh
./compile-and-install.sh
```

This builds the release binary and installs it to:

```text
~/.local/bin/az
```

You can specify a custom installation directory:

```sh
AZ_BIN_DIR="$HOME/bin" ./compile-and-install.sh
```

The build requires either `cargo` or `rustc`.

For the lower-level build process:

```sh
./build.sh
```

---

## Extending az

`az` is designed to be extended through its plugin architecture.

The repository includes an [AI Agent Guide](AGENTS.md) containing information about the architecture, plugin API, and testing requirements.

To add a new language, an AI coding assistant can scaffold the implementation, connect it to the language router, and add the required regression tests.

---

## Documentation

* [User Manual](USER_MANUAL.md)
* [Changelog](CHANGELOG.md)
* [AI Agent Guide](AGENTS.md)

---

## License

`az` is licensed under the [WTFPL](https://www.wtfpl.net/).

You are free to copy, modify, distribute, and use `az` for any purpose, including commercial applications, without restriction.

The software is provided **"as is"**, without warranty of any kind.
