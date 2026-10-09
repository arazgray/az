# az — Changelog

## 4.8.0

- **Konsole context-menu title fix.** Context menu rows use first-strong bidi isolates so Konsole does not show a stray leading mark.
- **Clickable status controls.** Click `autosave:on/off` to toggle autosave, or `tabs` / `spaces:4` to change indentation.
- **Tree aligns with tabs.** The project root shares the tab-bar row; tree drawing, clicking, scrolling, and its vertical scrollbar account for the additional visible row.
- **Author/contact dialog.** The command palette's `Author`, `Contact`, and `Support` entries open the same dialog with the az author and contact details.
- **Repeatable screenshots.** Added `scripts/capture_screenshot.py`; the README screenshot is regenerated for this release and the pre-tag step is documented.

## 4.7.0

- **Narrow-terminal titlebar layout.** The titlebar shares one width calculation for rendering and mouse hitboxes, switches to a compact clock, and only shows actions that fit. Quit is always aligned with its click target.
- **Compact welcome dialog.** Small terminals show a compact, readable welcome message with the version, shortcuts hint, and continue instruction instead of letting the logo crowd them out.
- **Status bar handles long paths.** Paths are safely escaped and middle-ellipsized so cursor position and status messages remain visible.
- **Safer tree double-click.** A second click on the same row is deduplicated; it no longer unexpectedly opens Rename. Use `F2` or the context menu.
- **Clearer bulk replacement.** The destructive action is labeled `Replace all` in the dialog.
- **Menus follow terminal resizes.** Context menus recompute their position and size while open.
- **Updated README screenshot** to show the 4.7.0 UI.

## 4.6.1

- **Restart after update works when the binary was replaced.** `Restart now` used `current_exe()` (`/proc/self/exe`), which reads as `/path/az (deleted)` once dpkg/`cp` swaps the file — spawning it failed with `No such file or directory (os error 2)`. Restart now prefers the live binary, falls back to the reinstalled path, then to `az` on PATH.

- **Shortcuts for every palette command.** Every row shows its shortcut in parantes (`Save (Ctrl+S)`); the 150+ language rows show their `set <name>` filter. 12 new bindings: `Alt+S` save as, `Alt+N`/`Alt+M` new file/folder, `F2` rename, `Alt+D` delete, `Alt+U` check for update, `Alt+A` autosave, `Alt+T`/`Alt+I` indent tabs/spaces, `Alt+F` format, `Alt+W` welcome, `Alt+L` syntax menu (palette prefilled with `set `). All listed in `Ctrl+K` and `--help`.
- **Launch flags.** `az --autosave=1 --indent=spaces` (aliases `--as=1`, `--tabs`/`--spaces`, plus `--wrap`/`--no-wrap`, `--rtl`/`--no-rtl`). CLI beats `settings.txt` and is persisted to it. Unknown `--flags` ignored; `--` ends flag parsing.
- **Installer verifies `az` runs.** `install.sh` smoke-tests the binary, diagnoses glibc mismatches with the source-build fix, and prints/persists the PATH export when the binary lands outside PATH. Linux release builds moved to Ubuntu 22.04 (glibc 2.35 baseline) so the prebuilt runs on 22.04+.

- **Installer downloads from the GitHub release.** `install.sh` fetches versioned packages from the release assets first (`AZ_RELEASE_TAG` defaults to the short tag, e.g. `4.5.0` → `4.5`) and only falls back to raw `dist/` on `main`.
- **Autosave toggle.** `Ctrl+P` → `Enable autosave` / `Disable autosave` saves open files automatically after edits (throttled to one write/sec per file, silent, never prompts for a password). Toggling on flushes all dirty tabs; exiting with autosave on saves them too. Persisted in `settings.txt` in the state dir.
- **Indent style: Tabs or 4 spaces.** `Ctrl+P` → `Indent with Tabs` / `Indent with Spaces (4)` switches the `Tab` key and auto-indent unit for all languages. Persisted in `settings.txt`. Status bar shows `tabs` / `spaces:4` plus an `autosave` chip when on.
- **Format selection or file.** `Ctrl+P` → `Format selection or file` re-indents with the configured Tabs/Spaces unit (selection, else whole file). One undo entry. Heuristic: strings and `//` comments skipped; continuation lines and switch `case:` may need a nudge.

## 4.4

- **Check for update command.** `Ctrl+P` → `Check for update` runs the startup update check on demand (an explicit request beats the `AZ_NO_UPDATE_CHECK` opt-out). Shows the update dialog when a newer release exists, otherwise a status note.

## 4.3

- **Welcome dialog refresh.** Logo is followed by the tagline, a short description, and the current version number (pulled from `Cargo.toml` so it never goes stale).
- **Welcome Enter no longer toggles the tree.** Dismissing the welcome screen with Enter/Return used to fire the keypress into the editor as well, collapsing the folder under the tree cursor (or inserting a blank line). The dismissing Enter is now consumed by the dialog.
- **Installer banners.** `install.sh` and `compile-and-install.sh` show the same tagline, description, and version; desktop entries use the tagline.

## 4.2.1

