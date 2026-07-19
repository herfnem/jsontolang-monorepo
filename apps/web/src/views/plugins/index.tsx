import type { FC } from "react";
import { cn } from "@workspace/ui/lib/utils";
import { BUILTIN_PLUGINS } from "@/libs/plugins";
import { PluginCard } from "./components/plugin-card";

const PLUGIN_CONTRACT = `return {
  key = "typescript",
  render = function(document)
    -- build a string from \`document\` and return it
    return "..."
  end,
}`;

interface PluginsViewProps {
  className?: string;
}

export const PluginsView: FC<PluginsViewProps> = ({ className }) => {
  return (
    <main className={cn("mx-auto w-full max-w-5xl px-6 pb-24", className)}>
      <section className="py-12 sm:py-16">
        <h1 className="text-3xl font-semibold tracking-tight sm:text-4xl">Plugins</h1>
        <p className="text-muted-foreground mt-4 max-w-2xl leading-relaxed">
          A jsontolang plugin is a Lua file, not Rust code. These three ship embedded in the binary,
          so the CLI works with nothing configured — but they are ordinary plugins, and you can read,
          edit, or replace any of them.
        </p>
      </section>

      <section className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {BUILTIN_PLUGINS.map((plugin) => (
          <PluginCard key={plugin.key} plugin={plugin} />
        ))}
      </section>

      <section className="py-16">
        <h2 className="text-2xl font-semibold tracking-tight">Write your own</h2>
        <p className="text-muted-foreground mt-3 max-w-2xl leading-relaxed">
          Drop a <code className="font-mono">.lua</code> file into{" "}
          <code className="font-mono">~/.config/jsontolang/plugins/</code> and its{" "}
          <code className="font-mono">key</code> becomes a valid{" "}
          <code className="font-mono">--lang</code> value. A plugin must return a table with a{" "}
          <code className="font-mono">key</code> string and a{" "}
          <code className="font-mono">render</code> function:
        </p>

        <pre className="border-border bg-card mt-6 overflow-x-auto rounded-lg border p-4 font-mono text-sm leading-relaxed">
          <code>{PLUGIN_CONTRACT}</code>
        </pre>

        <p className="text-muted-foreground mt-6 max-w-2xl text-sm leading-relaxed">
          <strong className="text-foreground font-medium">Note:</strong> once that directory contains
          at least one usable plugin, the embedded defaults are no longer offered — the two sources
          are never merged. Copy the files above into it if you want to keep them alongside your own.
        </p>

        <p className="text-muted-foreground mt-4 max-w-2xl text-sm leading-relaxed">
          Plugins run in a sandboxed Lua 5.4 runtime with only{" "}
          <code className="font-mono">string</code>, <code className="font-mono">table</code>, and{" "}
          <code className="font-mono">math</code> loaded — no filesystem, process, or OS access. That
          also means custom plugins are CLI-only: the browser playground below runs native renderers
          for the three built-in languages instead.
        </p>
      </section>
    </main>
  );
};
