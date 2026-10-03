# az Text Editor

`az` is a high-performance, lightweight terminal text editor engineered for speed and keyboard-driven workflows. Written entirely in Rust with zero dependencies, it delivers instantaneous startup, robust syntax highlighting for 42 languages, and seamless project navigation without the overhead of configuration files or external plugins.

**Documentation:** [User Manual](https://www.google.com/search?q=USER_MANUAL.md) · [Changelog](CHANGELOG.md) · [AI Agent Guide](AGENTS.md) · [Plugin Development](https://www.google.com/search?q=PLUGIN_GUIDE.md)

![az](screenshot.png)

---

## Installation

Run the automatic single-command installer to clone, build, and install `az` directly on your machine.

```sh
curl -fsSL https://raw.githubusercontent.com/arazgray/az/refs/heads/main/install.sh | sh

```

This script handles both fresh installations and in-place upgrades (rebuilding and reinstalling to `~/.local/bin/az`). On launch, `az` automatically checks the main repository for updates and provides an upgrade prompt if a newer release is detected.
*(Note: Update checks are skipped offline. Opt-out by setting `AZ_NO_UPDATE_CHECK=1`).*

---

## Command-Line Usage

```sh
<<<<<<< HEAD
./build.sh          # release build; installs ~/.local/bin/az and /usr/local/bin/az
az --help
az file.php         # open file
az project/         # open folder
az file.php:20      # open at line 20
az newfile.txt      # new file tab
```

```text
az  [Open] [Commands] [Shortcuts]  tabs above editor (click switch, middle-click close)
Ctrl+S  save              Ctrl+O  quick open (file, symbol, file:line, :line)
Ctrl+P  command palette   Ctrl+K  shortcuts (searchable)   Ctrl+F  search & replace dialog
Ctrl+Shift+O find in files   Ctrl+Shift+H replace in files   Ctrl+R replace field   Ctrl+G go to line
Ctrl+L  find next         Ctrl+E  end of line                +/-   tree width (in tree)
Ctrl+D  close tab         Ctrl+N  new empty tab               Ctrl+Q  quit   Alt+1-9  visible tabs
Ctrl+Tab  cycle tabs      + on the tab bar  new empty tab
Ctrl+T  tree focus        Ctrl+H  hide/show tree (in tree)
Ctrl+Z / Ctrl+Y  undo / redo      Ctrl+C / X / V / A  copy / cut / paste / select all
Mouse: click move/open/switch, drag select, dbl-click word, wheel pans, click outside a dialog closes it
=======
az                  # Open empty editor
az file.php         # Open a specific file
az project/         # Open a directory and populate the project tree
az file.php:20      # Open a file and jump directly to line 20
az newfile.txt      # Initialize a new file in a new tab
az --help           # Display help and usage options
az --version        # Display current version

>>>>>>> refs/remotes/origin/main
```

---

## Interface Overview

`az` is designed with a clean, Tokyo Night-inspired interface that maximizes screen real estate while keeping essential information visible.

```text
┌ az  [Open] [Commands] [Shortcuts] ──────────  02:30 PM  30/09/2026 ─┐
├─────────────────────────────────────────────────────────────────────┤
│                             │ 1:main.blade.php  2:style.css         │
│ ▾ project/                  │  1  @extends('layouts.app')           │
│   ▾ resources/              │  2  @section('content')               │
│     ▾ views/                │  3  <style>                           │
│       main.blade.php        │  4  #header { background: #fff; }     │
│       style.css             │  5  </style>                          │
│   Enter open/fold           │  6  <div id="app">{{ $user->name }}</div>│
│   N file  Shift+N folder    │  7  @if($x) … @endif                  │
├─────────────────────────────────────────────────────────────────────┤
└ main.blade.php  BLADE  tree shown │ Found $user  Ln 6… ───┘

```

### UI Components

* **Titlebar & Tabs:** Features interactive mode chips and a real-time clock. Active tabs are visually distinct, and modified files are marked with an asterisk (`*`). Press `Esc` to reveal `Alt+1-9` tab-switching hints.
* **Project Tree:** Automatically color-codes files by extension (e.g., PHP is purple, HTML is orange, JS is yellow) for rapid visual parsing.
* **Editor:** Supports mixed-language syntax highlighting on a single line, visible line numbers, and horizontal scrolling (no word wrap by design).
* **Status Bar:** Displays file path, modification state, active language, and document statistics. System messages flash light blue for immediate user feedback.

### Keyboard & Mouse Controls

| Action | Shortcut | Action | Shortcut |
| --- | --- | --- | --- |
| **Save File** | `Ctrl+S` | **Command Palette** | `Ctrl+P` |
| **Quick Open** | `Ctrl+O` | **Shortcuts Menu** | `Ctrl+K` |
| **Find (Case-sensitive with `%`)** | `Ctrl+F` | **Find in Files** | `Ctrl+Shift+O` |
| **Replace** | `Ctrl+R` | **Replace in Files** | `Ctrl+Shift+H` |
| **Find Next** | `Ctrl+L` | **Go to Line** | `Ctrl+G` |
| **Undo / Redo** | `Ctrl+Z` / `Ctrl+Y` | **End of Line** | `Ctrl+E` |
| **Copy / Cut / Paste** | `Ctrl+C` / `X` / `V` | **Select All** | `Ctrl+A` |
| **New File** | `Ctrl+N` | **Close Tab** | `Ctrl+D` |
| **Switch Tabs** | `Alt+1-9` | **Quit** | `Ctrl+Q` |
| **Focus Tree** | `Ctrl+T` | **Toggle Tree Visibility** | `Ctrl+H` |
| **Adjust Tree Width** | `+` / `-` (in tree) |  |  |

* **Mouse Support:** Full SGR mouse integration. Click to move cursor, open files, or switch tabs. Middle-click to close tabs. Drag to select text, double-click for word selection. Scroll wheel supported across editor, sidebar, and pickers. Right-click opens context-aware action menus.

---

## Core Capabilities

<<<<<<< HEAD
For quick edits, small projects, server work, focused writing, and terminal code changes.
No config drama, no plugins to install. Keyboard-first, mouse supported for click/drag/scroll.


## Features

- Written in Rust
- Keyboard-first editing
- Opens files or folders
- Project tree sidebar
- Different tree colors for different file extensions
- Tabs with `Alt+1` to `Alt+9` on the visible window, `Ctrl+Tab` to cycle, and `+` for a new empty tab (inactive tabs shaded lighter)
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
- Find and replace in one dialog (path, find, replace, live count; `Tab` switches fields, `Enter` replaces, click outside cancels)
- Replace in files with `Ctrl+Shift+H` (same dialog, scoped via the path or a sidebar right-click)
- Mouse: click to move cursor / expand folders / open files / switch tabs, double-click file to rename, wheel pans the editor and moves the sidebar one row
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
| Navigate | Tree, quick open (fuzzy file + symbol), find in files, go-to-line/start/end, `Alt+1-9` on the visible tabs, `Ctrl+Tab` |
| Edit | Undo/redo (400; dirty flag clears when the text matches the save), auto-indent, `()` `{}` `[]` close, wrap a selection with those or quotes, `<div>` → `</div>`, copy/cut/paste (OS clipboard, OSC52 fallback), select-all, delete-line |
| Search | Search & replace dialog (`Ctrl+F` / `Ctrl+R` / `Ctrl+Shift+H`) with path, find, replace, and a live count. Find in files modal (`Ctrl+Shift+O`). Case-insensitive default, `%term` sensitive |
| Highlight | 42 languages (table below); PHP/Blade/HTML/CSS/JS mixed per-line; CSS `#id` + hex; logs levels + timestamps |
| Complete | `Tab` context items: PHP `$vars`/members, HTML/XML tags/attrs, CSS props/values/`@rules`, JS/TS members/snippets, Blade directives, Bash `$vars`, SQL keywords, Nginx/Apache/Dockerfile/Rust words, keywords + symbols for the 20 new languages |
| Safety | Atomic saves, root-password save when the file is not writable, session restore per project, throttled recovery (`$XDG_STATE_HOME/az-rust`), terminal cleanup |
| Term | Raw-mode `stty`, bracketed paste (5 MB), truecolor, tabs/wide-char aware, UTF-8 byte-safe, SGR mouse click/drag/scroll, horizontal scrollbar when a pane overflows |

> No word wrap by design — long lines scroll horizontally. No splits or regex.

### What is new in 3.0?

- **Wheel pans the editor** one row per report. The caret and the selection stay. The sidebar still moves one row. The wheel on the tab bar cycles tabs.
- **Tabs past the ninth stay reachable.** At most nine are drawn, windowed around the current one. `Alt+1-9` hits that window. `Ctrl+Tab` cycles every open tab. `+` and `Ctrl+N` open an empty tab; saving it asks for a filename.
- **Search and replace is one dialog** (`Ctrl+F`, `Ctrl+R`, `Ctrl+Shift+H`): path, find, replace, a live count, Replace, and Cancel. `Tab` moves between the fields. `Enter` replaces. A click outside closes it, and the same is true for quick open, the command palette, find in files, and shortcuts.
- **Horizontal scroll bars** appear on the last content row when a line or a sidebar name is wider than its pane. The caret stays on screen across tabs.
- **`~/.bashrc` highlights as Bash,** along with the other shell startup names and a shell `#!` on a file that would otherwise be plain.
- **The dirty flag follows the text.** Undo back to the saved buffer clears it. CRLF files are written back as CRLF. Copy and cut of a line match. `(` `{` `[` `"` `'` wrap a selection.
- **Clipboard, sudo save, and fast paste.** A confirmed copy pastes from the OS clipboard; `Copied (OSC52)` pastes the editor text. A permission error asks for the root password and saves with `sudo`. A long paste is one insert. The installer also copies `/usr/local/bin/az` so `sudo az` can find it.
- **Tests**: 56 total (55 run, 1 ignored Wayland roundtrip).

### What is new in 2.6?

- **20 new language plugins** (42 built in): Python, Java, C#, C++, C, Go, Kotlin, Swift, Ruby, Dart, Scala, R, Lua, Perl, Haskell, Elixir, Clojure, Zig, Julia, Objective-C. Auto-detected by extension/filename (`Gemfile` → Ruby, `.h` → C), switchable via `Ctrl+P` → `set …`. Each brings keywords/types/comments/numbers, call highlighting, symbol extraction + word completion.
- **Startup update check**: every launch compares against `Cargo.toml` on GitHub main (short `curl` timeout, silent when offline; `AZ_NO_UPDATE_CHECK=1` opts out). When a newer release exists, a modal shows the version and the one-line upgrade command above.
- **Wheel that keeps up**: fast scrolling and touchpads flush several mouse reports per read — the whole burst used to be silently dropped. Each report is now dispatched, so kinetic scrolling works in the editor, sidebar, and pickers.
- **Clipboard you can trust**: stdin is now closed before waiting on `wl-copy`/`xclip`/`xsel`/`pbcopy` (they read stdin to EOF — previously copy could hang when a tool was installed). Status still shows `Copied` (tool accepted) vs `Copied (OSC52)` (terminal fallback).
- **Tests**: 38 total (new: mouse-burst splitting, end-to-end wheel scrolling, clipboard-pipe EOF, remote-version parsing + comparison).

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
=======
* **Navigation:** Browse via the project tree, Quick Open (fuzzy finding for files and symbols), global find-in-files, and line jumping.
* **Editing:** Includes 400-step undo/redo, auto-indentation, automatic bracket/tag closing, whole-line deletion, and reliable clipboard integration via system tools (`wl-copy`, `xclip`, `pbcopy`) with OSC52 fallback for SSH sessions.
* **Search & Replace:** Case-insensitive by default, with `%term` support for exact case matching. Features dedicated modals for workspace-wide find and replace operations.
* **Intelligent Completion:** Press `Tab` for context-aware completions, including language-specific variables, tags, attributes, keywords, and structural symbols.
* **Data Safety:** Utilizes atomic saves, per-project session restoration, and throttled background recovery (saved to `$XDG_STATE_HOME/az-rust`).
* **Terminal Native:** Raw-mode `stty` integration, 5MB bracketed paste support, truecolor rendering, wide-character awareness, and clean terminal state restoration on exit.
>>>>>>> refs/remotes/origin/main

---

## Language Support (42 Built-In)

`az` automatically detects file types by extension and filename, applying targeted syntax highlighting, comment handling, and symbol extraction. Override the active language at any time via `Ctrl+P` → `set [Language]`.

<<<<<<< HEAD
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
| Bash | `*.sh`, `*.bash`, `*.zsh`, `.bashrc`, `.zshrc`, `.profile`, other shell rc names, `#!` when the name is plain | `#!`, `#` comments, `$VAR`/`${}`, keywords/builtins, `fn` symbols |
| Dotenv | `.env`, `*.env` | `KEY=`, strings/numbers/bools, `#` comments, `export` |
| INI | `*.ini`, `*.conf`, `*.cfg` | `[sections]`, `key =/:`, `#`/`;` comments, symbols |
| Logs | `*.log` | timestamps + `ERROR` red / `WARN` yellow / `INFO` green / `DEBUG` dim |
| Rust | `*.rs` | keywords/types/macros/`#[attrs]`, `//` + `/* */`, `fn/struct/enum` symbols |
| Nginx | `nginx.conf` | blocks/directives, `$vars`, `#` comments, symbols |
| Apache | `.htaccess`, `httpd.conf` | directives, `<Sections>`, `#` comments, symbols |
| Dockerfile | `Dockerfile*` | `FROM`/`RUN`/… instructions, `$vars`, `#` comments |
| systemd | `*.service`, `*.timer` | `[Sections]`, `key=`, `#`/`;` comments, symbols |
| SQL | `*.sql` | keywords, strings/numbers, `--` + `/* */`, `CREATE TABLE` symbols |
| Python | `*.py`, `*.pyw`, `*.pyi` | keywords/builtins, `#` comments, `@decorators`, `def`/`class` symbols |
| Java | `*.java` | keywords/types, `//` + `/* */`, `class`/`interface`/`enum` + method symbols |
| C# | `*.cs`, `*.csx` | keywords/types, `//` + `/* */`, `[attrs]`, `class`/`namespace` symbols |
| C++ | `*.cpp`, `*.cxx`, `*.cc`, `*.hpp`, `*.hh`, `*.hxx` | keywords/types, `#include`, `class`/`namespace` symbols |
| C | `*.c`, `*.h` | keywords/types, `#include`, `struct`/`enum` symbols |
| Go | `*.go` | keywords/types, `//` + `/* */`, `func`/`type` symbols |
| Kotlin | `*.kt`, `*.kts` | keywords/types, `//` + `/* */`, `fun`/`class`/`object` symbols |
| Swift | `*.swift` | keywords/types, `//` + `/* */`, `@attrs`, `func`/`struct`/`protocol` symbols |
| Ruby | `*.rb`, `Gemfile`, `Rakefile` | keywords/builtins, `#` comments, `:symbols`, `def`/`class`/`module` symbols |
| Dart | `*.dart` | keywords/types, `//` + `/* */`, `class`/`mixin` symbols |
| Scala | `*.scala`, `*.sc` | keywords/types, `//` + `/* */`, `def`/`class`/`object`/`trait` symbols |
| R | `*.r` | keywords/builtins, `#` comments, `name <- function()` symbols |
| Lua | `*.lua` | keywords/builtins, `--` comments, `function` symbols |
| Perl | `*.pl`, `*.pm`, `*.t` | keywords/builtins, `#` comments, `$@%` sigils, `sub`/`package` symbols |
| Haskell | `*.hs`, `*.lhs` | keywords/types, `--` + `{- -}`, `name :: Type` + definition symbols |
| Elixir | `*.ex`, `*.exs` | keywords/builtins, `#` comments, `@attrs`, `def`/`defmodule` symbols |
| Clojure | `*.clj`, `*.cljs`, `*.cljc`, `*.edn` | keywords/builtins, `;` comments, `:keywords`, `defn`/`ns` symbols |
| Zig | `*.zig` | keywords/types, `//` comments, `fn`/`const`/`test` symbols |
| Julia | `*.jl` | keywords/types, `#` + `#= =#`, `function`/`struct`/`macro` symbols |
| Objective-C | `*.m`, `*.mm` | keywords/types, `#import`, `@directives`, `@interface` + `- (…)` method symbols |
| Plain | `*.txt` + fallback | no highlighting, always available via `set Plain` |
=======
| Language | Extensions/Files | Highlighting & Symbol Support |
| --- | --- | --- |
| **PHP** | `*.php`, `*.phtml` | Keywords, functions, `$vars`, comments (string+CSS aware), symbols |
| **Blade** | `*.blade.php` | `@directives`, `{{ }}`, mixed HTML+CSS+JS |
| **HTML** | `*.html`, `*.htm` | Tags, attributes, entities, void-tag auto-close |
| **CSS** | `*.css` | Properties, hex codes, `#id`, `@rules`, `!important`, selector symbols |
| **JavaScript** | `*.js`, `*.mjs`, `*.cjs`, `*.jsx` | Keywords, built-ins, template strings, regex, members, snippets |
| **TypeScript** | `*.ts`, `*.tsx`, `*.mts`, `*.cts` | JS features + `interface`/`type`/`enum`, symbols |
| **XML** | `*.xml`, `*.svg` | Tags, attributes, entities, `<?…?>`, `<!-- -->` |
| **Markdown** | `*.md` | Headings, bold, inline code, links, lists, symbols |
| **JSON** | `*.json`, `*.jsonc` | Strings, numbers, booleans, `//` and `/* */` (jsonc) |
| **TOML** | `*.toml` | `[sections]`, keys, values, `#` comments, symbols |
| **YAML** | `*.yaml`, `*.yml` | Keys, values, `#` comments, lists |
| **Bash** | `*.sh`, `*.bash`, `*.zsh` | `#!`, `$VAR`, keywords, built-ins, function symbols |
| **Dotenv** | `.env`, `*.env` | Keys, values, `export`, `#` comments |
| **INI** | `*.ini`, `*.conf`, `*.cfg` | `[sections]`, keys, `#`/`;` comments, symbols |
| **Logs** | `*.log` | Timestamps, semantic log levels (ERROR, WARN, INFO, DEBUG) |
| **Rust** | `*.rs` | Keywords, types, macros, `#[attrs]`, `fn/struct/enum` symbols |
| **Nginx** | `nginx.conf` | Blocks, directives, `$vars`, `#` comments, symbols |
| **Apache** | `.htaccess`, `httpd.conf` | Directives, `<Sections>`, `#` comments, symbols |
| **Dockerfile** | `Dockerfile*` | Instructions, `$vars`, `#` comments |
| **systemd** | `*.service`, `*.timer` | `[Sections]`, keys, `#`/`;` comments, symbols |
| **SQL** | `*.sql` | Keywords, strings, numbers, table creation symbols |
| **Python** | `*.py`, `*.pyw`, `*.pyi` | Keywords, `@decorators`, `def`/`class` symbols |
| **Java** | `*.java` | Keywords, types, `class`/`interface`/`enum`, method symbols |
| **C#** | `*.cs`, `*.csx` | Keywords, types, `[attrs]`, `class`/`namespace` symbols |
| **C++ / C** | `*.cpp`, `*.hpp`, `*.c`, `*.h` | Keywords, types, `#include`, `class`/`namespace`/`struct` symbols |
| **Go** | `*.go` | Keywords, types, `func`/`type` symbols |
| **Kotlin** | `*.kt`, `*.kts` | Keywords, types, `fun`/`class`/`object` symbols |
| **Swift** | `*.swift` | Keywords, `@attrs`, `func`/`struct`/`protocol` symbols |
| **Ruby** | `*.rb`, `Gemfile`, `Rakefile` | Keywords, `:symbols`, `def`/`class`/`module` symbols |
| **Dart** | `*.dart` | Keywords, types, `class`/`mixin` symbols |
| **Scala** | `*.scala`, `*.sc` | Keywords, types, `def`/`class`/`trait` symbols |
| **R** | `*.r` | Keywords, built-ins, `name <- function()` symbols |
| **Lua** | `*.lua` | Keywords, built-ins, `function` symbols |
| **Perl** | `*.pl`, `*.pm`, `*.t` | Keywords, `$@%` sigils, `sub`/`package` symbols |
| **Haskell** | `*.hs`, `*.lhs` | Keywords, types, definition symbols |
| **Elixir** | `*.ex`, `*.exs` | Keywords, `@attrs`, `def`/`defmodule` symbols |
| **Clojure** | `*.clj`, `*.cljs`, `*.edn` | Keywords, `:keywords`, `defn`/`ns` symbols |
| **Zig** | `*.zig` | Keywords, types, `fn`/`const`/`test` symbols |
| **Julia** | `*.jl` | Keywords, types, `function`/`struct`/`macro` symbols |
| **Objective-C** | `*.m`, `*.mm` | Keywords, types, `#import`, `@directives`, method symbols |
| **Plain Text** | `*.txt` | Fallback mode. No highlighting, always available. |
>>>>>>> refs/remotes/origin/main

**Extending Languages via AI:**
`az` includes an [AI Agent Guide](AGENTS.md) that teaches LLM coding assistants the plugin API, architecture, and testing requirements. To add a new language, instruct your AI to scaffold the language implementation (`src/plugins/x.rs`), wire the routing, and add regression tests following the [Plugin Guide](https://www.google.com/search?q=PLUGIN_GUIDE.md).

---

## Recent Updates

### Version 2.6

* **20 New Language Plugins:** Added Python, Java, C#, C++, C, Go, Kotlin, Swift, Ruby, Dart, Scala, R, Lua, Perl, Haskell, Elixir, Clojure, Zig, Julia, and Objective-C. Includes auto-detection, dynamic keyword/type assignment, symbol extraction, and word completion.
* **Startup Update Check:** Silently compares the local version against the GitHub `main` branch (with short timeouts to prevent blocking). Alerts users to available upgrades via a modal dialog.
* **High-Frequency Mouse Scrolling:** Refactored event processing to capture high-burst touchpad scrolling. Kinetic scrolling is now fluid across the editor, sidebar, and pickers.
* **Robust Clipboard Processing:** Standard input is explicitly closed before awaiting system clipboards (`wl-copy`, `xclip`, etc.), eliminating hang states caused by broken clipboard pipes.

### Version 2.5

* **Global Replace in Files:** (`Ctrl+Shift+H`). Complete workflow for searching, reviewing match counts, and replacing terms across the entire project footprint (supports up to 3,000 files, skipping binaries and limits hits to 10k).
* **Right-Click Context Menus:** Native mouse integration for tab management, sidebar file operations (rename, delete, search here), and editor actions (cut/copy/paste, find/replace).
* **Enhanced Mouse Selection:** Drag-to-select functionality, double-click for word selection, triple-click for whole lines.
* **Blinking UI Alerts:** Status messages and prompts flash light blue to immediately draw attention.
* **Searchable Shortcuts Dialog:** (`Ctrl+K`). Browse and filter all available keyboard shortcuts within the editor.

---

## Building from Source

To compile the editor locally without relying on the remote script:

```sh
./build.sh

```

*This compiles `az` to `target/release/az` and installs it to `~/.local/bin/az`. It will automatically update your `~/.profile` to include `~/.local/bin` in your PATH if necessary (requires a terminal restart or sourcing `~/.profile`).*

To specify a custom binary directory:

```sh
AZ_BIN_DIR="$HOME/bin" ./build.sh

```

*Requires `cargo` (recommended) or `rustc`.*

---

## License

`az` is licensed under the [WTFPL](https://www.wtfpl.net/) (Do What The Fuck You Want To Public License).

You are free to copy, distribute, modify, and utilize this software for any purpose, including commercial applications, without restrictions. The software is provided "as is" without warranty of any kind.
