import { Page } from "@/components/page";

export default function RiskPage() {
  return (
    <Page title="Risk" note="Leverage is null when equity is unknown — never displayed as 0x. Stress numbers are simulated and labelled as such.">
      <div className="clay p-6 text-[14px] text-[var(--mk-muted)]">No venue equity. Headroom gauges wait on a linked account.</div>
    </Page>
  );
}
