import { STAGE, CAPS, PROGRAM_ID } from "@markov/facts";

const APP = process.env.NEXT_PUBLIC_APP_URL || "http://localhost:3000";

export default function Home() {
  return (
    <div className="wrap">
      <nav>
        <a className="mark" href="/">
          <img src="/brand/markov-mark.svg" width={28} height={28} alt="" />
          markov
        </a>
        <a className="btn" href={APP}>
          Open the terminal
        </a>
      </nav>
      <p className="mono" style={{ marginTop: 48 }}>
        {STAGE}
      </p>
      <h1>
        Perps that obey
        <br />
        <span className="blue">your rules</span>, not
        <br />
        just your orders.
      </h1>
      <p className="sub">
        Markov is a programmable perpetuals account on Solana. Humans, software and models express leveraged intent; the
        account enforces what the capital is allowed to do — across Pacifica, Drift and Phoenix.
      </p>
      <div className="grid">
        <div className="card">
          <div className="mono">Venues</div>
          <p style={{ fontSize: 28, fontFamily: "var(--font-display)", fontWeight: 800, margin: "8px 0 0" }}>1 executable</p>
          <p style={{ color: "var(--muted)", fontSize: 14 }}>
            Pacifica on its testnet. Phoenix live mainnet reads. Drift waits on RPC+SDK subscribe.
          </p>
        </div>
        <div className="card">
          <div className="mono">Receipts</div>
          <p style={{ fontSize: 28, fontFamily: "var(--font-display)", fontWeight: 800, margin: "8px 0 0" }}>0 on chain</p>
          <p style={{ color: "var(--muted)", fontSize: 14 }}>
            Program {PROGRAM_ID.slice(0, 4)}…{PROGRAM_ID.slice(-4)} is generated, not deployed. Control-plane receipts exist in-process after a request.
          </p>
        </div>
        <div className="card">
          <div className="mono">Caps</div>
          <p style={{ fontSize: 28, fontFamily: "var(--font-display)", fontWeight: 800, margin: "8px 0 0" }}>
            {CAPS.perPositionLeverageMajors.toFixed(1)}x · ${CAPS.perAccountGrossNotionalUsd.toLocaleString()}
          </p>
          <p style={{ color: "var(--muted)", fontSize: 14 }}>Per-account gross notional and leverage are code, not a PDF.</p>
        </div>
      </div>

      <section>
        <p className="mono">01 — The problem</p>
        <h2>A stop-loss is one trigger. A mandate is a policy.</h2>
        <div className="grid">
          <div className="card">
            <strong>Orders, not outcomes.</strong>
            <p>Interfaces think in tickets. People think in exposure and limits.</p>
          </div>
          <div className="card">
            <strong>Bots hold too much power.</strong>
            <p>A session key without a mandate is custody with extra steps.</p>
          </div>
          <div className="card">
            <strong>Every venue, a new dialect.</strong>
            <p>One account, one receipt format, adapters underneath.</p>
          </div>
        </div>
      </section>

      <section>
        <p className="mono">02 — The control layer</p>
        <h2>Own the account. Borrow the liquidity.</h2>
        <div className="ink">
          <p>
            Existing venues first; native markets later, if earned. Phoenix has no non-production cluster. The perpetual
            pool that is not in the route set stays off the route set.
          </p>
        </div>
      </section>

      <section>
        <p className="mono">03 — Position lifecycle</p>
        <h2>Simulate, then the owner signs.</h2>
        <div className="grid">
          <div className="card">
            <strong>1. Policy</strong>
            <p>Every ticket is checked against the mandate. Stale data rejects.</p>
          </div>
          <div className="card">
            <strong>2. Route</strong>
            <p>Eligible venues only. Phoenix is integrated, not executable.</p>
          </div>
          <div className="card">
            <strong>3. Receipt</strong>
            <p>Allow, reject, skip, or require approval — all leave a receipt.</p>
          </div>
        </div>
      </section>

      <section>
        <p className="mono">04 — MCP, not a trading bot</p>
        <h2>Models may propose. They may not spend.</h2>
        <p className="sub">
          MCP tools read, simulate and propose. Withdraw, execute, and set-mandate are absent. The owner signs in the
          Terminal.
        </p>
      </section>

      <section>
        <p className="mono">05 — Invest</p>
        <h2>Tell software how your money may be invested.</h2>
        <p className="sub">
          TSLAx, NVDAx, SPYx, AAPLx. Recurring buys inside Solana’s Subscriptions &amp; Allowances cap. Tokenized stocks
          are price exposure, not shares. Issuers keep pause and delegate powers. Availability follows the issuer’s
          eligibility rules.
        </p>
      </section>

      <section>
        <p className="mono">06 — Release path</p>
        <h2>Test stage now. Capped mainnet is a later gate.</h2>
        <div className="grid">
          <div className="card">
            <div className="mono">Now</div>
            <p>Devnet program + Pacifica testnet + Phoenix mainnet reads.</p>
          </div>
          <div className="card">
            <div className="mono">Next</div>
            <p>Owner-signed capped mainnet after deploy, audit scope, and gate P1.</p>
          </div>
        </div>
      </section>

      <footer>
        Programmable perpetuals on Solana. Existing venues first; native markets later, if earned. No token. Software
        infrastructure, not investment advice. Phoenix, Pacifica and Drift are independent venues; Markov is not
        affiliated with them and availability follows each venue’s access model. © 2026 Markov.
        <div>Markov — v0.1 · {STAGE}</div>
      </footer>
    </div>
  );
}
