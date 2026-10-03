# az — Changelog

## Unreleased

## 3.0

- **Wheel pans the editor.** Each report moves the viewport one row. The caret and the selection stay put. The sidebar selection still moves one row. The wheel on the tab bar cycles tabs. A burst of reports still applies every report before the next paint.
- **Tabs past the ninth stay reachable.** At most nine tabs are drawn, windowed around the current one. `Alt+1-9` hits that window. `Ctrl+Tab` / `Ctrl+Shift+Tab` cycle every open tab. `+` on the tab bar and `Ctrl+N` open an empty unsaved tab. Saving it asks for a filename.
- **Search and replace is a dialog.** `Ctrl+F`, `Ctrl+R`, and `Ctrl+Shift+H` open one box: path, find, replace, a live count, Replace, and Cancel. `Tab` moves between the fields. `Enter` replaces. A click outside closes it. The same click-outside rule covers quick open, the command palette, find in files, and shortcuts.
- **Horizontal scroll bars.** When a line or a sidebar name is wider than its pane, the last content row is a scrollbar. The editor offset is a visual column, so a tab no longer leaves the caret off the right edge.
- **Shell rc files highlight as Bash.** `*.sh`, `*.bash`, and `*.zsh` already did. `~/.bashrc` has no extension, so it was plain. Basenames (`.bashrc`, `.bash_profile`, `.zshrc`, `.profile`, and the other shell startup names) and a `#!` line for sh/bash/zsh/ksh/dash/ash/mksh now select Bash when the name would otherwise be plain. A syntax mode you set by hand still wins.
- **The dirty flag follows the text.** Undo back to the saved buffer clears `modified`. `revision` stays monotonic.
- **CRLF files stay CRLF.** The buffer is LF. Save and replace-in-files write `\r\n` back when the file had it.
- **Copy and cut of a whole line match,** including the newline, except on the last line. `(` `{` `[` `"` `'` wrap a selection. `(` `{` `[` still insert an empty pair when nothing is selected.
- **Paste keeps the copy you just made.** A confirmed OS copy reads the system clipboard. An OSC52-only copy pastes the editor text, so an older system clipboard does not replace it. A typed character plus Enter is a newline. A real paste (16 bytes, or a newline inside 8 or more bytes) is still one insert.
- **Prompts move the cursor.** Left/Right/Home/End, and Up/Down through recent answers. The root-password prompt has no history.
- **The screen follows the terminal.** A size change redraws on the next input wake. A status flash paints one clear frame when the timer ends. The key that dismisses welcome or the update notice is handled. Status `Col` is the screen column. On a narrow terminal the message stays by dropping the tree and syntax chips first.
- **System clipboard both ways.** `Ctrl+C` / `Ctrl+X` copy to the OS clipboard (`wl-copy`, `xclip`/`xsel`, `pbcopy`, or Wayland `ext-data-control` when those tools are missing). Status shows `Copied` when a tool or the compositor confirmed, and `Copied (OSC52)` when only the terminal sequence was sent.
- **`sudo az`.** The installer also copies the binary to `/usr/local/bin/az`. `sudo`'s `secure_path` includes that directory and does not include `~/.local/bin`, which is why `sudo az` was "command not found".
- **Save when the file is not writable.** A permission error blinks the status message red, asks for the root password (shown as `*`), and writes the buffer with `sudo -S`. The password is sent on stdin and is not put on the command line. Cancel with `Esc`.
- **Long pastes insert in one step.** Bracketed paste and a raw burst of text are one insert and one undo step, instead of a character-by-character redraw.

## 2.6

- **20 new language plugins** (42 built in): Python, Java, C#, C++, C, Go, Kotlin, Swift, Ruby, Dart, Scala, R, Lua, Perl, Haskell, Elixir, Clojure, Zig, Julia, Objective-C. Auto-detected by extension/filename (`Gemfile` → Ruby, `.h` → C), switchable via `Ctrl+P` → `set …`. Each brings keywords/types/comments/numbers, call highlighting, symbol extraction + word completion.
- **Startup update check**: every launch compares against `Cargo.toml` on GitHub main (short `curl` timeout, silent when offline; `AZ_NO_UPDATE_CHECK=1` opts out). When a newer release exists, a modal shows the version and the one-line upgrade command (the installer script installs fresh and upgrades in place).
- **Wheel burst fix**: fast scrolling/touchpads flush several mouse reports per stdin read; the whole burst used to be silently dropped (`parse_sgr_mouse` failed on the glued chunk). Each report is now dispatched via `parse_mouse_events()`, so kinetic scrolling works in the editor, sidebar, and pickers.
- **Clipboard deadlock fix**: `pipe_to_clipboard_tool()` now closes stdin before `wait()` — `wl-copy`/`xclip`/`xsel`/`pbcopy` all read stdin to EOF, so copy could previously hang the editor whenever a tool was installed.
- **`--version`/`--help` read `CARGO_PKG_VERSION`** instead of a hardcoded string, so the reported version can never drift from `Cargo.toml`.
- **Tests**: 38 total (new: mouse-burst splitting, end-to-end wheel scrolling, clipboard-pipe EOF, remote-version parsing + comparison).

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
