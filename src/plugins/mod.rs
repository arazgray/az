//! Language plugin facade for Az.
//!
//! Each language lives in its own file in this folder. The editor calls only
//! the small functions in this module, then this module delegates to the right
//! plugin. To add a new built in plugin:
//!
//! 1. Create `src/plugins/my_language.rs`.
//! 2. Add `pub(crate) mod my_language;` below.
//! 3. Add its extension and command word to `from_path` and `from_word`.
//! 4. Add it to `highlight_segments`, `completion_context`, `completion_items`,
//!    and `extract_symbols` if the language supports those features.
//!
//! See `example.rs` for a tiny documented skeleton.

use std::ffi::OsStr;
use std::path::Path;

use crate::{BLUE, CYAN, FG_DARK, GREEN, MAGENTA, ORANGE, PURPLE, RED, Segment, SyntaxMode, YELLOW, CompletionItem};

pub(crate) mod php;
pub(crate) mod html;
pub(crate) mod css;
pub(crate) mod javascript;
pub(crate) mod blade;
pub(crate) mod markdown;
pub(crate) mod json;
pub(crate) mod toml;
pub(crate) mod yaml;
pub(crate) mod bash;
pub(crate) mod dotenv;
pub(crate) mod ini;
pub(crate) mod log;
pub(crate) mod rust;
pub(crate) mod nginx;
pub(crate) mod apache;
pub(crate) mod dockerfile;
pub(crate) mod systemd;
pub(crate) mod sql;
pub(crate) mod typescript;
pub(crate) mod xml;
pub(crate) mod python;
pub(crate) mod java;
pub(crate) mod csharp;
pub(crate) mod cpp;
pub(crate) mod c;
pub(crate) mod go;
pub(crate) mod kotlin;
pub(crate) mod swift;
pub(crate) mod ruby;
pub(crate) mod dart;
pub(crate) mod scala;
pub(crate) mod r;
pub(crate) mod lua;
pub(crate) mod perl;
pub(crate) mod haskell;
pub(crate) mod elixir;
pub(crate) mod clojure;
pub(crate) mod zig;
pub(crate) mod julia;
pub(crate) mod objc;
pub(crate) mod makefile;
pub(crate) mod cmake;
pub(crate) mod powershell;
pub(crate) mod proto;
pub(crate) mod graphql;
pub(crate) mod terraform;
pub(crate) mod vue;
pub(crate) mod svelte;
pub(crate) mod asm;
pub(crate) mod tex;
pub(crate) mod erlang;
pub(crate) mod solidity;
pub(crate) mod ada;
pub(crate) mod arduino;
pub(crate) mod asciidoc;
pub(crate) mod ats;
pub(crate) mod awk;
pub(crate) mod bat;
pub(crate) mod b;
pub(crate) mod caddyfile;
pub(crate) mod cake;
pub(crate) mod coffeescript;
pub(crate) mod conky;
pub(crate) mod crontab;
pub(crate) mod crystal;
pub(crate) mod cuda;
pub(crate) mod cython;
pub(crate) mod d;
pub(crate) mod dot;
pub(crate) mod ebuild;
pub(crate) mod elm;
pub(crate) mod erb;
pub(crate) mod etcportage;
pub(crate) mod fish;
pub(crate) mod forth;
pub(crate) mod fortran;
pub(crate) mod freebsd;
pub(crate) mod fsharp;
pub(crate) mod gdscript;
pub(crate) mod gemini;
pub(crate) mod gitcommit;
pub(crate) mod gitconfig;
pub(crate) mod gitrebase;
pub(crate) mod gleam;
pub(crate) mod glsl;
pub(crate) mod gnuplot;
pub(crate) mod godoc;
pub(crate) mod golo;
pub(crate) mod gomod;
pub(crate) mod groff;
pub(crate) mod groovy;
pub(crate) mod haml;
pub(crate) mod hare;
pub(crate) mod hc;
pub(crate) mod inputrc;
pub(crate) mod jinja2;
pub(crate) mod jsonnet;
pub(crate) mod justfile;
pub(crate) mod keymap;
pub(crate) mod kickstart;
pub(crate) mod kvlang;
pub(crate) mod ledger;
pub(crate) mod lfe;
pub(crate) mod lilypond;
pub(crate) mod lisp;
pub(crate) mod mail;
pub(crate) mod man;
pub(crate) mod mc;
pub(crate) mod meson;
pub(crate) mod micro;
pub(crate) mod mpd;
pub(crate) mod msbuild;
pub(crate) mod nanorc;
pub(crate) mod nftables;
pub(crate) mod nim;
pub(crate) mod nix;
pub(crate) mod nu;
pub(crate) mod ocaml;
pub(crate) mod octave;
pub(crate) mod odin;
pub(crate) mod pascal;
pub(crate) mod patch;
pub(crate) mod pkgconfig;
pub(crate) mod peg;
pub(crate) mod po;
pub(crate) mod pony;
pub(crate) mod pov;
pub(crate) mod privoxy;
pub(crate) mod prql;
pub(crate) mod puppet;
pub(crate) mod raku;
pub(crate) mod renpy;
pub(crate) mod rpmspec;
pub(crate) mod rest;
pub(crate) mod sage;
pub(crate) mod salt;
pub(crate) mod sed;
pub(crate) mod smalltalk;
pub(crate) mod stata;
pub(crate) mod tcl;
pub(crate) mod twig;
pub(crate) mod v;
pub(crate) mod vala;
pub(crate) mod verilog;
pub(crate) mod vhdl;
pub(crate) mod vi;
pub(crate) mod xresources;
pub(crate) mod yum;
pub(crate) mod zscript;
pub(crate) mod scad;
pub(crate) mod plain;
pub(crate) mod example;

#[derive(Clone, Copy)]
pub(crate) struct CompletionContext<'a> {
    pub(crate) lines: &'a [String],
    pub(crate) scan_limit: usize,
}

pub(crate) fn mode_label(mode: SyntaxMode) -> &'static str {
    match mode {
        SyntaxMode::Php => "PHP",
        SyntaxMode::Blade => "BLADE",
        SyntaxMode::Html => "HTML",
        SyntaxMode::Css => "CSS",
        SyntaxMode::JavaScript => "JS",
        SyntaxMode::TypeScript => "TS",
        SyntaxMode::Xml => "XML",
        SyntaxMode::Markdown => "MD",
        SyntaxMode::Json => "JSON",
        SyntaxMode::Toml => "TOML",
        SyntaxMode::Yaml => "YAML",
        SyntaxMode::Bash => "SH",
        SyntaxMode::Dotenv => "ENV",
        SyntaxMode::Ini => "INI",
        SyntaxMode::Log => "LOG",
        SyntaxMode::Rust => "RUST",
        SyntaxMode::Nginx => "NGINX",
        SyntaxMode::Apache => "APACHE",
        SyntaxMode::Dockerfile => "DOCKER",
        SyntaxMode::Systemd => "SYSTEMD",
        SyntaxMode::Sql => "SQL",
        SyntaxMode::Python => "PY",
        SyntaxMode::Java => "JAVA",
        SyntaxMode::Csharp => "CS",
        SyntaxMode::Cpp => "CPP",
        SyntaxMode::C => "C",
        SyntaxMode::Go => "GO",
        SyntaxMode::Kotlin => "KT",
        SyntaxMode::Swift => "SWIFT",
        SyntaxMode::Ruby => "RB",
        SyntaxMode::Dart => "DART",
        SyntaxMode::Scala => "SCALA",
        SyntaxMode::R => "R",
        SyntaxMode::Lua => "LUA",
        SyntaxMode::Perl => "PL",
        SyntaxMode::Haskell => "HS",
        SyntaxMode::Elixir => "EX",
        SyntaxMode::Clojure => "CLJ",
        SyntaxMode::Zig => "ZIG",
        SyntaxMode::Julia => "JL",
        SyntaxMode::Objc => "OBJC",
        SyntaxMode::Makefile => "MAKE",
        SyntaxMode::Cmake => "CMAKE",
        SyntaxMode::Powershell => "PS1",
        SyntaxMode::Proto => "PROTO",
        SyntaxMode::Graphql => "GQL",
        SyntaxMode::Terraform => "TF",
        SyntaxMode::Vue => "VUE",
        SyntaxMode::Svelte => "SVELTE",
        SyntaxMode::Asm => "ASM",
        SyntaxMode::Tex => "TEX",
        SyntaxMode::Erlang => "ERL",
        SyntaxMode::Solidity => "SOL",
        SyntaxMode::Ada => "ADA",
        SyntaxMode::Arduino => "ARDUINO",
        SyntaxMode::Asciidoc => "ADOC",
        SyntaxMode::Ats => "ATS",
        SyntaxMode::Awk => "AWK",
        SyntaxMode::Bat => "BAT",
        SyntaxMode::B => "B",
        SyntaxMode::Caddyfile => "CADDY",
        SyntaxMode::Cake => "CAKE",
        SyntaxMode::Coffeescript => "COFFEE",
        SyntaxMode::Conky => "CONKY",
        SyntaxMode::Crontab => "CRON",
        SyntaxMode::Crystal => "CR",
        SyntaxMode::Cuda => "CUDA",
        SyntaxMode::Cython => "CYTHON",
        SyntaxMode::D => "D",
        SyntaxMode::Dot => "DOT",
        SyntaxMode::Ebuild => "EBUILD",
        SyntaxMode::Elm => "ELM",
        SyntaxMode::Erb => "ERB",
        SyntaxMode::Etcportage => "PORTAGE",
        SyntaxMode::Fish => "FISH",
        SyntaxMode::Forth => "FORTH",
        SyntaxMode::Fortran => "FORTRAN",
        SyntaxMode::Freebsd => "FREEBSD",
        SyntaxMode::Fsharp => "FS",
        SyntaxMode::Gdscript => "GD",
        SyntaxMode::Gemini => "GEMINI",
        SyntaxMode::Gitcommit => "GIT-COMMIT",
        SyntaxMode::Gitconfig => "GITCONFIG",
        SyntaxMode::Gitrebase => "GIT-REBASE",
        SyntaxMode::Gleam => "GLEAM",
        SyntaxMode::Glsl => "GLSL",
        SyntaxMode::Gnuplot => "GNUPLOT",
        SyntaxMode::Godoc => "GODOC",
        SyntaxMode::Golo => "GOLO",
        SyntaxMode::Gomod => "GOMOD",
        SyntaxMode::Groff => "GROFF",
        SyntaxMode::Groovy => "GROOVY",
        SyntaxMode::Haml => "HAML",
        SyntaxMode::Hare => "HARE",
        SyntaxMode::Hc => "HC",
        SyntaxMode::Inputrc => "INPUTRC",
        SyntaxMode::Jinja2 => "JINJA",
        SyntaxMode::Jsonnet => "JSONNET",
        SyntaxMode::Justfile => "JUST",
        SyntaxMode::Keymap => "KEYMAP",
        SyntaxMode::Kickstart => "KICKSTART",
        SyntaxMode::Kvlang => "KV",
        SyntaxMode::Ledger => "LEDGER",
        SyntaxMode::Lfe => "LFE",
        SyntaxMode::Lilypond => "LY",
        SyntaxMode::Lisp => "LISP",
        SyntaxMode::Mail => "MAIL",
        SyntaxMode::Man => "MAN",
        SyntaxMode::Mc => "MC",
        SyntaxMode::Meson => "MESON",
        SyntaxMode::Micro => "MICRO",
        SyntaxMode::Mpd => "MPD",
        SyntaxMode::Msbuild => "MSBUILD",
        SyntaxMode::Nanorc => "NANORC",
        SyntaxMode::Nftables => "NFT",
        SyntaxMode::Nim => "NIM",
        SyntaxMode::Nix => "NIX",
        SyntaxMode::Nu => "NU",
        SyntaxMode::Ocaml => "OCAML",
        SyntaxMode::Octave => "OCTAVE",
        SyntaxMode::Odin => "ODIN",
        SyntaxMode::Pascal => "PASCAL",
        SyntaxMode::Patch => "DIFF",
        SyntaxMode::Pkgconfig => "PKGCONF",
        SyntaxMode::Peg => "PEG",
        SyntaxMode::Po => "PO",
        SyntaxMode::Pony => "PONY",
        SyntaxMode::Pov => "POV",
        SyntaxMode::Privoxy => "PRIVOXY",
        SyntaxMode::Prql => "PRQL",
        SyntaxMode::Puppet => "PUPPET",
        SyntaxMode::Raku => "RAKU",
        SyntaxMode::Renpy => "RENPY",
        SyntaxMode::Rpmspec => "SPEC",
        SyntaxMode::Rest => "RST",
        SyntaxMode::Sage => "SAGE",
        SyntaxMode::Salt => "SALT",
        SyntaxMode::Sed => "SED",
        SyntaxMode::Smalltalk => "SMALLTALK",
        SyntaxMode::Stata => "STATA",
        SyntaxMode::Tcl => "TCL",
        SyntaxMode::Twig => "TWIG",
        SyntaxMode::V => "V",
        SyntaxMode::Vala => "VALA",
        SyntaxMode::Verilog => "VERILOG",
        SyntaxMode::Vhdl => "VHDL",
        SyntaxMode::Vi => "VIM",
        SyntaxMode::Xresources => "XRES",
        SyntaxMode::Yum => "YUM",
        SyntaxMode::Zscript => "ZSCRIPT",
        SyntaxMode::Scad => "SCAD",
        SyntaxMode::Plain => "PLAIN",
    }
}

