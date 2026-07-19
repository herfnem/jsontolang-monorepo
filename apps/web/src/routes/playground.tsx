import { createFileRoute } from "@tanstack/react-router";
import { canonical } from "@/libs/site";
import { PlaygroundView } from "@/views/playground";

const TITLE = "Playground — jsontolang";
const DESCRIPTION =
  "Paste JSON and generate type definitions live in your browser. No install, no server — runs entirely on WASM.";

const BREADCRUMB_LD = {
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  itemListElement: [
    { "@type": "ListItem", position: 1, name: "Home", item: canonical("/") },
    { "@type": "ListItem", position: 2, name: "Playground", item: canonical("/playground") },
  ],
};

export const Route = createFileRoute("/playground")({
  head: () => ({
    meta: [
      { title: TITLE },
      { name: "description", content: DESCRIPTION },
      { property: "og:title", content: TITLE },
      { property: "og:description", content: DESCRIPTION },
      { property: "og:url", content: canonical("/playground") },
      { name: "twitter:title", content: TITLE },
      { name: "twitter:description", content: DESCRIPTION },
    ],
    links: [{ rel: "canonical", href: canonical("/playground") }],
    scripts: [{ type: "application/ld+json", children: JSON.stringify(BREADCRUMB_LD) }],
  }),
  component: PlaygroundView,
});
