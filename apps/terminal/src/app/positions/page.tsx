import { Page } from "@/components/page";

export default function Positions() {
  return (
    <Page title="Positions" note="Cross-venue table. Liquidation figures are venue estimates. Reduce/close require an owner signature.">
      <div className="clay p-6 text-[14px] text-[var(--mk-muted)]">No positions. Link Pacifica or Drift on /venues.</div>
    </Page>
  );
}
