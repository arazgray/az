<p align="center">
  <img src="logo.png" alt="az Logo" height="64"/>
  <h1 align="center">az</h1>
  <h3 align="center">The TUI text editor you've always wanted</h3>
</p>

<p align="center">
  <strong>A ridiculously fast, lightweight & sane terminal text editor built in Rust. Batteries included.</strong><br>
  Keyboard-first but <strong>mouse-supported</strong>, zero-configuration, and designed to stay out of your way<br>A perfect alternative to Vim and Nano<br>With more than <strong>150 language/syntax</strong> support.</i><br>
  <a href="USER_MANUAL.md">User Manual</a> |
  <a href="CHANGELOG.md">Changelog</a> |
  <a href="PLUGIN_GUIDE.md">Plugin Guide</a>
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

> **Update checks:** `az` automatically checks the main repository for newer releases when it starts. When one is found, a dialog offers a one-click **Update and restart** (with a success notice and restart button) instead of a command to retype. Checks are skipped when offline. Set `AZ_NO_UPDATE_CHECK=1` to disable update checks.

---

## Packages

Prebuilt packages are available for Linux, macOS, and Windows (x86_64 + ARM64)
on the [releases page](https://github.com/arazgray/az/releases/latest):

- Linux (generic tarball, x86_64 + ARM64)
- Debian / Ubuntu / Mint / Pop!_OS (`.deb`, x86_64 + ARM64)
- Fedora / RHEL / openSUSE (`.rpm`, x86_64 + ARM64)
- macOS Apple Silicon and Intel (tarball)
- Windows (`.exe`, x86_64 + ARM64)

Tagged releases are built by [`.github/workflows/release.yml`](.github/workflows/release.yml);
Linux packages can also be reproduced locally with `./dist/package.sh`.
The one-line installer below already knows these names and picks the
right one for your OS + CPU.

### Debian / Ubuntu

```sh
sudo dpkg -i az_4.9.0_amd64.deb
```

ARM (Raspberry Pi, Ampere, AWS Graviton):

```sh
sudo dpkg -i az_4.9.0_arm64.deb
```

### Arch Linux (AUR)

```sh
yay -S az-bin
# or: paru -S az-bin
```

Packaging files live in `dist/aur/az-bin/` (`PKGBUILD` + `.SRCINFO`, sums
verified against the release assets). Publishing to the AUR needs an AUR
account — steps are in `dist/aur/README.md`.

### Fedora / RHEL / openSUSE

```sh
# Fedora / RHEL / CentOS:
sudo dnf install ./az-4.9.0-1.x86_64.rpm
# or, without a network solver:
sudo rpm -i az-4.9.0-1.x86_64.rpm
# openSUSE:
sudo zypper install ./az-4.9.0-1.x86_64.rpm
```

### Arch / Alpine / other Linux (generic tarball)

```sh
tar -xzf az-4.9.0-linux-amd64.tar.gz
sudo install -m755 az /usr/local/bin/az
```

### macOS

```sh
brew install arazgray/tap/az
```

Or manually from the release tarball:

```sh
tar -xzf az-4.9.0-macos-arm64.tar.gz   # Apple Silicon
# tar -xzf az-4.9.0-macos-amd64.tar.gz # Intel
mkdir -p ~/.local/bin
install -m755 az ~/.local/bin/az
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
* Automatic indentation (Tabs or 4 spaces, all languages; switchable from the status bar)
* Format selection or file (re-indent, command palette)
* Autosave toggle (command palette or status bar)
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
* Project root aligned with the tabs; one-row tree wheel navigation
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
* Clickable tree, autosave, and indentation controls
* Author, contact, and support dialog
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
| Autosave           | `Alt+A`              | Format           | `Alt+F`        |
| Indent Tabs/Spaces | `Alt+T` / `Alt+I`    | Syntax Menu      | `Alt+L`        |
| Save As            | `Alt+S`              | New File/Folder  | `Alt+N`/`Alt+M`|
| Rename             | `F2`                 | Delete File      | `Alt+D`        |
| Welcome            | `Alt+W`              | Check Update     | `Alt+U`        |
| Adjust Tree Width  | `+` / `-`            |                  |                |

---

## Language Support

`az` currently includes syntax highlighting and language-aware features for **152 languages and formats**.

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
| Makefile    | `Makefile`, `*.mk`, `*.mak`         |
| CMake       | `CMakeLists.txt`, `*.cmake`         |
| PowerShell  | `*.ps1`, `*.psm1`, `*.psd1`        |
| Proto       | `*.proto`                           |
| GraphQL     | `*.graphql`, `*.gql`                |
| Terraform   | `*.tf`, `*.hcl`                     |
| Vue         | `*.vue`                             |
| Svelte      | `*.svelte`                          |
| Assembly    | `*.[sS]`, `*.asm`                   |
| TeX         | `*.tex`, `*.bib`, `*.cls`, `*.sty`  |
| Erlang      | `*.erl`, `*.hrl`                    |
| Solidity    | `*.sol`                             |
| Ada | `*.ads` `*.adb` `*.ada` |
| Arduino | `*.ino` |
| AsciiDoc | `*.asc` `*.asciidoc` `*.adoc` |
| ATS | `*.dats` `*.hats` `*.sats` |
| Awk | `*.awk` |
| B | `*.b` |
| Batch | `*.bat` `*.cmd` |
| Caddyfile | `caddyfile` |
| Cake | `*.cake` |
| CoffeeScript | `*.coffee` |
| Conky | `conky.conf` `*conkyrc*` |
| Crontab | `crontab` `crontab.*` |
| Crystal | `*.cr` |
| CUDA | `*.cu` `*.cuh` |
| Cython | `*.pyx` `*.pxd` |
| D | `*.d` `*.di` `*.dd` |
| Ebuild | `*.ebuild` `*.eclass` |
| Elm | `*.elm` |
| ERB | `*.erb` `*.rhtml` |
| F# | `*.fs` `*.fsi` `*.fsx` |
| Fish | `*.fish` |
| Forth | `*.forth` `*.4th` `*.fs8` `*.ft` `*.fth` `*.frt` |
| Fortran | `*.f` `*.f90` `*.f95` `*.for` |
| FreeBSD kernel | `generic` |
| GDScript | `*.gd` |
| Gemini | `*.gmi` `*.gemini` |
| Git commit | `commit_editmsg` `tag_editmsg` `merge_msg` |
| Git config | `.gitconfig` `gitconfig` `gitmodules` `.git/config` |
| Git rebase | `git-rebase-todo` |
| Gleam | `*.gleam` |
| GLSL | `*.frag` `*.vert` `*.fp` `*.vp` `*.glsl` |
| Gnuplot | `*.gnu` `*.gpi` `*.plt` `*.gp` |
| Go doc | `*.godoc` |
| Go mod | `go.mod` |
| Golo | `*.golo` |
| Graphviz | `*.dot` `*.gv` |
| Groff | `*.me` `*.ms` `*.rof` `*.tmac` `tmac.*` |
| Groovy | `jenkinsfile` `*.groovy` `*.gy` `*.gvy` `*.gsh` `*.gradle` |
| Haml | `*.haml` |
| Hare | `*.ha` |
| HolyC | `*.hc` |
| Inputrc | `inputrc` `.inputrc` |
| Jinja2 | `*.j2` `*.jinja` `*.jinja2` |
| Jsonnet | `*.jsonnet` `*.libsonnet` |
| Just | `justfile` `.justfile` `*.just` |
| Keymap | `xmodmap` `*.map` `*.kmap` `*.keymap` |
| Kickstart | `*.ks` `*.kickstart` |
| Kvlang | `*.kv` |
| Ledger | `ledger` `ldgr` `beancount` `bnct` `*.ledger` `*.ldgr` `*.beancount` `*.bnct` |
| LFE | `*.lfe` |
| LilyPond | `*.ly` `*.ily` `*.lly` |
| Lisp | `emacs` `zile` `*.el` `*.lisp` `*.lsp` `*.scm` `*.ss` `*.rkt` |
| Mail | `*.eml` `mutt-*` |
| Man page | `*.1` `*.2` `*.3` `*.4` `*.5` `*.6` `*.7` `*.8` `*.9` |
| MC (sendmail) | `*.mc` |
| Meson | `meson.build` `meson_options.txt` `meson.options` |
| Micro config | `*.micro` |
| MPD config | `mpd.conf` |
| MSBuild | `*.props` `*.targets` `*.tasks` `*proj` |
| Nanorc | `nanorc` `.nanorc` |
| nftables | `nftables.conf` `nftables.rules` |
| Nim | `nim.cfg` `*.nim` `*.nims` |
| Nix | `*.nix` |
| Nushell | `*.nu` |
| OCaml | `*.ml` `*.mli` |
| Octave | (palette only) |
| Odin | `*.odin` |
| OpenSCAD | `*.scad` |
| Pascal | `*.pas` |
| Patch | `*.patch` `*.diff` |
| PEG | `*.peg` `*.lpeg` |
| pkg-config | `*.pc` |
| PO file | `*.po` `*.pot` |
| Pony | `*.pony` |
| Portage | `*.keywords` `*.mask` `*.unmask` `*.use` |
| POV-Ray | `*.pov` `*.povray` |
| Privoxy | `*.action` `*.filter` `privoxy/config` |
| PRQL | `*.prql` |
| Puppet | `*.pp` |
| Raku | `*.p6` `*.pl6` `*.pm6` `*.pod6` `*.raku` `*.rakumod` `*.rakudoc` `*.rakutest` `*.nqp` |
| Ren'Py | `*.rpy` |
| reST | `*.rest` `*.rst` |
| RPM spec | `*.spec` `*.rpmspec` |
| Sage | `*.sage` |
| SaltStack | `*.sls` |
| Sed | `*.sed` |
| Smalltalk | `*.st` `*.sources` `*.changes` |
| Stata | `*.do` `*.ado` |
| Tcl | `*.tcl` |
| Twig | `*.twig` |
| V | (palette only) |
| Vala | `*.vala` |
| Verilog | `*.v` `*.vh` `*.sv` `*.svh` |
| VHDL | `*.vhdl` `*.vhd` |
| Vimscript | `vimrc` `.vimrc` `exrc` `.exrc` `gvimrc` `.gvimrc` `*.vim` |
| Xresources | `xdefaults` `xresources` |
| Yum repo | `yum.conf` `*.repo` |
| ZScript | `*.zc` `*.zsc` |
| Plain Text  | `*.txt` + fallback                |

Language-aware completion is available for many supported languages, including variables, members, tags, attributes, keywords, directives, and structural symbols.

Change the active language through the command palette:

```text
Ctrl+P → set [Language]
```

---

## What's New in 4.9

Version 4.9 brings the latest interface improvements and gathers the user-facing features added throughout the 4.x series.

### Interface and editing

* The project root now shares the tab row; tree entries start directly beneath it.
* Click the status bar's `tree shown/hidden`, `autosave:on/off`, and `tabs` / `spaces:4` chips to change those settings.
* Open the author and contact dialog from the command palette with `Author`, `Contact`, or `Support`.
* Create, open, rename, and delete project files and folders from the tree and context menus.
* Use up to nine visible tabs, `Alt+1–9` to switch between them, and keyboard shortcuts for palette actions.
* Automatic indentation, selectable Tabs or 4 spaces, and re-indent selected text or a full file.
* Automatic bracket pairs and HTML closing tags, selection wrapping, whole-line deletion, and 400-step undo/redo.
* Autosave for open files, large-file editing, and session restoration with recovery for unsaved work.
* Prompts support cursor movement and recent-answer history; password entries are excluded.

### Navigation and search

* Fuzzy Quick Open for files, symbols, and `file:line` locations.
* Find and replace in the current file or across a project, plus Find in Files.
* Case-insensitive search by default; prefix a query with `%` for case-sensitive matching.
* `Ctrl+Tab` to cycle all tabs; `Ctrl+K` opens searchable keyboard shortcuts.

### Language support

* Syntax highlighting for 152 languages and formats, with context-aware completion and symbol navigation in supported modes.
* Select a language from the command palette or let `az` detect it from the filename.
* Shell startup files are recognized by filename or shebang; plain text highlights brackets and punctuation with alternating rows.

### Terminal, clipboard, and file safety

* Word wrap and RTL editing, with tab- and wide-character-aware cursor placement.
* Truecolor, mouse selection and menus, horizontal/vertical scrollbars, and terminal-state restoration.
* System clipboard integration with OSC52 fallback and bracketed paste up to 5 MB.
* Atomic saves, CRLF preservation, recovery files, and password-assisted saves for protected files.

### Setup and updates

* Launch options for autosave, indentation, wrapping, and RTL mode; preferences persist between runs.
* Optional startup update checks and a one-click update-and-restart flow.
* Prebuilt packages for Linux, macOS, and Windows, with a one-line installer.

See the [Changelog](CHANGELOG.md) for the detailed release history.

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

`az` supports additional language plugins. The [Plugin Guide](PLUGIN_GUIDE.md) explains how to add a language module, register it, and test it.

---

## Documentation

* [User Manual](USER_MANUAL.md)
* [Changelog](CHANGELOG.md)
* [Plugin Guide](PLUGIN_GUIDE.md)

---

## License

`az` is licensed under the [WTFPL](https://www.wtfpl.net/).

You are free to copy, modify, distribute, and use `az` for any purpose, including commercial applications, without restriction.

The software is provided **"as is"**, without warranty of any kind.
