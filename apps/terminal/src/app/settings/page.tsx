import { cluster, rpcUrl, programId, apiUrl } from "@/lib/env";
import { Page } from "@/components/page";

export default function Settings() {
  return (
    <Page title="Settings" note="RPC is from config, not a free-form field that could silently switch clusters.">
      <div className="clay p-5 grid gap-2 text-[13px] font-mono">
        <div>cluster {cluster}</div>
        <div className="break-all">rpc {rpcUrl}</div>
        <div className="break-all">program {programId}</div>
        <div className="break-all">api {apiUrl}</div>
      </div>
    </Page>
  );
}
