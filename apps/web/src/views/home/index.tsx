import type { FC } from "react";
import { Link } from "@tanstack/react-router";
import { Button } from "@workspace/ui/components/button";
import { cn } from "@workspace/ui/lib/utils";
import { TAGLINE } from "@/libs/site";
import { CodeBlock } from "./components/code-block";
import { LiveDemo } from "./components/live-demo";
import { SurfaceCard } from "./components/surface-card";

const FACTS = ["Rust, compiled to WebAssembly", "Nothing leaves your browser"] as const;

const CONTAINER = "mx-auto w-full max-w-5xl px-6";

const SECTION_HEADING =
  "font-display font-stretch-75% text-5xl leading-[0.9] font-extrabold tracking-[-0.03em] sm:text-7xl";

const TEXT_LINK =
  "hover:text-primary font-medium whitespace-nowrap underline decoration-2 underline-offset-4 transition-colors";

interface HomeViewProps {
  className?: string;
}

export const HomeView: FC<HomeViewProps> = ({ className }) => {
  return (
    <main className={cn("w-full", className)}>
      <section className={cn(CONTAINER, "pt-12 pb-12 sm:pt-20 sm:pb-16")}>
        <h1 className="font-display font-stretch-75% min-w-0 text-[clamp(3rem,22vw,17rem)] leading-[0.86] font-extrabold tracking-[-0.03em] [overflow-wrap:anywhere]">
          {TAGLINE.split(/(?<=\.)\s/).map((line, index) => (
            <span key={line} className={cn("block", index > 0 && "text-primary")}>
              {line}
            </span>
          ))}
        </h1>

        <div className="mt-10 grid grid-cols-1 gap-8 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
          <div>
            <p className="text-muted-foreground max-w-[52ch] text-lg leading-relaxed">
              jsontolang infers a schema from any JSON and generates matching type definitions
              for TypeScript, Rust, Go, or a custom Lua plugin of your own. One core, three
              surfaces.
            </p>
            <ul className="mt-5 flex flex-wrap gap-x-6 gap-y-2 text-sm">
              {FACTS.map((fact) => (
                <li key={fact} className="flex items-center gap-2">
                  <span aria-hidden className="bg-primary size-1.5 shrink-0" />
                  {fact}
                </li>
              ))}
            </ul>
          </div>
          <div className="flex flex-wrap items-center gap-x-6 gap-y-4">
            <Button
              nativeButton={false}
              render={<Link to="/playground" />}
              className="h-12 px-6 text-base"
            >
              Open the playground
            </Button>
            <Link to="/plugins" className={cn(TEXT_LINK, "text-base")}>
              Browse plugins
            </Link>
          </div>
        </div>
      </section>

      <section className="border-border border-t">
        <div className={cn(CONTAINER, "py-16 sm:py-24")}>
          <div className="flex flex-wrap items-end justify-between gap-x-6 gap-y-4">
            <h2 className={SECTION_HEADING}>Try it right here</h2>
            <Link to="/playground" className={cn(TEXT_LINK, "text-sm")}>
              Open full playground →
            </Link>
          </div>
          <LiveDemo className="mt-10" />
        </div>
      </section>

      <section className="border-border border-t pt-16 sm:pt-24">
        <h2 className={cn(SECTION_HEADING, CONTAINER)}>Three surfaces, one core</h2>
        <ul className="mt-10">
          <SurfaceCard title="CLI" status="Available">
            The full tool. Reads from a file, stdin, or an inline string, and renders through
            sandboxed Lua plugins — including any you write yourself.
          </SurfaceCard>
          <SurfaceCard title="Web" status="You are here" current>
            The playground runs the same schema inference compiled to WebAssembly. Nothing is
            uploaded; every keystroke is rendered in your browser.
          </SurfaceCard>
          <SurfaceCard title="TUI" status="Available">
            A two-pane vim-style editor: JSON on the left, generated types on the right,
            re-rendered on every keystroke. hjkl navigation, insert mode, yank/paste, and system
            clipboard copy — no mouse required.
          </SurfaceCard>
        </ul>
      </section>

      <section className="bg-foreground text-background">
        <div className={cn(CONTAINER, "grid grid-cols-1 gap-10 py-16 sm:py-24")}>
          <div>
            <h2 className={SECTION_HEADING}>Try it from a terminal</h2>
            <p className="text-background/70 mt-6 max-w-[52ch] leading-relaxed">
              Exactly one input source is required —{" "}
              <code className="text-background font-mono">--json</code>,{" "}
              <code className="text-background font-mono">--file</code>, or{" "}
              <code className="text-background font-mono">--stdin</code>. Use{" "}
              <code className="text-background font-mono">--root</code> to name the generated root
              type; it defaults to <code className="text-background font-mono">Root</code>.
            </p>
          </div>
          <CodeBlock label="shell">
            {`# Inline JSON
jsontolang --lang typescript --json '{"name":"Neko"}'

# From a file, into Rust
jsontolang --lang rust --file ./example.json

# From a pipe, into Go
curl -s https://api.example.com/user | jsontolang --lang go --stdin`}
          </CodeBlock>
        </div>
      </section>
    </main>
  );
};
