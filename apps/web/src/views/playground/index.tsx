import { useEffect, useMemo, useState, type FC } from "react";
import { Check, Copy } from "lucide-react";
import { Button } from "@workspace/ui/components/button";
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "@workspace/ui/components/resizable";
import { cn } from "@workspace/ui/lib/utils";
import { useJsontolang } from "@/hooks/jsontolang/use-jsontolang";
import { useMediaQuery } from "@/hooks/use-media-query";
import { LANGUAGES, type Language } from "@/types/jsontolang";
import { CodeEditor } from "@/components/code-editor";
import { ActionsRail } from "./components/actions-rail";
import { EditorPanel } from "./components/editor-panel";

/** Wide enough to show optionality, nesting, and arrays on first paint. */
const SAMPLE_JSON = `{
  "id": 1,
  "name": "Neko",
  "tags": ["cat", "small"],
  "owner": { "name": "Anup", "email": "anup@example.com" },
  "sightings": [
    { "at": "2026-07-19", "place": "roof" },
    { "at": "2026-07-18" }
  ]
}`;

const JSON_STORAGE_KEY = "jsontolang:playground:json";

const OUTPUT_EXTENSION: Record<Language, string> = {
  typescript: "ts",
  rust: "rs",
  go: "go",
};

interface PlaygroundViewProps {
  className?: string;
}

export const PlaygroundView: FC<PlaygroundViewProps> = ({ className }) => {
  const { ready, loadError, renderTypes } = useJsontolang();
  const isDesktop = useMediaQuery("(min-width: 640px)");
  const [json, setJson] = useState(() => localStorage.getItem(JSON_STORAGE_KEY) ?? SAMPLE_JSON);
  const [root, setRoot] = useState("Root");
  const [language, setLanguage] = useState<Language>("typescript");
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    localStorage.setItem(JSON_STORAGE_KEY, json);
  }, [json]);

  // `renderTypes` is a new closure each render, so it is deliberately not a
  // dependency — `ready` is what actually gates it.
  const result = useMemo(
    () => renderTypes(root.trim() || "Root", json, language),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [ready, root, json, language],
  );

  const handleFormat = () => {
    try {
      setJson(JSON.stringify(JSON.parse(json), null, 2));
    } catch {
      // Invalid JSON — leave the input as-is; the error already surfaces
      // in the Types panel.
    }
  };

  const handleMinify = () => {
    try {
      setJson(JSON.stringify(JSON.parse(json)));
    } catch {
      // Invalid JSON — leave the input as-is.
    }
  };

  const handleUpload = (file: File) => {
    void file.text().then(setJson);
  };

  const handleDownload = () => {
    if (!result?.output) return;

    const blob = new Blob([result.output], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `types.${OUTPUT_EXTENSION[language]}`;
    link.click();
    URL.revokeObjectURL(url);
  };

  const handleCopy = () => {
    if (!result?.output) return;
    void navigator.clipboard.writeText(result.output);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <main className={cn("flex min-h-0 flex-1 flex-col overflow-hidden", className)}>
      {loadError && (
        <p
          role="alert"
          className="border-destructive/40 bg-destructive/10 text-destructive shrink-0 border-b px-6 py-3 text-sm"
        >
          Failed to load the WebAssembly module: {loadError}
        </p>
      )}

      <ResizablePanelGroup
        orientation={isDesktop ? "horizontal" : "vertical"}
        className="min-h-0 flex-1"
      >
        <ResizablePanel defaultSize="35%" minSize="20%">
          <EditorPanel
            title="JSON"
            actions={
              <label className="flex items-center gap-2">
                <span className="text-muted-foreground text-xs">root</span>
                <input
                  value={root}
                  onChange={(event) => setRoot(event.target.value)}
                  placeholder="Root"
                  spellCheck={false}
                  autoComplete="off"
                  autoCorrect="off"
                  autoCapitalize="off"
                  className="border-border bg-background focus-visible:border-ring focus-visible:ring-ring/50 h-7 w-32 rounded-md border px-2 font-mono text-sm outline-none focus-visible:ring-3"
                />
              </label>
            }
          >
            <CodeEditor value={json} onChange={setJson} language="json" ariaLabel="JSON input" />
          </EditorPanel>
        </ResizablePanel>

        <ResizableHandle className="bg-border hover:bg-primary/50 data-[separator=active]:bg-primary w-1 transition-colors" />

        <ResizablePanel defaultSize={220} minSize={200} maxSize={320}>
          <ActionsRail
            onUpload={handleUpload}
            onFormat={handleFormat}
            onMinify={handleMinify}
            onDownload={handleDownload}
            downloadDisabled={!result?.output}
            language={language}
            languages={LANGUAGES}
            onLanguageChange={setLanguage}
          />
        </ResizablePanel>

        <ResizableHandle className="bg-border hover:bg-primary/50 data-[separator=active]:bg-primary w-1 transition-colors" />

        <ResizablePanel defaultSize="45%" minSize="20%">
          <EditorPanel
            title="Types"
            actions={
              <Button
                type="button"
                variant="ghost"
                size="xs"
                disabled={!result?.output}
                onClick={handleCopy}
              >
                {copied ? <Check /> : <Copy />}
                {copied ? "Copied" : "Copy"}
              </Button>
            }
          >
            {!ready && !loadError ? (
              <p className="text-muted-foreground p-4 font-mono text-sm">Loading WebAssembly…</p>
            ) : result?.error ? (
              <p role="alert" className="text-destructive p-4 font-mono text-sm break-words">
                {result.error}
              </p>
            ) : (
              <CodeEditor
                value={result?.output ?? ""}
                language={language}
                readOnly
                ariaLabel="Generated types output"
              />
            )}
          </EditorPanel>
        </ResizablePanel>
      </ResizablePanelGroup>
    </main>
  );
};
