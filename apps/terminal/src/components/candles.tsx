type Candle = { openTime: number; open: number; high: number; low: number; close: number };

export function CandleChart({ candles, label }: { candles: Candle[]; label: string }) {
  if (!candles.length) {
    return <p className="text-[13px] text-[var(--mk-muted)]">No kline yet from Pacifica.</p>;
  }
  const w = 640;
  const h = 180;
  const pad = 8;
  const highs = candles.map((c) => c.high);
  const lows = candles.map((c) => c.low);
  const min = Math.min(...lows);
  const max = Math.max(...highs);
  const span = max - min || 1;
  const bw = Math.max(2, (w - pad * 2) / candles.length - 1);
  const y = (v: number) => pad + ((max - v) / span) * (h - pad * 2);
  return (
    <div>
      <p className="text-[11px] font-mono uppercase tracking-[0.08em] text-[var(--mk-muted)]">{label}</p>
      <svg viewBox={`0 0 ${w} ${h}`} className="mt-2 w-full h-[180px]" role="img" aria-label={label}>
        {candles.map((c, i) => {
          const x = pad + i * ((w - pad * 2) / candles.length) + bw / 2;
          const up = c.close >= c.open;
          return (
            <g key={c.openTime}>
              <line x1={x} x2={x} y1={y(c.high)} y2={y(c.low)} stroke={up ? "#2e8b57" : "#e8552b"} strokeWidth="1" />
              <rect
                x={x - bw / 2}
                y={y(Math.max(c.open, c.close))}
                width={bw}
                height={Math.max(1, Math.abs(y(c.open) - y(c.close)))}
                fill={up ? "#2e8b57" : "#e8552b"}
              />
            </g>
          );
        })}
      </svg>
    </div>
  );
}
