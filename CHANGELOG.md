# az — Changelog

## 2.2

- **Find in Files modal** (`Ctrl+Shift+O`, or `Ctrl+P` → `Find in files`): separate live-search popup across the opened folder (≤3000 files, 80 results, `file:line + snippet`, `Enter` jumps to the match). Same `%Foo` rule as Find: case-insensitive by default, `%` prefix = case-sensitive.
- **Go to Start/End**: `Home`/`End` for line ends, `Ctrl+E` for end of line, `Ctrl+Home/End` or `Alt+Up/Down` for file top/bottom (hold `Shift` to select) — all four also in the command palette.
- **Terminal-friendly shortcuts**: `Ctrl+Shift+O` (most terminals reserve `Ctrl+Shift+F` for their own search) and `Alt+Up/Down` (Guake uses `Ctrl+Up/Down` for height).
- **Tests**: 24 total (new: `%` query parsing, case-sensitive project matching, navigation key bindings).

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
