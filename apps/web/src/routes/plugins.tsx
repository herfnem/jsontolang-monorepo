import { createFileRoute } from "@tanstack/react-router";
import { PluginsView } from "@/views/plugins";

export const Route = createFileRoute("/plugins")({
  head: () => ({
    meta: [
      { title: "Plugins — jsontolang" },
      {
        name: "description",
        content:
          "Browse and download jsontolang's Lua plugins for TypeScript, Rust, and Go — or write your own to target any language.",
      },
      { property: "og:title", content: "Plugins — jsontolang" },
      {
        property: "og:description",
        content:
          "Browse and download jsontolang's Lua plugins for TypeScript, Rust, and Go — or write your own to target any language.",
      },
      { name: "twitter:title", content: "Plugins — jsontolang" },
      {
        name: "twitter:description",
        content:
          "Browse and download jsontolang's Lua plugins for TypeScript, Rust, and Go — or write your own to target any language.",
      },
    ],
  }),
  component: PluginsView,
});
