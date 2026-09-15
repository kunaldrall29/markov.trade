const KEY = "markov.siws.token";
const PUB = "markov.siws.pubkey";

export function getToken(): string | null {
  if (typeof window === "undefined") return null;
  return localStorage.getItem(KEY);
}

export function getSessionPubkey(): string | null {
  if (typeof window === "undefined") return null;
  return localStorage.getItem(PUB);
}

export function setSession(token: string, pubkey: string) {
  localStorage.setItem(KEY, token);
  localStorage.setItem(PUB, pubkey);
}

export function clearSession() {
  localStorage.removeItem(KEY);
  localStorage.removeItem(PUB);
}
