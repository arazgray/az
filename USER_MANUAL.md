# az 2.1 — User Manual

`az` is a small, sane terminal text editor. Open fast, type immediately, stay keyboard-first.

## 1. Install & Run

```sh
./build.sh
# installs to ~/.local/bin/az
az --help
az --version
```

Requirements: `cargo` (recommended) or `rustc`. No external Rust crates.

Remote install:

```sh
curl -fsSL https://raw.githubusercontent.com/arazgholami/az/refs/heads/main/install.sh | sh
```

## 2. Opening Things

```sh
az file.php          # open file (parent dir becomes project root)
az project/          # open folder (tree focused, welcome screen)
az file.php:20       # open file at line 20
az :20               # open CWD, same as `az .` (line arg ignored for folders)
az newfile.txt       # non-existent path -> new file tab with that path
az -h / --help
az -V / --version
```

If you pass a folder, you start in the tree. Press `Enter` on a file to edit, or `Ctrl+O` to quick-open.

## 3. Screen Layout

```
┌ topbar: az | sane editor | tabs 1:main.rs 2:README.md | 02:30 PM 30/09/2026 ┐
│ tree (28 cols default) │ gutter Ln │ editor text (horizontal scroll)        │
│                        │           │                                        │
└ status: tree/editor path saved/modified SYNTAX tree shown/hidden | message ┘
  └ popups: welcome, help (Ctrl+/), quick open (Ctrl+O), palette (Ctrl+P)     ┘
```

- Topbar: tabs `1:name`, `*` = modified. Press `Esc` to flash `Alt+1-9` hints.
- Tree: `▾` open dir, `▸` closed dir. Colors by extension (PHP purple, HTML orange, JS yellow, etc.).
- Gutter: line numbers, min width 4.
- Status: `focus path state syntax tree | message stats Ln,Col Lines Words`.
- No word wrap: long lines scroll horizontally. Cursor stays visible.

## 4. Keyboard — Complete Map

### Global (works in tree + editor)

