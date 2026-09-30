# az text editor 2.2

`az` is a small, sane terminal text editor for code and text.

> Open fast. Type immediately. Stay keyboard-first. Zero dependencies.

![az](az-editor.jpg)

Docs: [`USER_MANUAL.md`](USER_MANUAL.md) (full usage + troubleshooting) · [`CHANGELOG.md`](CHANGELOG.md) (release notes) · [`AGENTS.md`](AGENTS.md) (AI-agent guide) · [`PLUGIN_GUIDE.md`](PLUGIN_GUIDE.md) (add a language)

---
# Install (Automatic single-command installer, builds on your machine):

```sh
curl -fsSL https://raw.githubusercontent.com/arazgholami/az/refs/heads/main/install.sh | sh
```

---

## Quick start

```sh
./build.sh          # builds release + installs to ~/.local/bin/az
az --help
az file.php         # open file
az project/         # open folder
az file.php:20      # open at line 20
az newfile.txt      # new file tab
```

```text
Ctrl+S  save              Ctrl+O  quick open (file, symbol, file:line, :line)
Ctrl+P  command palette   Ctrl+F  find (%term = case-sensitive)   Ctrl+L  find next
Ctrl+Shift+O find in files (%term = case-sensitive)   Ctrl+R  replace   Ctrl+G  go to line
Ctrl+E  end of line       Ctrl+Home/End or Alt+Up/Down  top/bottom of file
Ctrl+T  tree focus        Ctrl+H  hide/show tree (in tree)        +/-   tree width (in tree)
Ctrl+D  close tab         Ctrl+N  new file                        Ctrl+Q  quit   Alt+1-9  switch tab
Ctrl+Z / Ctrl+Y  undo / redo      Ctrl+C / X / V / A  copy / cut / paste / select all
Ctrl+/  help
```