pub(crate) fn from_word(word: &str) -> Option<SyntaxMode> {
    match word.trim().to_ascii_lowercase().as_str() {
        "php" => Some(SyntaxMode::Php),
        "blade" => Some(SyntaxMode::Blade),
        "html" | "html4" | "html5" => Some(SyntaxMode::Html),
        "css" => Some(SyntaxMode::Css),
        "js" | "javascript" | "mjs" | "cjs" | "jsx" => Some(SyntaxMode::JavaScript),
        "ts" | "typescript" | "tsx" | "mts" | "cts" => Some(SyntaxMode::TypeScript),
        "xml" | "svg" => Some(SyntaxMode::Xml),
        "md" | "markdown" | "mkd" => Some(SyntaxMode::Markdown),
        "json" | "jsonc" | "json5" => Some(SyntaxMode::Json),
        "toml" => Some(SyntaxMode::Toml),
        "yaml" | "yml" => Some(SyntaxMode::Yaml),
        "sh" | "bash" | "zsh" | "shell" => Some(SyntaxMode::Bash),
        "dotenv" | "env" => Some(SyntaxMode::Dotenv),
        "ini" | "conf" | "cfg" | "config" => Some(SyntaxMode::Ini),
        "log" => Some(SyntaxMode::Log),
        "rust" | "rs" => Some(SyntaxMode::Rust),
        "nginx" => Some(SyntaxMode::Nginx),
        "apache" | "htaccess" => Some(SyntaxMode::Apache),
        "dockerfile" | "docker" | "containerfile" => Some(SyntaxMode::Dockerfile),
        "systemd" | "service" | "unit" => Some(SyntaxMode::Systemd),
        "sql" => Some(SyntaxMode::Sql),
        "python" | "py" | "python2" | "python3" => Some(SyntaxMode::Python),
        "java" => Some(SyntaxMode::Java),
        "csharp" | "c#" | "cs" | "csx" => Some(SyntaxMode::Csharp),
        "cpp" | "c++" | "cxx" => Some(SyntaxMode::Cpp),
        "c" => Some(SyntaxMode::C),
        "go" | "golang" => Some(SyntaxMode::Go),
        "kotlin" | "kt" => Some(SyntaxMode::Kotlin),
        "swift" => Some(SyntaxMode::Swift),
        "ruby" | "rb" => Some(SyntaxMode::Ruby),
        "dart" => Some(SyntaxMode::Dart),
        "scala" => Some(SyntaxMode::Scala),
        "r" => Some(SyntaxMode::R),
        "lua" => Some(SyntaxMode::Lua),
        "perl" | "pl" => Some(SyntaxMode::Perl),
        "haskell" | "hs" => Some(SyntaxMode::Haskell),
        "elixir" | "ex" => Some(SyntaxMode::Elixir),
        "clojure" | "clj" => Some(SyntaxMode::Clojure),
        "zig" => Some(SyntaxMode::Zig),
        "julia" | "jl" => Some(SyntaxMode::Julia),
        "objc" | "objective-c" | "objectivec" | "mm" => Some(SyntaxMode::Objc),
        "makefile" | "make" | "mk" => Some(SyntaxMode::Makefile),
        "cmake" => Some(SyntaxMode::Cmake),
        "powershell" | "ps1" | "psm1" => Some(SyntaxMode::Powershell),
        "proto" | "protobuf" => Some(SyntaxMode::Proto),
        "graphql" | "gql" => Some(SyntaxMode::Graphql),
        "terraform" | "tf" | "hcl" => Some(SyntaxMode::Terraform),
        "vue" => Some(SyntaxMode::Vue),
        "svelte" => Some(SyntaxMode::Svelte),
        "asm" | "assembly" | "nasm" => Some(SyntaxMode::Asm),
        "tex" | "latex" => Some(SyntaxMode::Tex),
        "erlang" | "erl" => Some(SyntaxMode::Erlang),
        "solidity" | "sol" => Some(SyntaxMode::Solidity),
        "ada" => Some(SyntaxMode::Ada),
        "arduino" | "ino" => Some(SyntaxMode::Arduino),
        "asciidoc" | "adoc" => Some(SyntaxMode::Asciidoc),
        "ats" => Some(SyntaxMode::Ats),
        "awk" => Some(SyntaxMode::Awk),
        "batch" | "bat" | "dosbatch" => Some(SyntaxMode::Bat),
        "b" => Some(SyntaxMode::B),
        "caddyfile" | "caddy" => Some(SyntaxMode::Caddyfile),
        "cake" => Some(SyntaxMode::Cake),
        "coffeescript" | "coffee" => Some(SyntaxMode::Coffeescript),
        "conky" => Some(SyntaxMode::Conky),
        "crontab" | "cron" => Some(SyntaxMode::Crontab),
        "crystal" | "cr" => Some(SyntaxMode::Crystal),
        "cuda" => Some(SyntaxMode::Cuda),
        "cython" | "pyx" => Some(SyntaxMode::Cython),
        "d" | "dlang" => Some(SyntaxMode::D),
        "dot" | "graphviz" => Some(SyntaxMode::Dot),
        "ebuild" | "gentoo" => Some(SyntaxMode::Ebuild),
        "elm" => Some(SyntaxMode::Elm),
        "erb" => Some(SyntaxMode::Erb),
        "portage" | "etc-portage" => Some(SyntaxMode::Etcportage),
        "fish" => Some(SyntaxMode::Fish),
        "forth" => Some(SyntaxMode::Forth),
        "fortran" | "f90" | "f95" => Some(SyntaxMode::Fortran),
        "freebsd" | "bsd" => Some(SyntaxMode::Freebsd),
        "fsharp" | "f#" | "fs" => Some(SyntaxMode::Fsharp),
        "gdscript" | "godot" | "gd" => Some(SyntaxMode::Gdscript),
        "gemini" | "gemtext" => Some(SyntaxMode::Gemini),
        "git-commit" => Some(SyntaxMode::Gitcommit),
        "git-config" | "gitconfig" => Some(SyntaxMode::Gitconfig),
        "git-rebase" | "git-rebase-todo" => Some(SyntaxMode::Gitrebase),
        "gleam" => Some(SyntaxMode::Gleam),
        "glsl" => Some(SyntaxMode::Glsl),
        "gnuplot" => Some(SyntaxMode::Gnuplot),
        "godoc" => Some(SyntaxMode::Godoc),
        "golo" => Some(SyntaxMode::Golo),
        "gomod" => Some(SyntaxMode::Gomod),
        "groff" | "troff" => Some(SyntaxMode::Groff),
        "groovy" | "gradle" => Some(SyntaxMode::Groovy),
        "haml" => Some(SyntaxMode::Haml),
        "hare" => Some(SyntaxMode::Hare),
        "hc" | "holyc" => Some(SyntaxMode::Hc),
        "inputrc" => Some(SyntaxMode::Inputrc),
        "jinja2" | "jinja" => Some(SyntaxMode::Jinja2),
        "jsonnet" => Some(SyntaxMode::Jsonnet),
        "just" | "justfile" => Some(SyntaxMode::Justfile),
        "keymap" | "xmodmap" => Some(SyntaxMode::Keymap),
        "kickstart" | "ks" => Some(SyntaxMode::Kickstart),
        "kvlang" | "kivy" | "kv" => Some(SyntaxMode::Kvlang),
        "ledger" => Some(SyntaxMode::Ledger),
        "lfe" => Some(SyntaxMode::Lfe),
        "lilypond" | "ly" => Some(SyntaxMode::Lilypond),
        "lisp" | "elisp" | "scheme" => Some(SyntaxMode::Lisp),
        "mail" | "email" => Some(SyntaxMode::Mail),
        "man" | "manpage" => Some(SyntaxMode::Man),
        "mc" | "m4" => Some(SyntaxMode::Mc),
        "meson" => Some(SyntaxMode::Meson),
        "micro" => Some(SyntaxMode::Micro),
        "mpd" => Some(SyntaxMode::Mpd),
        "msbuild" => Some(SyntaxMode::Msbuild),
        "nanorc" | "nano" => Some(SyntaxMode::Nanorc),
        "nftables" | "nft" => Some(SyntaxMode::Nftables),
        "nim" => Some(SyntaxMode::Nim),
        "nix" => Some(SyntaxMode::Nix),
        "nu" | "nushell" => Some(SyntaxMode::Nu),
        "ocaml" => Some(SyntaxMode::Ocaml),
        "octave" => Some(SyntaxMode::Octave),
        "odin" => Some(SyntaxMode::Odin),
        "pascal" | "pas" => Some(SyntaxMode::Pascal),
        "patch" | "diff" => Some(SyntaxMode::Patch),
        "pkg-config" | "pkgconfig" | "pkgconf" => Some(SyntaxMode::Pkgconfig),
        "peg" => Some(SyntaxMode::Peg),
        "po" | "gettext" => Some(SyntaxMode::Po),
        "pony" => Some(SyntaxMode::Pony),
        "pov" | "povray" => Some(SyntaxMode::Pov),
        "privoxy" => Some(SyntaxMode::Privoxy),
        "prql" => Some(SyntaxMode::Prql),
        "puppet" => Some(SyntaxMode::Puppet),
        "raku" | "perl6" => Some(SyntaxMode::Raku),
        "renpy" => Some(SyntaxMode::Renpy),
        "rpmspec" | "spec" => Some(SyntaxMode::Rpmspec),
        "rst" | "rest" | "restructuredtext" => Some(SyntaxMode::Rest),
        "sage" => Some(SyntaxMode::Sage),
        "salt" | "saltstack" => Some(SyntaxMode::Salt),
        "sed" => Some(SyntaxMode::Sed),
        "smalltalk" => Some(SyntaxMode::Smalltalk),
        "stata" => Some(SyntaxMode::Stata),
        "tcl" => Some(SyntaxMode::Tcl),
        "twig" => Some(SyntaxMode::Twig),
        "v" | "vlang" => Some(SyntaxMode::V),
        "vala" => Some(SyntaxMode::Vala),
        "verilog" => Some(SyntaxMode::Verilog),
        "vhdl" => Some(SyntaxMode::Vhdl),
        "vi" | "vim" | "vimscript" => Some(SyntaxMode::Vi),
        "xresources" | "xres" => Some(SyntaxMode::Xresources),
        "yum" => Some(SyntaxMode::Yum),
        "zscript" => Some(SyntaxMode::Zscript),
        "openscad" | "scad" => Some(SyntaxMode::Scad),
        "plain" | "text" | "txt" => Some(SyntaxMode::Plain),
        _ => None,
    }
}

