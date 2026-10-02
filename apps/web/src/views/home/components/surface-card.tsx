import type { FC, ReactNode } from "react";
import { cn } from "@workspace/ui/lib/utils";

interface SurfaceCardProps {
  title: string;
  status: string;
  /** True for the surface the user is currently on. */
  current?: boolean;
  children: ReactNode;
  className?: string;
}

/** One of jsontolang's three surfaces: CLI, web playground, TUI. */
export const SurfaceCard: FC<SurfaceCardProps> = ({
  title,
  status,
  current,
  children,
  className,
}) => {
  return (
    <li className={cn("border-border border-t", className)}>
      <div className="mx-auto grid w-full max-w-5xl grid-cols-1 gap-x-10 gap-y-3 px-6 py-8 md:grid-cols-[minmax(0,13rem)_minmax(0,1fr)_minmax(0,8rem)] md:items-baseline">
        <h3
          className={cn(
            "font-display font-stretch-75% text-7xl leading-none font-extrabold tracking-[-0.03em] uppercase sm:text-8xl",
            current && "text-primary",
          )}
        >
          {title}
        </h3>
        <p className="text-muted-foreground max-w-[60ch] leading-relaxed">{children}</p>
        <span
          className={cn(
            "text-sm font-medium md:text-right",
            current ? "text-primary" : "text-muted-foreground",
          )}
        >
          {status}
        </span>
      </div>
    </li>
  );
};
