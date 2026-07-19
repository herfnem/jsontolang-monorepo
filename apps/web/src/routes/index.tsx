import { createFileRoute } from "@tanstack/react-router";
import { HomeView } from "@/views/home";

export const Route = createFileRoute("/")({
  head: () => ({
    meta: [
      { title: "jsontolang — JSON to Types, Instantly" },
      {
        name: "description",
        content:
          "Infer a schema from any JSON and generate TypeScript, Rust, or Go types instantly in your browser, powered by WASM. Free CLI and plugin system included.",
      },
      { property: "og:title", content: "jsontolang — JSON to Types, Instantly" },
      {
        property: "og:description",
        content:
          "Infer a schema from any JSON and generate TypeScript, Rust, or Go types instantly in your browser, powered by WASM.",
      },
      { name: "twitter:title", content: "jsontolang — JSON to Types, Instantly" },
      {
        name: "twitter:description",
        content:
          "Infer a schema from any JSON and generate TypeScript, Rust, or Go types instantly in your browser, powered by WASM.",
      },
    ],
  }),
  component: HomeView,
});
