import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type ChangeEvent,
  type ComponentProps,
  type FC,
} from "react";
import { Braces, Check, Copy, Download, FileUp, Minimize2 } from "lucide-react";
import { Button } from "@workspace/ui/components/button";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@workspace/ui/components/resizable";
import { cn } from "@workspace/ui/lib/utils";
import { useJsontolang } from "@/hooks/jsontolang/use-jsontolang";
import { useMediaQuery } from "@/hooks/use-media-query";
import { LANGUAGES, type Language } from "@/types/jsontolang";
import { CodeEditor } from "@/components/code-editor";
import { Brand, NavLinks } from "@/components/nav";
import { EditorPanel } from "./components/editor-panel";
import { LanguageSelect } from "./components/language-select";

/** Wide enough to show optionality, nesting, and arrays on first paint. */
const SAMPLE_JSON = `{
  "pets": [
    {
      "name" : "Tom",
      "species" : "Cat",
      "foods" : ["fish"],
      "birth_year" : 2000,
      "photo" : null
    },
    {
      "name" : "Spike",
      "species" : "Dog",
      "birth_year" : 2000,
      "photo" : "https://cataas.com/cat"
    },
    {
      "name" : "Meowstronaught",
      "species" : "Cat",
      "foods" : ["tuna", "catnip", "celery"],
      "birth_year" : 2012,
      "photo" : "https://cataas.com/cat"
    }
  ]
}`;

const JSON_STORAGE_KEY = "jsontolang:playground:json";

const OUTPUT_EXTENSION: Record<Language, string> = {
  typescript: "ts",
  rust: "rs",
  go: "go",
};

const DOWNLOAD_LABEL = {
  idle: (file: string) => `Download ${file}`,
  confirm: (file: string) => `Confirm download of ${file}`,
  done: (file: string) => `Downloaded ${file}`,
};

interface IconButtonProps extends ComponentProps<typeof Button> {
  label: string;
}

const IconButton: FC<IconButtonProps> = ({ label, ...props }) => (
  <Button
    type="button"
    variant="ghost"
    size="icon-sm"
    aria-label={label}
    title={label}
    {...props}
  />
);

interface PlaygroundViewProps {
  className?: string;
}