pub(crate) fn from_path(path: Option<&Path>) -> SyntaxMode {
    let Some(path) = path else { return SyntaxMode::Plain; };
    let name = path.file_name().and_then(OsStr::to_str).unwrap_or("").to_ascii_lowercase();
    if name.ends_with(".blade.php") { return SyntaxMode::Blade; }
    // Filename-based modes (no useful extension)
    if name == "dockerfile" || name.starts_with("dockerfile.") || name == "containerfile" {
        return SyntaxMode::Dockerfile;
    }
    if name == ".env" || name.starts_with(".env.") || name.ends_with(".env") {
        return SyntaxMode::Dotenv;
    }
    if name == ".htaccess" || name == "httpd.conf" || name == "apache2.conf" {
        return SyntaxMode::Apache;
    }
    if name == "nginx.conf" || (name.starts_with("nginx") && name.ends_with(".conf")) {
        return SyntaxMode::Nginx;
    }
    if name == "gemfile" || name == "rakefile" {
        return SyntaxMode::Ruby;
    }
    if name == "makefile" || name == "gnumakefile" || name.ends_with(".mk") || name.ends_with(".mak") {
        return SyntaxMode::Makefile;
    }
    if name == "cmakelists.txt" || name.ends_with(".cmake") {
        return SyntaxMode::Cmake;
    }
    if name == "caddyfile" {
        return SyntaxMode::Caddyfile;
    }
    if name == "conky.conf" {
        return SyntaxMode::Conky;
    }
    if name.contains("conkyrc") {
        return SyntaxMode::Conky;
    }
    if name == "crontab" {
        return SyntaxMode::Crontab;
    }
    if name.starts_with("crontab.") {
        return SyntaxMode::Crontab;
    }
    if name == "generic" {
        return SyntaxMode::Freebsd;
    }
    if name == "commit_editmsg" {
        return SyntaxMode::Gitcommit;
    }
    if name == "tag_editmsg" {
        return SyntaxMode::Gitcommit;
    }
    if name == "merge_msg" {
        return SyntaxMode::Gitcommit;
    }
    if name == ".gitconfig" {
        return SyntaxMode::Gitconfig;
    }
    if name == "gitconfig" {
        return SyntaxMode::Gitconfig;
    }
    if name == "gitmodules" {
        return SyntaxMode::Gitconfig;
    }
    if name == "git-rebase-todo" {
        return SyntaxMode::Gitrebase;
    }
    if name == "go.mod" {
        return SyntaxMode::Gomod;
    }
    if name.starts_with("tmac.") {
        return SyntaxMode::Groff;
    }
    if name == "jenkinsfile" {
        return SyntaxMode::Groovy;
    }
    if name == "inputrc" {
        return SyntaxMode::Inputrc;
    }
    if name == ".inputrc" {
        return SyntaxMode::Inputrc;
    }
    if name == "justfile" {
        return SyntaxMode::Justfile;
    }
    if name == ".justfile" {
        return SyntaxMode::Justfile;
    }
    if name == "xmodmap" {
        return SyntaxMode::Keymap;
    }
    if name == "ledger" {
        return SyntaxMode::Ledger;
    }
    if name == "ldgr" {
        return SyntaxMode::Ledger;
    }
    if name == "beancount" {
        return SyntaxMode::Ledger;
    }
    if name == "bnct" {
        return SyntaxMode::Ledger;
    }
    if name == "emacs" {
        return SyntaxMode::Lisp;
    }
    if name == "zile" {
        return SyntaxMode::Lisp;
    }
    if name.starts_with("mutt-") {
        return SyntaxMode::Mail;
    }
    if name == "meson.build" {
        return SyntaxMode::Meson;
    }
    if name == "meson_options.txt" {
        return SyntaxMode::Meson;
    }
    if name == "meson.options" {
        return SyntaxMode::Meson;
    }
    if name == "mpd.conf" {
        return SyntaxMode::Mpd;
    }
    if name == "nanorc" {
        return SyntaxMode::Nanorc;
    }
    if name == ".nanorc" {
        return SyntaxMode::Nanorc;
    }
    if name == "nftables.conf" {
        return SyntaxMode::Nftables;
    }
    if name == "nftables.rules" {
        return SyntaxMode::Nftables;
    }
    if name == "nim.cfg" {
        return SyntaxMode::Nim;
    }
    if name == "vimrc" {
        return SyntaxMode::Vi;
    }
    if name == ".vimrc" {
        return SyntaxMode::Vi;
    }
    if name == "exrc" {
        return SyntaxMode::Vi;
    }
    if name == ".exrc" {
        return SyntaxMode::Vi;
    }
    if name == "gvimrc" {
        return SyntaxMode::Vi;
    }
    if name == ".gvimrc" {
        return SyntaxMode::Vi;
    }
    if name == "xdefaults" {
        return SyntaxMode::Xresources;
    }
    if name == "xresources" {
        return SyntaxMode::Xresources;
    }
    if name == "yum.conf" {
        return SyntaxMode::Yum;
    }
    if name == "config" {
        if path.components().any(|c| c.as_os_str() == OsStr::new(".git")) {
            return SyntaxMode::Gitconfig;
        }
        if path.components().any(|c| c.as_os_str() == OsStr::new("privoxy")) {
            return SyntaxMode::Privoxy;
        }
    }
    if name == "pkgfile" {
        return SyntaxMode::Bash;
    }
    if is_shell_rc_name(&name) {
        return SyntaxMode::Bash;
    }
    match path.extension().and_then(OsStr::to_str).unwrap_or("").to_ascii_lowercase().as_str() {
        "php" | "phtml" => SyntaxMode::Php,
        "html" | "htm" => SyntaxMode::Html,
        "xml" | "svg" | "plist" | "xsl" => SyntaxMode::Xml,
        "css" => SyntaxMode::Css,
        "js" | "mjs" | "cjs" | "jsx" => SyntaxMode::JavaScript,
        "ts" | "tsx" | "mts" | "cts" => SyntaxMode::TypeScript,
        "md" | "mkd" | "markdown" => SyntaxMode::Markdown,
        "json" | "jsonc" | "json5" => SyntaxMode::Json,
        "toml" => SyntaxMode::Toml,
        "yaml" | "yml" => SyntaxMode::Yaml,
        "sh" | "bash" | "zsh" => SyntaxMode::Bash,
        "env" => SyntaxMode::Dotenv,
        "ini" | "conf" | "cfg" => SyntaxMode::Ini,
        "log" => SyntaxMode::Log,
        "rs" => SyntaxMode::Rust,
        "sql" => SyntaxMode::Sql,
        "py" | "pyw" | "pyi" => SyntaxMode::Python,
        "java" => SyntaxMode::Java,
        "cs" | "csx" => SyntaxMode::Csharp,
        "cpp" | "cxx" | "cc" | "hpp" | "hh" | "hxx" => SyntaxMode::Cpp,
        "c" | "h" => SyntaxMode::C,
        "go" => SyntaxMode::Go,
        "kt" | "kts" => SyntaxMode::Kotlin,
        "swift" => SyntaxMode::Swift,
        "rb" | "gemspec" => SyntaxMode::Ruby,
        "dart" => SyntaxMode::Dart,
        "scala" | "sc" => SyntaxMode::Scala,
        "r" => SyntaxMode::R,
        "lua" => SyntaxMode::Lua,
        "pl" | "pm" | "t" => SyntaxMode::Perl,
        "hs" | "lhs" => SyntaxMode::Haskell,
        "ex" | "exs" => SyntaxMode::Elixir,
        "clj" | "cljs" | "cljc" | "edn" => SyntaxMode::Clojure,
        "zig" => SyntaxMode::Zig,
        "jl" => SyntaxMode::Julia,
        "m" | "mm" => SyntaxMode::Objc,
        "service" | "timer" | "socket" | "unit" => SyntaxMode::Systemd,
        "mk" | "mak" => SyntaxMode::Makefile,
        "cmake" => SyntaxMode::Cmake,
        "ps1" | "psm1" | "psd1" => SyntaxMode::Powershell,
        "proto" => SyntaxMode::Proto,
        "graphql" | "gql" => SyntaxMode::Graphql,
        "tf" | "hcl" => SyntaxMode::Terraform,
        "vue" => SyntaxMode::Vue,
        "svelte" => SyntaxMode::Svelte,
        "s" | "asm" => SyntaxMode::Asm,
        "tex" | "bib" | "cls" | "sty" => SyntaxMode::Tex,
        "erl" | "hrl" => SyntaxMode::Erlang,
        "sol" => SyntaxMode::Solidity,
        "ads" | "adb" | "ada" => SyntaxMode::Ada,
        "ino" => SyntaxMode::Arduino,
        "asc" | "asciidoc" | "adoc" => SyntaxMode::Asciidoc,
        "dats" | "hats" | "sats" => SyntaxMode::Ats,
        "awk" => SyntaxMode::Awk,
        "bat" | "cmd" => SyntaxMode::Bat,
        "b" => SyntaxMode::B,
        "cake" => SyntaxMode::Cake,
        "coffee" => SyntaxMode::Coffeescript,
        "cr" => SyntaxMode::Crystal,
        "cu" | "cuh" => SyntaxMode::Cuda,
        "pyx" | "pxd" => SyntaxMode::Cython,
        "d" | "di" | "dd" => SyntaxMode::D,
        "dot" | "gv" => SyntaxMode::Dot,
        "ebuild" | "eclass" => SyntaxMode::Ebuild,
        "elm" => SyntaxMode::Elm,
        "erb" | "rhtml" => SyntaxMode::Erb,
        "keywords" | "mask" | "unmask" | "use" => SyntaxMode::Etcportage,
        "fish" => SyntaxMode::Fish,
        "forth" | "4th" | "fs8" | "ft" | "fth" | "frt" => SyntaxMode::Forth,
        "f" | "f90" | "f95" | "for" => SyntaxMode::Fortran,
        "fs" | "fsi" | "fsx" => SyntaxMode::Fsharp,
        "gd" => SyntaxMode::Gdscript,
        "gmi" | "gemini" => SyntaxMode::Gemini,
        "gleam" => SyntaxMode::Gleam,
        "frag" | "vert" | "fp" | "vp" | "glsl" => SyntaxMode::Glsl,
        "gnu" | "gpi" | "plt" | "gp" => SyntaxMode::Gnuplot,
        "godoc" => SyntaxMode::Godoc,
        "golo" => SyntaxMode::Golo,
        "me" | "ms" | "rof" | "tmac" => SyntaxMode::Groff,
        "groovy" | "gy" | "gvy" | "gsh" | "gradle" => SyntaxMode::Groovy,
        "haml" => SyntaxMode::Haml,
        "ha" => SyntaxMode::Hare,
        "hc" => SyntaxMode::Hc,
        "j2" | "jinja" | "jinja2" => SyntaxMode::Jinja2,
        "jsonnet" | "libsonnet" => SyntaxMode::Jsonnet,
        "just" => SyntaxMode::Justfile,
        "map" | "kmap" | "keymap" => SyntaxMode::Keymap,
        "ks" | "kickstart" => SyntaxMode::Kickstart,
        "kv" => SyntaxMode::Kvlang,
        "ledger" | "ldgr" | "beancount" | "bnct" => SyntaxMode::Ledger,
        "lfe" => SyntaxMode::Lfe,
        "ly" | "ily" | "lly" => SyntaxMode::Lilypond,
        "el" | "lisp" | "lsp" | "scm" | "ss" | "rkt" => SyntaxMode::Lisp,
        "eml" => SyntaxMode::Mail,
        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => SyntaxMode::Man,
        "mc" => SyntaxMode::Mc,
        "micro" => SyntaxMode::Micro,
        "props" | "targets" | "tasks" => SyntaxMode::Msbuild,
        "nim" | "nims" => SyntaxMode::Nim,
        "nix" => SyntaxMode::Nix,
        "nu" => SyntaxMode::Nu,
        "ml" | "mli" => SyntaxMode::Ocaml,
        "odin" => SyntaxMode::Odin,
        "pas" => SyntaxMode::Pascal,
        "patch" | "diff" => SyntaxMode::Patch,
        "pc" => SyntaxMode::Pkgconfig,
        "peg" | "lpeg" => SyntaxMode::Peg,
        "po" | "pot" => SyntaxMode::Po,
        "pony" => SyntaxMode::Pony,
        "pov" | "povray" => SyntaxMode::Pov,
        "action" | "filter" => SyntaxMode::Privoxy,
        "prql" => SyntaxMode::Prql,
        "pp" => SyntaxMode::Puppet,
        "p6" | "pl6" | "pm6" | "pod6" | "raku" | "rakumod" | "rakudoc" | "rakutest" | "nqp" => SyntaxMode::Raku,
        "rpy" => SyntaxMode::Renpy,
        "spec" | "rpmspec" => SyntaxMode::Rpmspec,
        "rest" | "rst" => SyntaxMode::Rest,
        "sage" => SyntaxMode::Sage,
        "sls" => SyntaxMode::Salt,
        "sed" => SyntaxMode::Sed,
        "st" | "sources" | "changes" => SyntaxMode::Smalltalk,
        "do" | "ado" => SyntaxMode::Stata,
        "tcl" => SyntaxMode::Tcl,
        "twig" => SyntaxMode::Twig,
        "vala" => SyntaxMode::Vala,
        "v" | "vh" | "sv" | "svh" => SyntaxMode::Verilog,
        "vhdl" | "vhd" => SyntaxMode::Vhdl,
        "vim" => SyntaxMode::Vi,
        "repo" => SyntaxMode::Yum,
        "zc" | "zsc" => SyntaxMode::Zscript,
        "scad" => SyntaxMode::Scad,
        _ => SyntaxMode::Plain,
    }
}

