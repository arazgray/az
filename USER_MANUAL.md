# az 2.6 — User Manual

`az` is a fast, small & sane text editor. Open fast, type immediately, stay keyboard-first.

## 1. Install & Run

```sh
./build.sh
# installs to ~/.local/bin/az
az --help
az --version
```

Requirements: `cargo` (recommended) or `rustc`. No external Rust crates.

Remote install (also upgrades an existing install in place):

```sh
curl -fsSL https://raw.githubusercontent.com/arazgholami/az/refs/heads/main/install.sh | sh
```

On every launch `az` checks GitHub for a newer release (short timeout, silent when offline). When one exists, a modal shows the version and the upgrade command above. Set `AZ_NO_UPDATE_CHECK=1` to skip the check.

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
┌ az  [Open] [Commands] [Shortcuts]                        02:30 PM 30/09/2026 ┐
├──────────────────────────────────────────────────────────────────────────┤
│                            ├ tabs 1:main.rs 2:README.md (`*` = modified)          ┤
│ tree (28 cols default) │ gutter Ln │ editor text (horizontal scroll)        │
│                        │           │                                        │
├──────────────────────────────────────────────────────────────────────────┤
└ status chips: path state(modified-only) syntax tree | message | stats    ┘
  └ popups: welcome, shortcuts (Ctrl+K), quick open (Ctrl+O), palette (Ctrl+P) ┘
```

- Titlebar: blue `az` chip + mode (`editor`/`tree`) chip, clickable plain-text buttons (`Open` → quick open, `Commands` → palette, `Shortcuts` → shortcut list), clock — full-width separator below. Tab bar under the separator, above the editor: tabs `1:name`, `*` = modified, inactive tabs shaded lighter; click to switch, middle-click to close.
- Tree: `▾` open dir, `▸` closed dir. Colors by extension (PHP purple, HTML orange, JS yellow, etc.).
- Gutter: line numbers, min width 4.
- Status: one chip per item — path, state (`modified`, orange, only when dirty), syntax (purple), tree (cyan) | message (flashes light blue on change, red on a permission error) | stats (yellow). Prompts blink light blue until answered. The root-password prompt blinks red.
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
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` / `Ctrl+A` | Copy / Cut / Paste / Select all (OS clipboard; OSC52 fallback). Paste reads the OS clipboard first |
| `Ctrl+W` / `Ctrl+Backspace` | Delete current line |
| `Ctrl+K` | Keyboard shortcuts (searchable dialog, `Enter`/`Esc` closes) |
| `Ctrl+Shift+O` | Find in files (separate modal, `%term` = case-sensitive) |
| `Ctrl+Shift+H` | Replace in files (dialog, `%term` = case-sensitive) |
| `Alt+1`..`Alt+9` (`Esc` then `1..9`) | Switch tab (first 9 visible) |
| Click tab | Switch tab (inactive tabs shaded lighter) |
| Middle-click tab | Close tab (asks if modified) |
| Right-click | Context menu: tab (Close, Copy file path), sidebar and editor menus (see Mouse) |
| `Esc` | Clear selection + show Alt hints |

### Editor Only

| Keys | Action |
|------|--------|
| Arrows, Home/End, PgUp/PgDn | Move (Shift = select) |
| `Ctrl+Left/Right` | Word jump (Shift = select) |
| `Ctrl+E` | Go to end of line |
| `Ctrl+Home` / `Alt+Up` | Go to start of file (Shift = select) |
| `Ctrl+End` / `Alt+Down` | Go to end of file (Shift = select) |
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
| `Enter` or click | Open file / expand-collapse dir |
| `Left` / `Right` | Collapse / Expand dir |
| Double-click file/dir | Rename (never root, refuses existing target) |
| Click editor | Move cursor there |
| Wheel over tree/editor | Scroll one line (tree selection / editor cursor) |
| `n` / `N` (Shift+N) | New file / New folder in selected dir |
| `r` / `R` | Rename (never root, refuses existing target) |
| `Del` | Delete file/folder (asks, closes affected tabs) |
| `+` / `=` / `-` / `_` | Tree width +2 / -2 (18..44) |

Popups (`Ctrl+O`, `Ctrl+Shift+O`, `Ctrl+P`, prompts): `Up/Down` or `Ctrl+P`/`Ctrl+N` navigate, `Enter` confirm, `Esc` cancel, `Ctrl+U` clear line.

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
`Save, Save as, New file/folder, Rename/Delete, Go to line, Go to Start/End of Line, Go to Start/End of File, Find in files, Replace in files, Set syntax …, Find/Replace, Toggle sidebar, Focus tree/editor, Close tab, Demo mode, Keyboard shortcuts, Quit`. Type `set php` to force language.

