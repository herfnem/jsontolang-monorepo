import { useRef, type ChangeEvent, type FC } from "react";
import { Download, FileUp, Minimize2, Sparkles } from "lucide-react";
import { Button } from "@workspace/ui/components/button";
import { cn } from "@workspace/ui/lib/utils";
import type { Language } from "@/types/jsontolang";
import { LanguageSelect } from "./language-select";

interface ActionsRailProps {
  onUpload: (file: File) => void;
  onFormat: () => void;
  onMinify: () => void;
  onDownload: () => void;
  downloadDisabled: boolean;
  language: Language;
  languages: readonly Language[];
  onLanguageChange: (language: Language) => void;
  className?: string;
}

export const ActionsRail: FC<ActionsRailProps> = ({
  onUpload,
  onFormat,
  onMinify,
  onDownload,
  downloadDisabled,
  language,
  languages,
  onLanguageChange,
  className,
}) => {
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleFileChange = (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) onUpload(file);
    event.target.value = "";
  };

  return (
    <div
      className={cn(
        "bg-card flex h-full flex-col justify-center gap-4 overflow-y-auto p-4",
        className,
      )}
    >
      <input
        ref={fileInputRef}
        type="file"
        accept=".json,application/json"
        className="hidden"
        onChange={handleFileChange}
      />

      <div className="flex flex-col gap-1.5">
        <span className="text-muted-foreground text-xs font-medium tracking-wide uppercase">
          Convert to
        </span>
        <LanguageSelect
          value={language}
          languages={languages}
          onChange={onLanguageChange}
          className="w-full"
        />
      </div>

      <div className="border-border flex flex-col gap-2 border-t pt-4">
        <Button
          type="button"
          variant="outline"
          className="w-full"
          onClick={() => fileInputRef.current?.click()}
        >
          <FileUp /> Upload data
        </Button>

        <Button type="button" variant="outline" className="w-full" onClick={onFormat}>
          <Sparkles /> Format / Beautify
        </Button>

        <Button type="button" variant="outline" className="w-full" onClick={onMinify}>
          <Minimize2 /> Minify / Compact
        </Button>

        <Button
          type="button"
          variant="outline"
          className="w-full"
          disabled={downloadDisabled}
          onClick={onDownload}
        >
          <Download /> Download
        </Button>
      </div>
    </div>
  );
};
