import type { FC } from "react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@workspace/ui/components/select";
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
    <Select
      value={value}
      onValueChange={(next) => {
        if (next !== null && isLanguage(next)) onChange(next);
      }}
    >
      <SelectTrigger className={className}>
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {languages.map((language) => (
          <SelectItem key={language} value={language}>
            {language}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
};
