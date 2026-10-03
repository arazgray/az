# AGENTS.md — AI Agent Guide for `az` 2.6

> Read this before editing. `az` is a single-binary Rust TUI editor (~4900 lines, zero crates). Keep changes small, test with `cargo test`, never break raw-mode cleanup.

## 1. Quick Facts

- Lang: Rust 2021, no dependencies (`Cargo.toml` only package + release profile).
- Entry: `src/main.rs` (~4900 lines) + `src/plugins/*.rs` (42 files: 41 languages + `example.rs` skeleton).
- Build: `cargo check` (fast), `cargo test` (45 unit tests), `cargo build` / `cargo build --release`, `./build.sh` (installs `~/.local/bin/az` and `/usr/local/bin/az`).
- Run: `./target/debug/az --help`, `./target/debug/az file:line`.
- License: WTFPL (matches README; `Cargo.toml` fixed from MIT).
- State: `$XDG_STATE_HOME/az-rust` or `~/.local/state/az-rust` (`session-*.txt`, `recovery/*.rec`).

## 2. Architecture Map

```
src/main.rs
  consts BG/FG/... (Tokyo Night), HISTORY_LIMIT=400, QUICK_OPEN_LIMIT=2500, ...
  enum SyntaxMode { Php, Blade, Html, Css, JavaScript, Plain }
  struct Pos { line, col }          // col = BYTE index, always char-boundary
  enum TextOp { Insert, Delete }
  struct HistoryEntry { ops, before, after }
  struct Tab { path, name, lines: Vec<String>, cursor, row_offset, col_offset,
               modified, revision, saved_revision, syntax_mode, undo/redo, large_file }
  struct TreeRow, PickerItem, CompletionItem
  enum Focus { Editor, Tree }
  struct Editor { root, tabs, tab_index, ... cached_clock_* , last_recovery_write,
                last_tree_click_time/path, pending_input, pending_update }
  impl Editor {
    new(args) / run() / enable_raw_mode() / cleanup()
    read_key(), read_escape(), handle_key(), handle_global_shortcut(),
    handle_tree_key(), handle_editor_key(), handle_mouse*()
    render(), render_titlebar(), render_topbar_separator(), render_tabbar(), titlebar_button_regions(), render_content(), render_status_separator(), render_status_line(),
    render_popup_box(), render_simple_picker(), render_autocomplete_dropdown()
    open_file(), new_tab(), close_current_tab(), save_current_tab/_as()
    insert_text(), apply_insert_at(), apply_delete_range(), backspace(), delete_forward()
    undo(), redo(), copy/cut/paste, move_*, go_to_line()
    find_next(), replace_one/all(), prompt(), quick_open(), command_palette(), shortcuts_dialog(),
    project_search_*, replace_in_files_*, replace_in_line(), count_matches_in_line(),
    context_menu(), render_context_menu(), tree ops, reveal_path_in_tree(), toggle_*()
    autocomplete_*, state_dir/session/recovery, try_restore_session/save_session
  }
  helpers: absolute_path(), parse_cli_path(), relative_path(), atomic_write_file(),
           ansi_*, visual_width(), fit_plain/fit_ansi(), clamp_char_boundary(),
           next/prev_char_boundary(), find_in_line(), parse_quick_open_query(),
           quick_score(), base64_encode(), escape/unescape_state(), ...
           MouseEvent, parse_sgr_mouse() // SGR `ESC[<Cb;Cx;CyM/m`, 1-indexed
           editor_click_col(), tab_hit_index(), context_menu_geometry()
           REPLACE_MATCH_LIMIT=10_000, BG_TAB (#3a405c, inactive tabs)

src/plugins/mod.rs   // facade: from_word/from_path, tree_color, highlight_segments,
                     // completion_context/items, extract_symbols, string utils
  php.rs | html.rs | css.rs | javascript.rs | blade.rs | example.rs
src/wayland_clip.rs  // ext-data-control clipboard; `az --clipboard-hold` serves a copy
```

Rendering: immediate-mode ANSI, `render()` each keystroke + each minute (clock). `read_terminal_size()` via ioctl then `stty size` then env.

Input: raw mode via `stty -echo -icanon -isig -ixon ... min 0 time 1`. `read_key()` returns `String` (escape seqs as text, paste as `\0AZPASTE:…`). `is_printable()` filters. Mouse: SGR `1000`+`1002`+`1006` enabled in `enable_raw_mode()`, disabled in `cleanup()`; `read_key()` breaks on `M/m` for `ESC[<…` (or 6-byte `ESC[M` legacy); `handle_key()` routes both via `parse_sgr_mouse()` / `parse_legacy_mouse()` → `handle_mouse()`. Picker loops (quick open, palette, find-in-files, shortcuts) and `context_menu()` scroll selection on wheel.

Mouse: left-click sidebar (`x <= tree_width`) toggles dir / opens file; double-click same path <500ms calls `rename_tree_path_prompt(false)`. Title row buttons hit-tested via `titlebar_button_regions()`. Left-click editor maps `(x,y)` via `editor_start_col()+gutter` + `editor_click_col()` (visual→byte, tab=4/wide=2 aware) and moves cursor. Wheel (`Cb&64`, up=`Cb&1==0`) scrolls tree by moving `tree_index ±1` or editor by moving cursor `±1` (keeps `ensure_*_visible()` invariants; scroll never changes focus, click sets it). Right-click (`Cb&3==2`) opens `context_menu()` (tab/sidebar/editor items, `context_menu_geometry()` clamps to screen minus status line); middle-click (`==1`) on tab bar closes via `close_tab_at()`. Left-drag motion (`Cb&32`, button 0) extends selection from `mouse_drag_start` (tab-index guarded); double-click selects `word_range_at()`, triple-click the line. Layout rows: 1 titlebar, 2 separator, 3 tab bar, 4.. content (`content_height = rows-5`), separator, `rows` status.

## 3. Critical Invariants (do not break)

1. **Cols are byte indices.** Always `clamp_char_boundary()` after arithmetic. Use `prev/next_char_boundary()`, never `col±1` on UTF-8. Tests cover this indirectly.
2. **Raw mode must restore.** `enable_raw_mode()` saves `stty -g`, `cleanup()` restores or `stty sane`, plus `\x1b[?1002l \x1b[?1006l \x1b[?1000l \x1b[?2004l \x1b[0m \x1b[?25h \x1b[?1049l`. Every early return in `run()` must call `cleanup()`.
3. **`0x08` duality.** `Ctrl+H` == Backspace on some terms. Rule: in `Focus::Editor`, `0x08` falls through to editor backspace; in `Tree`, toggles sidebar. Don't re-add global `0x08` → toggle unconditionally.
4. **`absolute_path()` must stay absolute.** Resolves against CWD + lexical `..` normalisation without FS access (supports new files). Session/recovery hashes depend on it.
5. **History byte positions.** `apply_insert/delete` mutate `lines` + set `cursor=start`. `undo()` iterates `ops.rev()`, `redo()` forward. `mark_edited()` bumps `revision` (monotonic — undo stays dirty by design).
6. **No crates.** Std only (`fs`, `io`, `env`, `process::Command` for `date`, raw `ioctl` FFI). Don't add deps without discussion.
7. **Picker widths.** Clamp to `cols-2`. Narrow terminals (30 cols) must not overflow. See `render_simple_picker()`.
8. **Skip lists must stay in sync.** `add_tree_rows()` + `collect_quick_open_files()` both skip `.git node_modules vendor .idea .vscode target dist build __pycache__ .next .nuxt`. Update both together.

## 4. Plugin API (add language in 15 min)

1. Copy `src/plugins/example.rs` → `src/plugins/mylang.rs`. Implement any of:
   ```rust
   pub(crate) fn segments(line: &str) -> Vec<Segment>
   pub(crate) fn completion_context(before: &str, explicit: bool) -> Option<(String,String,usize)>
   pub(crate) fn completion_items(kind: &str, ctx: CompletionContext) -> Vec<CompletionItem>
   pub(crate) fn symbols(line: &str) -> Vec<String>
   // optional helpers: looks_like_line(), looks_like_context()
   ```
   `Segment { start, end, color }` byte ranges, colors from `crate::{BLUE,…}`.
2. Register in `src/plugins/mod.rs`: `pub(crate) mod mylang;`, extend `from_word()`, `from_path()`, `highlight_segments()`, `completion_context/items()`, `extract_symbols()`, `tree_color()` if needed.
3. Add `SyntaxMode::MyLang` in `main.rs`, plus palette items in `command_items()` + `run_command()` (`set-syntax-mylang`).
4. Add test in `main.rs` `#[cfg(test)]` for `from_word/from_path` + `segments()` smoke.
5. Update `PLUGIN_GUIDE.md` + `README.md` language list.

See `PLUGIN_GUIDE.md` JavaScript wiring example. Keep highlighting line-local (no multi-line state) for perf.

## 5. Common Tasks — Where to Edit

| Task | Location |
|------|----------|
| New keybinding | `handle_global_shortcut()` (global) or `handle_editor_key()` / `handle_tree_key()`. Update `shortcut_help_lines()`, `print_help()`, `USER_MANUAL.md`. |
| New palette command | `command_items()` + `run_command()` + handler method. |
| Rendering glitch | `render_content()`, `render_editor_line()`, `cursor_screen_position()`, `ensure_editor_visible()`. Remember `visual_width()` for tabs/wide/control chars. |
| Find/replace | `find_next()` (wrap-aware, selection-aware), `replace_one/all()`, `parse_find_query()` (`%` prefix). |
| Tree / file ops | `add_tree_rows()`, `collect_quick_open_files()`, `reveal_path_in_tree()`, `create/rename/delete_*_prompt()`. |
| Autocomplete | `autocomplete_context()` → `plugins::completion_context()`, `refresh_autocomplete()`, per-plugin `completion_*`. |
| Session/recovery | `state_dir()`, `session_file()`, `try_restore_session()` (reads `tab_index`), `save_session()`, `write_recovery_for_current_tab()` (250ms throttle), `offer_recovery()`. |
| Perf | `clock_text()` caches `date` subprocess per minute; symbol scan caps (600 files/1MB), search caps (3000 files/5MB). Don't remove caps. |

## 6. Testing

```sh
cargo check   # fast gate
cargo test    # 45 tests: cli_path, absolute, quick_open parse, html auto-close, find, escape, search %, navigation keys, plugins, mouse SGR + click-col, replace counting, menu geometry, Ctrl+Shift+H, word range, Ctrl+K + shortcuts, OSC52, legacy mouse, wheel ±1, paste, sudo message, wayland socketpair
cargo build   # debug binary ./target/debug/az
```

Add tests for pure fns (`parse_*`, `find_in_line`, `quick_score`, `escape_state`, plugin `segments/symbols`, `last_unclosed_tag`). TUI `Editor` methods need terminal — test logic via extracted pure helpers, not `Editor::new()` with FS side-effects.

Manual smoke (no PTY in CI):
```sh
./target/debug/az --help
./target/debug/az --version
./target/debug/az src/main.rs:20   # should report Opened main.rs:20 if run interactively
cargo build --release
```

## 7. Bugs Fixed (2.0.1 + 2.1 + 2.2 + 2.5 + 2.6) — Don't Regress

2.6 (languages + update check + input/clipboard fixes):
- 20 new plugins (41 total + Plain = 42 modes): python, java, csharp, cpp, c, go, kotlin, swift, ruby, dart, scala, r, lua, perl, haskell, elixir, clojure, zig, julia, objc. Each wires `mod X;` + `mode_label` + `from_word` + `from_path` + `tree_color` + `highlight_segments` + word completion/symbols, plus `SyntaxMode::X` + palette `set-syntax-x` + `run_command` in `main.rs`. `.h` → C (C++ headers highlight as C — accepted, documented). `Gemfile`/`Rakefile` → Ruby.
- Startup update check: `check_for_updates()` (curl `--max-time 5` of raw GitHub `Cargo.toml`, silent on failure, `AZ_NO_UPDATE_CHECK=1` opt-out) runs in `main()` before raw mode; newer version shown via `render_update_notice()` modal in `run()`. `--version`/`--help` use `env!("CARGO_PKG_VERSION")`, never a hardcoded string.
- Wheel bursts: `read_key()` glues multi-report stdin reads; `handle_key()` now dispatches `parse_mouse_events()` (splits concatenated SGR/legacy reports, ignores trailing partials) instead of parsing the chunk as one event. Never route menu keys through global shortcuts (unchanged).
- Clipboard deadlock: `pipe_to_clipboard_tool()` drops stdin before `wait()` — tools read stdin to EOF, so waiting first hung the editor. Tests: `cat`-based EOF test (hangs pre-fix, passes post-fix).
- Tests: 38 total (new: mouse-burst splitting, end-to-end wheel scrolling, clipboard-pipe EOF, remote-version parsing + comparison).

Unreleased (wheel step, OS clipboard, sudo save, fast paste):
- Wheel is one row per report (`handle_mouse_wheel` ±1). Do not put the ±3 step back.
- Clipboard write order: SSH prints OSC52 first; then `try_clipboard_tool()`, then `wayland_clip::copy()`. Local tries tools, then Wayland, then OSC52. True means a tool or Wayland confirmed. `wayland_clip` speaks `ext-data-control-v1` with no crates. `copy` re-execs `az --clipboard-hold` (handled in `main` before the editor) so the selection stays alive. `Ctrl+V` uses `read_system_clipboard()` first.
- `./build.sh` `install_system_wide` copies `/usr/local/bin/az` (directly when root or the dir is writable, otherwise `sudo cp`). Failure must not fail the `~/.local/bin` install. `sudo` `secure_path` does not include `~/.local/bin`.
- Permission-denied save (`ErrorKind::PermissionDenied`, and not already root): status blinks red (`message_is_error`), `prompt_secret` (RED, `*` masking), then `write_file_with_sudo` (`sudo -k -S -p ''`, password + newline on stdin only, payload staged in a temp file). Other status flashes stay cyan.
- Paste: `read_escape` treats `\x1b[200~` as a paste prefix and reads through `\x1b[201~` (5MB cap, stall gives up after a few quiet reads). `coalesce_burst` turns an already-buffered run (newline/CR, or 16+ bytes) into one `\0AZPASTE:` insert. `apply_insert_at` rebuilds the line vec once for multi-line text.
- Tests: 45, plus an ignored Wayland roundtrip (`wayland_clipboard_roundtrip`) that restores the previous clipboard.

2.5 (replace-all + context menus + chrome):
- Replace in Files is `Ctrl+Shift+H` only (`is_ctrl_shift_h`); caps: 3000 files, 5MB, 10k matches. Open modified tabs are skipped (never clobber unsaved buffers); reloaded tabs get `undo/redo` cleared (positions refer to old content).
- Right-click (`button&3==2`) opens `context_menu()`; middle-click (`==1`) on topbar closes via `close_tab_at()`. Menu loop swallows its own mouse (click-away/Esc cancel, `1-9` pick). Never route menu keys through global shortcuts.
- Inactive tabs use `BG_TAB` (lighter than bar); open file row uses `BG_HIGHLIGHT` in `render_tree_line()`; status chips in `render_status_line()` (`modified`-only orange, stats yellow) — no `saved` chip, it was display-only noise. Mode chip lives in the titlebar via `focus_label()`.
- `Ctrl+K` opens `shortcuts_dialog()` (`shortcut_defs()` + shared `filter_command_items()`); `Ctrl+/` (`is_ctrl_slash`) is gone. Titlebar buttons (plain ACCENT text, no bg chip) hit-tested via `titlebar_button_regions()` → quick-open/palette/shortcuts.
- Clipboard: `copy_to_system_clipboard()` returns verified bool; cascade `try_clipboard_tool()` (`wl-copy`/`xclip`/`xsel`/`pbcopy` via `pipe_to_clipboard_tool()`) then `osc52_sequence()`; OSC52-first over SSH. Messages `Copied` vs `Copied (OSC52)`.
- Status flash: `render()` arms `status_flash_until` on message change (750ms, 250ms phases, CYAN); `prompt()` blinks every prompt light blue (350ms) using non-blocking `read_key()` ticks + `redraw` flag.
- Welcome logo is generated bytes from ttfx `highlight` (`ttfx_logo()` + `TTFX_LOGO_WIDTH`, per-row widths — rows are ragged, never pad to a single width).

2.2 (search + navigation):
- Find in Files is `Ctrl+Shift+O` only (`is_ctrl_shift_o`); never re-add `Ctrl+Shift+F` — terminals reserve it for their own search bar.
- File top/bottom secondary is `Alt+Up/Down` (`is_alt_up/down`, CSI `1;3` + legacy `Esc+Arrow`); never re-add `Ctrl+Up/Down` — Guake uses it for height.
- `%` case-sensitivity shared via `parse_search_query()` between in-file find and project search. Tests: `search_query_percent_is_case_sensitive`, `project_line_matches_respects_case`, `ctrl_navigation_bindings`, `ctrl_shift_find_bindings`.

2.0.1:
- `last_unclosed_tag()` stripped just-typed `>`; before fix HTML auto-close never fired. Test `html_auto_close_recovers_tag_after_gt`.
- CLI `file:line`, `:line`, `newfile` (non-existent) + `--help/--version`; `absolute_path()` now absolute + `..` normalised. Tests `cli_path_*`, `absolute_*`.
- `save_as` writes before switching `tab.path` (no orphan on failure).
- `find_next()` starts from selection end, wraps with `(wrapped)` notice, avoids `+1` overlap bug.
- `tab_index` persisted/restored (was written but ignored).
- Picker width clamped for narrow terms; tree/quick-open skip `target/dist/build/__pycache__/.next/.nuxt`.
- `date` subprocess cached per minute (`cached_clock_*`); removed dead `comp()`; license MIT→WTFPL; help lists `Ctrl+N`, `Ctrl+L`, `+/-`.
- New: `Ctrl+L` find-next, tree `+/-` resize (18..44).

2.1 (Blade/CSS `#` fix):
- PHP `#` no longer marks CSS `#header`, `#fff`, `href="#section"` as COMMENT. New `hash_comment_start()` skips strings + `is_css_hash()` (ID/hex + `{`/`}` heuristic). CSS adds `id_selector_ranges()` (YELLOW).
- `//` no longer marks `https://…` inside strings/templates. New `line_comment_start()` in `php.rs`/`javascript.rs` skips strings + `://`.
- Blade `@` no longer marks `user@example.com`. New `directive_ranges()` requires boundary + outside strings, skips `@@`.
- Tests: `blade_css_id_is_not_php_comment`, `blade_hex_color_is_not_comment`, `blade_href_hash_is_not_comment`, `blade_https_is_not_comment`, `php_real_comments_still_work`, `blade_directive_vs_email`.

## 8. Known Issues / TODO for Agents

- `revision` monotonic → undo-after-save stays `modified`. Fix requires content-hash or saved-text compare; currently by design — document, don't “fix” by decrementing revision (breaks redo).
- `word wrap` in README was misleading — now documented as horizontal scroll. Real wrap needs `render_editor_line()` + `cursor_screen_position()` + `ensure_editor_visible()` rework (screen-row vs file-line mapping).
- `collect_quick_open_symbols()` (600 files) + `collect_project_search_results()` (3000 files) block UI. Future: cache index or background thread with `mpsc`.
- Recovery separator `---TEXT---\n` collides if filename ends with that string (filenames can't contain `\n`, so risk tiny). Proper fix: length-prefixed body or NUL separator with migration.
- Prompt line editing: no Left/Right, no history. Add `prompt_history` if needed.
- No drag-select, no splits, no regex — out of scope unless requested.

## 9. Style & PR Rules

- `cargo check` + `cargo test` clean, no warnings (`unused_assignments`, `dead_code` fail review).
- Keep diffs minimal; single concern per change; update `USER_MANUAL.md` + help text + `--help` together when adding keys.
- Byte-safe Rust: no `s[col..]` without clamp; prefer `clamp_char_boundary()`.
- ANSI: always reset `\x1b[0m`, pad with `fit_plain/fit_ansi()` to avoid ghost text.
- Don't spawn subprocesses per keystroke (see clock cache pattern).
- Don't add crates; don't use `unsafe` except existing `ioctl`.
- Update this file + `USER_MANUAL.md` when changing architecture or shortcuts.
