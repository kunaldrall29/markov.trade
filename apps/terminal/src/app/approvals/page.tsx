import { Page } from "@/components/page";

export default function Approvals() {
  return (
    <Page title="Approvals" note="MCP proposals land here. The model never receives a signable. Sign or decline; both are receipted.">
      <div className="clay p-6 text-[14px] text-[var(--mk-muted)]">Inbox empty.</div>
    </Page>
  );
}
