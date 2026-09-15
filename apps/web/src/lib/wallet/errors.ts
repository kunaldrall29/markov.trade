/**
 * Program errors as the program names them. The wallet's message is shown
 * verbatim underneath; nothing is reworded (docs/13 §6 rule 3).
 */
import { MARKOV_MANDATE_ERROR__INVALID_AMOUNT, getMarkovMandateErrorMessage, isMarkovMandateError } from "@markov/sdk/generated/mandate";

const CUSTOM_RE = /custom program error:\s*(0x[0-9a-f]+|\d+)/i;

/** The Anchor error name and message for a custom error code from the mandate program, if it is one. */
export function describeProgramError(err: unknown): { code: number; message: string } | null {
  const text = err instanceof Error ? err.message : String(err);
  const m = CUSTOM_RE.exec(text);
  let code: number | null = null;
  if (m) code = m[1]!.startsWith("0x") ? parseInt(m[1]!, 16) : parseInt(m[1]!, 10);
  const asAny = err as { context?: { code?: number }; cause?: unknown };
  if (code == null && typeof asAny?.context?.code === "number") code = asAny.context.code;
  if (code == null) return null;
  const mandateCode = code as typeof MARKOV_MANDATE_ERROR__INVALID_AMOUNT;
  if (isMarkovMandateError(mandateCode, {} as never)) {
    return { code, message: getMarkovMandateErrorMessage(mandateCode) };
  }
  return { code, message: `program error ${code}` };
}

export function errorText(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "string") return err;
  try {
    return JSON.stringify(err);
  } catch {
    return String(err);
  }
}
