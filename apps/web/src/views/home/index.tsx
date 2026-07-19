import type { FC } from "react";
import { Link } from "@tanstack/react-router";
import { Globe, SquareTerminal, Terminal } from "lucide-react";
import { Button } from "@workspace/ui/components/button";
import { cn } from "@workspace/ui/lib/utils";
import { TAGLINE } from "@/libs/site";
import { CodeBlock } from "./components/code-block";
import { LiveDemo } from "./components/live-demo";
import { SurfaceCard } from "./components/surface-card";

const BADGES = ["Rust, compiled to WebAssembly", "Nothing leaves your browser"] as const;

interface HomeViewProps {
  className?: string;
}

export const HomeView: FC<HomeViewProps> = ({ className }) => {
  return (
    <main className={cn("mx-auto w-full max-w-5xl px-6 pb-24", className)}>
      <section className="relative overflow-hidden py-16 sm:py-24">
        <div
          aria-hidden
          className="bg-primary/25 pointer-events-none absolute top-0 left-1/2 -z-10 size-[36rem] -translate-x-1/2 -translate-y-1/2 rounded-full blur-3xl"
        />

        <div className="flex flex-wrap items-center gap-2">
          {BADGES.map((badge) => (
            <span
              key={badge}
              className="border-border text-muted-foreground rounded-full border px-3 py-1 text-xs"
            >
              {badge}
            </span>
          ))}
        </div>

        <h1 className="mt-5 text-4xl font-semibold tracking-tight sm:text-5xl">{TAGLINE}</h1>
        <p className="text-muted-foreground mt-5 max-w-2xl text-lg leading-relaxed">
          jsontolang infers a schema from any JSON and generates matching type definitions for
          TypeScript, Rust, Go, or a custom Lua plugin of your own. One core, three surfaces.
        </p>
        <div className="mt-8 flex flex-wrap gap-3">
          <Button nativeButton={false} render={<Link to="/playground" />}>
            Open the playground
          </Button>
          <Button nativeButton={false} variant="outline" render={<Link to="/plugins" />}>
            Browse plugins
          </Button>
        </div>
      </section>

      <section>
        <div className="flex items-baseline justify-between gap-3">
          <h2 className="text-2xl font-semibold tracking-tight">Try it right here</h2>
          <Link
            to="/playground"
            className="text-muted-foreground hover:text-foreground text-sm whitespace-nowrap underline-offset-4 hover:underline"
          >
            Open full playground →
          </Link>
        </div>
        <LiveDemo className="mt-6" />
      </section>

      <section className="py-16">
        <h2 className="text-2xl font-semibold tracking-tight">Three surfaces, one core</h2>
        <div className="mt-6 grid gap-4 sm:grid-cols-3">
          <SurfaceCard title="CLI" status="available" icon={<Terminal />}>
            The full tool. Reads from a file, stdin, or an inline string, and renders through
            sandboxed Lua plugins — including any you write yourself.
          </SurfaceCard>
          <SurfaceCard title="Web" status="you are here" icon={<Globe />} current>
            The playground runs the same schema inference compiled to WebAssembly. Nothing is
            uploaded; every keystroke is rendered in your browser.
          </SurfaceCard>
          <SurfaceCard title="TUI" status="planned" icon={<SquareTerminal />}>
            An interactive terminal front end over the same core. Not built yet — the crate layout
            already leaves room for it.
          </SurfaceCard>
        </div>
      </section>

      <section>
        <h2 className="text-2xl font-semibold tracking-tight">Try it from a terminal</h2>
        <p className="text-muted-foreground mt-3 max-w-2xl leading-relaxed">
          Exactly one input source is required — <code className="font-mono">--json</code>,{" "}
          <code className="font-mono">--file</code>, or <code className="font-mono">--stdin</code>.
          Use <code className="font-mono">--root</code> to name the generated root type; it defaults
          to <code className="font-mono">Root</code>.
        </p>
        <CodeBlock className="mt-6" label="shell">
          {`# Inline JSON
jsontolang --lang typescript --json '{"name":"Neko"}'

# From a file, into Rust
jsontolang --lang rust --file ./example.json

# From a pipe, into Go
curl -s https://api.example.com/user | jsontolang --lang go --stdin`}
        </CodeBlock>
      </section>
    </main>
  );
};
