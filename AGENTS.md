# AGENTS.md — AI Agent Guide for `az` 3.1

> Read this before editing. `az` is a single-binary Rust TUI editor (zero crates). Keep changes small, test with `cargo test`, never break raw-mode cleanup.

## 1. Quick Facts

- Lang: Rust 2021, no dependencies (`Cargo.toml` only package + release profile).
- Entry: `src/main.rs` (~4900 lines) + `src/plugins/*.rs` (42 files: 41 languages + `example.rs` skeleton).
- Build: `cargo check` (fast), `cargo test` (61 tests, 1 ignored Wayland roundtrip), `cargo build` / `cargo build --release`, `./build.sh` (installs `~/.local/bin/az` and `/usr/local/bin/az`).
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
  struct Tab { path, name, lines: Vec<String>, cursor, row_offset,
               col_offset,  // VISUAL column, not a byte index
               modified, revision, saved_revision, saved_hash, syntax_mode,
               undo/redo, large_file, crlf }
  struct TreeRow, PickerItem, CompletionItem
  enum Focus { Editor, Tree }
  struct Editor { root, tabs, tab_index, ... cached_clock_* , last_recovery_write,
                last_tree_click_time/path, pending_input, pending_update,
                clipboard_verified, follow_cursor, follow_tree, tree_h_offset, show_hscroll,
                show_editor_vscroll, show_tree_vscroll, prompt_history, hscroll_drag, vscroll_drag }
  impl Editor {
    new(args) / run() / enable_raw_mode() / cleanup()
    read_key(), read_escape(), handle_key(), handle_global_shortcut(),
    handle_tree_key(), handle_editor_key(), handle_mouse*()
    render(), render_titlebar(), render_topbar_separator(), render_tabbar(), titlebar_button_regions(), render_content(), render_status_separator(), render_status_line(),
    popup_box(), simple_picker_string(), render_autocomplete_dropdown(), present_overlay()
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

Mouse: left-click sidebar (`x <= tree_width`) toggles dir / opens file; double-click same path <500ms calls `rename_tree_path_prompt(false)`. Title row buttons hit-tested via `titlebar_button_regions()` (plain prefix is ` az   {mode} `, 7 + mode length; the hit starts on the label; red ` Quit ` sits left of the clock and calls `confirm_quit()`). Left-click editor maps `(x,y)` via `editor_start_col()+gutter` + `editor_click_col()` (visual→byte, tab=4/wide=2 aware) and moves cursor. Wheel (`Cb&64`, up=`Cb&1==0`) is ±1 per report: tab bar (`y==3`) cycles tabs, sidebar moves `tree_index` and sets `follow_tree = true`, editor sets `follow_cursor = false` and pans `row_offset` (caret and selection stay). Do not put the ±3 step back, and do not move the caret from the wheel. Right-click (`Cb&3==2`) opens `context_menu()`; middle-click (`==1`) on a tab closes via `close_tab_at()` (the `+` button is not a tab). A click on a vertical bar pans that pane and does not open a menu or move the caret. Left-drag motion (`Cb&32`, button 0) extends selection from `mouse_drag_start`, unless `vscroll_drag` or `hscroll_drag` is set. Picker dialogs and the search/replace dialog close on a click outside `picker_frame`. Autocomplete: click inside accepts, click outside closes and the click still lands. Layout rows: 1 titlebar, 2 separator, 3 tab bar, 4.. content (`content_height = rows-5`), separator, `rows` status. When `show_hscroll`, the last content row is the shared horizontal bar and `editor_view_rows` is one shorter. Vertical bars (`show_editor_vscroll`, `show_tree_vscroll`) occupy the pane's right column on text rows only (`┃` / `│`); the corner stays on the horizontal bar. `refresh_hscroll` recomputes horizontal then vertical twice so a stolen column or row settles. `>` not `>=`. `ensure_tree_visible` uses `editor_view_rows` and honors `follow_tree`.

## 3. Critical Invariants (do not break)

1. **Cols are byte indices.** Always `clamp_char_boundary()` after arithmetic. Use `prev/next_char_boundary()`, never `col±1` on UTF-8. Tests cover this indirectly.
2. **Raw mode must restore.** `enable_raw_mode()` saves `stty -g`, `cleanup()` restores or `stty sane`, plus `\x1b[?1002l \x1b[?1006l \x1b[?1000l \x1b[?2004l \x1b[0m \x1b[?25h \x1b[?1049l`. Every early return in `run()` must call `cleanup()`.
3. **`0x08` duality.** `Ctrl+H` == Backspace on some terms. Rule: in `Focus::Editor`, `0x08` falls through to editor backspace; in `Tree`, toggles sidebar. Don't re-add global `0x08` → toggle unconditionally.
4. **`absolute_path()` must stay absolute.** Resolves against CWD + lexical `..` normalisation without FS access (supports new files). Session/recovery hashes depend on it.
5. **History byte positions.** `apply_insert/delete` mutate `lines` + set `cursor=start`. `undo()` iterates `ops.rev()`, `redo()` forward. `mark_edited()` bumps `revision` (monotonic — do not decrement it; that breaks redo). `modified` is `buffer_hash(lines) != saved_hash`. Undo back to the saved text clears the dirty flag. `col_offset` is a visual column (`fit_visual_offset`), not a byte index.
6. **No crates.** Std only (`fs`, `io`, `env`, `process::Command` for `date`, raw `ioctl` FFI). Don't add deps without discussion.
7. **Picker widths.** Clamp to `cols-2`. Narrow terminals (30 cols) must not overflow. See `simple_picker_string()`.
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
| Find/replace | `search_replace_dialog()` (path, find, replace, count, Tab, Enter, Cancel, click-outside). `find_next()`, `replace_all()`, `apply_project_replace()`, `parse_search_query()` (`%` prefix). |
| Tree / file ops | `add_tree_rows()`, `collect_quick_open_files()`, `reveal_path_in_tree()`, `create/rename/delete_*_prompt()`. |
| Autocomplete | `autocomplete_context()` → `plugins::completion_context()`, `refresh_autocomplete()`, per-plugin `completion_*`. |
| Session/recovery | `state_dir()`, `session_file()`, `try_restore_session()` (reads `tab_index`), `save_session()`, `write_recovery_for_current_tab()` (250ms throttle), `offer_recovery()`. |
| Perf | `clock_text()` caches `date` subprocess per minute. Quick-open symbols scan 20 files per `read_key` wake; find-in-files scans 25; the replace count scans 20. Caps stay (600 files/1MB symbols, 3000 files/5MB search, 10k matches). Don't remove caps. Don't add a thread. |

## 6. Testing

```sh
cargo check   # fast gate
cargo test    # 61 tests (60 run, 1 ignored Wayland roundtrip): cli_path, absolute, quick_open parse, html auto-close, find, escape, search %, navigation keys, plugins, bashrc/shebang, mouse SGR + click-col, replace counting, menu geometry, Ctrl+Shift+H, word range, Ctrl+K + shortcuts, OSC52, legacy mouse, wheel pan, paste, sudo message, wayland socketpair, tab window, visual scroll, vertical scrollbar, wrap, dirty-hash, CRLF, welcome logo, dialog chrome
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

## 7. Bugs Fixed (2.0.1 + 2.1 + 2.2 + 2.5 + 2.6 + 3.0) — Don't Regress

Unreleased (vertical scrollbars):
- `show_editor_vscroll` / `show_tree_vscroll` when `lines` or `tree_rows` exceed `editor_view_rows`. Hidden sidebar draws no tree bar. The text column shrinks by one while the bar is up (`editor_text_width`, `tree_text_width`). Drag sets `follow_cursor` or `follow_tree` false. Tree keys, wheel, tree clicks, and `reveal_path_in_tree` set `follow_tree` true. Right-click and middle-click on a bar return before the menu. Do not draw the vertical bar on the horizontal-bar row.

3.0 (viewport, dialogs, tabs, bash rc files, dirty flag, CRLF, scrollbars, clipboard, sudo save, paste):
- Wheel is one row per report. Editor pans `row_offset` with `follow_cursor = false` (caret and selection stay). Tree moves `tree_index` ±1. Tab-bar wheel calls `cycle_tab`. Do not put the ±3 step back and do not move the caret from the wheel.
- Tabs: `visible_tab_indexes` uses `window_indexes` (max 9, centered on the current tab). `Alt+1-9` hits that window. `+` is an extra width after the tabs (`tab_hit_index` == `visible.len()`); middle/right-click on it must not close a tab. `Ctrl+N` and `+` call `new_tab(true)`. Saving a pathless tab prompts `Filename:`.
- `Ctrl+F` opens `find_dialog` (current buffer: query, count, Next, Close). `Ctrl+R` and `Ctrl+Shift+H` open `search_replace_dialog`. Enter replaces all in the target (empty path or the current file = buffer; a directory = project replace, no y/N). Caps stay. Click-outside uses `picker_frame` / `PickerMouse` for every list dialog.
- `col_offset` is a visual column (`fit_visual_offset`, `byte_at_visual`). Horizontal bar is the last content row when either pane overflows (`refresh_hscroll`). Do not steal the status row. Do not add word wrap unless asked.
- Bash: `is_shell_rc_name` in `from_path`, `syntax_from_shebang` only when `from_path` is Plain. `Tab::syntax()` checks a manual mode first. `bash.rs` itself is unchanged.
- `modified` compares `buffer_hash` to `saved_hash`. Do not decrement `revision`. Recovery sets `saved_hash` to `hash.wrapping_add(1)` so the restored tab stays dirty. `Tab.crlf` is detected before LF normalization; `text()` joins with `\r\n` when set. Project replace preserves it.
- Clipboard: `clipboard_verified` is true only when a tool or Wayland confirmed. Paste prefers the OS clipboard only then; otherwise the internal clipboard when it is non-empty. SSH still prints OSC52 first. Do not call the live Wayland clipboard from tests. `wayland_clipboard_roundtrip` stays ignored.
- Copy/cut of an unselected line uses `line_as_clipboard` (newline unless it is the last line). `(` `{` `[` `"` `'` on a selection call `wrap_selection`. Empty pairs are `(` `{` `[` only.
- Paste burst: `pending_is_paste_burst` is 16+ bytes, or a newline AND length >= 8. A lone Enter must stay auto-indent.
- Prompts: `apply_line_edit` (Left/Right/Home/End, Ctrl+U). Up/Down walk `prompt_history` (cap 50, skip secrets).
- Resize: `terminal_resized()` in the input loop. Status flash uses `was_flashing` for one clear frame. Welcome and the update notice pass the dismissing key to `handle_key`.
- `./build.sh` `install_system_wide` copies `/usr/local/bin/az` (directly when root or the dir is writable, otherwise `sudo cp`). Failure must not fail the `~/.local/bin` install. `sudo` `secure_path` does not include `~/.local/bin`.
- Permission-denied save (`ErrorKind::PermissionDenied`, and not already root): status blinks red (`message_is_error`), `prompt_secret` (RED, `*` masking), then `write_file_with_sudo` (`sudo -k -S -p ''`, password + newline on stdin only, payload staged in a temp file). Other status flashes stay cyan.
- Tests: 56 (55 run). Ignored `wayland_clipboard_roundtrip` restores the previous clipboard. Do not run it.

2.6 (languages + update check + input/clipboard fixes):
- 20 new plugins (41 total + Plain = 42 modes): python, java, csharp, cpp, c, go, kotlin, swift, ruby, dart, scala, r, lua, perl, haskell, elixir, clojure, zig, julia, objc. Each wires `mod X;` + `mode_label` + `from_word` + `from_path` + `tree_color` + `highlight_segments` + word completion/symbols, plus `SyntaxMode::X` + palette `set-syntax-x` + `run_command` in `main.rs`. `.h` → C (C++ headers highlight as C — accepted, documented). `Gemfile`/`Rakefile` → Ruby.
- Startup update check: `check_for_updates()` (curl `--max-time 5` of raw GitHub `Cargo.toml`, silent on failure, `AZ_NO_UPDATE_CHECK=1` opt-out) runs in `main()` before raw mode; newer version shown via `render_update_notice()` modal in `run()`. `--version`/`--help` use `env!("CARGO_PKG_VERSION")`, never a hardcoded string.
- Wheel bursts: `read_key()` glues multi-report stdin reads; `handle_key()` now dispatches `parse_mouse_events()` (splits concatenated SGR/legacy reports, ignores trailing partials) instead of parsing the chunk as one event. Never route menu keys through global shortcuts (unchanged).
- Clipboard deadlock: `pipe_to_clipboard_tool()` drops stdin before `wait()` — tools read stdin to EOF, so waiting first hung the editor. Tests: `cat`-based EOF test (hangs pre-fix, passes post-fix).
- Tests: 38 total (new: mouse-burst splitting, end-to-end wheel scrolling, clipboard-pipe EOF, remote-version parsing + comparison).

2.5 (replace-all + context menus + chrome):
- Replace in Files is `Ctrl+Shift+H` only (`is_ctrl_shift_h`); caps: 3000 files, 5MB, 10k matches. Open modified tabs are skipped (never clobber unsaved buffers); reloaded tabs get `undo/redo` cleared (positions refer to old content).
- Right-click (`button&3==2`) opens `context_menu()`; middle-click (`==1`) on topbar closes via `close_tab_at()`. Menu loop swallows its own mouse (click-away/Esc cancel, `1-9` pick). Never route menu keys through global shortcuts.
- Inactive tabs use `BG_TAB` (lighter than bar); open file row uses `BG_HIGHLIGHT` in `render_tree_line()`; status chips in `render_status_line()` (`modified`-only orange, stats yellow) — no `saved` chip, it was display-only noise. Mode chip lives in the titlebar via `focus_label()`.
- `Ctrl+K` opens `shortcuts_dialog()` (`shortcut_defs()` + shared `filter_command_items()`); `Ctrl+/` (`is_ctrl_slash`) is gone. Titlebar buttons (plain ACCENT text, no bg chip; red `Quit` with `QUIT_LABEL` left of the clock) hit-tested via `titlebar_button_regions()` → quick-open/palette/shortcuts/quit. Palette `welcome` shows the welcome dialog again.
- Clipboard: `copy_to_system_clipboard()` returns verified bool; cascade `try_clipboard_tool()` (`wl-copy`/`xclip`/`xsel`/`pbcopy` via `pipe_to_clipboard_tool()`) then `osc52_sequence()`; OSC52-first over SSH. Messages `Copied` vs `Copied (OSC52)`.
- Status flash: `render()` arms `status_flash_until` on message change (750ms, 250ms phases, CYAN); `prompt()` blinks every prompt light blue (350ms) using non-blocking `read_key()` ticks + `redraw` flag.
- Welcome logo is `logo.txt` at the repo root, embedded with `include_str!` (`welcome_logo()`). Dark ink (`30`/`90`) is `#578bfb`; light ink (`37`/`97`) and the solid fills (`47`) are `#7aa2f7`. The mark is centered in the box. Read the terminal size before building the welcome or update dialog — `Editor::new` still has 80×24, and `present_overlay` paints after the string is built. The sidebar draws no hardware caret (the selection highlight is the indicator). Dialogs repaint only when their contents change: search, quick open, and find-in-files must not call `frame()` on every input wake. Shared chrome is `dialog_border/title/body/selected/input/muted`.

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

- `revision` stays monotonic. `modified` is `buffer_hash != saved_hash`. Do not “fix” a stale dirty flag by decrementing revision (breaks redo).
- No word wrap. Horizontal scroll plus a scrollbar row, and a vertical bar when the pane is taller than the viewport, is the current behavior. Real wrap needs `render_editor_line()` + `cursor_screen_position()` + `ensure_editor_visible()` rework (screen-row vs file-line mapping).
- Quick-open symbols and find-in-files scan a chunk per input wake. The file list is still collected up front (caps unchanged) and can stall once. A background index is future work. Do not add a crate or a thread for it.
- Recovery separator `---TEXT---\n` collides if filename ends with that string (filenames can't contain `\n`, so risk tiny). Proper fix: length-prefixed body or NUL separator with migration.
- No splits, no regex — out of scope unless requested. Drag-select already exists (left-drag, double-click word, triple-click line).

## 9. Style & PR Rules

- `cargo check` + `cargo test` clean, no warnings (`unused_assignments`, `dead_code` fail review).
- Keep diffs minimal; single concern per change; update `USER_MANUAL.md` + help text + `--help` together when adding keys.
- Byte-safe Rust: no `s[col..]` without clamp; prefer `clamp_char_boundary()`.
- ANSI: always reset `\x1b[0m`, pad with `fit_plain/fit_ansi()` to avoid ghost text.
- Don't spawn subprocesses per keystroke (see clock cache pattern).
- Don't add crates; don't use `unsafe` except existing `ioctl`.
- Update this file + `USER_MANUAL.md` when changing architecture or shortcuts.
