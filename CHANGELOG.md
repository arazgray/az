# az — Changelog

## 2.5

- **Replace in Files dialog** (`Ctrl+Shift+H`, or `Ctrl+P` → `Replace in files`): prompts search → replacement → confirm with real counts, then rewrites files across the project (≤3000 files, skips >5MB/binaries, 10k-match cap, `%Foo` = case-sensitive). Open unmodified tabs reload (undo cleared); files with unsaved buffers are skipped and reported. Also scoped via sidebar right-click → `Search & Replace here`.
- **Right-click context menus** (SGR mouse): tab (`Close tab`, `Copy file path`), sidebar (`Open`, `Copy file path`, `Rename`, `Delete`, `Search here`, `Search & Replace here`), editor (`Cut/Copy/Paste`, `Select All`, `Find/Replace in File`, `Find/Replace in Files`, `Go to Line`). Keyboard navigable (`Up/Down`, `Enter`, `Esc`, `1-9`).
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

## 2.2

- **Find in Files modal** (`Ctrl+Shift+O`, or `Ctrl+P` → `Find in files`): separate live-search popup across the opened folder (≤3000 files, 80 results, `file:line + snippet`, `Enter` jumps to the match). Same `%Foo` rule as Find: case-insensitive by default, `%` prefix = case-sensitive.
- **Go to Start/End**: `Home`/`End` for line ends, `Ctrl+E` for end of line, `Ctrl+Home/End` or `Alt+Up/Down` for file top/bottom (hold `Shift` to select) — all four also in the command palette.
- **Terminal-friendly shortcuts**: `Ctrl+Shift+O` (most terminals reserve `Ctrl+Shift+F` for their own search) and `Alt+Up/Down` (Guake uses `Ctrl+Up/Down` for height).
- **Mouse support** (SGR `1000`/`1006`): click tabs to switch (inactive tabs now render on a lighter chip), click sidebar folders to expand/collapse and files to open, double-click a file to rename, click in the editor to move the cursor, wheel scrolls sidebar and editor. No drag-select.
- **Tests**: 26 total (new: `%` query parsing, case-sensitive project matching, navigation key bindings, SGR mouse parsing, click visual-to-byte mapping).

## 2.1

- **16 new language plugins** (22 total, table below): Markdown, JSON, TOML, YAML, Bash, Dotenv, INI, Logs, Rust, Nginx, Apache, Dockerfile, systemd, SQL, TypeScript, XML — plus existing PHP/Blade/HTML/CSS/JS. Auto-detected by filename/extension, switchable via `Ctrl+P` → `set …`.
- **Blade + CSS fix**: `#header`, `#fff`, `href="#section"`, `https://…` no longer gray as PHP `#` / `//` comments. `#id` now yellow, hex orange, URLs stay green. `user@example.com` no longer purple as Blade directive. 8 new regression tests (17 total).
- **Smarter comments**: `//` / `#` ignored inside `"strings"`, `` `templates` ``, and `://` protocols.
- **`--help` / `--version`**, `file:line`, `:line`, `newfile` CLI handling; absolute-path session keys.
- **Faster topbar**: `date` cached per minute. Pickers safe on 30-col terminals. Tree/quick-open skip `target dist build __pycache__ .next .nuxt`.
- **New keys**: `Ctrl+L` find-next, `+/-` tree width, `Ctrl+N` new file everywhere in help.
- **Repo hygiene**: `.gitignore` for `target/`, `/az` binary, `*.tmp`, OS/IDE noise; binaries untracked.

## 2.0

- Completely rewritten in Rust
- Faster startup and rendering
- Better handling for huge files
- Modular language support through Rust plugins
- Mixed syntax highlighting for files that contain PHP, HTML, CSS, Blade, and JavaScript together
- File colors in the project tree based on extension
