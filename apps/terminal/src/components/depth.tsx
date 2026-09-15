type Level = { price: number; size: number };

export function Depth({ bids, asks, venue }: { bids: Level[]; asks: Level[]; venue: string }) {
  const max = Math.max(...bids.map((l) => l.size), ...asks.map((l) => l.size), 0.0001);
  return (
    <div>
      <h2 className="text-[13px] font-semibold">Book · {venue}</h2>
      <div className="mt-2 grid grid-cols-2 gap-3 text-[12px] font-mono">
        <DepthCol levels={bids.slice(0, 10)} max={max} side="bid" />
        <DepthCol levels={asks.slice(0, 10)} max={max} side="ask" />
      </div>
    </div>
  );
}

function DepthCol({ levels, max, side }: { levels: Level[]; max: number; side: "bid" | "ask" }) {
  return (
    <div>
      <div className="text-[var(--mk-muted)] uppercase tracking-[0.08em] text-[11px]">{side === "bid" ? "Bids" : "Asks"}</div>
      {levels.map((l, i) => (
        <div key={`${side}${i}`} className="relative flex justify-between py-[2px]">
          <span
            className="absolute inset-y-0 left-0 opacity-20"
            style={{
              width: `${(l.size / max) * 100}%`,
              background: side === "bid" ? "#2e8b57" : "#e8552b",
            }}
          />
          <span className="relative">{l.price}</span>
          <span className="relative">{l.size}</span>
        </div>
      ))}
    </div>
  );
}
