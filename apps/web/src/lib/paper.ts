/**
 * The daily paper log, bundled from the committed `paper/*.md` files at build
 * time. Those files are written by the paper runner (P08) and never edited by
 * hand; this module only parses `key: value` lines for display.
 */
const files = import.meta.glob("../../../paper/*.md", { query: "?raw", import: "default", eager: true }) as Record<string, string>;

export type PaperDay = { date: string; fields: { key: string; value: string }[]; raw: string };

export const PAPER_DAYS: PaperDay[] = Object.entries(files)
  .map(([path, raw]) => {
    const date = path.split("/").pop()!.replace(/\.md$/, "");
    const fields: { key: string; value: string }[] = [];
    for (const line of raw.split("\n")) {
      const m = /^([a-z_ ()|-]+?):\s*(.*)$/i.exec(line);
      if (m) fields.push({ key: m[1]!.trim(), value: m[2]!.trim() });
    }
    return { date, fields, raw };
  })
  .sort((a, b) => (a.date < b.date ? 1 : -1));