pub(crate) fn is_programming_mode(mode: SyntaxMode) -> bool {
    !matches!(mode, SyntaxMode::Plain | SyntaxMode::Log)
}

/// Extensionless shell startup files (`~/.bashrc` and friends).
fn is_shell_rc_name(name: &str) -> bool {
    matches!(
        name,
        ".bashrc"
            | ".bash_profile"
            | ".bash_login"
            | ".bash_logout"
            | ".bash_aliases"
            | ".profile"
            | ".zshrc"
            | ".zprofile"
            | ".zshenv"
            | ".zlogin"
            | ".zlogout"
            | ".kshrc"
            | ".mkshrc"
            | "bashrc"
            | "bash_profile"
            | "profile"
    )
}

/// `#!/bin/bash`, `#!/usr/bin/env sh`, and the other common shells.
/// Used when the filename has no extension, which is why `~/.bashrc`
/// used to stay Plain even though `bash.rs` was already loaded.
pub(crate) fn syntax_from_shebang(line: &str) -> Option<SyntaxMode> {
    let rest = line.trim_start().strip_prefix("#!")?;
    let mut parts = rest.split_whitespace();
    let token = parts.next().unwrap_or("").to_ascii_lowercase();
    let base = token.rsplit('/').next().unwrap_or("");
    if base == "env" {
        return shell_syntax(parts.next().unwrap_or(""));
    }
    shell_syntax(base)
}

fn shell_syntax(name: &str) -> Option<SyntaxMode> {
    match name.to_ascii_lowercase().as_str() {
        "sh" | "bash" | "zsh" | "ksh" | "dash" | "ash" | "mksh" => Some(SyntaxMode::Bash),
        _ => None,
    }
}

