# Plan: jsontolang as a 3-surface product in the ReASM monorepo

## Context

This repo (`jsontox-monorepo`) currently hosts one product, ReASM (Rust VM/parser + React visualizer), via a root Cargo workspace (`crates/core`, `crates/parser`, `crates/wasm`) and a pnpm/turborepo frontend (`apps/web`, `packages/{ui,tsconfig,eslint-config}`). Separately, a gitignored `jsontox/` folder holds a working standalone Rust CLI, package name `jsontolang`: reads JSON, infers a schema, renders type definitions via Lua plugin scripts (typescript/rust/go), loaded either from `~/.config/jsontolang/plugins/*.lua` or 3 embedded defaults.

Goal: turn `jsontolang` into a proper 3-surface product — CLI (unchanged behavior), Web (landing + plugin showcase + live WASM playground), TUI (future, no code now) — by joining it into this monorepo's existing Cargo/pnpm workspaces rather than building new tooling.

Two decisions already made (user-confirmed, not open questions):
1. **Playground = built-in languages only.** mlua vendors C Lua via a `cc` build script, which doesn't target `wasm32-unknown-unknown` cleanly. Rather than rewriting the plugin engine on a pure-Rust Lua VM, the browser playground runs native Rust renderers for ts/go/rust. Custom `.lua` plugin authoring/execution stays CLI-only, backed by native mlua, unchanged.
2. **Join existing workspaces**, not a separate subtree — new crates go into the root `Cargo.toml` `members`, new app goes into the pnpm workspace, reusing `packages/ui`/`tsconfig`/`eslint-config` and following `crates/wasm`'s existing wasm-bindgen → `apps/web/public` wiring pattern exactly.

## Verified current state

- `jsontox/src/schema.rs` (385 lines): pure `serde_json::Value → Document/TypeExpr`, zero I/O, zero mlua — already wasm-safe as-is.
- `jsontox/src/plugins/mod.rs`: `LanguagePlugin` trait, discovers `.lua` from config dir, falls back to `include_str!`'d embedded defaults (`plugins/{typescript,rust,go}.lua`, ~665 lines total).
- `jsontox/src/plugins/lua_plugin.rs`: sole mlua touchpoint, sandboxed Lua 5.4.
- `jsontox/src/{main,lib,cli,input}.rs`: CLI I/O, already cleanly separated from schema/render logic.
- Tests: `tests/schema_inference.rs` (594 lines, pure schema), `tests/plugin_output.rs` (629 lines, Lua-path), `tests/cli.rs` (116 lines, binary-level).
- `crates/wasm/src/lib.rs` + `package.json`: the pattern to mirror — plain `#[wasm_bindgen]` fns, `serde_wasm_bindgen::to_value`, build script does `wasm-pack build --target web && cp pkg/wasm_bg.wasm ../../apps/web/public/`.
- Root `package.json` chains wasm before app build by hand (`"build": "pnpm wasm && pnpm --dir apps/web build"`) — turbo isn't doing that graph edge; follow the same manual-sequencing convention for jsontolang.
- `pnpm-workspace.yaml` lists `crates/wasm` and `crates/wasm/pkg` explicitly (the generated `pkg/package.json` becomes an importable workspace package). Root `.gitignore` has `crates/wasm/pkg/`, `/apps/web/public/wasm_bg.wasm`, and `/jsontox`.

## Target layout

