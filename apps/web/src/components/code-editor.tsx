import { useMemo, useState, type FC } from "react";
import CodeMirror, { EditorView, type Extension, type Statistics } from "@uiw/react-codemirror";
import { json } from "@codemirror/lang-json";
import { javascript } from "@codemirror/lang-javascript";
import { rust } from "@codemirror/lang-rust";
import { StreamLanguage } from "@codemirror/language";
import { go } from "@codemirror/legacy-modes/mode/go";
import { githubDark } from "@uiw/codemirror-theme-github";
import { cn } from "@workspace/ui/lib/utils";
import type { Language } from "@/types/jsontolang";

export type EditorLanguage = Language | "json";

const LANGUAGE_EXTENSIONS: Record<EditorLanguage, Extension> = {
  json: json(),
  typescript: javascript({ typescript: true }),
  rust: rust(),
  go: StreamLanguage.define(go),
};

interface CodeEditorProps {
  value: string;
  onChange?: (value: string) => void;
  language: EditorLanguage;
  readOnly?: boolean;
  ariaLabel: string;
  className?: string;
}

export const CodeEditor: FC<CodeEditorProps> = ({
  value,
  onChange,
  language,
  readOnly,
  ariaLabel,
  className,
}) => {
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
        extensions={extensions}
        theme={githubDark}
        height="100%"
        className="min-h-0 flex-1 text-sm [&_.cm-editor]:h-full"
      />
      <div className="border-border text-muted-foreground flex h-6 shrink-0 items-center justify-end border-t px-3 font-mono text-[11px]">
        <span>
          Ln {cursor.line}, Col {cursor.col}
        </span>
      </div>
    </div>
  );
};
