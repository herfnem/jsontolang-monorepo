import type { FC } from "react";
import { cn } from "@workspace/ui/lib/utils";

interface CodeBlockProps {
  children: string;
  label?: string;
  className?: string;
}

export const CodeBlock: FC<CodeBlockProps> = ({ children, label, className }) => {
  return (
    <div className={cn("border-border bg-card overflow-hidden rounded-lg border", className)}>
      {label && (
        <div className="border-border text-muted-foreground border-b px-4 py-2 font-mono text-xs">
          {label}
        </div>
      )}
      <pre className="overflow-x-auto p-4 font-mono text-sm leading-relaxed">
        <code>{children}</code>
      </pre>
    </div>
  );
};
