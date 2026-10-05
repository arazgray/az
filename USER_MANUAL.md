# az 3.2 — User Manual

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

- Titlebar: blue `az` chip + mode (`editor`/`tree`) chip, clickable plain-text buttons (`Open` → quick open, `Commands` → palette, `Shortcuts` → shortcut list), red `Quit` button left of the clock — full-width separator below. Tab bar under the separator, above the editor: at most nine tabs, windowed around the current one (`1:name` numbers that window), `*` = modified, inactive tabs shaded lighter. Click a tab to switch, middle-click to close. `+` at the end opens an empty unsaved tab (middle-click and right-click on `+` do nothing).
- Tree: `▾` open dir, `▸` closed dir. Colors by extension (PHP purple, HTML orange, JS yellow, shell rc files red, etc.). The selected row is a highlight. The blinking caret is only in the editor.
- Gutter: line numbers, min width 4.
- Status: one chip per item — path, state (`modified`, orange, only when the text differs from the last save), syntax (purple), tree (cyan) | message (flashes light blue on change, red on a permission error, then one clear frame) | stats (yellow). `Col` is the screen column. On a narrow terminal the tree chip, then the syntax chip, drop so the message stays. Prompts blink light blue until answered. The root-password prompt blinks red.
- No word wrap. Long lines scroll horizontally, and the caret stays on screen (a tab is four columns). When the editor or the sidebar is wider than its pane, the last content row is a horizontal scrollbar for both. Wheel on that row moves four columns. Drag the thumb to jump. When a pane is taller than the viewport, it draws a vertical bar on its right edge (`┃` thumb, `│` track). Click or drag that bar to pan. The editor caret stays where it is. A click, the wheel, or the arrow keys in the sidebar follow the selection again.

## 4. Keyboard — Complete Map

### Global (works in tree + editor)

