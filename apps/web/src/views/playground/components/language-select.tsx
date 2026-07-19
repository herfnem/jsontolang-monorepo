import type { FC } from "react";
import { cn } from "@workspace/ui/lib/utils";
import { isLanguage, type Language } from "@/types/jsontolang";

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
    <label className="flex items-center gap-2">
      <span className="sr-only">Target language</span>
      <select
        value={value}
        onChange={(event) => {
          const next = event.target.value;
          if (isLanguage(next)) onChange(next);
        }}
        className={cn(
          "border-border bg-background focus-visible:border-ring focus-visible:ring-ring/50 h-7 rounded-md border px-2 text-sm outline-none focus-visible:ring-3",
          className,
        )}
      >
        {languages.map((language) => (
          <option key={language} value={language}>
            {language}
          </option>
        ))}
      </select>
    </label>
  );
};
