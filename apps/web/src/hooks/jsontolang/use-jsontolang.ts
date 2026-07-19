import { useEffect, useState } from "react";
import init, { render_types } from "jsontolang-wasm";
import { asset } from "@/libs/site";
import type { Language } from "@/types/jsontolang";

/**
 * Module-level so the WASM module is instantiated once per page load no matter
 * how many components ask for it. `init()` is not idempotent — calling it again
 * re-fetches and re-instantiates.
 */
let instantiation: Promise<void> | null = null;

const ensureInstantiated = (): Promise<void> => {
  instantiation ??= init({
    module_or_path: asset("jsontolang_wasm_bg.wasm"),
  }).then(() => undefined);

  return instantiation;
};

export interface Jsontolang {
  /** False until the WASM module has finished instantiating. */
  ready: boolean;
  /** Set if instantiation itself failed, e.g. the `.wasm` file 404'd. */
  loadError: string | null;
  /**
   * Renders `json` as `language` source text, or returns the error message
   * jsontolang produced. Returns `null` before the module is ready.
   *
   * Errors are returned rather than thrown: invalid JSON is the *expected*
   * state while someone is typing, not an exception.
   */
  renderTypes: (
    root: string,
    json: string,
    language: Language,
  ) => { output: string; error: null } | { output: null; error: string } | null;
}

/** Loads the jsontolang WASM module and exposes its renderer. */
export const useJsontolang = (): Jsontolang => {
  const [ready, setReady] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;

    ensureInstantiated()
      .then(() => {
        if (active) setReady(true);
      })
      .catch((error: unknown) => {
        if (active) setLoadError(error instanceof Error ? error.message : String(error));
      });

    return () => {
      active = false;
    };
  }, []);

  const renderTypes: Jsontolang["renderTypes"] = (root, json, language) => {
    if (!ready) return null;

    try {
      return { output: render_types(root, json, language), error: null };
    } catch (error: unknown) {
      return { output: null, error: error instanceof Error ? error.message : String(error) };
    }
  };

  return { ready, loadError, renderTypes };
};
