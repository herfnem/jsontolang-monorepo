import type { FC } from "react";
import { cn } from "@workspace/ui/lib/utils";

interface PluginsViewProps {
  className?: string;
}

export const PluginsView: FC<PluginsViewProps> = ({ className }) => {
  return (
    <div className={cn("", className)}>
      <h1>Plugins</h1>
      <p>Browse and download language plugins.</p>
    </div>
  );
};
