import { apiUrl } from "./env";
import { getToken } from "./session";

export async function api<T>(path: string, init?: RequestInit): Promise<T> {
  const headers: Record<string, string> = {
    accept: "application/json",
    "content-type": "application/json",
    ...((init?.headers as Record<string, string>) || {}),
  };
  const token = getToken();
  if (token && !headers.authorization) headers.authorization = `Bearer ${token}`;
  if (!init?.body) delete headers["content-type"];
  const res = await fetch(`${apiUrl}${path}`, {
    ...init,
    headers,
    cache: "no-store",
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`${res.status} ${path} ${text}`);
  }
  return res.json() as Promise<T>;
}