export const PlaygroundView: FC<PlaygroundViewProps> = ({ className }) => {
  const { ready, loadError, renderTypes } = useJsontolang();
  const isDesktop = useMediaQuery("(min-width: 640px)");
  const isWide = useMediaQuery("(min-width: 1024px)");
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [json, setJson] = useState(
    () => localStorage.getItem(JSON_STORAGE_KEY) ?? SAMPLE_JSON,
  );
  const [root, setRoot] = useState("Root");
  const [language, setLanguage] = useState<Language>("typescript");
  const [copied, setCopied] = useState(false);
  const [download, setDownload] = useState<"idle" | "confirm" | "done">("idle");
  const [lastOutput, setLastOutput] = useState("");

  useEffect(() => {
    localStorage.setItem(JSON_STORAGE_KEY, json);
  }, [json]);

  useEffect(() => {
    if (download === "idle") return;
    const timer = setTimeout(
      () => setDownload("idle"),
      download === "confirm" ? 4000 : 1500,
    );
    return () => clearTimeout(timer);
  }, [download]);

  // `renderTypes` is a new closure each render, so it is deliberately not a
  // dependency — `ready` is what actually gates it.
  const result = useMemo(
    () => renderTypes(root.trim() || "Root", json, language),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [ready, root, json, language],
  );

  const output = result?.output ?? null;
  const error = result?.error ?? null;
  if (output !== null && output !== lastOutput) setLastOutput(output);

  const handleFormat = () => {
    try {
      setJson(JSON.stringify(JSON.parse(json), null, 2));
    } catch {
      // Invalid JSON — leave the input as-is; the error already surfaces
      // in the JSON panel's status bar.
    }
  };

  const handleMinify = () => {
    try {
      setJson(JSON.stringify(JSON.parse(json)));
    } catch {
      // Invalid JSON — leave the input as-is.
    }
  };

  const handleFileChange = (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) void file.text().then(setJson);
    event.target.value = "";
  };

  const handleDownload = () => {
    if (!output) return;
    if (download !== "confirm") {
      setDownload("confirm");
      return;
    }

    const blob = new Blob([output], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `types.${OUTPUT_EXTENSION[language]}`;
    link.click();
    URL.revokeObjectURL(url);
    setDownload("done");
  };

  const handleCopy = () => {
    if (!output) return;
    void navigator.clipboard.writeText(output);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  const minSize = isWide ? 420 : isDesktop ? 280 : "20%";

  return (
    <main className={cn("flex h-full flex-col overflow-hidden", className)}>
      {loadError && (
        <p
          role="alert"
          className="border-destructive/40 bg-destructive/10 text-destructive shrink-0 border-b px-6 py-3 text-sm"
        >
          Failed to load the WebAssembly module: {loadError}
        </p>
      )}

      <input
        ref={fileInputRef}
        type="file"
        accept=".json,application/json"
        className="hidden"
        onChange={handleFileChange}
      />

      <ResizablePanelGroup
        orientation={isDesktop ? "horizontal" : "vertical"}
        className="min-h-0 flex-1"
      >
        <ResizablePanel defaultSize="50%" minSize={minSize}>
          <EditorPanel
            title="JSON"
            actions={
              <>
                <div className="flex items-center gap-4">
                  <div className="hidden lg:block">
                    <Brand wordmarkClassName="sr-only @md:not-sr-only" />
                  </div>
                  <label className="flex items-center gap-1.5">
                    <span className="text-muted-foreground text-xs">root</span>
                    <input
                      value={root}
                      onChange={(event) => setRoot(event.target.value)}
                      placeholder="Root"
                      spellCheck={false}
                      autoComplete="off"
                      autoCorrect="off"
                      autoCapitalize="off"
                      className="border-foreground focus-visible:border-ring focus-visible:ring-ring/50 h-7 w-24 border bg-transparent px-2 font-mono text-xs outline-none focus-visible:ring-3"
                    />
                  </label>
                </div>
                <div className="flex items-center gap-0.5">
                  <IconButton
                    label="Upload JSON file"
                    onClick={() => fileInputRef.current?.click()}
                  >
                    <FileUp />
                  </IconButton>
                  <IconButton label="Format" onClick={handleFormat}>
                    <Braces />
                  </IconButton>
                  <IconButton label="Minify" onClick={handleMinify}>
                    <Minimize2 />
                  </IconButton>
                </div>
              </>
            }
          >
            <CodeEditor
              value={json}
              onChange={setJson}
              language="json"
              ariaLabel="JSON input"
              status={
                <span role="status" className="text-destructive">
                  {error}
                </span>
              }
            />
          </EditorPanel>
        </ResizablePanel>

        <ResizableHandle className="bg-border hover:bg-primary data-[separator=active]:bg-primary transition-colors" />

        <ResizablePanel defaultSize="50%" minSize={minSize}>
          <EditorPanel
            title="Types"
            actions={
              <>
                <LanguageSelect
                  value={language}
                  languages={LANGUAGES}
                  onChange={setLanguage}
                />
                <div className="flex items-center gap-1">
                  <IconButton
                    label={DOWNLOAD_LABEL[download](
                      `types.${OUTPUT_EXTENSION[language]}`,
                    )}
                    variant={download === "confirm" ? "outline" : "ghost"}
                    size={download === "confirm" ? "sm" : "icon-sm"}
                    disabled={!output}
                    onClick={handleDownload}
                    onBlur={() => {
                      if (download === "confirm") setDownload("idle");
                    }}
                    onKeyDown={(event) => {
                      if (event.key === "Escape") setDownload("idle");
                    }}
                  >
                    {download === "done" ? <Check /> : <Download />}
                    {download === "confirm" && "Confirm"}
                  </IconButton>
                  <Button
                    type="button"
                    size="sm"
                    aria-label={copied ? "Copied" : "Copy"}
                    disabled={!output}
                    onClick={handleCopy}
                  >
                    {copied ? <Check /> : <Copy />}
                    <span className="lg:@max-lg:hidden">
                      {copied ? "Copied" : "Copy"}
                    </span>
                  </Button>
                  <div className="ml-2 hidden lg:block">
                    <NavLinks omit="/playground" />
                  </div>
                </div>
              </>
            }
          >
            <CodeEditor
              value={lastOutput}
              language={language}
              readOnly
              ariaLabel="Generated types output"
              placeholder={
                !ready && !loadError ? "Loading WebAssembly…" : undefined
              }
              className={cn(
                "[&_.cm-editor]:transition-opacity",
                error && "[&_.cm-editor]:opacity-50",
              )}
            />
          </EditorPanel>
        </ResizablePanel>
      </ResizablePanelGroup>
    </main>
  );
};
