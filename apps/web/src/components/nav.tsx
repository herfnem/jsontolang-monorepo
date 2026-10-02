import { Link } from "@tanstack/react-router";
import { cn } from "@workspace/ui/lib/utils";
import { asset } from "@/libs/site";

export const LINKS = [
  { to: "/plugins", label: "Plugins" },
  { to: "/playground", label: "Playground" },
] as const;

interface BrandProps {
  wordmarkClassName?: string;
}

export function Brand({ wordmarkClassName }: BrandProps) {
  return (
    <Link to="/" className="flex shrink-0 items-center gap-2">
      <img
        src={asset("logo.svg")}
        alt=""
        className="size-5 max-[359px]:hidden sm:size-6"
      />
      <span
        className={cn(
          "font-display text-xl leading-none font-extrabold tracking-[-0.03em] font-stretch-75% sm:text-2xl",
          wordmarkClassName,
        )}
      >
        jsontolang
      </span>
    </Link>
  );
}

interface NavLinksProps {
  omit?: (typeof LINKS)[number]["to"];
}

export function NavLinks({ omit }: NavLinksProps) {
  return (
    <nav className="flex items-center gap-1">
      {LINKS.filter((link) => link.to !== omit).map((link) => (
        <Link
          key={link.to}
          to={link.to}
          className="data-[status=active]:bg-foreground data-[status=active]:text-background px-2 py-1.5 text-sm font-semibold whitespace-nowrap decoration-2 underline-offset-4 hover:underline data-[status=active]:hover:no-underline sm:px-2.5"
        >
          {link.label}
        </Link>
      ))}
    </nav>
  );
}

interface NavProps {
  className?: string;
}

export function Nav({ className }: NavProps) {
  return (
    <header className={cn("border-border h-14 shrink-0 border-b", className)}>
      <div className="mx-auto flex h-full w-full max-w-5xl items-center justify-between gap-3 px-6">
        <Brand />
        <NavLinks />
      </div>
    </header>
  );
}
