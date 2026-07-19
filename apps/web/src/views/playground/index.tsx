import { useMemo, useState, type FC } from "react";
import { cn } from "@workspace/ui/lib/utils";
import { useJsontolang } from "@/hooks/jsontolang/use-jsontolang";
import { LANGUAGES, type Language } from "@/types/jsontolang";
import { EditorPanel } from "./components/editor-panel";
import { LanguageSelect } from "./components/language-select";

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

interface PlaygroundViewProps {
  className?: string;
}

export const PlaygroundView: FC<PlaygroundViewProps> = ({ className }) => {
  const { ready, loadError, renderTypes } = useJsontolang();
  const [json, setJson] = useState(SAMPLE_JSON);
  const [root, setRoot] = useState("Root");
  const [language, setLanguage] = useState<Language>("typescript");

  // `renderTypes` is a new closure each render, so it is deliberately not a
  // dependency — `ready` is what actually gates it.
  const result = useMemo(
    () => renderTypes(root, json, language),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [ready, root, json, language],
  );

  return (
    <main className={cn("mx-auto flex w-full max-w-6xl flex-col px-6 pb-12", className)}>
      <section className="py-8">
        <h1 className="text-3xl font-semibold tracking-tight">Playground</h1>
        <p className="text-muted-foreground mt-3 max-w-2xl leading-relaxed">
          Paste JSON and get type definitions back. jsontolang is compiled to WebAssembly and runs
          entirely in this tab — nothing is uploaded to a server.
        </p>
      </section>

      {loadError && (
        <p
          role="alert"
          className="border-destructive/40 bg-destructive/10 text-destructive mb-4 rounded-lg border px-4 py-3 text-sm"
        >
          Failed to load the WebAssembly module: {loadError}
        </p>
      )}

      <div className="grid min-h-[28rem] gap-4 lg:grid-cols-2">
        <EditorPanel
          title="JSON"
          actions={
            <label className="flex items-center gap-2">
              <span className="text-muted-foreground text-xs">root</span>
              <input
                value={root}
                onChange={(event) => setRoot(event.target.value)}
                spellCheck={false}
                className="border-border bg-background focus-visible:border-ring focus-visible:ring-ring/50 h-7 w-32 rounded-md border px-2 font-mono text-sm outline-none focus-visible:ring-3"
              />
            </label>
          }
        >
          <textarea
            value={json}
            onChange={(event) => setJson(event.target.value)}
            spellCheck={false}
            aria-label="JSON input"
            className="min-h-[24rem] flex-1 resize-none bg-transparent p-4 font-mono text-sm leading-relaxed outline-none"
          />
        </EditorPanel>

        <EditorPanel
          title="Types"
          actions={
            <LanguageSelect value={language} languages={LANGUAGES} onChange={setLanguage} />
          }
        >
          {!ready && !loadError ? (
            <p className="text-muted-foreground p-4 font-mono text-sm">Loading WebAssembly…</p>
          ) : result?.error ? (
            <p role="alert" className="text-destructive p-4 font-mono text-sm break-words">
              {result.error}
            </p>
          ) : (
            <pre className="flex-1 overflow-auto p-4 font-mono text-sm leading-relaxed">
              <code>{result?.output}</code>
            </pre>
          )}
        </EditorPanel>
      </div>
    </main>
  );
};
