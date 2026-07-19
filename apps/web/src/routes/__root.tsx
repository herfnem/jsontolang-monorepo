import { createRootRoute, HeadContent, Outlet } from "@tanstack/react-router";
import { TanStackDevtools } from "@tanstack/react-devtools";
import { TanStackRouterDevtoolsPanel } from "@tanstack/react-router-devtools";
import { Nav } from "@/components/nav";
import { canonical, SITE_NAME } from "@/libs/site";

const DESCRIPTION =
  "Infer a schema from any JSON and generate TypeScript, Rust, or Go types instantly in your browser, powered by WASM.";
// Absolute: crawlers and social scrapers don't resolve relative OG images.
const OG_IMAGE = canonical("/og-image.png");

export const Route = createRootRoute({
  head: () => ({
    meta: [
      { title: SITE_NAME },
      { name: "description", content: DESCRIPTION },
      { property: "og:site_name", content: SITE_NAME },
      { property: "og:type", content: "website" },
      { property: "og:image", content: OG_IMAGE },
      { name: "twitter:card", content: "summary_large_image" },
      { name: "twitter:image", content: OG_IMAGE },
    ],
  }),
  component: RootLayout,
});

function RootLayout() {
  return (
    <>
      <HeadContent />
      <Nav />
      <Outlet />
      <TanStackDevtools
        plugins={[
          {
            name: "TanStack Router",
            render: <TanStackRouterDevtoolsPanel />,
          },
          // Custom product devtools mount here too, e.g.:
          // { name: "EPUB Engine", render: <EpubDevtoolsPanel /> },
        ]}
      />
    </>
  );
}