Full map: [`USER_MANUAL.md`](USER_MANUAL.md#4-keyboard--complete-map).

---

## What it looks like

```text
┌ az | sane editor │ 1:main.blade.php  2:style.css │  02:30 PM  30/09/2026 ─┐
│ ▾ project/              │  1  @extends('layouts.app')                    │
│   ▾ resources/          │  2  @section('content')                        │
│     ▾ views/            │  3  <style>                                    │
│       main.blade.php    │  4  #header { background: #fff; }  ← ID yellow │
│       style.css         │  5  </style>         hex #fff orange, not gray │
│   Enter open/fold       │  6  <div id="app">{{ $user->name }}</div>      │
│   N file  Shift+N folder│  7  @if($x) … @endif   ← @ purple, $x red     │
└ editor  main.blade.php  saved  BLADE  tree shown │ Found $user  Ln 6… ───┘
```

- Topbar: tabs (`*` = modified). Press `Esc` to flash `Alt+1-9` hints.
- Tree: colors by extension (PHP purple, Blade magenta, HTML orange, JS yellow, CSS blue).
- Editor: line numbers, horizontal scroll, Tokyo Night colors, mixed-language highlighting.
- Status: focus · path · saved/modified · language · message · `Ln,Col Lines Words`.

---

## Why az?

For quick edits, small projects, server work, focused writing, and terminal code changes.
No config drama, no plugins to install, no mouse required.


## Features

- Written in Rust
- Keyboard-first editing
- Opens files or folders
- Project tree sidebar
- Different tree colors for different file extensions
- Tabs with `Alt+1` to `Alt+9`
- Quick open with `Ctrl+O`
- Open files and jump to a line with `file.php:20`
- Jump to a line in the current file with `:20`
- Find in files with `Ctrl+Shift+O` (separate modal, live search across opened folder)
- Function and symbol opening from quick open
- Command palette with `Ctrl+P`
- Save, create files, switch language mode, and run editor actions from the command palette
- Visible line numbers
- Welcome screen on startup
- `Ctrl+/` shows the same welcome/help screen
- Find and replace
- Find in files modal (`Ctrl+Shift+O`, also via `Ctrl+P` → `Find in files`)
- Case-insensitive search by default
- Case-sensitive search with `%term` (works in both Find and Find in files)
- Horizontal scroll (no word wrap by design)
- UTF-8 input support
- Tokyo Night inspired interface colors
- Syntax highlighting through plugins
- Autocomplete through plugins
- Huge file editing support
- Recovery files for unsaved work
- Terminal cleanup on quit


### Features in one shot

| Area | Details |
|------|---------|
| Open | Files, folders, `file:line`, `:line`, non-existent → new tab |
| Navigate | Tree, quick open (fuzzy file + symbol), find in files, go-to-line/start/end, `Alt+1-9` |
| Edit | Undo/redo (400), auto-indent, `()` `{}` close, `<div>` → `</div>`, copy/cut/paste (OSC52), select-all, delete-line |
| Search | Find in file + Find in files modal (`Ctrl+Shift+O`), case-insensitive default, `%term` sensitive, wrap notice, replace one/all (undoable) |
| Highlight | 22 languages (table below); PHP/Blade/HTML/CSS/JS mixed per-line; CSS `#id` + hex; logs levels + timestamps |
| Complete | `Tab` context items: PHP `$vars`/members, HTML/XML tags/attrs, CSS props/values/`@rules`, JS/TS members/snippets, Blade directives, Bash `$vars`, SQL keywords, Nginx/Apache/Dockerfile/Rust words |
| Safety | Atomic saves, session restore per project, throttled recovery (`$XDG_STATE_HOME/az-rust`), terminal cleanup |
| Term | Raw-mode `stty`, bracketed paste (5 MB), truecolor, tabs/wide-char aware, UTF-8 byte-safe |

> No word wrap by design — long lines scroll horizontally. No mouse, splits, or regex.

### What is new in 2.2?

- **Find in Files modal** (`Ctrl+Shift+O`, or `Ctrl+P` → `Find in files`): separate live-search popup across the opened folder (≤3000 files, 80 results, `file:line + snippet`, `Enter` jumps to the match). Same `%Foo` rule as Find: case-insensitive by default, `%` prefix = case-sensitive.
- **Go to Start/End**: `Home`/`End` for line ends, `Ctrl+E` for end of line, `Ctrl+Home/End` or `Alt+Up/Down` for file top/bottom (hold `Shift` to select) — all four also in the command palette.
- **Terminal-friendly shortcuts**: `Ctrl+Shift+O` (most terminals reserve `Ctrl+Shift+F` for their own search) and `Alt+Up/Down` (Guake uses `Ctrl+Up/Down` for height).
- **Tests**: 24 total (new: `%` query parsing, case-sensitive project matching, navigation key bindings).

Older releases: [`CHANGELOG.md`](CHANGELOG.md).

---

## Supported languages (22 built in)

`src/plugins/` — one file per language, std-only, line-local. Auto-detected by filename/extension; override anytime via `Ctrl+P` → `set …`.

| Language | Files | What you get |
|----------|-------|--------------|
| PHP | `*.php`, `*.phtml` | keywords/functions/`$vars`, `#` `//` (string + CSS aware), symbols |
| Blade | `*.blade.php` | `@directives` (boundary + string aware), `{{ }}` / `{{-- --}}`, HTML+CSS+JS mixed |
| HTML | `*.html`, `*.htm` | tags/attrs/entities, void-tag auto-close |
| CSS | `*.css` | props/hex/`#id`/`@rules`/`!important`, selector symbols |
| JavaScript | `*.js`, `*.mjs`, `*.cjs`, `*.jsx` | keywords/builtins/templates/regex, members, snippets |
| TypeScript | `*.ts`, `*.tsx`, `*.mts`, `*.cts` | JS highlighting + `interface`/`type`/`enum`, symbols |
| XML | `*.xml`, `*.svg` | tags/attrs/entities, `<?…?>`, `<!-- -->` |
| Markdown | `*.md` | headings, bold, `code`, links, lists, symbols |
| JSON | `*.json`, `*.jsonc` | strings/numbers/`true`/`false`/`null`, `//` + `/* */` for jsonc |
| TOML | `*.toml` | `[sections]`, `key =`, strings/numbers/bools, `#` comments, symbols |
| YAML | `*.yaml`, `*.yml` | `key:`, strings/numbers/bools, `#` comments, `-` lists |
| Bash | `*.sh`, `*.bash`, `*.zsh` | `#!`, `#` comments, `$VAR`/`${}`, keywords/builtins, `fn` symbols |
| Dotenv | `.env`, `*.env` | `KEY=`, strings/numbers/bools, `#` comments, `export` |
| INI | `*.ini`, `*.conf`, `*.cfg` | `[sections]`, `key =/:`, `#`/`;` comments, symbols |
| Logs | `*.log` | timestamps + `ERROR` red / `WARN` yellow / `INFO` green / `DEBUG` dim |
| Rust | `*.rs` | keywords/types/macros/`#[attrs]`, `//` + `/* */`, `fn/struct/enum` symbols |
| Nginx | `nginx.conf` | blocks/directives, `$vars`, `#` comments, symbols |
| Apache | `.htaccess`, `httpd.conf` | directives, `<Sections>`, `#` comments, symbols |
| Dockerfile | `Dockerfile*` | `FROM`/`RUN`/… instructions, `$vars`, `#` comments |
| systemd | `*.service`, `*.timer` | `[Sections]`, `key=`, `#`/`;` comments, symbols |
| SQL | `*.sql` | keywords, strings/numbers, `--` + `/* */`, `CREATE TABLE` symbols |
| Plain | `*.txt` + fallback | no highlighting, always available via `set Plain` |

> **Want more? Just ask your AI agent.** `az` ships [`AGENTS.md`](AGENTS.md) — a guide that teaches any AI coding agent the plugin API, architecture, and test rules. Ask it to *"add language support for X"* and it can scaffold `src/plugins/x.rs`, wire `mod.rs` + `SyntaxMode` + palette, and add regression tests, following [`PLUGIN_GUIDE.md`](PLUGIN_GUIDE.md).

```blade
{{-- real Blade: all three languages on one screen --}}
@extends('layouts.app')
@section('content')
<style>
  #header { background: #fff; color: #333; }   /* # = ID/hex, not comment */
  .container { max-width: 1200px; }
</style>
<div id="app" class="container">
  <a href="#section">jump</a>
  <a href="https://example.com">link</a>       {{-- // inside string, not comment --}}
  {{ $user->name }}                            {{-- $var red, {{ }} teal --}}
  @if($x) … @endif                             {{-- @ purple, not email --}}
</div>
@endsection
```

---

## Build and install

```sh
./build.sh
# → target/release/az → ./az → ~/.local/bin/az
# adds ~/.local/bin to PATH via ~/.profile if needed
. ~/.profile   # first time only, or restart terminal
az
```

Custom dir:

```sh
AZ_BIN_DIR="$HOME/bin" ./build.sh
```

Requirements: `cargo` (recommended) or `rustc`. No crates. See [`AGENTS.md`](AGENTS.md#6-testing) for `cargo check/test/build`.

---

## Basic usage

```sh
az file.php
az project/
az file.php:20
az newfile.txt
az --help
az --version
```

State lives in `$XDG_STATE_HOME/az-rust` (or `~/.local/state/az-rust`): `session-<hash>.txt`, `recovery/*.rec`.

Troubleshooting (Backspace vs `Ctrl+H`, Alt numbers, colors, slow quick-open): [`USER_MANUAL.md`](USER_MANUAL.md#8-troubleshooting).

---

## License

WTFPL
