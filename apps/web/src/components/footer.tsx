import { Link } from "@tanstack/react-router";
import { LINKS } from "@/components/nav";
import { TAGLINE } from "@/libs/site";

export function Footer() {
  return (
    <footer className="border-border border-t">
      <div className="mx-auto flex w-full max-w-5xl flex-wrap items-end justify-between gap-x-10 gap-y-6 px-6 pt-10 pb-12">
        <div>
          <p className="font-display text-5xl leading-none font-extrabold tracking-[-0.03em] font-stretch-75% sm:text-6xl">
            jsontolang
          </p>
          <p className="text-muted-foreground mt-3">{TAGLINE}</p>
        </div>
        <nav aria-label="Footer" className="flex items-center gap-6">
          {LINKS.map((link) => (
            <Link
              key={link.to}
              to={link.to}
              className="hover:text-primary text-sm font-medium whitespace-nowrap underline decoration-2 underline-offset-4 transition-colors"
            >
              {link.label}
            </Link>
          ))}
        </nav>
      </div>
    </footer>
  );
}
