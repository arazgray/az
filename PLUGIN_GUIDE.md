# Az plugins

Language support now lives in `src/plugins`.

Files included (54 hand-written + 98 table-driven = 152 languages):

- `php.rs` / `blade.rs` / `html.rs` / `css.rs` / `javascript.rs`: as before (mixed highlighting for Blade/PHP/HTML)
- `typescript.rs`: JS highlighting + `interface`/`type`/`enum`, TS symbols
- `xml.rs`: tags/attrs/entities, `<?…?>`, comments
- `markdown.rs`: headings/bold/code/links, heading symbols
- `json.rs` / `toml.rs` / `yaml.rs`: keys/values/comments, section symbols (TOML)
- `bash.rs`: `$VAR`s, keywords/builtins, `fn` symbols, var completion. `*.sh`/`*.bash`/`*.zsh` plus extensionless rc names (`.bashrc`, `.zshrc`, `.profile`, …). A `#!` for sh/bash/zsh/ksh/dash/ash/mksh is Bash when the filename would otherwise be plain. A manual syntax mode still wins. `~/.bashrc` used to stay plain because it has no extension.
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
- `ada.rs`: Ada, -- comments, marker symbols
- `arduino.rs`: Arduino, // comments, block comments, function symbols
- `asciidoc.rs`: AsciiDoc, // comments, heading symbols
- `ats.rs`: ATS, // comments, block comments, marker symbols
- `awk.rs`: Awk, # comments, marker symbols
- `bat.rs`: Batch, REM/:: comments, label symbols
- `b.rs`: B, // comments, block comments, function symbols
- `caddyfile.rs`: Caddyfile, # comments, no symbols
- `cake.rs`: Cake, // comments, block comments, task symbols
- `coffeescript.rs`: CoffeeScript, # comments, block comments, marker symbols
- `conky.rs`: Conky, # comments, no symbols
- `crontab.rs`: Crontab, # comments, no symbols
- `crystal.rs`: Crystal, # comments, marker symbols
- `cuda.rs`: CUDA, // comments, block comments, function symbols
- `cython.rs`: Cython, # comments, marker symbols
- `d.rs`: D, // comments, block comments, marker symbols
- `dot.rs`: Graphviz, ///# comments, block comments, marker symbols
- `ebuild.rs`: Ebuild, # comments, function symbols
- `elm.rs`: Elm, -- comments, block comments, marker symbols
- `erb.rs`: ERB, block comments, marker symbols
- `etcportage.rs`: Portage, # comments, no symbols
- `fish.rs`: Fish, # comments, marker symbols
- `forth.rs`: Forth, block comments, backslash comments, colon definitions
- `fortran.rs`: Fortran, ! comments, marker symbols
- `freebsd.rs`: FreeBSD kernel, # comments, no symbols
- `fsharp.rs`: F#, // comments, block comments, marker symbols
- `gdscript.rs`: GDScript, # comments, marker symbols
- `gemini.rs`: Gemini, spaced #, heading symbols
- `gitcommit.rs`: Git commit, line-start #, no symbols
- `gitconfig.rs`: Git config, spaced #, section symbols
- `gitrebase.rs`: Git rebase, line-start #, commit subjects
- `gleam.rs`: Gleam, // comments, marker symbols
- `glsl.rs`: GLSL, // comments, block comments, function symbols
- `gnuplot.rs`: Gnuplot, # comments, function symbols
- `godoc.rs`: Go doc, // comments, no symbols
- `golo.rs`: Golo, # comments, marker symbols
- `gomod.rs`: Go mod, // comments, marker symbols
- `groff.rs`: Groff, roff requests, title symbols
- `groovy.rs`: Groovy, // comments, block comments, marker symbols
- `haml.rs`: Haml, spaced #, line-start -#//, no symbols
- `hare.rs`: Hare, // comments, marker symbols
- `hc.rs`: HolyC, // comments, block comments, function symbols
- `inputrc.rs`: Inputrc, spaced #, no symbols
- `jinja2.rs`: Jinja2, block comments, block symbols
- `jsonnet.rs`: Jsonnet, ///# comments, block comments, no symbols
- `justfile.rs`: Just, # comments, target symbols
- `keymap.rs`: Keymap, line-start !, no symbols
- `kickstart.rs`: Kickstart, # comments, section symbols
- `kvlang.rs`: Kvlang, # comments, rule symbols
- `ledger.rs`: Ledger, line-start ;, dated entries
- `lfe.rs`: LFE, spaced ;, marker symbols
- `lilypond.rs`: LilyPond, % comments, block comments, no symbols
- `lisp.rs`: Lisp, spaced ;, marker symbols
- `mail.rs`: Mail, line-start >, no symbols
- `man.rs`: Man page, roff requests, title symbols
- `mc.rs`: MC (sendmail), # comments, line-start dnl, marker symbols
- `meson.rs`: Meson, # comments, marker symbols
- `micro.rs`: Micro config, # comments, no symbols
- `mpd.rs`: MPD config, spaced #, no symbols
- `msbuild.rs`: MSBuild, block comments, no symbols
- `nanorc.rs`: Nanorc, line-start #, no symbols
- `nftables.rs`: nftables, spaced #, marker symbols
- `nim.rs`: Nim, # comments, block comments, marker symbols
- `nix.rs`: Nix, # comments, block comments, no symbols
- `nu.rs`: Nushell, # comments, marker symbols
- `ocaml.rs`: OCaml, block comments, marker symbols
- `octave.rs`: Octave, %/# comments, block comments, marker symbols
- `odin.rs`: Odin, // comments, block comments, proc symbols
- `pascal.rs`: Pascal, // comments, block comments, marker symbols
- `patch.rs`: Patch, diff colors, diff paths
- `pkgconfig.rs`: pkg-config, # comments, no symbols
- `peg.rs`: PEG, spaced --, no symbols
- `po.rs`: PO file, spaced #, marker symbols
- `pony.rs`: Pony, // comments, block comments, marker symbols
- `pov.rs`: POV-Ray, // comments, block comments, marker symbols
- `privoxy.rs`: Privoxy, spaced #, no symbols
- `prql.rs`: PRQL, # comments, marker symbols
- `puppet.rs`: Puppet, spaced #, marker symbols
- `raku.rs`: Raku, # comments, marker symbols
- `renpy.rs`: Ren'Py, # comments, marker symbols
- `rpmspec.rs`: RPM spec, spaced #, section symbols
- `rest.rs`: reST, line-start .., no symbols
- `sage.rs`: Sage, # comments, marker symbols
- `salt.rs`: SaltStack, line-start #, target symbols
- `sed.rs`: Sed, spaced #, no symbols
- `smalltalk.rs`: Smalltalk, plain text, no symbols
- `stata.rs`: Stata, // comments, line-start *, block comments, marker symbols
- `tcl.rs`: Tcl, Tcl # comments, marker symbols
- `twig.rs`: Twig, block comments, block symbols
- `v.rs`: V, // comments, block comments, marker symbols
- `vala.rs`: Vala, // comments, block comments, marker symbols
- `verilog.rs`: Verilog, // comments, block comments, marker symbols
- `vhdl.rs`: VHDL, -- comments, marker symbols
- `vi.rs`: Vimscript, line-start ", marker symbols
- `xresources.rs`: Xresources, ! comments, no symbols
- `yum.rs`: Yum repo, spaced #, section symbols
- `zscript.rs`: ZScript, // comments, block comments, marker symbols
- `scad.rs`: OpenSCAD, // comments, block comments, marker symbols


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
