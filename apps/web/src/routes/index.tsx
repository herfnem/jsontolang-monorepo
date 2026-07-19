import { createFileRoute } from "@tanstack/react-router";
import { HomeView } from "@/views/home";

export const Route = createFileRoute("/")({
  head: () => ({
    meta: [
      { title: "title" },
      {
        name: "description",
        content:
          "description content",
      },
    ],
  }),
  component: HomeView,
});