### Find / Replace
- Find: case-insensitive default, `%Foo` = case-sensitive. `Enter` finds, selection covers match. `Ctrl+L` finds next (wraps with `(wrapped)` notice).
- Replace: prompts `Replace:`, `Replace with:`, `Replace all? y/N`. Single replaces current match; all scans whole file (cap 20k ops, undoable as one entry).

### Find in Files (palette or `Ctrl+Shift+O`)
Separate modal from Quick Open. Type to live-search file contents across the opened folder:
- Case-insensitive by default, `%Foo` = case-sensitive (same `%` rule as `Ctrl+F`).
- Substring match across ≤3000 files, skips >5MB and binaries (via `read_to_string` failure). Shows `file:line + snippet` (80 results max).
- `Enter` opens the selected match at that line, `Esc` cancels.
- Requires a Kitty-protocol terminal for the direct shortcut (`CSI-u`); otherwise use `Ctrl+P` → `Find in files`. (`Ctrl+Shift+O` is used because most terminals reserve `Ctrl+Shift+F` for their own search.)

### Replace in Files (palette or `Ctrl+Shift+H`)
Dialog across the opened folder (or a sidebar subfolder via right-click → `Search & Replace here`):
- Prompts `Replace in files:`, `Replace with:`, then `Replace N in M files (…) ? y/N` with the real counts. `%Foo` = case-sensitive (same `%` rule as `Ctrl+F`).
- Same file scope as Find in Files: ≤3000 files, skips >5MB and binaries, 10k-match cap (narrow your search if capped). CRLF files are normalized to LF on write.
- Open unmodified tabs reload from disk (cursor kept, undo cleared); files with unsaved open buffers are skipped and reported.
- Requires a Kitty-protocol terminal for the direct shortcut; otherwise use `Ctrl+P` → `Replace in files`.

### Go to Line / Start / End
- `Ctrl+G`, `:20` in quick open, or `az file:20` for a numbered line. Clamps to EOF.
- `Home` / `Ctrl+P` → `Go to Start of Line`, `End` or `Ctrl+E` / `Go to End of Line` for line ends.
- `Ctrl+Home` or `Alt+Up` / `Go to Start of File`, `Ctrl+End` or `Alt+Down` / `Go to End of File` for file top/bottom. Hold `Shift` with any of these to select.

### Autocomplete (`Tab`)
Context-aware per language + document words. `Tab`/`Enter` accept, `Esc` close. Sources:
- PHP: `$vars`, `->`/`::` members, keywords/functions/symbols
- HTML: `<tag`, attributes inside tags
- CSS: properties, `prop: value`, `@rules`
- JS: `obj.` members, keywords/builtins/snippets
- Blade: `@directives`
- All other languages: keywords (+types/builtins where relevant) and file symbols (`def`/`class`/`func`/`fn`/…)

