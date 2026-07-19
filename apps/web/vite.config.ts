import path from "path";
import { devtools } from "@tanstack/devtools-vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import { VitePWA } from "vite-plugin-pwa";

// Project GitHub Pages serves from `/<repo>/`, not the domain root, so the
// deploy workflow sets BASE_PATH. Read from the environment rather than passed
// as `vite build --base=...`: the `build` script is a `&&` chain, and appended
// pnpm args would land on whichever command happens to be last.
//
// Everything downstream follows from this one value — `router.ts` takes its
// basepath from `import.meta.env.BASE_URL`, and `libs/site.ts`'s `asset()`
// resolves the `.wasm` and `.lua` files against it.
const base = process.env.BASE_PATH ?? "/";

// https://vite.dev/config/
export default defineConfig({
  base,

  plugins: [
    devtools(),
    tanstackRouter({
      target: "react",
      autoCodeSplitting: true,
      routesDirectory: "./src/routes",
      generatedRouteTree: "./src/routeTree.gen.ts",
    }),
    react(),
    tailwindcss(),
    VitePWA({
      registerType: "autoUpdate",
      includeAssets: ["favicon.svg", "apple-touch-icon.png", "og-image.png", "og-image.svg"],
      manifest: {
        name: "jsontolang",
        short_name: "jsontolang",
        description:
          "Infer a schema from any JSON and generate TypeScript, Rust, or Go types instantly in your browser.",
        // Relative — resolves against wherever the manifest itself is served
        // from, so it survives the GitHub Pages `/<repo>/` subpath without
        // needing to know `base` at manifest-authoring time.
        start_url: ".",
        display: "standalone",
        background_color: "#0f172a",
        theme_color: "#0f172a",
        icons: [
          { src: "icon-192.png", sizes: "192x192", type: "image/png" },
          { src: "icon-512.png", sizes: "512x512", type: "image/png" },
        ],
      },
      workbox: {
        // The WASM binary and Lua plugin sources are fetched at runtime, not
        // pulled in through the JS import graph, so Workbox's default
        // js/css/html glob won't see them — list the extensions explicitly.
        globPatterns: ["**/*.{js,css,html,wasm,svg,png,ico,lua}"],
      },
    }),
  ],

  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});

