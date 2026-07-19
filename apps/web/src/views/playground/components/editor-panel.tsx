import type { FC, ReactNode } from "react";
import { cn } from "@workspace/ui/lib/utils";

interface EditorPanelProps {
  title: string;
  /** Rendered on the panel's header row, e.g. a language picker. */
  actions?: ReactNode;
  children: ReactNode;
  className?: string;
}

export const EditorPanel: FC<EditorPanelProps> = ({ title, actions, children, className }) => {
  return (
    <section className={cn("flex h-full min-h-0 flex-col overflow-hidden", className)}>
      <header className="border-border flex h-11 shrink-0 items-center justify-between gap-3 border-b px-3">
        <h2 className="text-muted-foreground text-xs font-medium tracking-wide uppercase">
          {title}
        </h2>
        {actions}
      </header>
      <div className="min-h-0 flex-1">{children}</div>
    </section>
  );
};
