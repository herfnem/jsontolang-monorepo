import { createFileRoute } from "@tanstack/react-router";
import { canonical, SITE_URL } from "@/libs/site";
import { HomeView } from "@/views/home";

const TITLE = "jsontolang — JSON to Types, Instantly";
const DESCRIPTION =
  "Infer a schema from any JSON and generate TypeScript, Rust, or Go types instantly in your browser, powered by WASM. Free CLI and plugin system included.";
const SOCIAL_DESCRIPTION =
  "Infer a schema from any JSON and generate TypeScript, Rust, or Go types instantly in your browser, powered by WASM.";

/** Search engines read this to surface jsontolang as an installable free tool. */
const SOFTWARE_APPLICATION_LD = {
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  name: "jsontolang",
  description:
    "Infers a schema from JSON and generates type definitions for TypeScript, Rust, Go, or custom Lua plugins.",
  applicationCategory: "DeveloperApplication",
  operatingSystem: "Web, macOS, Linux, Windows",
  url: SITE_URL,
  offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
};

export const Route = createFileRoute("/")({
  head: () => ({
    meta: [
      { title: TITLE },
      { name: "description", content: DESCRIPTION },
      { property: "og:title", content: TITLE },
      { property: "og:description", content: SOCIAL_DESCRIPTION },
      { property: "og:url", content: canonical("/") },
      { name: "twitter:title", content: TITLE },
      { name: "twitter:description", content: SOCIAL_DESCRIPTION },
    ],
    links: [{ rel: "canonical", href: canonical("/") }],
    scripts: [
      {
        type: "application/ld+json",
        children: JSON.stringify(SOFTWARE_APPLICATION_LD),
      },
    ],
  }),
  component: HomeView,
});
