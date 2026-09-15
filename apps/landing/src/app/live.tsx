"use client";

import { useEffect, useState } from "react";

const API = process.env.NEXT_PUBLIC_API_URL || "http://localhost:4000";

export function LiveStrip() {
  const [text, setText] = useState("control plane not reached");
  useEffect(() => {
    fetch(`${API}/health`)
      .then((r) => r.json())
      .then((h: { pacifica_markets?: number; phoenix_markets?: number; program_deployed?: boolean }) => {
        setText(
          `live: ${h.pacifica_markets ?? 0} pacifica · ${h.phoenix_markets ?? 0} phoenix · program ${h.program_deployed ? "on chain" : "undeployed"} · drift adapter closed`,
        );
      })
      .catch(() => setText("control plane not reached"));
  }, []);
  return <p className="mono">{text}</p>;
}
