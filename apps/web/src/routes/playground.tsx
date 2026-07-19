import { createFileRoute } from "@tanstack/react-router";
import { PlaygroundView } from "@/views/playground";

export const Route = createFileRoute("/playground")({
  head: () => ({
    meta: [
      { title: "Playground — jsontolang" },
      {
        name: "description",
        content:
          "Paste JSON and generate type definitions live in your browser. No install, no server — runs entirely on WASM.",
      },
      { property: "og:title", content: "Playground — jsontolang" },
      {
        property: "og:description",
        content:
          "Paste JSON and generate type definitions live in your browser. No install, no server — runs entirely on WASM.",
      },
      { name: "twitter:title", content: "Playground — jsontolang" },
      {
        name: "twitter:description",
        content:
          "Paste JSON and generate type definitions live in your browser. No install, no server — runs entirely on WASM.",
      },
    ],
  }),
  component: PlaygroundView,
});
