import { Page } from "@/components/page";

export default function MandatePage() {
  return (
    <Page title="Mandate" note="Hard rules. Changing them is owner-signed. Soft preferences never override these.">
      <div className="clay p-5 grid gap-2 text-[14px]">
        <Row k="Max leverage" v="2.0x" />
        <Row k="Max notional" v="$2,000 / account (global cap)" />
        <Row k="Min safety buffer" v="20%" />
        <Row k="Daily loss" v="$50" />
        <Row k="Approved markets" v="SOL / BTC / ETH / DOGE / FARTCOIN / PUMP perps" />
        <p className="text-[12px] text-[var(--mk-muted)] mt-2">Hash appears after set_mandate lands on chain. Program is undeployed until FACTS says otherwise.</p>
      </div>
    </Page>
  );
}

function Row({ k, v }: { k: string; v: string }) {
  return (
    <div className="flex justify-between gap-4 border-b border-black/5 py-2">
      <span className="text-[var(--mk-muted)]">{k}</span>
      <span className="font-medium text-right">{v}</span>
    </div>
  );
}