```
Cargo.toml                          # members += 3 new crates
crates/
  jsontolang-core/                  # pure lib, no mlua, wasm-safe
    src/lib.rs
    src/schema.rs                   # moved verbatim from jsontox/src/schema.rs
    src/render/{mod,typescript,rust,go}.rs   # NEW native ports of the 3 .lua files
    tests/schema_inference.rs       # moved, only import path changes
    tests/render.rs                 # NEW, parity-pinning subset of plugin_output.rs assertions
  jsontolang-cli/                   # binary crate, binary name stays "jsontolang"
    src/{main,lib,cli,input}.rs     # moved verbatim (schema:: refs -> jsontolang_core::schema::)
    src/plugins/{mod,lua_plugin}.rs # moved verbatim, mlua stays CLI-only
    plugins/{typescript,rust,go}.lua # moved from jsontox/plugins/
    tests/{plugin_output,cli}.rs    # moved verbatim (crate/binary name unchanged, no edits needed)
  jsontolang-wasm/                  # wasm-bindgen glue, mirrors crates/wasm
    Cargo.toml, package.json, src/lib.rs
apps/
  jsontolang-web/                   # landing + plugin showcase + playground
    package.json  # deps: @workspace/{ui,tsconfig,eslint-config}, jsontolang-wasm (workspace:*)
    public/{wasm_bg.wasm, plugins/*.lua}   # populated by build-time copy steps
    src/routes/{index,plugins,playground}.tsx
    src/views/{home,plugins,playground}/
    src/data/plugins.ts              # hand-written metadata for the 3 built-ins
```

Skipped: no `jsontolang-parser` crate. ReASM splits parser/core because assembly parsing and VM execution are genuinely separable. jsontolang's "parsing" is one `serde_json::from_str` call — not worth a crate.

## Core vs CLI split

- `schema.rs` moves into `jsontolang-core` with no logic changes.
- `LanguagePlugin` trait (dyn dispatch over Lua-backed plugins, discovery across embedded + user files) stays CLI-only — it's a CLI concept, core exposes plain functions instead: `jsontolang_core::render::render(lang: &str, doc: &Document) -> Result<String>` + `BUILTIN_LANGS: &[&str]`.
- CLI's embedded defaults stay exactly as today — `include_str!` + mlua interpretation. CLI never calls `jsontolang_core::render`; that function exists solely for wasm. This preserves requirement 1 (CLI output byte-identical to current `jsontox`).
- `input.rs`/`cli.rs` move to `jsontolang-cli` untouched (std::fs/io/clap, never needed by core or wasm).

## wasm-bindgen surface (`crates/jsontolang-wasm/src/lib.rs`)

```rust
#[wasm_bindgen] pub fn infer(root: &str, json: &str) -> Result<JsValue, JsValue>
#[wasm_bindgen] pub fn render_types(root: &str, json: &str, lang: &str) -> Result<String, JsValue>
#[wasm_bindgen] pub fn builtin_languages() -> Vec<String>
```
Same `serde_wasm_bindgen`/`JsValue::from_str(&e.to_string())` error pattern as `crates/wasm`. `package.json` build script is a literal copy of `crates/wasm/package.json`'s, pointed at `apps/jsontolang-web/public`.

## apps/jsontolang-web

Scaffolded from `apps/web` (same TanStack Router + `@workspace/*` deps + vite config shape).

- **Landing** (`routes/index.tsx`): what jsontolang is, links to showcase/playground/CLI install (reuse `jsontox/README.md` copy).
- **Plugin showcase** (`routes/plugins.tsx`): renders cards from hand-written `src/data/plugins.ts` (key/title/description/file for the 3 built-ins), download links to `/plugins/*.lua` served from `public/plugins/`, populated by a build-time copy step from `crates/jsontolang-cli/plugins/` (same "copy artifact into public/" pattern already used for `wasm_bg.wasm`). No CMS/registry for v1 — 3 hand-edited entries.
- **Playground** (`routes/playground.tsx`): `import init, { render_types, builtin_languages } from "jsontolang-wasm"`, JSON textarea + root-name input (default `"Root"`, matches CLI) + language `<select>` populated from `builtin_languages()` + read-only output panel. All in-browser, no server round-trip.

## Migration steps (dependency order)

