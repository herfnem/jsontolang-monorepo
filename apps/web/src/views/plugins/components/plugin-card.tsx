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
    <article className={cn("border-foreground flex flex-col border-2 p-5", className)}>
      <header className="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1">
        <h2 className="font-display font-stretch-75% text-3xl leading-none font-extrabold tracking-[-0.03em]">
          {plugin.title}
        </h2>
        <code className="text-muted-foreground font-mono text-xs whitespace-nowrap">--lang {plugin.key}</code>
      </header>

      <p className="text-muted-foreground mt-3 flex-1 text-sm leading-relaxed">
        {plugin.description}
      </p>

      <pre className="border-foreground mt-4 overflow-x-auto border p-3 font-mono text-xs leading-relaxed">
        <code>{plugin.sample}</code>
      </pre>

      <footer className="mt-4 flex flex-wrap items-center gap-3">
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
          className="hover:text-primary text-sm font-medium whitespace-nowrap underline decoration-2 underline-offset-4 transition-colors"
        >
          View source
        </a>
      </footer>
    </article>
  );
};
