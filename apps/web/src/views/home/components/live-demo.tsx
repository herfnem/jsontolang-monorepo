import { useMemo, useState, type FC } from "react";
import { CodeEditor } from "@/components/code-editor";
import { useJsontolang } from "@/hooks/jsontolang/use-jsontolang";
import { cn } from "@workspace/ui/lib/utils";

const DEMO_JSON = `{
  "id": 1,
  "name": "Neko",
  "tags": ["cat", "small"]
}`;

interface LiveDemoProps {
  className?: string;
}

/** A live, editable teaser of the playground — type, get types back, right on the homepage. */
export const LiveDemo: FC<LiveDemoProps> = ({ className }) => {
  const { ready, loadError, renderTypes } = useJsontolang();
  const [json, setJson] = useState(DEMO_JSON);

  // `renderTypes` is a new closure each render, so it is deliberately not a
  // dependency — `ready` is what actually gates it.
  const result = useMemo(
    () => renderTypes("Root", json, "typescript"),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [ready, json],
  );

  return (
    <div className={cn("grid gap-4 sm:grid-cols-2", className)}>
      <div className="border-border bg-card flex flex-col overflow-hidden rounded-lg border">
        <div className="border-border text-muted-foreground border-b px-4 py-2 font-mono text-xs">
          input.json <span className="text-muted-foreground/70">— edit me</span>
        </div>
        <CodeEditor
          value={json}
          onChange={setJson}
          language="json"
          ariaLabel="Live demo JSON input"
          className="h-64"
        />
      </div>

      <div className="border-border bg-card flex flex-col overflow-hidden rounded-lg border">
        <div className="border-border text-muted-foreground border-b px-4 py-2 font-mono text-xs">
          typescript
        </div>
        {!ready && !loadError ? (
          <p className="text-muted-foreground h-64 p-4 font-mono text-sm">
            Loading WebAssembly…
          </p>
        ) : loadError ? (
          <p role="alert" className="text-destructive h-64 p-4 font-mono text-sm break-words">
            Failed to load the WebAssembly module: {loadError}
          </p>
        ) : result?.error ? (
          <p role="alert" className="text-destructive h-64 p-4 font-mono text-sm break-words">
            {result.error}
          </p>
        ) : (
          <CodeEditor
            value={result?.output ?? ""}
            language="typescript"
            readOnly
            ariaLabel="Live demo TypeScript output"
            className="h-64"
          />
        )}
      </div>
    </div>
  );
};
