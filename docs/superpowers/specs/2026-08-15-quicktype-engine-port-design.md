# Port quick typw's type-conversion logic into jsontolang-core

## Why

`quick typw` (top-level dir, standalone `quicktype-rs` crate, not a workspace
member) has a more thorough JSON type-inference and rendering engine than
`crates/core`: proper nullable unification (`Shape::Nullable`), quicktype's
`InferMaps` map-shape detection (all-numeric keys always; 20+/50+ property
thresholds otherwise), and structural dedup of identical object shapes.
`crates/core/src/schema.rs` currently has none of these — nulls collapse to
`Any`, there's no map inference, and structural dedup doesn't exist.

User directive: reference `quick typw` only for the type-conversion logic,
and update the rest of the app around it.

## Scope

**Keep stable:** the serialized public IR (`Document`, `NamedType`, `Field`,
`TypeExpr`) — `crates/wasm` exports it as a JS object via `serde_wasm_bindgen`
and `crates/cli`'s Lua plugins consume it as an `mlua` table. Neither consumer
changes shape.

**Port, don't vendor:** quick typw's `infer`/`merge`/`normalize` algorithm
becomes a new internal `Shape` type inside `crates/core` (not a dependency on
the `quick typw` crate — it's not a workspace member and has a different
public API). The existing 3-way `Integer`/`UnsignedInteger`/`Float` split in
`TypeExpr` is untouched; quick typw only distinguishes `Int`/`Double` and that
distinction doesn't map cleanly onto the existing IR's finer split.

**New `TypeExpr` variants (additive):**
- `Nullable(Box<TypeExpr>)` — a value that is sometimes JSON `null`.
- `Map { value: Box<TypeExpr> }` — an object whose keys are data, not field
  names (quick typw's `InferMaps` thresholds).

## Components

1. **`crates/core/src/shape.rs` (new)** — port of quick typw's `Shape` enum,
   `infer`, `merge`, `nullable`, `normalize`, `map_value`/thresholds, and the
   `Names` structural-dedup/naming pass. Internal only, not re-exported.

2. **`crates/core/src/schema.rs`** — `infer_document` builds a `Shape` via the
   ported `infer`/`normalize`, then converts `Shape` → `Document` using
   `Names` for naming and dedup (replacing today's path-keyed `TypeRegistry`).
   Public signature (`infer_document(root_name, &Value) -> Result<Document>`)
   is unchanged.

3. **`crates/core/src/render/{go,rust,typescript}.rs`** — extended to handle
   `TypeExpr::Nullable`/`TypeExpr::Map`, adopting quick typw's per-language
   conventions for them: Go pointer-for-value-types + `,omitempty`,
   `map[string]T`; Rust `Option<T>`, `HashMap<String, T>`; TypeScript `?`
   member, `Record<string, T>`. Still walks `Document`, not quick typw's
   `Shape` — this is a port of formatting rules, not a code copy.

4. **`crates/cli/plugins/{go,rust,typescript}.lua`** — mirrored changes so
   `crates/cli/tests/render_parity.rs` (which pins native-Rust and Lua output
   byte-identical) stays green. CLI-generated output changes accordingly.

5. **Tests** — `crates/core/tests/schema_inference.rs` and `render.rs`,
   `crates/cli/tests/render_parity.rs` and `plugin_output.rs` assert exact
   output strings today; most existing cases get their expected strings
   updated for the new format, plus new cases for nullable fields and map
   inference (ported from quick typw's own `#[cfg(test)]` blocks).

**Untouched:** `crates/tui` (drives the CLI only), `crates/wasm` (calls
`core::render`, gets new variants for free), `apps/web` (consumes wasm
bindings).

## Data flow

`JSON Value` → `shape::infer` (per-sample) → `shape::merge` (unify samples) →
`shape::normalize` (nullable collapse + map-threshold rewrite) → `Shape` tree
→ `Names::gather` (structural dedup, naming) → `Document`/`NamedType`/
`TypeExpr` → per-language renderer (native Rust for wasm, Lua for CLI).

## Testing

- `cargo test -p jsontolang-core` — inference + native renderer cases.
- `cargo test -p jsontolang-cli` — Lua plugin output + `render_parity`
  (byte-identical native vs. Lua).
- `cargo build --target wasm32-unknown-unknown -p jsontolang-wasm` — confirms
  the new `Shape`/`Names` code has no non-wasm-safe dependencies (must stay
  `mlua`-free per the crate's existing doc comment).
