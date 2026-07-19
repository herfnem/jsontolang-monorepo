/**
 * Canonical origin for `<link rel="canonical">` and `og:url`.
 *
 * Placeholder until the real domain is registered — swap this one constant and
 * every route's metadata follows.
 */
export const SITE_URL = "https://jsontolang.dev";

export const SITE_NAME = "jsontolang";

export const TAGLINE = "Paste JSON. Get types.";

/** Builds an absolute URL for a route path, e.g. `canonical("/plugins")`. */
export const canonical = (path: string): string => `${SITE_URL}${path}`;

/**
 * Resolves a path against Vite's base URL.
 *
 * Deploys under a GitHub Pages project subpath, so anything fetched from
 * `public/` must be base-relative — a hard-coded `/plugins/go.lua` 404s there.
 */
export const asset = (path: string): string => `${import.meta.env.BASE_URL}${path}`;