pub(crate) fn tree_color(path: &Path, is_dir: bool) -> &'static str {
    if is_dir { return BLUE; }
    let name = path.file_name().and_then(OsStr::to_str).unwrap_or("").to_ascii_lowercase();
    if name.ends_with(".blade.php") { return MAGENTA; }
    if name == "dockerfile" || name.starts_with("dockerfile.") {
        return BLUE;
    }
    if name == ".env" || name.starts_with(".env.") {
        return YELLOW;
    }
    if name == ".htaccess" || name == "nginx.conf" {
        return PURPLE;
    }
    if is_shell_rc_name(&name) {
        return RED;
    }
    if name == "makefile" || name == "gnumakefile" || name == "cmakelists.txt" {
        return BLUE;
    }
    if name == "caddyfile" {
        return BLUE;
    }
    if name == "conky.conf" {
        return GREEN;
    }
    if name == "crontab" {
        return FG_DARK;
    }
    if name == "generic" {
        return FG_DARK;
    }
    if name == "commit_editmsg" || name == "tag_editmsg" || name == "merge_msg" {
        return YELLOW;
    }
    if name == ".gitconfig" || name == "gitconfig" || name == "gitmodules" {
        return CYAN;
    }
    if name == "git-rebase-todo" {
        return YELLOW;
    }
    if name == "go.mod" {
        return CYAN;
    }
    if name == "inputrc" || name == ".inputrc" {
        return FG_DARK;
    }
    if name == "meson.build" || name == "meson_options.txt" || name == "meson.options" {
        return BLUE;
    }
    if name == "mpd.conf" {
        return FG_DARK;
    }
    if name == "nanorc" || name == ".nanorc" {
        return FG_DARK;
    }
    if name == "nftables.conf" || name == "nftables.rules" {
        return RED;
    }
    if name == "xdefaults" || name == "xresources" {
        return FG_DARK;
    }
    match path.extension().and_then(OsStr::to_str).unwrap_or("").to_ascii_lowercase().as_str() {
        "php" | "phtml" => PURPLE,
        "html" | "htm" | "xml" | "svg" => ORANGE,
        "css" | "scss" | "sass" | "less" => BLUE,
        "js" | "mjs" | "cjs" | "jsx" => YELLOW,
        "ts" | "tsx" | "mts" | "cts" => CYAN,
        "rs" => ORANGE,
        "py" | "pyw" | "pyi" => GREEN,
        "java" => RED,
        "cs" | "csx" => CYAN,
        "cpp" | "cxx" | "cc" | "hpp" | "hh" | "hxx" | "c" | "h" => BLUE,
        "go" => CYAN,
        "kt" | "kts" => ORANGE,
        "swift" => RED,
        "rb" => RED,
        "dart" => CYAN,
        "scala" | "sc" => RED,
        "r" => BLUE,
        "lua" => PURPLE,
        "pl" | "pm" | "t" => YELLOW,
        "hs" | "lhs" => PURPLE,
        "ex" | "exs" => MAGENTA,
        "clj" | "cljs" | "cljc" | "edn" => GREEN,
        "zig" => ORANGE,
        "jl" => MAGENTA,
        "m" | "mm" => ORANGE,
        "vue" | "svelte" => GREEN,
        "ps1" | "psm1" | "psd1" => CYAN,
        "proto" | "graphql" | "gql" => ORANGE,
        "tf" | "hcl" => PURPLE,
        "s" | "asm" => FG_DARK,
        "tex" | "bib" | "cls" | "sty" => GREEN,
        "erl" | "hrl" => RED,
        "sol" => CYAN,
        "ads" | "adb" | "ada" => PURPLE,
        "ino" => CYAN,
        "asc" | "asciidoc" | "adoc" => GREEN,
        "dats" | "hats" | "sats" => CYAN,
        "awk" => ORANGE,
        "bat" | "cmd" => YELLOW,
        "b" => FG_DARK,
        "cake" => RED,
        "coffee" => YELLOW,
        "cr" => RED,
        "cu" | "cuh" => GREEN,
        "pyx" | "pxd" => GREEN,
        "d" | "di" | "dd" => RED,
        "dot" | "gv" => CYAN,
        "ebuild" | "eclass" => PURPLE,
        "elm" => BLUE,
        "erb" | "rhtml" => RED,
        "keywords" | "mask" | "unmask" | "use" => FG_DARK,
        "fish" => GREEN,
        "forth" | "4th" | "fs8" | "ft" | "fth" | "frt" => ORANGE,
        "f" | "f90" | "f95" | "for" => BLUE,
        "fs" | "fsi" | "fsx" => CYAN,
        "gd" => CYAN,
        "gmi" | "gemini" => FG_DARK,
        "gleam" => MAGENTA,
        "frag" | "vert" | "fp" | "vp" | "glsl" => PURPLE,
        "gnu" | "gpi" | "plt" | "gp" => ORANGE,
        "godoc" => FG_DARK,
        "golo" => ORANGE,
        "me" | "ms" | "rof" | "tmac" => FG_DARK,
        "groovy" | "gy" | "gvy" | "gsh" | "gradle" => GREEN,
        "haml" => ORANGE,
        "ha" => ORANGE,
        "hc" => YELLOW,
        "j2" | "jinja" | "jinja2" => GREEN,
        "jsonnet" | "libsonnet" => CYAN,
        "just" => BLUE,
        "map" | "kmap" | "keymap" => FG_DARK,
        "ks" | "kickstart" => RED,
        "kv" => ORANGE,
        "ledger" | "ldgr" | "beancount" | "bnct" => GREEN,
        "lfe" => GREEN,
        "ly" | "ily" | "lly" => PURPLE,
        "el" | "lisp" | "lsp" | "scm" | "ss" | "rkt" => GREEN,
        "eml" => FG_DARK,
        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => FG_DARK,
        "mc" => ORANGE,
        "micro" => CYAN,
        "props" | "targets" | "tasks" => BLUE,
        "nim" | "nims" => YELLOW,
        "nix" => CYAN,
        "nu" => GREEN,
        "ml" | "mli" => ORANGE,
        "odin" => CYAN,
        "pas" => BLUE,
        "patch" | "diff" => FG_DARK,
        "pc" => CYAN,
        "peg" | "lpeg" => PURPLE,
        "po" | "pot" => YELLOW,
        "pony" => ORANGE,
        "pov" | "povray" => CYAN,
        "action" | "filter" => FG_DARK,
        "prql" => ORANGE,
        "pp" => PURPLE,
        "p6" | "pl6" | "pm6" | "pod6" | "raku" | "rakumod" | "rakudoc" | "rakutest" | "nqp" => PURPLE,
        "rpy" => MAGENTA,
        "spec" | "rpmspec" => RED,
        "rest" | "rst" => FG_DARK,
        "sage" => GREEN,
        "sls" => YELLOW,
        "sed" => ORANGE,
        "st" | "sources" | "changes" => YELLOW,
        "do" | "ado" => BLUE,
        "tcl" => CYAN,
        "twig" => GREEN,
        "vala" => PURPLE,
        "v" | "vh" | "sv" | "svh" => GREEN,
        "vhdl" | "vhd" => GREEN,
        "vim" => GREEN,
        "repo" => RED,
        "zc" | "zsc" => ORANGE,
        "scad" => YELLOW,
        "mk" | "mak" | "cmake" => BLUE,
        "md" | "mkd" | "markdown" | "txt" => GREEN,
        "json" | "jsonc" | "json5" | "toml" | "yaml" | "yml" => CYAN,
        "sh" | "bash" | "zsh" => RED,
        "env" => YELLOW,
        "ini" | "conf" | "cfg" => BLUE,
        "log" => FG_DARK,
        "sql" => ORANGE,
        "service" | "timer" | "socket" | "unit" => CYAN,
        _ => FG_DARK,
    }
}

pub(crate) fn highlight_segments(line: &str, syntax: SyntaxMode) -> Vec<Segment> {
    match syntax {
        SyntaxMode::Php => mixed_segments(line, true, false, false),
        SyntaxMode::Blade => mixed_segments(line, true, true, true),
        SyntaxMode::Html => mixed_segments(line, line_contains_php(line), false, true),
        SyntaxMode::Css => css::segments(line),
        SyntaxMode::JavaScript => javascript::segments(line),
        SyntaxMode::TypeScript => typescript::segments(line),
        SyntaxMode::Xml => xml::segments(line),
        SyntaxMode::Markdown => markdown::segments(line),
        SyntaxMode::Json => json::segments(line),
        SyntaxMode::Toml => toml::segments(line),
        SyntaxMode::Yaml => yaml::segments(line),
        SyntaxMode::Bash => bash::segments(line),
        SyntaxMode::Dotenv => dotenv::segments(line),
        SyntaxMode::Ini => ini::segments(line),
        SyntaxMode::Log => log::segments(line),
        SyntaxMode::Rust => rust::segments(line),
        SyntaxMode::Nginx => nginx::segments(line),
        SyntaxMode::Apache => apache::segments(line),
        SyntaxMode::Dockerfile => dockerfile::segments(line),
        SyntaxMode::Systemd => systemd::segments(line),
        SyntaxMode::Sql => sql::segments(line),
        SyntaxMode::Python => python::segments(line),
        SyntaxMode::Java => java::segments(line),
        SyntaxMode::Csharp => csharp::segments(line),
        SyntaxMode::Cpp => cpp::segments(line),
        SyntaxMode::C => c::segments(line),
        SyntaxMode::Go => go::segments(line),
        SyntaxMode::Kotlin => kotlin::segments(line),
        SyntaxMode::Swift => swift::segments(line),
        SyntaxMode::Ruby => ruby::segments(line),
        SyntaxMode::Dart => dart::segments(line),
        SyntaxMode::Scala => scala::segments(line),
        SyntaxMode::R => r::segments(line),
        SyntaxMode::Lua => lua::segments(line),
        SyntaxMode::Perl => perl::segments(line),
        SyntaxMode::Haskell => haskell::segments(line),
        SyntaxMode::Elixir => elixir::segments(line),
        SyntaxMode::Clojure => clojure::segments(line),
        SyntaxMode::Zig => zig::segments(line),
        SyntaxMode::Julia => julia::segments(line),
        SyntaxMode::Objc => objc::segments(line),
        SyntaxMode::Makefile => makefile::segments(line),
        SyntaxMode::Cmake => cmake::segments(line),
        SyntaxMode::Powershell => powershell::segments(line),
        SyntaxMode::Proto => proto::segments(line),
        SyntaxMode::Graphql => graphql::segments(line),
        SyntaxMode::Terraform => terraform::segments(line),
        SyntaxMode::Vue => vue::segments(line),
        SyntaxMode::Svelte => svelte::segments(line),
        SyntaxMode::Asm => asm::segments(line),
        SyntaxMode::Tex => tex::segments(line),
        SyntaxMode::Erlang => erlang::segments(line),
        SyntaxMode::Solidity => solidity::segments(line),
        SyntaxMode::Ada => ada::segments(line),
        SyntaxMode::Arduino => arduino::segments(line),
        SyntaxMode::Asciidoc => asciidoc::segments(line),
        SyntaxMode::Ats => ats::segments(line),
        SyntaxMode::Awk => awk::segments(line),
        SyntaxMode::Bat => bat::segments(line),
        SyntaxMode::B => b::segments(line),
        SyntaxMode::Caddyfile => caddyfile::segments(line),
        SyntaxMode::Cake => cake::segments(line),
        SyntaxMode::Coffeescript => coffeescript::segments(line),
        SyntaxMode::Conky => conky::segments(line),
        SyntaxMode::Crontab => crontab::segments(line),
        SyntaxMode::Crystal => crystal::segments(line),
        SyntaxMode::Cuda => cuda::segments(line),
        SyntaxMode::Cython => cython::segments(line),
        SyntaxMode::D => d::segments(line),
        SyntaxMode::Dot => dot::segments(line),
        SyntaxMode::Ebuild => ebuild::segments(line),
        SyntaxMode::Elm => elm::segments(line),
        SyntaxMode::Erb => erb::segments(line),
        SyntaxMode::Etcportage => etcportage::segments(line),
        SyntaxMode::Fish => fish::segments(line),
        SyntaxMode::Forth => forth::segments(line),
        SyntaxMode::Fortran => fortran::segments(line),
        SyntaxMode::Freebsd => freebsd::segments(line),
        SyntaxMode::Fsharp => fsharp::segments(line),
        SyntaxMode::Gdscript => gdscript::segments(line),
        SyntaxMode::Gemini => gemini::segments(line),
        SyntaxMode::Gitcommit => gitcommit::segments(line),
        SyntaxMode::Gitconfig => gitconfig::segments(line),
        SyntaxMode::Gitrebase => gitrebase::segments(line),
        SyntaxMode::Gleam => gleam::segments(line),
        SyntaxMode::Glsl => glsl::segments(line),
        SyntaxMode::Gnuplot => gnuplot::segments(line),
        SyntaxMode::Godoc => godoc::segments(line),
        SyntaxMode::Golo => golo::segments(line),
        SyntaxMode::Gomod => gomod::segments(line),
        SyntaxMode::Groff => groff::segments(line),
        SyntaxMode::Groovy => groovy::segments(line),
        SyntaxMode::Haml => haml::segments(line),
        SyntaxMode::Hare => hare::segments(line),
        SyntaxMode::Hc => hc::segments(line),
        SyntaxMode::Inputrc => inputrc::segments(line),
        SyntaxMode::Jinja2 => jinja2::segments(line),
        SyntaxMode::Jsonnet => jsonnet::segments(line),
        SyntaxMode::Justfile => justfile::segments(line),
        SyntaxMode::Keymap => keymap::segments(line),
        SyntaxMode::Kickstart => kickstart::segments(line),
        SyntaxMode::Kvlang => kvlang::segments(line),
        SyntaxMode::Ledger => ledger::segments(line),
        SyntaxMode::Lfe => lfe::segments(line),
        SyntaxMode::Lilypond => lilypond::segments(line),
        SyntaxMode::Lisp => lisp::segments(line),
        SyntaxMode::Mail => mail::segments(line),
        SyntaxMode::Man => man::segments(line),
        SyntaxMode::Mc => mc::segments(line),
        SyntaxMode::Meson => meson::segments(line),
        SyntaxMode::Micro => micro::segments(line),
        SyntaxMode::Mpd => mpd::segments(line),
        SyntaxMode::Msbuild => msbuild::segments(line),
        SyntaxMode::Nanorc => nanorc::segments(line),
        SyntaxMode::Nftables => nftables::segments(line),
        SyntaxMode::Nim => nim::segments(line),
        SyntaxMode::Nix => nix::segments(line),
        SyntaxMode::Nu => nu::segments(line),
        SyntaxMode::Ocaml => ocaml::segments(line),
        SyntaxMode::Octave => octave::segments(line),
        SyntaxMode::Odin => odin::segments(line),
        SyntaxMode::Pascal => pascal::segments(line),
        SyntaxMode::Patch => patch::segments(line),
        SyntaxMode::Pkgconfig => pkgconfig::segments(line),
        SyntaxMode::Peg => peg::segments(line),
        SyntaxMode::Po => po::segments(line),
        SyntaxMode::Pony => pony::segments(line),
        SyntaxMode::Pov => pov::segments(line),
        SyntaxMode::Privoxy => privoxy::segments(line),
        SyntaxMode::Prql => prql::segments(line),
        SyntaxMode::Puppet => puppet::segments(line),
        SyntaxMode::Raku => raku::segments(line),
        SyntaxMode::Renpy => renpy::segments(line),
        SyntaxMode::Rpmspec => rpmspec::segments(line),
        SyntaxMode::Rest => rest::segments(line),
        SyntaxMode::Sage => sage::segments(line),
        SyntaxMode::Salt => salt::segments(line),
        SyntaxMode::Sed => sed::segments(line),
        SyntaxMode::Smalltalk => smalltalk::segments(line),
        SyntaxMode::Stata => stata::segments(line),
        SyntaxMode::Tcl => tcl::segments(line),
        SyntaxMode::Twig => twig::segments(line),
        SyntaxMode::V => v::segments(line),
        SyntaxMode::Vala => vala::segments(line),
        SyntaxMode::Verilog => verilog::segments(line),
        SyntaxMode::Vhdl => vhdl::segments(line),
        SyntaxMode::Vi => vi::segments(line),
        SyntaxMode::Xresources => xresources::segments(line),
        SyntaxMode::Yum => yum::segments(line),
        SyntaxMode::Zscript => zscript::segments(line),
        SyntaxMode::Scad => scad::segments(line),
        SyntaxMode::Plain => plain::segments(line),
    }
}

