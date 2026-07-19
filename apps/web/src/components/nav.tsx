import { Link } from "@tanstack/react-router";
import { asset } from "@/libs/site";

const LINKS = [
  { to: "/plugins", label: "Plugins" },
  { to: "/playground", label: "Playground" },
] as const;

export function Nav() {
  return (
    <header className="flex items-center justify-between px-6 py-4">
      <Link to="/" className="flex items-center gap-2">
        <img src={asset("logo.svg")} alt="jsontolang" className="size-7" />
        <span className="font-semibold">jsontolang</span>
      </Link>
      <nav className="flex items-center gap-4">
        {LINKS.map((link) => (
          <Link
            key={link.to}
            to={link.to}
            className="text-sm text-muted-foreground transition-colors hover:text-foreground"
            activeProps={{ className: "text-primary font-medium" }}
          >
            {link.label}
          </Link>
        ))}
      </nav>
    </header>
  );
}
