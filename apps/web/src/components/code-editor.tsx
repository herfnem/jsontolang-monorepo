import { useMemo, useState, type FC, type ReactNode } from "react";
import CodeMirror, { EditorView, type Extension, type Statistics } from "@uiw/react-codemirror";
import { json } from "@codemirror/lang-json";
import { javascript } from "@codemirror/lang-javascript";
import { rust } from "@codemirror/lang-rust";
import { StreamLanguage } from "@codemirror/language";
import { go } from "@codemirror/legacy-modes/mode/go";
import { githubDarkInit, githubLightInit } from "@uiw/codemirror-theme-github";
import { cn } from "@workspace/ui/lib/utils";
import { useTheme } from "@/components/theme-provider";
import { useMediaQuery } from "@/hooks/use-media-query";
import type { Language } from "@/types/jsontolang";

export type EditorLanguage = Language | "json";

const LANGUAGE_EXTENSIONS: Record<EditorLanguage, Extension> = {
  json: json(),
  typescript: javascript({ typescript: true }),
  rust: rust(),
  go: StreamLanguage.define(go),
};

const SURFACE = {
  background: "transparent",
  gutterBackground: "transparent",
  gutterBorder: "transparent",
  gutterForeground: "var(--muted-foreground)",
  gutterActiveForeground: "var(--foreground)",
  lineHighlight: "color-mix(in oklch, var(--muted) 60%, transparent)",
  fontFamily: "var(--font-mono)",
};

// module-level: inline object = new ref per render, useCodeMirror reconfigures on it, loops via onStatistics
const READ_ONLY_SETUP = {
  foldGutter: false,
  highlightActiveLine: false,
  highlightActiveLineGutter: false,
};

const LIGHT_THEME = githubLightInit({ settings: SURFACE });
const DARK_THEME = githubDarkInit({ settings: SURFACE });

interface CodeEditorProps {
  value: string;
  onChange?: (value: string) => void;
  language: EditorLanguage;
  readOnly?: boolean;
  ariaLabel: string;
  placeholder?: string;
  status?: ReactNode;
  className?: string;
}

export const CodeEditor: FC<CodeEditorProps> = ({
  value,
  onChange,
  language,
  readOnly,
  ariaLabel,
  placeholder,
  status,
  className,
}) => {
  const { theme } = useTheme();
  const prefersDark = useMediaQuery("(prefers-color-scheme: dark)");
  const dark = theme === "dark" || (theme === "system" && prefersDark);
  const [cursor, setCursor] = useState({ line: 1, col: 1 });

  const handleStatistics = (stats: Statistics) => {
    setCursor({
      line: stats.line.number,
      col: stats.selectionAsSingle.head - stats.line.from + 1,
    });
  };

  const extensions = useMemo(
    () => [
      LANGUAGE_EXTENSIONS[language],
      EditorView.lineWrapping,
      EditorView.contentAttributes.of({ "aria-label": ariaLabel }),
    ],
    [language, ariaLabel],
  );

  return (
    <div className={cn("flex h-full flex-col", className)}>
      <CodeMirror
        value={value}
        onChange={onChange}
        onStatistics={handleStatistics}
        readOnly={readOnly}
        placeholder={placeholder}
        basicSetup={readOnly ? READ_ONLY_SETUP : true}
        extensions={extensions}
        theme={dark ? DARK_THEME : LIGHT_THEME}
        height="100%"
        className="[&_.cm-placeholder]:text-muted-foreground min-h-0 flex-1 text-sm [&_.cm-editor]:h-full"
      />
      {!readOnly && (
        <div className="border-border text-muted-foreground flex min-h-6 shrink-0 items-start justify-between gap-4 border-t px-3 py-1 font-mono text-[11px]">
          <div className="min-w-0 break-words">{status}</div>
          <span className="shrink-0 tabular-nums">
            Ln {cursor.line}, Col {cursor.col}
          </span>
        </div>
      )}
    </div>
  );
};