- **Release pipeline fixes (no editor changes).** Windows binaries are now named per architecture before upload (previously both jobs overwrote a single `az.exe` and one arch was lost), artifact uploads and the publish filter use version-precise globs so stale packages can never leak into a release, and Linux jobs install `rpmbuild` so `.rpm`s are actually built.

## 4.2

- **One-click update and restart.** The startup update notice is now a real choice (`Update and restart` / `Later`) instead of a command to retype. Updating runs the installer with the terminal restored, then shows `Updated <old> → <new>` with `Restart now` / `Stay in editor`. Restart replaces the process in place and the saved session restores tabs.
- **Richer welcome dialog.** Logo is followed by the tagline, a short description, and the current version number.

## 4.1

- **Full micro syntax parity (152 modes).** 98 new one-file-per-language plugins (`ada.rs` … `zscript.rs`, same shape as the hand-written ones: highlighting, word completion, `def`/`class`-style symbols, tree colors, `set-syntax-*` palette items). Keyword data extracted from micro's own `runtime/syntax/*.yaml`; detection mirrors micro's `filename:` patterns. Aliases folded into existing modes: `python2/3` → Python, `html4/5` → HTML, `csx` → C#, `Pkgfile` → Bash.
- **12 new language modes (54 total, micro parity batch 1).** New built-in plugins: Makefile (`Makefile`, `*.mk`), CMake (`CMakeLists.txt`, `*.cmake`), PowerShell (`*.ps1/psm1/psd1`), Proto (`*.proto`), GraphQL (`*.graphql/gql`), Terraform (`*.tf/hcl`), Vue (`*.vue`), Svelte (`*.svelte`), Asm (`*.[sS]`, `*.asm`), TeX (`*.tex/bib/cls/sty`), Erlang (`*.erl/hrl`), Solidity (`*.sol`). Each wires `from_word`/`from_path`, highlighting, word completion, symbols, tree colors, and a `set-syntax-*` palette item. `Makefile` no longer falls back to Plain.
- **Multi-OS releases.** New CI matrix (`.github/workflows/release.yml`) builds Linux (x86_64 + ARM64), macOS (Intel + Apple Silicon), and Windows (x86_64 + ARM64) on every tag. New artifacts: `az-<ver>-linux-<arch>.tar.gz` (any distro: Arch, Alpine, NixOS...), `az_<ver>_<arch>.deb` (amd64 + arm64), `az-<ver>-1.<arch>.rpm` (Fedora/RHEL/openSUSE), `az-<ver>-macos-<arch>.tar.gz`, plus a Homebrew formula at `dist/homebrew/az.rb`. `install.sh` now detects OS + CPU and picks `.deb` → `.rpm` → tarball on Linux, tarball on macOS, and the arch-matched `.exe` on Windows. `dist/package.sh` reproduces the Linux packages locally (`--arch`, `--all`, SHA256SUMS).
- **macOS terminal fix.** `TIOCGWINSZ` is now `0x40087468` on macOS (was the Linux-only `0x5413`), so window-size detection works on Mac terminals; other Unix keeps the Linux value with `stty size` as fallback.
- **Homebrew tap.** New [`arazgray/homebrew-tap`](https://github.com/arazgray/homebrew-tap) repo: `brew install arazgray/tap/az` builds from source (zero deps, sha256-pinned tag archive, `livecheck` included). Mirror copy at `dist/homebrew/az.rb`; tap CI recipe staged at `dist/homebrew/test-workflow.yml` (needs one web-UI commit — see the tap README).
- **AUR.** `dist/aur/az-bin/` holds a ready-to-publish `PKGBUILD` + `.SRCINFO` for `az-bin` (name verified free on the AUR; sums verified against the 4.0 release assets), with an `aarch64` stanza to uncomment after the first multi-arch release. Publish steps in `dist/aur/README.md`.

## 4.0

- **Word wrap (`Alt+Z`).** Soft-folds long lines onto the next screen row (palette `Toggle word wrap`, editor right-click, status `wrap` chip). While on, the horizontal scrollbar hides and `col_offset` stays at 0. Continuation rows show a blank gutter. Click, caret, vertical scrollbar, and viewport follow the folds; `Up/Down` still move by file line.
- **RTL mode switch (`Alt+R`).** Palette `Enable/Disable RTL Mode`, editor right-click, status `rtl` chip. RTL right-aligns editor text inside an RTL isolate (`RLI…PDI`) and mirrors the caret/click mapping; the context menu is right-aligned too. Tree, gutter, dialogs, and row framing stay LTR. All dialog/menu rows are `LRI`-isolated so bidi terminals cannot merge them with RTL text.
- **Prebuilt packages.** `install.sh` now installs the ready package for your OS (`.deb` on Debian/Ubuntu, `.exe` on Windows) with `logo.png` icons; the old build-from-source flow moved to `compile-and-install.sh`. Release tarballs live under `dist/`.
- **Tests**: 70 total (66 run, 1 ignored Wayland roundtrip + new wrap/RTL/binding tests).

## 3.2

General stability and performance improvements and bug fixes.

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
