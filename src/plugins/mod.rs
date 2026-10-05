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
        SyntaxMode::Plain => "PLAIN",
    }
}

pub(crate) fn from_word(word: &str) -> Option<SyntaxMode> {
    match word.trim().to_ascii_lowercase().as_str() {
        "php" => Some(SyntaxMode::Php),
        "blade" => Some(SyntaxMode::Blade),
        "html" => Some(SyntaxMode::Html),
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
        "python" | "py" => Some(SyntaxMode::Python),
        "java" => Some(SyntaxMode::Java),
        "csharp" | "c#" | "cs" => Some(SyntaxMode::Csharp),
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