1. Root plumbing: add 3 crates to `Cargo.toml` members, add `crates/jsontolang-wasm` + `.../pkg` to `pnpm-workspace.yaml`, add their build artifacts (`crates/jsontolang-wasm/pkg/`, `apps/jsontolang-web/public/{wasm_bg.wasm,plugins/}`) to root `.gitignore`.
2. `jsontolang-core`: move `schema.rs` + `tests/schema_inference.rs` verbatim (import path only), write new `render/*.rs`. `cargo test -p jsontolang-core` passes.
3. `jsontolang-cli`: move `main/lib/cli/input.rs`, `plugins/{mod,lua_plugin}.rs`, `plugins/*.lua`, `tests/{plugin_output,cli}.rs` — update only `schema::` references to `jsontolang_core::schema::`. Package name stays `jsontolang` so test files need zero edits. `cargo test -p jsontolang` passes.
4. `jsontolang-wasm`: new crate depending on `jsontolang-core` only, mirrors `crates/wasm`'s `Cargo.toml`/`package.json` shape.
5. `apps/jsontolang-web`: scaffold from `apps/web`, add 3 routes/views, `data/plugins.ts`, plugin-sync copy step, `jsontolang-wasm: workspace:*` dep.
6. Root `package.json`: add `"jsontolang-wasm"` / `"watch:jsontolang-wasm"` scripts mirroring `wasm`/`watch:wasm`, extend `"build"` to include it. No `turbo.json` changes needed (turbo already covers `apps/*` generically).
7. CI: port `jsontox/.github/workflows/ci.yml` + `scripts/check.sh` into a new root `.github/workflows/` (none exists yet), updating cargo invocations to the 3 new `-p` package names, add `apps/jsontolang-web` build/lint/typecheck steps.
8. Cleanup: once all tests + builds pass, delete `jsontox/` and remove `/jsontox` from root `.gitignore` in the same change.

## Test strategy

| File | Destination | Change |
|---|---|---|
| `tests/schema_inference.rs` | `crates/jsontolang-core/tests/` | import path only |
| `tests/plugin_output.rs` | `crates/jsontolang-cli/tests/` | none |
| `tests/cli.rs` | `crates/jsontolang-cli/tests/` | none |
| *(new)* `tests/render.rs` | `crates/jsontolang-core/tests/` | ports ~10-15 assertions from `plugin_output.rs` per language, run against native renderers directly — pins native-renderer behavior without a full golden-diff harness against Lua output |
| `jsontolang-wasm` | — | no dedicated tests for v1; it's a ~40-line pass-through over already-tested core functions |

## Duplication tradeoff (named explicitly)

ts/go/rust rendering logic exists twice: as `.lua` templates (CLI, mlua) and as native Rust (`jsontolang-core`, used by wasm only). ~665 lines mirrored in two languages. Accepted because mlua can't target `wasm32-unknown-unknown`, and requirement 1 (CLI output byte-identical to today) means the Lua source can't be replaced as the CLI's source of truth anyway. Mark each of the 6 files with `// ponytail: rendering logic duplicated between here and <other path> — see docs/jsontolang-monorepo-plan.md for why and the upgrade path`.

**Upgrade path** if drift/bugs appear or a 4th language is needed: drop CLI's Lua embedded defaults, have CLI call `jsontolang_core::render` directly too (same fn wasm uses), keep `.lua` plugin mechanism only for user-authored custom plugins. Deliberately out of scope now — it changes CLI behavior.

## Future: TUI

Out of scope for this plan, no `crates/jsontolang-tui` stubbed now — nothing requests it yet. When built, it depends on `jsontolang-core` exactly like `jsontolang-cli`/`jsontolang-wasm` do (reuses `infer_document` + `render` as-is), needing only its own `ratatui`-based presentation layer and a 4th workspace member. No changes to the layout above are needed to add it later.

## App identity

**Name:** jsontolang
**Tagline:** Paste JSON. Get types.
**Description:** JSON in, type definitions out — infers a schema from any JSON and generates matching type definitions for TypeScript, Rust, Go, or a custom Lua plugin of your own. CLI, browser playground, one core.

## SEO & metadata (apps/jsontolang-web)

Domain placeholder used below: `https://jsontolang.dev` — swap for the real domain once registered. Shared OG image placeholder: `/og-image.png` (1200x630px) — needs designing before launch.

### Route: `/` (landing)

