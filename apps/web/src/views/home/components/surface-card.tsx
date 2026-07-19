import type { FC, ReactNode } from "react";
import { cn } from "@workspace/ui/lib/utils";

interface SurfaceCardProps {
  title: string;
  status: string;
  children: ReactNode;
  className?: string;
}

/** One of jsontolang's three surfaces: CLI, web playground, TUI. */
export const SurfaceCard: FC<SurfaceCardProps> = ({ title, status, children, className }) => {
  return (
    <div className={cn("border-border bg-card rounded-lg border p-5", className)}>
      <div className="flex items-baseline justify-between gap-3">
        <h3 className="font-semibold">{title}</h3>
        <span className="text-muted-foreground font-mono text-xs">{status}</span>
      </div>
      <p className="text-muted-foreground mt-2 text-sm leading-relaxed">{children}</p>
    </div>
  );
};
