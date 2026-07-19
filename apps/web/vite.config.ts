import path from "path";
import { devtools } from "@tanstack/devtools-vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

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
  ],

  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});

