export type VenueQuote = {
  venueId: number;
  venue: string;
  executable: boolean;
  linked: boolean;
  stale: boolean;
  mark: number;
  slippageBps: number;
  takerFeeBps: number;
  fundingBpsPerDay: number;
  exitBps: number;
  riskPremiumBps: number;
  freshnessMs: number;
};

export type RankedRoute = VenueQuote & {
  horizonHours: number;
  entryBps: number;
  holdingBps: number;
  totalBps: number;
  selected: boolean;
  reason: string;
};

export function compareRoutes(
  quotes: VenueQuote[],
  horizonHours: number,
): RankedRoute[] {
  const eligible = quotes.filter((q) => q.executable && q.linked && !q.stale);
  const ranked = quotes.map((q) => {
    const holdingBps = q.fundingBpsPerDay * (horizonHours / 24);
    const entryBps = q.slippageBps + q.takerFeeBps;
    const totalBps = entryBps + holdingBps + q.exitBps + q.riskPremiumBps;
    let reason = `${q.venue}: entry ${entryBps.toFixed(1)} + holding ${holdingBps.toFixed(1)} + exit ${q.exitBps.toFixed(1)}`;
    if (!q.linked) reason = `${q.venue} has no linked funded account`;
    else if (!q.executable) reason = `${q.venue} is integrated, not executable`;
    else if (q.stale) reason = `${q.venue} data is stale`;
    return {
      ...q,
      horizonHours,
      entryBps,
      holdingBps,
      totalBps,
      selected: false,
      reason,
    };
  });
  ranked.sort((a, b) => {
    const ae = eligible.includes(quotes.find((q) => q.venueId === a.venueId)!);
    const be = eligible.includes(quotes.find((q) => q.venueId === b.venueId)!);
    if (ae !== be) return ae ? -1 : 1;
    return a.totalBps - b.totalBps;
  });
  const firstOk = ranked.find((r) => r.executable && r.linked && !r.stale);
  if (firstOk) {
    firstOk.selected = true;
    firstOk.reason = `selected: lowest lifecycle cost over ${horizonHours}h (${firstOk.totalBps.toFixed(1)} bps)`;
  }
  return ranked;
}
