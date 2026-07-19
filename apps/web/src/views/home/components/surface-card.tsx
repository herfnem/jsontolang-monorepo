import type { FC, ReactNode } from "react";
import { cn } from "@workspace/ui/lib/utils";

interface SurfaceCardProps {
  title: string;
  status: string;
  icon: ReactNode;
  /** True for the surface the user is currently on. */
  current?: boolean;
  children: ReactNode;
  className?: string;
}

/** One of jsontolang's three surfaces: CLI, web playground, TUI. */
export const SurfaceCard: FC<SurfaceCardProps> = ({
  title,
  status,
  icon,
  current,
  children,
  className,
}) => {
  return (
    <div
      className={cn(
        "bg-card rounded-lg border p-5",
        current ? "border-primary/40" : "border-border",
        className,
      )}
    >
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-2.5">
          <div
            className={cn(
              "flex size-8 shrink-0 items-center justify-center rounded-md [&_svg]:size-4",
              current ? "bg-primary/10 text-primary" : "bg-muted text-muted-foreground",
            )}
          >
            {icon}
          </div>
          <h3 className="font-semibold">{title}</h3>
        </div>
        <span
          className={cn(
            "font-mono text-xs",
            current ? "text-primary" : "text-muted-foreground",
          )}
        >
          {status}
        </span>
      </div>
      <p className="text-muted-foreground mt-3 text-sm leading-relaxed">{children}</p>
    </div>
  );
};
