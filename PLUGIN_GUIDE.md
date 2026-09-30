# Az plugins

Language support now lives in `src/plugins`.

Files included (42 languages):

- `php.rs` / `blade.rs` / `html.rs` / `css.rs` / `javascript.rs`: as before (mixed highlighting for Blade/PHP/HTML)
- `typescript.rs`: JS highlighting + `interface`/`type`/`enum`, TS symbols
- `xml.rs`: tags/attrs/entities, `<?…?>`, comments
- `markdown.rs`: headings/bold/code/links, heading symbols
- `json.rs` / `toml.rs` / `yaml.rs`: keys/values/comments, section symbols (TOML)
- `bash.rs`: `$VAR`s, keywords/builtins, `fn` symbols, var completion
- `dotenv.rs` / `ini.rs` / `systemd.rs`: `KEY=`/`key:`/sections, comments
- `log.rs`: timestamps + ERROR/WARN/INFO/DEBUG (no completion by design)
- `rust.rs`: keywords/types/macros, `fn/struct/enum` symbols + completion
- `nginx.rs` / `apache.rs`: directives/blocks, `$vars`, completion
- `dockerfile.rs`: `FROM`/`RUN`/… instructions, `$vars`, completion
- `sql.rs`: keywords, `--` comments, keyword completion
- `python.rs`: keywords/builtins, `#` comments, `@decorators`, `def`/`class` symbols
- `java.rs` / `csharp.rs` / `kotlin.rs` / `scala.rs`: keywords/types, `//` + `/* */`, class-ish symbols
- `cpp.rs` / `c.rs`: keywords/types, `#include`, `class`/`struct`/`namespace` symbols
- `go.rs`: keywords/types, `func`/`type` symbols
- `swift.rs`: keywords/types, `@attrs`, `func`/`struct`/`protocol` symbols
- `ruby.rs`: keywords/builtins, `#` comments, `:symbols`, `def`/`class`/`module` symbols
- `dart.rs`: keywords/types, `class`/`mixin` symbols
- `r.rs`: keywords/builtins, `#` comments, `name <- function()` symbols
- `lua.rs`: keywords/builtins, `--` comments, `function` symbols
- `perl.rs`: keywords/builtins, `#` comments, `$@%` sigils, `sub`/`package` symbols
- `haskell.rs`: keywords/types, `--` + `{- -}`, `name :: Type` symbols
- `elixir.rs`: keywords/builtins, `#` comments, `@attrs`, `def`/`defmodule` symbols
- `clojure.rs`: keywords/builtins, `;` comments, `:keywords`, `defn`/`ns` symbols
- `zig.rs`: keywords/types, `fn`/`const`/`test` symbols
- `julia.rs`: keywords/types, `#` + `#= =#`, `function`/`struct`/`macro` symbols
- `objc.rs`: keywords/types, `#import`, `@directives`, `@interface` + method symbols
- `example.rs`: documented skeleton for adding a new plugin
- `mod.rs`: plugin registry and mixed-language facade used by the editor

Tip: config-like languages (`toml`/`yaml`/`ini`/`dotenv`/`systemd`) share the same shape — keys, strings via `string_ranges()`, comments outside strings via `pos_in_ranges()`. Copy `ini.rs` as a starting point.

## Add a new plugin

1. Create `src/plugins/language.rs`.
2. Add `pub(crate) mod language;` in `src/plugins/mod.rs`.
3. Add its command word and extensions in `from_word` and `from_path`. For JavaScript, that is `js | javascript | mjs | cjs | jsx` and `.js/.mjs/.cjs/.jsx`.
4. Wire it into `highlight_segments`, `completion_context`, `completion_items`, and `extract_symbols` only if needed.

Plugins are compiled Rust modules. They do not need external crates.


## JavaScript plugin wiring example

The JavaScript plugin is enabled in these places:

- `src/plugins/javascript.rs`: actual highlighting, completion, and symbols.
- `src/plugins/mod.rs`: `pub(crate) mod javascript;` registers the file.
- `src/plugins/mod.rs`: `from_word()` enables `set js` / `set javascript`.
- `src/plugins/mod.rs`: `from_path()` enables `.js`, `.mjs`, `.cjs`, and `.jsx` files.
- `src/plugins/mod.rs`: `highlight_segments()`, `completion_context()`, `completion_items()`, and `extract_symbols()` delegate editor features to the plugin.
- `src/main.rs`: `SyntaxMode::JavaScript` adds JS as an editor mode.
- `src/main.rs`: `command_items()` and `run_command()` add the command palette item.