### Syntax Modes
Auto by extension (`.blade.php` -> Blade, `.php/.phtml` -> PHP, `.html/.htm` -> HTML, `.css` -> CSS, `.js/.mjs/.cjs/.jsx` -> JS, `.ts/.tsx/.mts/.cts` -> TS, `.xml/.svg` -> XML, `.py` -> Python, `.java` -> Java, `.cs` -> C#, `.cpp/.hpp` -> C++, `.c/.h` -> C, `.go` -> Go, `.kt` -> Kotlin, `.swift` -> Swift, `.rb` (+`Gemfile`) -> Ruby, `.dart` -> Dart, `.scala` -> Scala, `.r` -> R, `.lua` -> Lua, `.pl` -> Perl, `.hs` -> Haskell, `.ex` -> Elixir, `.clj` -> Clojure, `.zig` -> Zig, `.jl` -> Julia, `.m/.mm` -> Objective-C, plus Markdown/JSON/TOML/YAML/Bash/Dotenv/INI/Log/Rust/Nginx/Apache/Dockerfile/systemd/SQL, else Plain). Override via palette `Set syntax …` or `Set syntax Auto` to revert. Status shows `PHP manual` when forced.

### Tree File Ops
Select dir or file, then `n/N/r/Del` or palette equivalents. Create auto-makes parent dirs. Rename updates open tabs (including children if dir renamed). Delete removes tabs pointing inside.

### Mouse
Requires a terminal with SGR mouse reporting (`1000`/`1002`/`1006`; most modern terminals, works over SSH).
- Titlebar + tab bar: click a tab to switch (inactive tabs render lighter). Middle-click a tab to close it (asks if modified).
- Sidebar: click a folder to expand/collapse, click a file to open it. Double-click a file (<500ms, same path) to rename it. The open file's row is highlighted.
- Editor: click to move the cursor there (gutter click goes to line start; tab/wide chars map correctly). Drag with the button held to select text (selection follows the cursor). Double-click selects the word under the caret, triple-click selects the whole line. Clicking focuses the editor.
- Wheel: scrolls one line per notch — the sidebar (`tree_index ±1`), the editor (cursor `±1` line), and picker dialogs/menus (moves selection by one). Several reports that arrive together each move one line, then the screen paints once. Scroll never steals focus; clicks set it. Terminals without SGR mouse fall back to legacy X10 reports.
- Right-click opens a context menu (`Up/Down` or `Ctrl+P`/`Ctrl+N`, `Enter` confirm, `Esc` or click-away cancels, `1-9` quick-pick):
  - Tab: `Close tab`, `Copy file path`.
  - Sidebar: `Open`, `Copy file path`, `Rename`, `Delete`, `Search here`, `Search & Replace here` (each opens its dialog; searches scope to that folder).
  - Editor: `Cut`, `Copy`, `Paste`, `Select All`, `Find/Replace in File`, `Find/Replace in Files`, `Go to Line` (right-click keeps an active selection so Copy/Cut work on it).

## 6. Recovery & Sessions

State dir: `$XDG_STATE_HOME/az-rust` or `~/.local/state/az-rust`.

- `session-<hash>.txt`: per-project open tabs, cursor, `tab_index`, expanded dirs. Restored only when starting from empty Untitled (folder open).
- `recovery/<hash>.rec`: unsaved buffers, throttled to 1 write/250ms per tab. Deleted on save/close.
- On launch, `Recover unsaved changes for NAME? y/N` per file. `y` restores as modified tab, `N` deletes recovery.

Atomic saves: write temp `.NAME.aztmp.PID` then rename, preserving permissions. If that fails with permission denied, the status line blinks red and asks `Permission denied. Root password:` (characters show as `*`). `Enter` writes the buffer as root via `sudo -S` (the password is sent on stdin, never on the command line). `Esc` or an empty entry cancels. A wrong password blinks the sudo error in red. This does not run when `az` is already root.

## 7. Copy/Paste

- System copy cascade: `wl-copy` (Wayland) → `xclip`/`xsel` (X11) → `pbcopy` (macOS) → Wayland `ext-data-control` (no extra tool; a helper process holds the selection until the next copy), then best-effort OSC52 `\x1b]52;c;BASE64\a`. Over SSH, OSC52 goes first (only path to the local clipboard). Status shows `Copied` when a tool or the compositor confirmed, `Copied (OSC52)` when only the terminal sequence was sent.
- `Ctrl+V` reads the OS clipboard first and pastes that text. If nothing outside is available, it pastes the editor's own clipboard.
- Bracketed paste (`\x1b[200~ … \x1b[201~`) and a raw burst of text insert in one step (up to 5MB), so a long paste is one undo entry.

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
| Copy says `Copied` but outside paste is empty | Plain `Copied` means a clipboard tool or the Wayland compositor accepted the text. `Copied (OSC52)` means only the terminal sequence was sent (common miss in Guake/VTE). On Wayland, `az` speaks `ext-data-control` itself when `wl-copy` is not installed. In tmux set `set -g set-clipboard on`. |
| `sudo az` says command not found | `sudo` does not search `~/.local/bin`. Re-run `./build.sh` and approve the prompt so it can install `/usr/local/bin/az`. |
| Save says permission denied | The status line blinks red and asks for the root password, then saves with `sudo`. `Esc` cancels. |
| Mouse clicks/scroll do nothing | Terminal doesn't forward SGR mouse (`1000`/`1002`/`1006`). Try Kitty, WezTerm, Alacritty, or recent GNOME Terminal. Legacy X10 (`ESC[M`) is handled as fallback. Keyboard works everywhere. |
| Wheel does nothing | Needs content to move (short files / few tree rows won't visibly scroll). Pickers scroll their selection; prompts ignore wheel. |

## 9. Known Limitations (by design)

- No word wrap (horizontal scroll only), no split panes.
- No regex search, no multi-cursor.
- Mouse: click + drag-select + wheel supported in tree/editor; requires a terminal with SGR mouse reporting (`1000`/`1002`/`1006`).
- Undo `revision` is monotonic: undo after save still shows `modified` until next save (content matches but dirty flag stays). Save to clear.
- Replace-all limit 20k ops, find in files 80 results, quick open 14 shown.
- Binary files open via lossy UTF-8; saving rewrites as UTF-8.

## 10. Tips

- `az .` + `Ctrl+O` is fastest project navigation.
- `Ctrl+P` then `set js` forces JS highlighting in mixed files.
- `+`/`-` in tree to fit long filenames.
- `Tab` in PHP after `$` lists all `$vars` in file (up to 20k lines scanned).
- `Ctrl+L` repeatedly to hop through matches; watch `(wrapped)`.