fn mixed_segments(line: &str, php_is_primary: bool, include_blade: bool, html_is_primary: bool) -> Vec<Segment> {
    let mut out = Vec::new();

    if php_is_primary || html_is_primary || line.contains('<') || line.contains('>') {
        out.extend(html::segments(line));
    }

    if css::looks_like_line(line) || html::has_inline_style(line) {
        out.extend(css::segments(line));
    }

    if javascript::looks_like_line(line) || line.to_ascii_lowercase().contains("<script") {
        out.extend(javascript::segments(line));
    }

    if php_is_primary || line_contains_php(line) {
        out.extend(php::segments(line));
    }

    if include_blade {
        out.extend(blade::segments(line));
    }

    out
}

pub(crate) fn completion_context(syntax: SyntaxMode, before: &str, explicit: bool) -> Option<(String, String, usize)> {
    if syntax == SyntaxMode::Plain { return None; }

    if syntax == SyntaxMode::Blade {
        if let Some(ctx) = blade::completion_context(before, explicit) { return Some(ctx); }
    }

    if matches!(syntax, SyntaxMode::Html | SyntaxMode::Blade | SyntaxMode::Php) || before.contains('<') {
        if let Some(ctx) = html::completion_context(before, explicit) { return Some(ctx); }
    }

    if syntax == SyntaxMode::Css || css::looks_like_context(before) || html::has_open_inline_style(before) {
        if let Some(ctx) = css::completion_context(before, explicit) { return Some(ctx); }
    }

    if syntax == SyntaxMode::JavaScript || javascript::looks_like_context(before) {
        if let Some(ctx) = javascript::completion_context(before, explicit) { return Some(ctx); }
    }

    if syntax == SyntaxMode::TypeScript {
        if let Some(ctx) = typescript::completion_context(before, explicit) { return Some(ctx); }
    }

    if syntax == SyntaxMode::Bash || before.starts_with("#!") {
        if let Some(ctx) = bash::completion_context(before, explicit) { return Some(ctx); }
    }

    if syntax == SyntaxMode::Sql {
        if let Some(ctx) = sql::completion_context(before, explicit) { return Some(ctx); }
    }

    if syntax == SyntaxMode::Nginx {
        if let Some(ctx) = nginx::completion_context(before, explicit) { return Some(ctx); }
    }

    if syntax == SyntaxMode::Apache {
        if let Some(ctx) = apache::completion_context(before, explicit) { return Some(ctx); }
    }

    if syntax == SyntaxMode::Dockerfile {
        if let Some(ctx) = dockerfile::completion_context(before, explicit) { return Some(ctx); }
    }

    if syntax == SyntaxMode::Rust {
        if let Some(ctx) = rust::completion_context(before, explicit) { return Some(ctx); }
    }

    macro_rules! word_mode {
        ($mode:ident, $plug:ident) => {
            if syntax == SyntaxMode::$mode {
                if let Some(ctx) = $plug::completion_context(before, explicit) { return Some(ctx); }
            }
        };
    }
    word_mode!(Python, python);
    word_mode!(Java, java);
    word_mode!(Csharp, csharp);
    word_mode!(Cpp, cpp);
    word_mode!(C, c);
    word_mode!(Go, go);
    word_mode!(Kotlin, kotlin);
    word_mode!(Swift, swift);
    word_mode!(Ruby, ruby);
    word_mode!(Dart, dart);
    word_mode!(Scala, scala);
    word_mode!(R, r);
    word_mode!(Lua, lua);
    word_mode!(Perl, perl);
    word_mode!(Haskell, haskell);
    word_mode!(Elixir, elixir);
    word_mode!(Clojure, clojure);
    word_mode!(Zig, zig);
    word_mode!(Julia, julia);
    word_mode!(Objc, objc);
    word_mode!(Makefile, makefile);
    word_mode!(Cmake, cmake);
    word_mode!(Powershell, powershell);
    word_mode!(Proto, proto);
    word_mode!(Graphql, graphql);
    word_mode!(Terraform, terraform);
    word_mode!(Vue, vue);
    word_mode!(Svelte, svelte);
    word_mode!(Asm, asm);
    word_mode!(Tex, tex);
    word_mode!(Erlang, erlang);
    word_mode!(Solidity, solidity);
    word_mode!(Ada, ada);
    word_mode!(Arduino, arduino);
    word_mode!(Asciidoc, asciidoc);
    word_mode!(Ats, ats);
    word_mode!(Awk, awk);
    word_mode!(Bat, bat);
    word_mode!(B, b);
    word_mode!(Caddyfile, caddyfile);
    word_mode!(Cake, cake);
    word_mode!(Coffeescript, coffeescript);
    word_mode!(Conky, conky);
    word_mode!(Crontab, crontab);
    word_mode!(Crystal, crystal);
    word_mode!(Cuda, cuda);
    word_mode!(Cython, cython);
    word_mode!(D, d);
    word_mode!(Dot, dot);
    word_mode!(Ebuild, ebuild);
    word_mode!(Elm, elm);
    word_mode!(Erb, erb);
    word_mode!(Etcportage, etcportage);
    word_mode!(Fish, fish);
    word_mode!(Forth, forth);
    word_mode!(Fortran, fortran);
    word_mode!(Freebsd, freebsd);
    word_mode!(Fsharp, fsharp);
    word_mode!(Gdscript, gdscript);
    word_mode!(Gemini, gemini);
    word_mode!(Gitcommit, gitcommit);
    word_mode!(Gitconfig, gitconfig);
    word_mode!(Gitrebase, gitrebase);
    word_mode!(Gleam, gleam);
    word_mode!(Glsl, glsl);
    word_mode!(Gnuplot, gnuplot);
    word_mode!(Godoc, godoc);
    word_mode!(Golo, golo);
    word_mode!(Gomod, gomod);
    word_mode!(Groff, groff);
    word_mode!(Groovy, groovy);
    word_mode!(Haml, haml);
    word_mode!(Hare, hare);
    word_mode!(Hc, hc);
    word_mode!(Inputrc, inputrc);
    word_mode!(Jinja2, jinja2);
    word_mode!(Jsonnet, jsonnet);
    word_mode!(Justfile, justfile);
    word_mode!(Keymap, keymap);
    word_mode!(Kickstart, kickstart);
    word_mode!(Kvlang, kvlang);
    word_mode!(Ledger, ledger);
    word_mode!(Lfe, lfe);
    word_mode!(Lilypond, lilypond);
    word_mode!(Lisp, lisp);
    word_mode!(Mail, mail);
    word_mode!(Man, man);
    word_mode!(Mc, mc);
    word_mode!(Meson, meson);
    word_mode!(Micro, micro);
    word_mode!(Mpd, mpd);
    word_mode!(Msbuild, msbuild);
    word_mode!(Nanorc, nanorc);
    word_mode!(Nftables, nftables);
    word_mode!(Nim, nim);
    word_mode!(Nix, nix);
    word_mode!(Nu, nu);
    word_mode!(Ocaml, ocaml);
    word_mode!(Octave, octave);
    word_mode!(Odin, odin);
    word_mode!(Pascal, pascal);
    word_mode!(Patch, patch);
    word_mode!(Pkgconfig, pkgconfig);
    word_mode!(Peg, peg);
    word_mode!(Po, po);
    word_mode!(Pony, pony);
    word_mode!(Pov, pov);
    word_mode!(Privoxy, privoxy);
    word_mode!(Prql, prql);
    word_mode!(Puppet, puppet);
    word_mode!(Raku, raku);
    word_mode!(Renpy, renpy);
    word_mode!(Rpmspec, rpmspec);
    word_mode!(Rest, rest);
    word_mode!(Sage, sage);
    word_mode!(Salt, salt);
    word_mode!(Sed, sed);
    word_mode!(Smalltalk, smalltalk);
    word_mode!(Stata, stata);
    word_mode!(Tcl, tcl);
    word_mode!(Twig, twig);
    word_mode!(V, v);
    word_mode!(Vala, vala);
    word_mode!(Verilog, verilog);
    word_mode!(Vhdl, vhdl);
    word_mode!(Vi, vi);
    word_mode!(Xresources, xresources);
    word_mode!(Yum, yum);
    word_mode!(Zscript, zscript);
    word_mode!(Scad, scad);

    if syntax == SyntaxMode::Xml {
        if let Some(ctx) = xml::completion_context(before, explicit) { return Some(ctx); }
    }

    // Config/log/markdown modes intentionally offer no completion yet,
    // but consult them so new plugins stay wired as the API grows.
    match syntax {
        SyntaxMode::Markdown => { let _ = markdown::completion_context(before, explicit); }
        SyntaxMode::Json => { let _ = json::completion_context(before, explicit); }
        SyntaxMode::Toml => { let _ = toml::completion_context(before, explicit); }
        SyntaxMode::Yaml => { let _ = yaml::completion_context(before, explicit); }
        SyntaxMode::Dotenv => { let _ = dotenv::completion_context(before, explicit); }
        SyntaxMode::Ini => { let _ = ini::completion_context(before, explicit); }
        SyntaxMode::Log => { let _ = log::completion_context(before, explicit); }
        SyntaxMode::Systemd => { let _ = systemd::completion_context(before, explicit); }
        _ => {}
    }

    if matches!(syntax, SyntaxMode::Php | SyntaxMode::Blade) || line_contains_php(before) || before.contains('$') {
        if let Some(ctx) = php::completion_context(before, explicit) { return Some(ctx); }
    }

    None
}

