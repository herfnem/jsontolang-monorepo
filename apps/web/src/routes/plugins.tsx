import { createFileRoute } from "@tanstack/react-router";
import { canonical } from "@/libs/site";
import { PluginsView } from "@/views/plugins";

const TITLE = "Plugins — jsontolang";
const DESCRIPTION =
  "Browse and download jsontolang's Lua plugins for TypeScript, Rust, and Go — or write your own to target any language.";

const BREADCRUMB_LD = {
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  itemListElement: [
    { "@type": "ListItem", position: 1, name: "Home", item: canonical("/") },
    { "@type": "ListItem", position: 2, name: "Plugins", item: canonical("/plugins") },
  ],
};

export const Route = createFileRoute("/plugins")({
  head: () => ({
    meta: [
      { title: TITLE },
      { name: "description", content: DESCRIPTION },
      { property: "og:title", content: TITLE },
      { property: "og:description", content: DESCRIPTION },
      { property: "og:url", content: canonical("/plugins") },
      { name: "twitter:title", content: TITLE },
      { name: "twitter:description", content: DESCRIPTION },
    ],
    links: [{ rel: "canonical", href: canonical("/plugins") }],
    scripts: [{ type: "application/ld+json", children: JSON.stringify(BREADCRUMB_LD) }],
  }),
  component: PluginsView,
});
