"use client";

import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import { Page } from "@/components/page";

export default function Integrations() {
  const [tools, setTools] = useState<string[]>([]);
  const [absent, setAbsent] = useState<string[]>([]);
  useEffect(() => {
    api<{ tools: string[]; absent: string[] }>("/mcp/tools").then((r) => {
      setTools(r.tools);
      setAbsent(r.absent);
    }).catch(() => undefined);
  }, []);
  return (
    <Page title="Integrations" note="MCP clients may read, simulate and propose. They cannot execute, withdraw, or change a mandate.">
      <div className="clay p-4">
        <h2 className="font-semibold">Tools</h2>
        <ul className="mt-2 font-mono text-[12px] grid gap-1">{tools.map((t) => <li key={t}>{t}</li>)}</ul>
      </div>
      <div className="clay p-4">
        <h2 className="font-semibold">Absent by design</h2>
        <ul className="mt-2 font-mono text-[12px] grid gap-1">{absent.map((t) => <li key={t}>{t}</li>)}</ul>
      </div>
    </Page>
  );
}
