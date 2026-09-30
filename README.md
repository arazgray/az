# az
 sane text editor | v2.5

`az` is a fast, small & sane text editor.

> Open fast. Type immediately. Stay keyboard-first. Zero dependencies.

![az](screenshot.png.jpg)

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
az  [Open] [Commands] [Shortcuts]  tabs above editor (click switch, middle-click close)
Ctrl+S  save              Ctrl+O  quick open (file, symbol, file:line, :line)
Ctrl+P  command palette   Ctrl+K  shortcuts (searchable)   Ctrl+F  find (%term sensitive)
Ctrl+Shift+O find in files   Ctrl+Shift+H replace in files   Ctrl+R replace   Ctrl+G go to line
Ctrl+L  find next         Ctrl+E  end of line                +/-   tree width (in tree)
Ctrl+D  close tab         Ctrl+N  new file                        Ctrl+Q  quit   Alt+1-9  switch tab
Ctrl+T  tree focus        Ctrl+H  hide/show tree (in tree)
Ctrl+Z / Ctrl+Y  undo / redo      Ctrl+C / X / V / A  copy / cut / paste / select all
Mouse: click move/open/switch, drag select, dbl-click word, wheel scroll, right-click menu
```

Full map: [`USER_MANUAL.md`](USER_MANUAL.md#4-keyboard--complete-map).

---

## What it looks like

```text
┌ az  [Open] [Commands] [Shortcuts] ──────────  02:30 PM  30/09/2026 ─┐
├─────────────────────────────────────────────────────────────────────┤
│                             │ 1:main.blade.php  2:style.css         │
│ ▾ project/              │  1  @extends('layouts.app')                    │
│   ▾ resources/          │  2  @section('content')                        │
│     ▾ views/            │  3  <style>                                    │
│       main.blade.php    │  4  #header { background: #fff; }  ← ID yellow │
│       style.css         │  5  </style>         hex #fff orange, not gray │
│   Enter open/fold       │  6  <div id="app">{{ $user->name }}</div>      │
│   N file  Shift+N folder│  7  @if($x) … @endif   ← @ purple, $x red     │
├─────────────────────────────────────────────────────────────────────┤
└ main.blade.php  BLADE  tree shown │ Found $user  Ln 6… ───┘
```

- Titlebar (`az` + mode chips, dialog buttons) + tab bar (`*` = modified, inactive tabs lighter). Press `Esc` to flash `Alt+1-9` hints.
- Tree: colors by extension (PHP purple, Blade magenta, HTML orange, JS yellow, CSS blue).
- Editor: line numbers, horizontal scroll, Tokyo Night colors, mixed-language highlighting.
- Status: path · modified (only when dirty) · language · message (flashes light blue) · `Ln,Col Lines Words` — each item on its own palette chip; mode lives in the titlebar.

---

## Why az?

For quick edits, small projects, server work, focused writing, and terminal code changes.
No config drama, no plugins to install. Keyboard-first, mouse supported for click/drag/scroll.


## Features

- Written in Rust
- Keyboard-first editing
- Opens files or folders
- Project tree sidebar
- Different tree colors for different file extensions
- Tabs with `Alt+1` to `Alt+9` (or click a tab; inactive tabs shaded lighter)
- Quick open with `Ctrl+O`
- Open files and jump to a line with `file.php:20`
- Jump to a line in the current file with `:20`
- Find in files with `Ctrl+Shift+O` (separate modal, live search across opened folder)
- Function and symbol opening from quick open
- Command palette with `Ctrl+P`
- Save, create files, switch language mode, and run editor actions from the command palette
- Visible line numbers
- Welcome screen on startup
- `Ctrl+K` opens the searchable keyboard-shortcuts dialog
- Find and replace
- Replace in files with `Ctrl+Shift+H` (separate dialog with counts + confirm, scoped via sidebar right-click)
- Mouse: click to move cursor / expand folders / open files / switch tabs, double-click file to rename, wheel to scroll
- Mouse right-click menus (sidebar, editor, tab), middle-click tab to close
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
| Term | Raw-mode `stty`, bracketed paste (5 MB), truecolor, tabs/wide-char aware, UTF-8 byte-safe, SGR mouse click/drag/scroll |

> No word wrap by design — long lines scroll horizontally. No splits or regex.

### What is new in 2.5?

- **Replace in Files dialog** (`Ctrl+Shift+H`, or `Ctrl+P` → `Replace in files`): prompts search → replacement → confirm with real counts (`Replace N in M files`), then rewrites files across the project (≤3000 files, skips >5MB/binaries, 10k-match cap, `%Foo` = case-sensitive). Open unmodified tabs reload (undo cleared); files with unsaved buffers are skipped and reported. Also scoped via sidebar right-click → `Search & Replace here`.
- **Right-click context menus** (SGR mouse): tab (`Close tab`, `Copy file path`), sidebar (`Open`, `Copy file path`, `Rename`, `Delete`, `Search here`, `Search & Replace here` — each opens its dialog), editor (`Cut/Copy/Paste`, `Select All`, `Find/Replace in File`, `Find/Replace in Files`, `Go to Line`). Keyboard navigable (`Up/Down`, `Enter`, `Esc`, `1-9`).
- **Middle-click a tab closes it** (asks if modified). Inactive tabs now use a lighter background (`BG_TAB`); the open file's sidebar row is highlighted.
- **Mouse text selection**: drag with the button held to select; double-click selects the word, triple-click the line — alongside click-to-move-cursor.
- **Dedicated titlebar**: blue `az` + mode chips, plain-text `[Open]`/`[Commands]`/`[Shortcuts]` buttons, clock — with a full-width separator below; tabs sit on the bar under it, above the editor, with click behavior intact.
- **Empty-space fill**: rows past end-of-file now paint the editor background instead of the terminal default.
- **Status bar chips**: path, state (`modified`, orange, only when dirty), syntax, tree, and stats each render on their own Tokyo Night chip; a matching separator sits above the bar.
- **Blinking alerts**: new status messages flash light blue once; every prompt blinks light blue until you answer.
- **Searchable shortcuts dialog** (`Ctrl+K`, replaces `Ctrl+/`): every shortcut filterable like the command palette. Welcome screen trimmed to 4 essentials + a highlighted `Ctrl+K` hint, with a ttfx-`highlight` gradient logo.
- **Reliable clipboard**: system copy now tries `wl-copy` → `xclip`/`xsel` → `pbcopy` before OSC52 (OSC52 first over SSH); status shows `Copied` vs `Copied (OSC52)` so you know what worked.
- **Wheel that works everywhere**: editor, sidebar, and all picker dialogs/menus scroll; legacy X10 mouse reports supported for terminals without SGR.
- **Tests**: 33 total (new: replace counting, menu geometry, `Ctrl+Shift+H`, word range, `Ctrl+K` + shortcut coverage, OSC52 bytes, legacy mouse).

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

`az` is released under the [WTFPL](https://www.wtfpl.net/) — the Do What The Fuck You Want To Public License.

In short: you may copy, distribute, modify, and use this software (including for commercial purposes) with no restrictions and no warranty. The full license text lives at <https://www.wtfpl.net/>.

- ✅ Use it for anything — personal, commercial, closed-source.
- ✅ Modify it, redistribute it, relicense your own changes however you like.
- ⚠️ No warranty: the software is provided "as is"; the authors are not liable for anything it does.
