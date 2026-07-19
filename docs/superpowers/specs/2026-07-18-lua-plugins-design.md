# Lua plugin system — design

Date: 2026-07-18

## Problem

`jsontolang` (crate `jsontolang`, repo `jsontox`) infers a schema from JSON and renders
type definitions for a target language. Target-language renderers currently live as
compiled Rust plugins in `src/plugins/{rust,go,typescript}.rs`, implementing a shared
`LanguagePlugin` trait (`key()`, `render(&Document) -> Result<String>`). Adding or
changing a target language today requires editing Rust and recompiling the binary.

Goal: replace the compiled Rust plugins with plugins authored in Lua, loaded and
executed at runtime, so new target languages can be added or edited by dropping a
`.lua` file — no recompile.

## Scope

Full replacement. `src/plugins/{rust.rs,go.rs,typescript.rs}` are deleted. Their logic
is reimplemented as `.lua` scripts. The Rust side of `src/plugins` becomes a generic
Lua-script loader; there is exactly one plugin mechanism, not two.

## Architecture

```
src/
  cli.rs              --lang: String (was a closed clap ValueEnum of 3 variants)
  lib.rs               unchanged shape (validate -> read_json -> infer_document -> lookup -> render)
  schema.rs            Document / NamedType / Field / TypeExpr gain #[derive(Serialize)]
  plugins/
    mod.rs              discovery + lookup_by_key, no more static TYPESCRIPT/RUST/GO instances
    lua_plugin.rs        new: LuaPlugin, wraps mlua::Lua + loaded plugin table

plugins/                 repo-level default scripts, also embedded into the binary
  typescript.lua
  rust.lua
  go.lua
```

`LanguagePlugin` trait is unchanged:

```rust
pub trait LanguagePlugin {
    fn key(&self) -> &'static str; // becomes owned String internally for Lua plugins
    fn render(&self, document: &Document) -> Result<String>;
}
```

(`key()` return type changes to `String`/`Cow<str>` since Lua plugins don't have
`&'static` keys — this is a trait signature change, all three current call sites in
`plugins/mod.rs` and `lib.rs` update accordingly.)

### Dependency

Add `mlua = { version = "...", features = ["lua54", "vendored", "serialize"] }`.
`vendored` bundles Lua source, no system Lua install required. `serialize` enables
`mlua::to_value`/`from_value` against `serde::Serialize`/`Deserialize` types.

### Discovery

`plugins::discover() -> Vec<LuaPlugin>`:

1. Resolve plugin dir: `$XDG_CONFIG_HOME/jsontolang/plugins`, falling back to
   `~/.config/jsontolang/plugins` if `XDG_CONFIG_HOME` is unset.