```html
<title>jsontolang — JSON to Types, Instantly</title>
<meta name="description" content="Infer a schema from any JSON and generate TypeScript, Rust, or Go types instantly in your browser, powered by WASM. Free CLI and plugin system included." />
<link rel="canonical" href="https://jsontolang.dev/" />

<meta property="og:title" content="jsontolang — JSON to Types, Instantly" />
<meta property="og:description" content="Infer a schema from any JSON and generate TypeScript, Rust, or Go types instantly in your browser, powered by WASM." />
<meta property="og:image" content="https://jsontolang.dev/og-image.png" />
<meta property="og:url" content="https://jsontolang.dev/" />
<meta property="og:type" content="website" />
<meta property="og:site_name" content="jsontolang" />

<meta name="twitter:card" content="summary_large_image" />
<meta name="twitter:title" content="jsontolang — JSON to Types, Instantly" />
<meta name="twitter:description" content="Infer a schema from any JSON and generate TypeScript, Rust, or Go types instantly in your browser, powered by WASM." />
<meta name="twitter:image" content="https://jsontolang.dev/og-image.png" />
```

JSON-LD (`SoftwareApplication`, placed on landing only):

```json
{
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  "name": "jsontolang",
  "description": "Infers a schema from JSON and generates type definitions for TypeScript, Rust, Go, or custom Lua plugins.",
  "applicationCategory": "DeveloperApplication",
  "operatingSystem": "Web, macOS, Linux, Windows",
  "url": "https://jsontolang.dev",
  "offers": { "@type": "Offer", "price": "0", "priceCurrency": "USD" }
}
```

### Route: `/plugins` (plugin showcase)

```html
<title>Plugins — jsontolang</title>
<meta name="description" content="Browse and download jsontolang's Lua plugins for TypeScript, Rust, and Go — or write your own to target any language." />
<link rel="canonical" href="https://jsontolang.dev/plugins" />

<meta property="og:title" content="Plugins — jsontolang" />
<meta property="og:description" content="Browse and download jsontolang's Lua plugins for TypeScript, Rust, and Go — or write your own to target any language." />
<meta property="og:image" content="https://jsontolang.dev/og-image.png" />
<meta property="og:url" content="https://jsontolang.dev/plugins" />
<meta property="og:type" content="website" />

<meta name="twitter:card" content="summary_large_image" />
<meta name="twitter:title" content="Plugins — jsontolang" />
<meta name="twitter:description" content="Browse and download jsontolang's Lua plugins for TypeScript, Rust, and Go — or write your own to target any language." />
<meta name="twitter:image" content="https://jsontolang.dev/og-image.png" />
```

JSON-LD (`BreadcrumbList`):

```json
{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    { "@type": "ListItem", "position": 1, "name": "Home", "item": "https://jsontolang.dev/" },
    { "@type": "ListItem", "position": 2, "name": "Plugins", "item": "https://jsontolang.dev/plugins" }
  ]
}
```

### Route: `/playground` (WASM playground)

```html
<title>Playground — jsontolang</title>
<meta name="description" content="Paste JSON and generate type definitions live in your browser. No install, no server — runs entirely on WASM." />
<link rel="canonical" href="https://jsontolang.dev/playground" />

<meta property="og:title" content="Playground — jsontolang" />
<meta property="og:description" content="Paste JSON and generate type definitions live in your browser. No install, no server — runs entirely on WASM." />
<meta property="og:image" content="https://jsontolang.dev/og-image.png" />
<meta property="og:url" content="https://jsontolang.dev/playground" />
<meta property="og:type" content="website" />

<meta name="twitter:card" content="summary_large_image" />
<meta name="twitter:title" content="Playground — jsontolang" />
<meta name="twitter:description" content="Paste JSON and generate type definitions live in your browser. No install, no server — runs entirely on WASM." />
<meta name="twitter:image" content="https://jsontolang.dev/og-image.png" />
```

JSON-LD (`BreadcrumbList`):

```json
{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    { "@type": "ListItem", "position": 1, "name": "Home", "item": "https://jsontolang.dev/" },
    { "@type": "ListItem", "position": 2, "name": "Playground", "item": "https://jsontolang.dev/playground" }
  ]
}
```

### Implementation note

`apps/jsontolang-web` uses TanStack Router (file-based), same as `apps/web` — set per-route `<title>`/meta via each route's `head`/`meta` export (TanStack Router's built-in head management), not `react-helmet`. No new dependency needed. Shared tags (viewport, charset, `og:site_name`) go in `routes/__root.tsx`; per-route tags override in each route file.
