import type { FC } from "react";
import { Button } from "@workspace/ui/components/button";
import { cn } from "@workspace/ui/lib/utils";
import { asset } from "@/libs/site";
import type { PluginMeta } from "@/types/jsontolang";

interface PluginCardProps {
  plugin: PluginMeta;
  className?: string;
}

export const PluginCard: FC<PluginCardProps> = ({ plugin, className }) => {
  const href = asset(`lua/${plugin.file}`);

  return (
    <article className={cn("border-border bg-card flex flex-col rounded-lg border p-5", className)}>
      <header className="flex items-baseline justify-between gap-3">
        <h2 className="text-lg font-semibold">{plugin.title}</h2>
        <code className="text-muted-foreground font-mono text-xs">--lang {plugin.key}</code>
      </header>

      <p className="text-muted-foreground mt-3 flex-1 text-sm leading-relaxed">
        {plugin.description}
      </p>

      <pre className="border-border bg-background mt-4 overflow-x-auto rounded-md border p-3 font-mono text-xs leading-relaxed">
        <code>{plugin.sample}</code>
      </pre>

      <footer className="mt-4 flex items-center gap-3">
        <Button
          size="sm"
          variant="outline"
          nativeButton={false}
          render={<a href={href} download={plugin.file} />}
        >
          Download {plugin.file}
        </Button>
        <a
          href={href}
          className="text-muted-foreground hover:text-foreground text-sm underline-offset-4 hover:underline"
        >
          View source
        </a>
      </footer>
    </article>
  );
};
