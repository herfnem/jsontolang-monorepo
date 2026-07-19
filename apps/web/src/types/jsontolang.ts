/**
 * Target languages the WASM build can render.
 *
 * Mirrors `jsontolang_core::render::BUILTIN_LANGS`. The wasm boundary is
 * untyped — `builtin_languages()` comes back as `string[]` — so this is the
 * hand-authored source of truth the app narrows against.
 */
export const LANGUAGES = ["typescript", "rust", "go"] as const;

export type Language = (typeof LANGUAGES)[number];

export const isLanguage = (value: string): value is Language =>
  (LANGUAGES as readonly string[]).includes(value);

/** Display metadata for one built-in Lua plugin, shown on `/plugins`. */
export interface PluginMeta {
  /** Matches the plugin's `key`, i.e. what `--lang` expects. */
  key: Language;
  title: string;
  description: string;
  /** File name under `public/lua/`, synced from `crates/cli/plugins/`. */
  file: string;
  /** Extension of the code this plugin emits, for the card's example. */
  outputExtension: string;
  /** A representative line of that plugin's output. */
  sample: string;
}