pub(crate) fn completion_items(kind: &str, prefix: &str, ctx: CompletionContext<'_>) -> Vec<CompletionItem> {
    let mut items = match kind.split_once(':').map(|(plugin, _)| plugin).unwrap_or(kind) {
        "blade" => blade::completion_items(kind, ctx),
        "html" => html::completion_items(kind, ctx),
        "xml" => xml::completion_items(kind, ctx),
        "css" => css::completion_items(kind, ctx),
        "javascript" => javascript::completion_items(kind, ctx),
        "typescript" => typescript::completion_items(kind, ctx),
        "php" => php::completion_items(kind, ctx),
        "bash" => bash::completion_items(kind, ctx),
        "sql" => sql::completion_items(kind, ctx),
        "nginx" => nginx::completion_items(kind, ctx),
        "apache" => apache::completion_items(kind, ctx),
        "dockerfile" => dockerfile::completion_items(kind, ctx),
        "rust" => rust::completion_items(kind, ctx),
        "python" => python::completion_items(kind, ctx),
        "java" => java::completion_items(kind, ctx),
        "csharp" => csharp::completion_items(kind, ctx),
        "cpp" => cpp::completion_items(kind, ctx),
        "c" => c::completion_items(kind, ctx),
        "go" => go::completion_items(kind, ctx),
        "kotlin" => kotlin::completion_items(kind, ctx),
        "swift" => swift::completion_items(kind, ctx),
        "ruby" => ruby::completion_items(kind, ctx),
        "dart" => dart::completion_items(kind, ctx),
        "scala" => scala::completion_items(kind, ctx),
        "r" => r::completion_items(kind, ctx),
        "lua" => lua::completion_items(kind, ctx),
        "perl" => perl::completion_items(kind, ctx),
        "haskell" => haskell::completion_items(kind, ctx),
        "elixir" => elixir::completion_items(kind, ctx),
        "clojure" => clojure::completion_items(kind, ctx),
        "zig" => zig::completion_items(kind, ctx),
        "julia" => julia::completion_items(kind, ctx),
        "objc" => objc::completion_items(kind, ctx),
        "makefile" => makefile::completion_items(kind, ctx),
        "cmake" => cmake::completion_items(kind, ctx),
        "powershell" => powershell::completion_items(kind, ctx),
        "proto" => proto::completion_items(kind, ctx),
        "graphql" => graphql::completion_items(kind, ctx),
        "terraform" => terraform::completion_items(kind, ctx),
        "vue" => vue::completion_items(kind, ctx),
        "svelte" => svelte::completion_items(kind, ctx),
        "asm" => asm::completion_items(kind, ctx),
        "tex" => tex::completion_items(kind, ctx),
        "erlang" => erlang::completion_items(kind, ctx),
        "solidity" => solidity::completion_items(kind, ctx),
        "ada" => ada::completion_items(kind, ctx),
        "arduino" => arduino::completion_items(kind, ctx),
        "asciidoc" => asciidoc::completion_items(kind, ctx),
        "ats" => ats::completion_items(kind, ctx),
        "awk" => awk::completion_items(kind, ctx),
        "batch" => bat::completion_items(kind, ctx),
        "b" => b::completion_items(kind, ctx),
        "caddyfile" => caddyfile::completion_items(kind, ctx),
        "cake" => cake::completion_items(kind, ctx),
        "coffeescript" => coffeescript::completion_items(kind, ctx),
        "conky" => conky::completion_items(kind, ctx),
        "crontab" => crontab::completion_items(kind, ctx),
        "crystal" => crystal::completion_items(kind, ctx),
        "cuda" => cuda::completion_items(kind, ctx),
        "cython" => cython::completion_items(kind, ctx),
        "d" => d::completion_items(kind, ctx),
        "dot" => dot::completion_items(kind, ctx),
        "ebuild" => ebuild::completion_items(kind, ctx),
        "elm" => elm::completion_items(kind, ctx),
        "erb" => erb::completion_items(kind, ctx),
        "etc-portage" => etcportage::completion_items(kind, ctx),
        "fish" => fish::completion_items(kind, ctx),
        "forth" => forth::completion_items(kind, ctx),
        "fortran" => fortran::completion_items(kind, ctx),
        "freebsd-kernel" => freebsd::completion_items(kind, ctx),
        "fsharp" => fsharp::completion_items(kind, ctx),
        "gdscript" => gdscript::completion_items(kind, ctx),
        "gemini" => gemini::completion_items(kind, ctx),
        "git-commit" => gitcommit::completion_items(kind, ctx),
        "git-config" => gitconfig::completion_items(kind, ctx),
        "git-rebase-todo" => gitrebase::completion_items(kind, ctx),
        "gleam" => gleam::completion_items(kind, ctx),
        "glsl" => glsl::completion_items(kind, ctx),
        "gnuplot" => gnuplot::completion_items(kind, ctx),
        "godoc" => godoc::completion_items(kind, ctx),
        "golo" => golo::completion_items(kind, ctx),
        "gomod" => gomod::completion_items(kind, ctx),
        "groff" => groff::completion_items(kind, ctx),
        "groovy" => groovy::completion_items(kind, ctx),
        "haml" => haml::completion_items(kind, ctx),
        "hare" => hare::completion_items(kind, ctx),
        "hc" => hc::completion_items(kind, ctx),
        "inputrc" => inputrc::completion_items(kind, ctx),
        "jinja2" => jinja2::completion_items(kind, ctx),
        "jsonnet" => jsonnet::completion_items(kind, ctx),
        "justfile" => justfile::completion_items(kind, ctx),
        "keymap" => keymap::completion_items(kind, ctx),
        "kickstart" => kickstart::completion_items(kind, ctx),
        "kvlang" => kvlang::completion_items(kind, ctx),
        "ledger" => ledger::completion_items(kind, ctx),
        "lfe" => lfe::completion_items(kind, ctx),
        "lilypond" => lilypond::completion_items(kind, ctx),
        "lisp" => lisp::completion_items(kind, ctx),
        "mail" => mail::completion_items(kind, ctx),
        "man" => man::completion_items(kind, ctx),
        "mc" => mc::completion_items(kind, ctx),
        "meson" => meson::completion_items(kind, ctx),
        "micro" => micro::completion_items(kind, ctx),
        "mpd" => mpd::completion_items(kind, ctx),
        "msbuild" => msbuild::completion_items(kind, ctx),
        "nanorc" => nanorc::completion_items(kind, ctx),
        "nftables" => nftables::completion_items(kind, ctx),
        "nim" => nim::completion_items(kind, ctx),
        "nix" => nix::completion_items(kind, ctx),
        "nu" => nu::completion_items(kind, ctx),
        "ocaml" => ocaml::completion_items(kind, ctx),
        "octave" => octave::completion_items(kind, ctx),
        "odin" => odin::completion_items(kind, ctx),
        "pascal" => pascal::completion_items(kind, ctx),
        "patch" => patch::completion_items(kind, ctx),
        "pc" => pkgconfig::completion_items(kind, ctx),
        "peg" => peg::completion_items(kind, ctx),
        "po" => po::completion_items(kind, ctx),
        "pony" => pony::completion_items(kind, ctx),
        "pov" => pov::completion_items(kind, ctx),
        "privoxy" => privoxy::completion_items(kind, ctx),
        "prql" => prql::completion_items(kind, ctx),
        "puppet" => puppet::completion_items(kind, ctx),
        "raku" => raku::completion_items(kind, ctx),
        "renpy" => renpy::completion_items(kind, ctx),
        "rpmspec" => rpmspec::completion_items(kind, ctx),
        "rst" => rest::completion_items(kind, ctx),
        "sage" => sage::completion_items(kind, ctx),
        "salt" => salt::completion_items(kind, ctx),
        "sed" => sed::completion_items(kind, ctx),
        "smalltalk" => smalltalk::completion_items(kind, ctx),
        "stata" => stata::completion_items(kind, ctx),
        "tcl" => tcl::completion_items(kind, ctx),
        "twig" => twig::completion_items(kind, ctx),
        "v" => v::completion_items(kind, ctx),
        "vala" => vala::completion_items(kind, ctx),
        "verilog" => verilog::completion_items(kind, ctx),
        "vhdl" => vhdl::completion_items(kind, ctx),
        "vi" => vi::completion_items(kind, ctx),
        "xresources" => xresources::completion_items(kind, ctx),
        "yum" => yum::completion_items(kind, ctx),
        "zscript" => zscript::completion_items(kind, ctx),
        "openscad" => scad::completion_items(kind, ctx),
        "markdown" => markdown::completion_items(kind, ctx),
        "json" => json::completion_items(kind, ctx),
        "toml" => toml::completion_items(kind, ctx),
        "yaml" => yaml::completion_items(kind, ctx),
        "dotenv" => dotenv::completion_items(kind, ctx),
        "ini" => ini::completion_items(kind, ctx),
        "log" => log::completion_items(kind, ctx),
        "systemd" => systemd::completion_items(kind, ctx),
        _ => Vec::new(),
    };
    let p = prefix.to_ascii_lowercase();
    items.retain(|i| i.label.to_ascii_lowercase().starts_with(&p));
    items.sort_by_key(|i| (i.label.len(), i.label.to_ascii_lowercase()));
    items.dedup_by(|a, b| a.label == b.label);
    items.truncate(30);
    items
}