| Keys | Action |
|------|--------|
| `Ctrl+S` | Save (prompts Save-as for Untitled) |
| `Ctrl+O` | Quick open file / symbol / `file:line` / `:line` |
| `Ctrl+P` | Command palette |
| `Ctrl+F` | Find in file (`%term` = case-sensitive) |
| `Ctrl+L` | Find next (uses last pattern) |
| `Ctrl+R` | Replace (one or all) |
| `Ctrl+G` | Go to line |
| `Ctrl+T` | Toggle tree focus (shows tree if hidden) |
| `Ctrl+H` | Hide/show tree — **tree focus only**. In editor, `0x08` acts as Backspace for terminals sending `^H` |
| `Ctrl+N` | New file tab |
| `Ctrl+D` | Close tab (asks if modified) |
| `Ctrl+Q` | Quit (asks if any tab modified) |
| `Ctrl+Z` / `Ctrl+Y` / `Ctrl+Shift+Z` | Undo / Redo |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` / `Ctrl+A` | Copy / Cut / Paste / Select all (OSC52 system copy) |
| `Ctrl+W` / `Ctrl+Backspace` | Delete current line |
| `Ctrl+/` (`Ctrl+_` / `0x1F`) | Help / welcome |
| `Ctrl+Shift+F` | Project search |
| `Alt+1`..`Alt+9` (`Esc` then `1..9`) | Switch tab (first 9 visible) |
| `Esc` | Clear selection + show Alt hints |

### Editor Only

| Keys | Action |
|------|--------|
| Arrows, Home/End, PgUp/PgDn | Move (Shift = select) |
| `Ctrl+Left/Right` | Word jump (Shift = select) |
| `Backspace` (`0x7F` or `0x08`) / `Del` | Delete backward / forward |
| `Enter` | Newline with auto-indent (`{ [ ( :` adds 4 spaces) |
| `(` `{` | Auto-close `()` `{}` |
| `>` | Auto-close HTML tag e.g. `<div>` -> `<div></div>` (skips void `<br>`, `<img>`) |
| `Tab` | Accept autocomplete, else insert `\t` (renders 4 spaces) |
| `Up/Down` in autocomplete | Navigate, `Enter/Tab` accept, `Esc` close |

### Tree Only

| Keys | Action |
|------|--------|
| `Up/Down`, `PgUp/PgDn` | Move selection |
| `Enter` | Open file / expand-collapse dir |
| `Left` / `Right` | Collapse / Expand dir |
| `n` / `N` (Shift+N) | New file / New folder in selected dir |
| `r` / `R` | Rename (never root, refuses existing target) |
| `Del` | Delete file/folder (asks, closes affected tabs) |
| `+` / `=` / `-` / `_` | Tree width +2 / -2 (18..44) |

Popups (`Ctrl+O`, `Ctrl+P`, prompts): `Up/Down` or `Ctrl+P`/`Ctrl+N` navigate, `Enter` confirm, `Esc` cancel, `Ctrl+U` clear line.

## 5. Core Workflows

### Tabs
- `Ctrl+N` new, `Ctrl+D` close, `Alt+1-9` switch. Modified `*` in topbar. Untitled hidden until edited when folder opened.
- Session auto-saves open file paths + cursor + syntax + expanded dirs per project root. Reopens on next `az project/`.

### Quick Open (`Ctrl+O`)
Type to fuzzy-match files + symbols. Forms:
- `main` -> files
- `run` -> symbols (`file #symbol`)
- `app.php:20` -> file at line
- `:20` -> go to line in current file
- Empty -> first 14 files

Skips: `.git node_modules vendor .idea .vscode target dist build __pycache__ .next .nuxt`. Limit 2500 files, symbols from first 600 programming files <1MB.

### Command Palette (`Ctrl+P`)
`Save, Save as, New file/folder, Rename/Delete, Go to line, Search project, Set syntax …, Find/Replace, Toggle sidebar, Focus tree/editor, Close tab, Demo mode, Welcome, Quit`. Type `set php` to force language.

### Find / Replace
- Find: case-insensitive default, `%Foo` = case-sensitive. `Enter` finds, selection covers match. `Ctrl+L` finds next (wraps with `(wrapped)` notice).
- Replace: prompts `Replace:`, `Replace with:`, `Replace all? y/N`. Single replaces current match; all scans whole file (cap 20k ops, undoable as one entry).

### Project Search (palette or `Ctrl+Shift+F`)
Substring case-insensitive across ≤3000 files, skips >5MB and binaries (via `read_to_string` failure). Shows `file:line + snippet`. `Enter` opens at line.

### Go to Line
`Ctrl+G`, `:20` in quick open, or `az file:20`. Clamps to EOF.

### Autocomplete (`Tab`)
Context-aware per language + document words. `Tab`/`Enter` accept, `Esc` close. Sources:
- PHP: `$vars`, `->`/`::` members, keywords/functions/symbols
- HTML: `<tag`, attributes inside tags
- CSS: properties, `prop: value`, `@rules`
- JS: `obj.` members, keywords/builtins/snippets
- Blade: `@directives`

### Syntax Modes
Auto by extension (`.blade.php` -> Blade, `.php/.phtml` -> PHP, `.html/.htm/.xml/.svg` -> HTML, `.css` -> CSS, `.js/.mjs/.cjs/.jsx` -> JS, else Plain). Override via palette `Set syntax …` or `Set syntax Auto` to revert. Status shows `PHP manual` when forced.

### Tree File Ops
Select dir or file, then `n/N/r/Del` or palette equivalents. Create auto-makes parent dirs. Rename updates open tabs (including children if dir renamed). Delete removes tabs pointing inside.

## 6. Recovery & Sessions

State dir: `$XDG_STATE_HOME/az-rust` or `~/.local/state/az-rust`.

- `session-<hash>.txt`: per-project open tabs, cursor, `tab_index`, expanded dirs. Restored only when starting from empty Untitled (folder open).
- `recovery/<hash>.rec`: unsaved buffers, throttled to 1 write/250ms per tab. Deleted on save/close.
- On launch, `Recover unsaved changes for NAME? y/N` per file. `y` restores as modified tab, `N` deletes recovery.

Atomic saves: write temp `.NAME.aztmp.PID` then rename, preserving permissions.

## 7. Copy/Paste

- Internal clipboard + OSC52 `\x1b]52;c;BASE64\a` for terminal/system copy. Works over SSH in supporting terminals.
- Bracketed paste (`\x1b[200~ … \x1b[201~`) inserts verbatim (up to 5MB). `Ctrl+V` pastes internal clipboard.

## 8. Troubleshooting

| Symptom | Fix |
|---------|-----|
| Backspace toggles tree | Your terminal sends `^H (0x08)`. In editor it now acts as Backspace; use tree focus for `Ctrl+H`. If still wrong, use `Del` or remap terminal Backspace to `0x7F`. |
| Colors look wrong | Needs truecolor (`38;2`/`48;2`). Use modern terminal (Kitty, WezTerm, Alacritty, recent GNOME). `TERM=xterm-256color` minimum. |
| `Alt+1-9` doesn't switch | Terminal sends `Esc+num`. Press `Esc` alone to see hints, then `Esc 1`. Or enable Alt-as-Esc in terminal settings. |
| Quick open slow / missing files | Large `target/` ignored by default, but 600-file symbol scan can still stall on huge repos. Open subfolder as root to narrow scope. |
| Clock frozen | Topbar clock caches per minute + per render; minute change triggers re-render. Normal. |
| Recovery prompt loop | `~/.local/state/az-rust/recovery/*.rec` — delete stale files if you always answer `N`. |
| `az file:20` opens file named `file:20` | Fixed in 2.0.1+: colon splits line. Quote paths with spaces. |
| Narrow terminal (<40 cols) | Picker clamps to `cols-2`, tree min 18. Hide tree (`Ctrl+H` in tree) for more width. |

## 9. Known Limitations (by design)

- No word wrap (horizontal scroll only), no mouse, no split panes.
- No regex search, no multi-cursor.
- Undo `revision` is monotonic: undo after save still shows `modified` until next save (content matches but dirty flag stays). Save to clear.
- Replace-all limit 20k ops, project search 80 results, quick open 14 shown.
- Binary files open via lossy UTF-8; saving rewrites as UTF-8.

## 10. Tips

- `az .` + `Ctrl+O` is fastest project navigation.
- `Ctrl+P` then `set js` forces JS highlighting in mixed files.
- `+`/`-` in tree to fit long filenames.
- `Tab` in PHP after `$` lists all `$vars` in file (up to 20k lines scanned).
- `Ctrl+L` repeatedly to hop through matches; watch `(wrapped)`.
