export function Page({ title, note, children }: { title: string; note: string; children?: React.ReactNode }) {
  return (
    <div className="grid gap-4">
      <h1 className="text-[32px] tracking-[-0.04em]" style={{ fontFamily: "var(--font-display)", fontWeight: 800 }}>
        {title}
      </h1>
      <p className="text-[14px] text-[var(--mk-muted)] max-w-2xl">{note}</p>
      {children}
    </div>
  );
}
