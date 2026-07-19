import type { FC } from "react";
import { cn } from "@workspace/ui/lib/utils";

interface HomeViewProps {
  className?: string;
}

export const HomeView: FC<HomeViewProps> = ({ className }) => {
  return (    <div className={cn("", className)}>
      HomeView
    </div>
  );
};
