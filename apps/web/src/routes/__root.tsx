import { createRootRoute, HeadContent, Outlet } from "@tanstack/react-router";
import { TanStackDevtools } from "@tanstack/react-devtools";
import { TanStackRouterDevtoolsPanel } from "@tanstack/react-router-devtools";

const SITE_NAME = "site name";
const DESCRIPTION =
  "site description";

export const Route = createRootRoute({
  head: () => ({
    meta: [
      { title: SITE_NAME},
      { name: "description", content: DESCRIPTION },
    ],
  }),
  component: RootLayout,
});

function RootLayout() {
  return (
    <>
      <HeadContent />
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