| Keys | Action |
|------|--------|
| `Ctrl+S` | Save (asks `Filename:` for an unsaved tab) |
| `Ctrl+O` | Quick open file / symbol / `file:line` / `:line` |
| `Ctrl+P` | Command palette |
| `Ctrl+F` | Find dialog for this file (`%term` = case-sensitive) |
| `Ctrl+L` | Find next (uses last pattern; also inside the find dialog) |
| `Ctrl+R` | Search & replace dialog |
| `Ctrl+G` | Go to line |
| `Ctrl+T` | Toggle tree focus (shows tree if hidden) |
| `Ctrl+H` | Hide/show tree — **tree focus only**. In editor, `0x08` acts as Backspace for terminals sending `^H` |
| `Ctrl+N` | New empty unsaved tab |
| `Ctrl+D` | Close tab (asks if modified) |
| `Ctrl+Q` | Quit (asks if any tab modified) |
| `Ctrl+Z` / `Ctrl+Y` / `Ctrl+Shift+Z` | Undo / Redo |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` / `Ctrl+A` | Copy / Cut / Paste / Select all. A confirmed OS copy pastes from the system clipboard; `Copied (OSC52)` pastes the editor text |
| `Ctrl+W` / `Ctrl+Backspace` | Delete current line |
| `Ctrl+K` | Keyboard shortcuts (searchable dialog, `Enter`/`Esc` closes) |
| `Ctrl+Shift+O` | Find in files (separate modal, `%term` = case-sensitive) |
| `Ctrl+Shift+H` | Search & replace dialog for the project (`%term` = case-sensitive) |
| `Alt+1`..`Alt+9` (`Esc` then `1..9`) | Switch a tab in the visible window |
| `Ctrl+Tab` / `Ctrl+Shift+Tab` | Next / previous open tab |
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
| `(` `{` `[` | Auto-close an empty pair. On a selection, these and `"` `'` wrap it |
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
| Wheel over tree/editor | One row: sidebar selection, or the editor viewport (caret stays) |
| `n` / `N` (Shift+N) | New file / New folder in selected dir |
| `r` / `R` | Rename (never root, refuses existing target) |
| `Del` | Delete file/folder (asks, closes affected tabs) |
| `+` / `=` / `-` / `_` | Tree width +2 / -2 (18..44) |

Popups (`Ctrl+O`, `Ctrl+Shift+O`, `Ctrl+P`, `Ctrl+K`, search & replace): `Up/Down` or `Ctrl+P`/`Ctrl+N` navigate a list, `Enter` confirm, `Esc` cancel, `Ctrl+U` clear the line, click a row to activate it, click outside to close. Prompts: Left/Right/Home/End move the cursor, Up/Down walk recent answers (not the root-password prompt).

## 5. Core Workflows

### Tabs
- `Ctrl+N` or `+` on the tab bar opens an empty unsaved tab. `Ctrl+S` asks `Filename:` and writes that path. `Ctrl+D` closes (asks if modified).
- At most nine tabs are drawn, windowed around the current one. `Alt+1-9` switches inside that window. `Ctrl+Tab` / `Ctrl+Shift+Tab` cycle every open tab. The wheel on the tab bar does the same. Modified `*` in the tab bar. The initial Untitled stays hidden until edited when a folder is opened.
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
`Save, Save as, New file/folder, Rename/Delete, Go to line, Go to Start/End of Line, Go to Start/End of File, Welcome, Undo, Redo, Select all, Find in files, Replace in files, Set syntax …, Find/Replace, Toggle sidebar, Focus tree/editor, Close tab, Keyboard shortcuts, Quit`. Type `set php` to force language. `Welcome` opens the welcome dialog again.

### Find
`Ctrl+F` opens the Find dialog for the current file:
- One query field. The count updates as you type (`%Foo` = case-sensitive). `Enter` or Next jumps to the next match and leaves the dialog open. `Tab` moves across the query, Next, and Close. `Esc`, Close, or a click outside closes it. `Ctrl+L` finds the next match with the last pattern.

### Replace
`Ctrl+R` opens Search & Replace for the current file. `Ctrl+Shift+H` opens the same dialog on the project:
- Path, find, and replace are editable. An empty path, or the path of the current file, means this buffer. A directory searches that folder. Another file rewrites that file.
- The count under the fields updates as you type (`%Foo` = case-sensitive). `Tab` / `Shift+Tab` move across the fields, Replace, and Cancel. `Enter` replaces every match in the target and leaves the dialog open so the count refreshes. `Esc`, Cancel, or a click outside closes it. `Ctrl+L` finds the next match in the current buffer.
- Project replace keeps the caps (3000 files, files over 5MB skipped, 10k matches). Open tabs with unsaved edits are skipped. A file that used CRLF is written back with CRLF. Replace in the current buffer is one undo entry.

### Find in Files (palette or `Ctrl+Shift+O`)
Separate modal from Quick Open. Type to live-search file contents across the opened folder:
- Case-insensitive by default, `%Foo` = case-sensitive (same `%` rule as `Ctrl+F`).
- Substring match across ≤3000 files, skips >5MB and binaries (via `read_to_string` failure). Shows `file:line + snippet` (80 results max).
- `Enter` opens the selected match at that line, `Esc` cancels.
- Requires a Kitty-protocol terminal for the direct shortcut (`CSI-u`); otherwise use `Ctrl+P` → `Find in files`. (`Ctrl+Shift+O` is used because most terminals reserve `Ctrl+Shift+F` for their own search.)

### Replace in Files (palette or `Ctrl+Shift+H`)
Same dialog as Replace, opened on the project root (or a sidebar folder via right-click → `Search & Replace here`). The count is the confirmation. There is no extra y/N prompt. Requires a Kitty-protocol terminal for the direct shortcut; otherwise use `Ctrl+P` → `Replace in files`.

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
Auto by extension (`.blade.php` -> Blade, `.php/.phtml` -> PHP, `.html/.htm` -> HTML, `.css` -> CSS, `.js/.mjs/.cjs/.jsx` -> JS, `.ts/.tsx/.mts/.cts` -> TS, `.xml/.svg` -> XML, `.py` -> Python, `.java` -> Java, `.cs` -> C#, `.cpp/.hpp` -> C++, `.c/.h` -> C, `.go` -> Go, `.kt` -> Kotlin, `.swift` -> Swift, `.rb` (+`Gemfile`) -> Ruby, `.dart` -> Dart, `.scala` -> Scala, `.r` -> R, `.lua` -> Lua, `.pl` -> Perl, `.hs` -> Haskell, `.ex` -> Elixir, `.clj` -> Clojure, `.zig` -> Zig, `.jl` -> Julia, `.m/.mm` -> Objective-C, plus Markdown/JSON/TOML/YAML/Bash/Dotenv/INI/Log/Rust/Nginx/Apache/Dockerfile/systemd/SQL, else Plain). Bash also matches `.bashrc`, `.bash_profile`, `.zshrc`, `.profile`, and the other shell startup names, and a `#!/bin/bash` or `#!/usr/bin/env sh` first line when the filename itself is plain (that is why `~/.bashrc` highlights). A `.py` file keeps Python even if the shebang says bash. Override via palette `Set syntax …` or `Set syntax Auto` to revert. Status shows `PHP manual` when forced. A manual mode wins over the filename and the shebang. Plain (`.txt` and any other unrecognized file) colors punctuation such as `~!@#$%^&*()[]` orange, and paints alternate rows in `#1a1b26` and `#1f2335`. Other languages keep a single background.

### Tree File Ops
Select dir or file, then `n/N/r/Del` or palette equivalents. Create auto-makes parent dirs. Rename updates open tabs (including children if dir renamed). Delete removes tabs pointing inside.

### Mouse
Requires a terminal with SGR mouse reporting (`1000`/`1002`/`1006`; most modern terminals, works over SSH).
- Titlebar + tab bar: click a tab to switch (inactive tabs render lighter). Middle-click a tab to close it (asks if modified).
- Sidebar: click a folder to expand/collapse, click a file to open it. Double-click a file (<500ms, same path) to rename it. The open file's row is highlighted.
- Editor: click to move the cursor there (gutter click goes to line start; tab/wide chars map correctly). Drag with the button held to select text (selection follows the cursor). Double-click selects the word under the caret, triple-click selects the whole line. Clicking focuses the editor.
- Wheel: one row per notch. The sidebar moves its selection. The editor pans, and the caret and selection stay. The tab bar cycles tabs. A horizontal scrollbar row moves that pane by four columns. A vertical bar, drawn only when that pane is taller than the viewport, pans on click or drag and leaves the caret where it is. A right-click on the bar does not open a menu. Picker dialogs move their selection by one, and a click outside the box closes them. Several reports that arrive together each apply, then the screen paints once. Scroll never steals focus; clicks set it. Terminals without SGR mouse fall back to legacy X10 reports.
- Autocomplete: click an item to accept it. A click outside the list closes it and still lands on the editor or the sidebar.
- Right-click opens a context menu (`Up/Down` or `Ctrl+P`/`Ctrl+N`, `Enter` confirm, `Esc` or click-away cancels, `1-9` quick-pick):
  - Tab: `Close tab`, `Copy file path`.
  - Sidebar: `Open`, `Copy file path`, `Rename`, `Delete`, `Search here`, `Search & Replace here` (each opens its dialog; searches scope to that folder).
  - Editor: `Cut`, `Copy`, `Paste`, `Select All`, `Find in File`, `Replace in File`, `Find in Files`, `Replace in Files`, `Go to Line` (right-click keeps an active selection so Copy/Cut work on it).

## 6. Recovery & Sessions

State dir: `$XDG_STATE_HOME/az-rust` or `~/.local/state/az-rust`.

- `session-<hash>.txt`: per-project open tabs, cursor, `tab_index`, expanded dirs. Restored only when starting from empty Untitled (folder open).
- `recovery/<hash>.rec`: unsaved buffers, throttled to 1 write/250ms per tab. Deleted on save/close.
- On launch, `Recover unsaved changes for NAME? y/N` per file. `y` restores as modified tab, `N` deletes recovery.

Atomic saves: write temp `.NAME.aztmp.PID` then rename, preserving permissions. If that fails with permission denied, the status line blinks red and asks `Permission denied. Root password:` (characters show as `*`). `Enter` writes the buffer as root via `sudo -S` (the password is sent on stdin, never on the command line). `Esc` or an empty entry cancels. A wrong password blinks the sudo error in red. This does not run when `az` is already root.

## 7. Copy/Paste

- System copy cascade: `wl-copy` (Wayland) → `xclip`/`xsel` (X11) → `pbcopy` (macOS) → Wayland `ext-data-control` (no extra tool; a helper process holds the selection until the next copy), then best-effort OSC52 `\x1b]52;c;BASE64\a`. Over SSH, OSC52 goes first (only path to the local clipboard). Status shows `Copied` when a tool or the compositor confirmed, `Copied (OSC52)` when only the terminal sequence was sent.
- Copy or cut with no selection takes the current line, including its newline, except on the last line (there is no extra newline to take).
- `Ctrl+V` prefers the system clipboard after a copy that a tool or the compositor confirmed. After `Copied (OSC52)` it pastes the editor text, so an older system clipboard does not replace the copy you just made. If the editor clipboard is empty, it tries the system clipboard.
- Bracketed paste (`\x1b[200~ … \x1b[201~`) and a raw burst of text insert in one step (up to 5MB), so a long paste is one undo entry. A single typed Enter is still a newline with auto-indent.

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
| Wheel does nothing | Needs content to move (short files / few tree rows won't visibly scroll). The editor pans; the caret stays. Pickers scroll their selection. |

## 9. Known Limitations (by design)

- No word wrap. Long lines scroll horizontally. A scrollbar appears when a pane is wider or taller than the viewport. No split panes.
- No regex search, no multi-cursor.
- Mouse: click + drag-select + wheel supported in tree/editor; requires a terminal with SGR mouse reporting (`1000`/`1002`/`1006`).
- Undo `revision` stays monotonic so redo keeps working. `modified` clears when the buffer matches the last save, including an undo back to that text.
- Replace in the current buffer caps at 20k operations. Find in files shows 80 results. Quick open shows 14. Symbol and project scans stay inside those caps and advance a chunk at a time while the dialog is open.
- Binary files open via lossy UTF-8; saving rewrites as UTF-8.

## 10. Tips

- `az .` + `Ctrl+O` is fastest project navigation.
- `Ctrl+P` then `set js` forces JS highlighting in mixed files.
- `+`/`-` in tree to fit long filenames.
- `Tab` in PHP after `$` lists all `$vars` in file (up to 20k lines scanned).
- `Ctrl+L` repeatedly to hop through matches; watch `(wrapped)`.
