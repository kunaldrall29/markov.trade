import type { InputHTMLAttributes, ReactNode } from "react";
import { cn } from "@/lib/utils";

export function Field({
  label,
  hint,
  children,
  htmlFor,
}: {
  label: string;
  hint?: ReactNode;
  children: ReactNode;
  htmlFor?: string;
}) {
  return (
    <div className="grid gap-1.5">
      <label htmlFor={htmlFor} className="font-mono text-micro uppercase tracking-eye text-subtle">
        {label}
      </label>
      {children}
      {hint ? <p className="font-mono text-nano text-subtle">{hint}</p> : null}
    </div>
  );
}

export function Input({ className, ...props }: InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      className={cn(
        "min-h-11 w-full rounded-sm bg-bg px-3 font-mono text-sm tabular-nums text-fg shadow-hairline outline-none placeholder:text-subtle focus-visible:shadow-hairline-strong",
        className,
      )}
      {...props}
    />
  );
}

/** A labelled value in mono: the unit of every stat on this surface. */
export function Stat({ label, value, note, tone }: { label: string; value: ReactNode; note?: ReactNode; tone?: "allow" | "refuse" | "muted" }) {
  return (
    <div className="bg-raised px-4 py-3">
      <dt className="font-mono text-nano uppercase tracking-wider text-subtle">{label}</dt>
      <dd className={cn("mt-1 font-mono text-xl tabular-nums", tone === "allow" && "text-allow", tone === "refuse" && "text-refuse", tone === "muted" && "text-subtle")}>
        {value}
        {note ? <span className="mt-0.5 block font-mono text-nano font-normal text-subtle">{note}</span> : null}
      </dd>
    </div>
  );
}

export function Notice({ tone = "muted", children }: { tone?: "muted" | "refuse" | "allow"; children: ReactNode }) {
  return (
    <p
      role={tone === "refuse" ? "alert" : undefined}
      className={cn(
        "rounded-sm px-3 py-2 font-mono text-xs leading-relaxed shadow-hairline",
        tone === "refuse" && "bg-refuse/10 text-refuse",
        tone === "allow" && "bg-allow/10 text-allow",
        tone === "muted" && "bg-raised text-muted",
      )}
    >
      {children}
    </p>
  );
}
