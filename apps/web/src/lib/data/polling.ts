/**
 * docs/13 §6: 5 s while the tab is visible, paused while hidden, 30 s after
 * three consecutive errors.
 */
type QueryLike = { state: { fetchFailureCount: number } };

export function pollInterval(query: QueryLike): number | false {
  if (typeof document !== "undefined" && document.hidden) return false;
  if (query.state.fetchFailureCount >= 3) return 30_000;
  return 5_000;
}

export function slowPollInterval(query: QueryLike): number | false {
  return pollInterval(query) === false ? false : 15_000;
}

export const FAST_POLL = { refetchInterval: pollInterval, refetchIntervalInBackground: false, retry: 1, staleTime: 2_000 } as const;
export const SLOW_POLL = { refetchInterval: slowPollInterval, refetchIntervalInBackground: false, retry: 1, staleTime: 5_000 } as const;