2. If the dir doesn't exist or contains no `.lua` files, fall back to 3 scripts
   embedded at compile time via `include_str!` (the same `typescript.lua`, `rust.lua`,
   `go.lua` shipped in the repo's top-level `plugins/` dir) — the app always has at
   least the 3 default plugins available, no first-run install step needed.
3. For each `.lua` file found (external dir takes full precedence over embedded
   defaults when the dir is non-empty — no merging), load it as a `LuaPlugin`. A file
   that fails to load (syntax error, missing `key`/`render`, wrong types) is skipped
   with a stderr warning; it does not abort loading the rest.

`lookup_by_key(key: &str)`: runs `discover()`, linear-searches for a matching `key()`.
No match → error listing all discovered keys, sorted.

`lookup(language: &str)` (called from `lib.rs`) is now just `lookup_by_key` — the old
`Language` enum -> key match function is deleted.

### Data bridge (Rust Document -> Lua table)

`TypeExpr` becomes internally tagged for a clean Lua-side shape:

```rust
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TypeExpr {
    Any,
    Bool,
    Integer,
    UnsignedInteger,
    Float,
    String,
    Array { item: Box<TypeExpr> },
    Named { name: String },
}
```

`Field` and `NamedType` and `Document` derive `Serialize` as-is (struct field names
carry over as Lua table keys).

Resulting Lua shape passed to `render(document)`:

```lua
{
  root_name = "Root",
  root = { kind = "named", name = "Root" },
  types = {
    {
      name = "Root",
      fields = {
        { name = "id", ty = { kind = "integer" }, optional = false },
        { name = "tags", ty = { kind = "array", item = { kind = "string" } }, optional = true },
      },
    },
  },
}
```

`LuaPlugin::render`:
1. `mlua::to_value(&lua, document)` -> `mlua::Value`.
2. Call the plugin's stored `render` function with that value.
3. Expect a Lua string back; convert to `Result<String>`. A Lua runtime error (nil
   index, type error, thrown `error()`) is caught and wrapped as an `anyhow::Error`
   that includes the plugin's `key` and Lua's error message.

### Plugin file contract

A plugin `.lua` file must, when executed, leave a single table on the stack (i.e.
`return { ... }` as the last statement) shaped as:

```lua
return {
  key = "typescript",      -- string, matched against --lang
  render = function(document)
    local out = {}
    -- ... build output from document.types / document.root ...
    return table.concat(out)
  end,
}
```

`LuaPlugin::load(path_or_embedded_source)`:
1. `lua.load(source).eval::<mlua::Table>()` — error if it doesn't evaluate to a table.
2. Read `key` field — error if missing or not a string.
3. Read `render` field — error if missing or not callable.
4. Store the `mlua::Lua` instance + the table (keeps `render` closure alive) inside
   `LuaPlugin`.

### Sandboxing

Each `LuaPlugin` gets its own `mlua::Lua` instance constructed with restricted stdlib:

```rust
Lua::new_with(
    StdLib::STRING | StdLib::TABLE | StdLib::MATH,
    LuaOptions::default(),
)?
```

No `io`, `os`, `package`, `debug`, `coroutine`. Plugins are pure `document in -> string
out` functions; they have no legitimate reason to touch the filesystem, environment,
or process. This is a hard boundary, not a default that plugins can opt out of.

### CLI

`Cli.lang` changes from `Language` (clap `ValueEnum`) to `String`. `clap`'s
`ArgGroup`/required/validate logic for `--file`/`--stdin`/`--json` is untouched.
`Cli::validate()` no longer needs a `Language` match; the lang string is validated
implicitly by `lookup_by_key` failing with a clear "unknown language, available: ..."
error at lookup time (not at CLI parse time, since valid values are only known after
plugin discovery).

## Error handling

| Failure | Behavior |
|---|---|
| Plugin dir missing/empty | Fall back to 3 embedded default scripts, no error |
| `.lua` file syntax error | Skip file, stderr warning with filename + Lua error, continue loading others |
| Plugin table missing `key`/`render` or wrong types | Skip file, stderr warning, continue loading others |
| `render()` throws / runtime errors | Propagate as `anyhow::Error` (includes plugin key + Lua message), CLI exits non-zero |
| `--lang` matches no loaded plugin | Error listing all discovered plugin keys, sorted, CLI exits non-zero |

## Testing

- `tests/plugin_output.rs` (existing): repointed at the 3 shipped `.lua` files instead
  of Rust impls; same fixture-JSON-in / expected-string-out assertions.
- `src/plugins/lua_plugin.rs` unit tests:
  - Load a minimal inline Lua source string plugin, assert `key()` and `render()`
    round-trip correctly against a hand-built `Document`.
  - Assert sandboxed globals are absent (`os`, `io` are `Nil` in the plugin's Lua
    globals table).
  - Assert a syntactically invalid `.lua` source does not panic `discover()` and is
    excluded from the result (rather than aborting the whole discovery pass).
- `tests/cli.rs` (existing): update `--lang typescript` (etc.) invocations — proves
  the free-string CLI path plus embedded-fallback discovery works end-to-end with no
  config dir present (the CI/test environment).

## Out of scope

- No plugin versioning/manifest format beyond the `key`/`render` table contract.
- No hot-reload of plugins within a single CLI invocation (each run re-discovers once).
- No network or filesystem access from within plugins (see Sandboxing).
- No changes to `schema.rs` inference logic itself, only the `Serialize` derives
  needed for the Lua data bridge.
