/** Venue numbers in, normalized risk out. Missing fields are null and fail closed upstream. */

export type VenuePosition = {
  marketId: string;
  venueId: number;
  side: "long" | "short";
  notionalUsd: number;
  entry: number;
  mark: number | null;
  leverage: number | null;
  liqPrice: number | null;
  marginUsd: number | null;
};

export type VenueAccountRaw = {
  venueId: number;
  equityUsd: number | null;
  grossNotionalUsd: number | null;
  maintenanceMarginUsd: number | null;
  dailyPnlUsd: number | null;
  health: number | null;
  positions: VenuePosition[];
  dataSlot: number;
  venueTs: number;
};

export type NormalizedRiskState = {
  equityUsd: number | null;
  grossNotionalUsd: number;
  netDeltaUsd: number;
  effectiveLeverage: number | null;
  safetyBufferBps: number | null;
  dailyLossUsd: number;
  headroom: {
    leverageBps: number | null;
    notionalUsd: number | null;
    dailyLossUsd: number | null;
  };
  venueHealth: Record<number, number | null>;
  dataSlot: number;
  stale: boolean;
};

export function normalize(
  venues: VenueAccountRaw[],
  mandate: { maxLeverageBps: number; maxNotionalUsd: number; maxDailyLossUsd: number },
  nowTs: number,
  freshnessMs: number,
): NormalizedRiskState {
  let equity = 0;
  let equityKnown = false;
  let gross = 0;
  let net = 0;
  let daily = 0;
  let slot = 0;
  let stale = false;
  const venueHealth: Record<number, number | null> = {};

  for (const v of venues) {
    if (nowTs - v.venueTs > freshnessMs) stale = true;
    slot = Math.max(slot, v.dataSlot);
    venueHealth[v.venueId] = v.health;
    if (v.equityUsd != null) {
      equity += v.equityUsd;
      equityKnown = true;
    }
    gross += v.grossNotionalUsd ?? v.positions.reduce((s, p) => s + Math.abs(p.notionalUsd), 0);
    for (const p of v.positions) {
      net += p.side === "long" ? p.notionalUsd : -p.notionalUsd;
    }
    daily += v.dailyPnlUsd ?? 0;
  }

  const effectiveLeverage = equityKnown && equity > 0 ? gross / equity : null;
  let safetyBufferBps: number | null = null;
  const mm = venues.reduce((s, v) => s + (v.maintenanceMarginUsd ?? 0), 0);
  if (equityKnown && gross > 0) {
    const dist = equity - mm;
    safetyBufferBps = Math.round((dist / gross) * 10_000);
  }

  return {
    equityUsd: equityKnown ? equity : null,
    grossNotionalUsd: gross,
    netDeltaUsd: net,
    effectiveLeverage,
    safetyBufferBps,
    dailyLossUsd: daily < 0 ? -daily : 0,
    headroom: {
      leverageBps:
        effectiveLeverage == null ? null : mandate.maxLeverageBps - Math.round(effectiveLeverage * 10_000),
      notionalUsd: mandate.maxNotionalUsd - gross,
      dailyLossUsd: mandate.maxDailyLossUsd - (daily < 0 ? -daily : 0),
    },
    venueHealth,
    dataSlot: slot,
    stale,
  };
}

export function simulateTrade(
  state: NormalizedRiskState,
  addNotionalUsd: number,
  addEquityUsd: number,
): NormalizedRiskState {
  const gross = state.grossNotionalUsd + Math.abs(addNotionalUsd);
  const equity = state.equityUsd == null ? null : state.equityUsd + addEquityUsd;
  const lev = equity && equity > 0 ? gross / equity : null;
  return {
    ...state,
    grossNotionalUsd: gross,
    equityUsd: equity,
    effectiveLeverage: lev,
    netDeltaUsd: state.netDeltaUsd + addNotionalUsd,
  };
}

export function simulateScenario(
  state: NormalizedRiskState,
  priceShockBps: number,
): { equityUsd: number | null; leverage: number | null } {
  if (state.equityUsd == null) return { equityUsd: null, leverage: null };
  const shock = state.netDeltaUsd * (priceShockBps / 10_000);
  const equity = state.equityUsd + shock;
  const lev = equity > 0 ? state.grossNotionalUsd / equity : null;
  return { equityUsd: equity, leverage: lev };
}
