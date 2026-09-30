# az text editor 2.1

`az` is a small, sane terminal text editor for code and text.

> Open fast. Type immediately. Stay keyboard-first. Zero dependencies.

![az](az-editor.jpg)

Docs: [`USER_MANUAL.md`](USER_MANUAL.md) (full usage + troubleshooting) · [`AGENTS.md`](AGENTS.md) (AI-agent guide) · [`PLUGIN_GUIDE.md`](PLUGIN_GUIDE.md) (add a language)

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
Ctrl+R  replace           Ctrl+G  go to line                      Ctrl+N  new file
Ctrl+T  tree focus        Ctrl+H  hide/show tree (in tree)        +/-   tree width (in tree)
Ctrl+D  close tab         Ctrl+Q  quit                            Alt+1-9  switch tab
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

## What is new in 2.1?

- **Blade + CSS fix**: `#header`, `#fff`, `href="#section"`, `https://…` no longer gray as PHP `#` / `//` comments. `#id` now yellow, hex orange, URLs stay green. `user@example.com` no longer purple as Blade directive. 6 new regression tests (15 total).
- **Smarter comments**: `//` / `#` ignored inside `"strings"`, `` `templates` ``, and `://` protocols.
- **`--help` / `--version`**, `file:line`, `:line`, `newfile` CLI handling; absolute-path session keys.
- **Faster topbar**: `date` cached per minute. Pickers safe on 30-col terminals. Tree/quick-open skip `target dist build __pycache__ .next .nuxt`.
- **New keys**: `Ctrl+L` find-next, `+/-` tree width, `Ctrl+N` new file everywhere in help.
- **Repo hygiene**: `.gitignore` for `target/`, `/az` binary, `*.tmp`, OS/IDE noise; binaries untracked.

2.0 recap: Rust rewrite, tabs, tree, quick open, palette, find/replace, autocomplete, recovery, zero crates.

---

## Features

| Area | Details |
|------|---------|
| Open | Files, folders, `file:line`, `:line`, non-existent → new tab |
| Navigate | Tree, quick open (fuzzy file + symbol), project search, go-to-line, `Alt+1-9` |
| Edit | Undo/redo (400), auto-indent, `()` `{}` close, `<div>` → `</div>`, copy/cut/paste (OSC52), select-all, delete-line |
| Search | Case-insensitive default, `%term` sensitive, wrap notice, replace one/all (undoable) |
| Highlight | PHP, Blade, HTML, CSS, JS mixed per-line; Blade `{{ }}` / `{{-- --}}` / `@dir`; CSS `#id` + hex; HTML tags/attrs/entities |
| Complete | `Tab` context items: PHP `$vars`/members, HTML tags/attrs, CSS props/values/`@rules`, JS members/snippets, Blade directives |
| Safety | Atomic saves, session restore per project, throttled recovery (`$XDG_STATE_HOME/az-rust`), terminal cleanup |
| Term | Raw-mode `stty`, bracketed paste (5 MB), truecolor, tabs/wide-char aware, UTF-8 byte-safe |

> No word wrap by design — long lines scroll horizontally. No mouse, splits, or regex.

---

## Language plugins

`src/plugins/` — one file per language, std-only, line-local.

| Plugin | File | Provides |
|--------|------|----------|
| PHP | `php.rs` | keywords/functions/`$vars`/`#` `//` (string + CSS aware) |
| Blade | `blade.rs` | `@directives` (boundary + string aware), `{{ }}`, symbols |
| HTML | `html.rs` | tags/attrs/entities, void-tag close, inline-style detect |
| CSS | `css.rs` | props/hex/`#id`/`@rules`/`!important`, selector symbols |
| JavaScript | `javascript.rs` | keywords/builtins/`` `templates` ``/regex/`//` (protocol aware) |
| Example | `example.rs` | skeleton for new languages |

Add one in ~15 min — see [`PLUGIN_GUIDE.md`](PLUGIN_GUIDE.md) + [`AGENTS.md`](AGENTS.md#4-plugin-api-add-language-in-15-min).

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

Remote:

```sh
curl -fsSL https://raw.githubusercontent.com/arazgholami/az/refs/heads/main/install.sh | sh
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
