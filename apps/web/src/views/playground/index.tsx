import type { FC } from "react";
import { cn } from "@workspace/ui/lib/utils";

interface PlaygroundViewProps {
  className?: string;
}

export const PlaygroundView: FC<PlaygroundViewProps> = ({ className }) => {
  return (
    <div className={cn("", className)}>
      <h1>Playground</h1>
      <p>Paste JSON, get types.</p>
    </div>
  );
};
