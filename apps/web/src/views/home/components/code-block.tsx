import { Fragment, type FC } from "react";
import { cn } from "@workspace/ui/lib/utils";

interface CodeBlockProps {
  children: string;
  label?: string;
  className?: string;
}

export const CodeBlock: FC<CodeBlockProps> = ({ children, label, className }) => {
  return (
    <div className={cn("border-background/25 border-2", className)}>
      {label && (
        <div className="border-background/25 text-background/60 border-b px-4 py-2 font-mono text-xs">
          {label}
        </div>
      )}
      <pre className="overflow-x-auto p-4 font-mono text-sm leading-relaxed sm:p-6">
        <code>
          {children.split("\n").map((line, index) => (
            <Fragment key={index}>
              {index > 0 && "\n"}
              <span className={cn(line.startsWith("#") && "text-background/60")}>{line}</span>
            </Fragment>
          ))}
        </code>
      </pre>
    </div>
  );
};