pub(crate) fn extract_symbols(text: &str, syntax: SyntaxMode) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        let no = idx + 1;
        match syntax {
            SyntaxMode::Php => {
                out.extend(php::symbols(line).into_iter().map(|s| (s, no)));
                out.extend(html::symbols(line).into_iter().map(|s| (s, no)));
                if css::looks_like_line(line) { out.extend(css::symbols(line).into_iter().map(|s| (s, no))); }
                if javascript::looks_like_line(line) { out.extend(javascript::symbols(line).into_iter().map(|s| (s, no))); }
            }
            SyntaxMode::Blade => {
                out.extend(php::symbols(line).into_iter().map(|s| (s, no)));
                out.extend(blade::symbols(line).into_iter().map(|s| (s, no)));
                out.extend(html::symbols(line).into_iter().map(|s| (s, no)));
                if css::looks_like_line(line) { out.extend(css::symbols(line).into_iter().map(|s| (s, no))); }
                if javascript::looks_like_line(line) { out.extend(javascript::symbols(line).into_iter().map(|s| (s, no))); }
            }
            SyntaxMode::Html => {
                if line_contains_php(line) { out.extend(php::symbols(line).into_iter().map(|s| (s, no))); }
                out.extend(html::symbols(line).into_iter().map(|s| (s, no)));
                if css::looks_like_line(line) { out.extend(css::symbols(line).into_iter().map(|s| (s, no))); }
                if javascript::looks_like_line(line) { out.extend(javascript::symbols(line).into_iter().map(|s| (s, no))); }
            }
            SyntaxMode::Css => out.extend(css::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::JavaScript => out.extend(javascript::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::TypeScript => out.extend(typescript::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Xml => out.extend(xml::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Markdown => out.extend(markdown::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Json => out.extend(json::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Toml => out.extend(toml::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Yaml => out.extend(yaml::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Bash => out.extend(bash::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Dotenv => out.extend(dotenv::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Ini => out.extend(ini::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Log => out.extend(log::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Rust => out.extend(rust::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Nginx => out.extend(nginx::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Apache => out.extend(apache::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Dockerfile => out.extend(dockerfile::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Systemd => out.extend(systemd::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Sql => out.extend(sql::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Python => out.extend(python::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Java => out.extend(java::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Csharp => out.extend(csharp::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Cpp => out.extend(cpp::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::C => out.extend(c::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Go => out.extend(go::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Kotlin => out.extend(kotlin::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Swift => out.extend(swift::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Ruby => out.extend(ruby::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Dart => out.extend(dart::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Scala => out.extend(scala::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::R => out.extend(r::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Lua => out.extend(lua::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Perl => out.extend(perl::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Haskell => out.extend(haskell::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Elixir => out.extend(elixir::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Clojure => out.extend(clojure::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Zig => out.extend(zig::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Julia => out.extend(julia::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Objc => out.extend(objc::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Makefile => out.extend(makefile::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Cmake => out.extend(cmake::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Powershell => out.extend(powershell::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Proto => out.extend(proto::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Graphql => out.extend(graphql::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Terraform => out.extend(terraform::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Vue => out.extend(vue::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Svelte => out.extend(svelte::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Asm => out.extend(asm::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Tex => out.extend(tex::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Erlang => out.extend(erlang::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Solidity => out.extend(solidity::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Ada => out.extend(ada::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Arduino => out.extend(arduino::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Asciidoc => out.extend(asciidoc::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Ats => out.extend(ats::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Awk => out.extend(awk::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Bat => out.extend(bat::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::B => out.extend(b::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Caddyfile => out.extend(caddyfile::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Cake => out.extend(cake::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Coffeescript => out.extend(coffeescript::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Conky => out.extend(conky::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Crontab => out.extend(crontab::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Crystal => out.extend(crystal::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Cuda => out.extend(cuda::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Cython => out.extend(cython::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::D => out.extend(d::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Dot => out.extend(dot::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Ebuild => out.extend(ebuild::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Elm => out.extend(elm::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Erb => out.extend(erb::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Etcportage => out.extend(etcportage::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Fish => out.extend(fish::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Forth => out.extend(forth::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Fortran => out.extend(fortran::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Freebsd => out.extend(freebsd::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Fsharp => out.extend(fsharp::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Gdscript => out.extend(gdscript::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Gemini => out.extend(gemini::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Gitcommit => out.extend(gitcommit::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Gitconfig => out.extend(gitconfig::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Gitrebase => out.extend(gitrebase::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Gleam => out.extend(gleam::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Glsl => out.extend(glsl::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Gnuplot => out.extend(gnuplot::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Godoc => out.extend(godoc::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Golo => out.extend(golo::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Gomod => out.extend(gomod::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Groff => out.extend(groff::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Groovy => out.extend(groovy::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Haml => out.extend(haml::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Hare => out.extend(hare::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Hc => out.extend(hc::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Inputrc => out.extend(inputrc::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Jinja2 => out.extend(jinja2::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Jsonnet => out.extend(jsonnet::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Justfile => out.extend(justfile::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Keymap => out.extend(keymap::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Kickstart => out.extend(kickstart::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Kvlang => out.extend(kvlang::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Ledger => out.extend(ledger::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Lfe => out.extend(lfe::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Lilypond => out.extend(lilypond::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Lisp => out.extend(lisp::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Mail => out.extend(mail::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Man => out.extend(man::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Mc => out.extend(mc::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Meson => out.extend(meson::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Micro => out.extend(micro::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Mpd => out.extend(mpd::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Msbuild => out.extend(msbuild::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Nanorc => out.extend(nanorc::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Nftables => out.extend(nftables::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Nim => out.extend(nim::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Nix => out.extend(nix::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Nu => out.extend(nu::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Ocaml => out.extend(ocaml::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Octave => out.extend(octave::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Odin => out.extend(odin::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Pascal => out.extend(pascal::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Patch => out.extend(patch::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Pkgconfig => out.extend(pkgconfig::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Peg => out.extend(peg::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Po => out.extend(po::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Pony => out.extend(pony::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Pov => out.extend(pov::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Privoxy => out.extend(privoxy::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Prql => out.extend(prql::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Puppet => out.extend(puppet::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Raku => out.extend(raku::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Renpy => out.extend(renpy::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Rpmspec => out.extend(rpmspec::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Rest => out.extend(rest::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Sage => out.extend(sage::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Salt => out.extend(salt::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Sed => out.extend(sed::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Smalltalk => out.extend(smalltalk::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Stata => out.extend(stata::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Tcl => out.extend(tcl::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Twig => out.extend(twig::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::V => out.extend(v::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Vala => out.extend(vala::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Verilog => out.extend(verilog::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Vhdl => out.extend(vhdl::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Vi => out.extend(vi::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Xresources => out.extend(xresources::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Yum => out.extend(yum::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Zscript => out.extend(zscript::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Scad => out.extend(scad::symbols(line).into_iter().map(|s| (s, no))),
            SyntaxMode::Plain => {}
        }
    }
    out
}

pub(crate) fn comp(label: &str, insert: &str, detail: &str) -> CompletionItem {
    CompletionItem { label: label.to_string(), insert: insert.to_string(), detail: detail.to_string() }
}

pub(crate) fn line_contains_php(line: &str) -> bool {
    line.contains("<?") || line.contains("?>") || line.contains("$") || line.contains("->") || line.contains("::")
}

pub(crate) fn find_between(line: &str, start_pat: &str, end_pat: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut pos = 0;
    while let Some(a) = line[pos..].find(start_pat) {
        let start = pos + a;
        let after = start + start_pat.len();
        if let Some(b) = line[after..].find(end_pat) {
            let end = after + b + end_pat.len();
            out.push((start, end));
            pos = end;
        } else {
            out.push((start, line.len()));
            break;
        }
    }
    out
}

pub(crate) fn string_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\'' || bytes[i] == b'"' {
            let quote = bytes[i];
            let start = i;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' { i += 2; continue; }
                if bytes[i] == quote { i += 1; break; }
                i += 1;
            }
            out.push((start, i.min(bytes.len())));
        } else { i += 1; }
    }
    out
}

pub(crate) fn find_words(line: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    for (i, c) in line.char_indices() {
        if c.is_alphanumeric() || c == '_' {
            if start.is_none() { start = Some(i); }
        } else if let Some(s) = start.take() {
            out.push((s, i));
        }
    }
    if let Some(s) = start { out.push((s, line.len())); }
    out
}

pub(crate) fn scan_ranges<F: Fn(char) -> bool>(line: &str, pred: F) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    for (i, c) in line.char_indices() {
        if pred(c) {
            if start.is_none() { start = Some(i); }
        } else if let Some(s) = start.take() {
            out.push((s, i));
        }
    }
    if let Some(s) = start { out.push((s, line.len())); }
    out
}

pub(crate) fn find_literals(line: &str, pats: &[&str]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for pat in pats {
        let mut pos = 0;
        while let Some(idx) = line[pos..].find(pat) {
            let s = pos + idx;
            out.push((s, s + pat.len()));
            pos = s + pat.len();
        }
    }
    out
}

pub(crate) fn find_prefixed_words(line: &str, prefix: char) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let p = prefix as u8;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == p {
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'-') { i += 1; }
            if i > start + 1 { out.push((start, i)); }
        } else { i += 1; }
    }
    out
}

pub(crate) fn after_nonspace_is(line: &str, from: usize, ch: char) -> bool {
    line[from..].chars().find(|c| !c.is_whitespace()) == Some(ch)
}

pub(crate) fn pos_in_ranges(pos: usize, ranges: &[(usize, usize)]) -> bool {
    ranges.iter().any(|(a, b)| pos >= *a && pos < *b)
}

pub(crate) fn is_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b':' | b'.')
}

pub(crate) fn word_suffix(before: &str) -> Option<(String, usize)> {
    let mut start = before.len();
    for (i, c) in before.char_indices().rev() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            start = i;
        } else { break; }
    }
    if start < before.len() { Some((before[start..].to_string(), start)) } else { None }
}

pub(crate) fn suffix_token(before: &str, marker: char) -> Option<(String, usize)> {
    let idx = before.rfind(marker)?;
    if before[idx + marker.len_utf8()..].chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        Some((before[idx..].to_string(), idx))
    } else { None }
}
