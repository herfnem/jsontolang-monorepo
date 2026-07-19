import type { PluginMeta } from "@/types/jsontolang";

/**
 * The three built-in plugins, hand-maintained.
 *
 * There is no registry or CMS for v1 — the `.lua` files themselves are copied
 * into `public/lua/` by the app's `sync:plugins` build step, and this array
 * is the display metadata that wraps them. Adding a fourth built-in means
 * adding it here, in `crates/cli/plugins/`, and in `LANGUAGES`.
 */
export const BUILTIN_PLUGINS: readonly PluginMeta[] = [
  {
    key: "typescript",
    title: "TypeScript",
    description:
      "Exported interfaces with optional properties, quoted keys where an identifier won't do, and a type alias when the root is an array.",
    file: "typescript.lua",
    outputExtension: "ts",
    sample: "export interface Root {\n  name: string;\n}",
  },
  {
    key: "rust",
    title: "Rust",
    description:
      "Serde-derived structs, `Option<T>` for fields that aren't always present, `#[serde(rename)]` whenever the JSON key can't be a Rust identifier, and raw identifiers for keywords.",
    file: "rust.lua",
    outputExtension: "rs",
    sample: "pub struct Root {\n    pub name: String,\n}",
  },
  {
    key: "go",
    title: "Go",
    description:
      "Structs with `json:` tags, pointer types for optional fields, and generated `MarshalJSON`/`UnmarshalJSON` for keys that a struct tag simply cannot express.",
    file: "go.lua",
    outputExtension: "go",
    sample: 'type Root struct {\n\tName string `json:"name"`\n}',
  },
];
