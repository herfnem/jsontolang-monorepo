import type { FC } from "react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@workspace/ui/components/select";
import { cn } from "@workspace/ui/lib/utils";
import { BUILTIN_PLUGINS } from "@/libs/plugins";
import { isLanguage, type Language } from "@/types/jsontolang";

const LABELS: Record<string, string> = Object.fromEntries(
  BUILTIN_PLUGINS.map((plugin) => [plugin.key, plugin.title]),
);

interface LanguageSelectProps {
  value: Language;
  languages: readonly Language[];
  onChange: (language: Language) => void;
  className?: string;
}

export const LanguageSelect: FC<LanguageSelectProps> = ({
  value,
  languages,
  onChange,
  className,
}) => {
  return (
    <Select
      value={value}
      items={LABELS}
      onValueChange={(next) => {
        if (next !== null && isLanguage(next)) onChange(next);
      }}
    >
      <SelectTrigger
        size="sm"
        aria-label="Output language"
        className={cn("border-foreground text-xs", className)}
      >
        <SelectValue />
      </SelectTrigger>
      <SelectContent className="ring-foreground shadow-none ring-2">
        {languages.map((language) => (
          <SelectItem key={language} value={language} className="text-xs">
            {LABELS[language]}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
};
