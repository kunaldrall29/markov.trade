# SESSION_LOG

One `##` per session, newest first. Facts, not narrative. What was verified, how, and what was not.

## 2026-09-01 — Session 0: verify, FACTS, gap list, decisions (no code)

- **Goal:** read the pack in order, verify the toolchain and the deployed program, create `docs/FACTS.md`, report gaps, surface D1–D3, propose the build order. Then stop.
- **Repo:** `git init` in `/home/kunal/markov.trade`; commit 1 = pack as received (flat `files/`), commit 2 = `git mv` into `docs/ prompts/ landing/ scripts/` as the pack's own README describes (no content edits), commit 3+ = FACTS, ADRs, this log. Authored as Kunal only.
- **Toolchain at start:** rustc 1.97.1, node 24.19.0, pnpm 9.15.9, bun 1.4.0, docker 29.7.2; **no `solana`, `anchor`, `avm`, no `~/.config/solana`**. Installed Agave stable (solana-cli 4.2.2), avm 1.1.2, anchor-cli 0.31.1 (the version both deployed programs use). Pin not decided (ADR-001).
- **Program identity resolved:** two `markov-mandate` programs on devnet, found through Kunal's `markov-landing` FACTS (read-only). `5o8E…` (`kunaldrall29/markov`, 98 txs, all 11 `BlockReason`s emitted 28–29 Aug, authority `2fpQ…`) is the base; `CT2n…` (`MarkovFyi/markov-program`, 4 deploy txs, never emitted) is the predecessor. Neither has an on-chain IDL; the checked-in IDL (sha256 `55c62d4b…`) was verified against the chain by event discriminators and by the reason byte in 20 `ActionRefused` payloads.
- **Spec conflicts found:** `docs/10` §4 numbering and two names contradict the chain (`OverTxCap`=0 … `Unauthorized`=10; `ProgramNotAllowed`, `Unauthorized`). Account seeds, state enum, policy fields, events (`emit!` vs `emit_cpi`), instruction set all differ. Written up in ADR-004.
- **Outside-world facts that changed since the pack was written:** Hermes price endpoints require an API key since 26 Aug (401 verified); Anchor stable is 1.1.2 not 1.0.x; `pyth-solana-receiver-sdk` 2.0.0 targets anchor ^1.0.2 (1.0.1 targets ^0.31.1); Pyth devnet receiver upgraded 31 Aug, address unchanged.
- **Hosting found:** Railway project `markov` (api, agents, bot, data-api, indexer, Postgres — all online, indexer stalled at lag 1.2M slots); Vercel deploys on a team this session cannot see; `markov.trade` registered at Cloudflare with **no DNS records**; `markovhq.grok.me/book` is a static sample. Emergency key lives in the `api` service env, not the bot. Written up in ADR-005.
- **Copy-grep:** clean on `landing/index.html`; red on injected `12% APY, guaranteed … Jupiter` (4 rules). B15 tooling works.
- **Not done / blocked:** faucet rate-limited (0 SOL on a throwaway key); no RPC-provider key; no Pyth key; `2fpQ…` custody unknown; LiteSVM ↔ Anchor compatibility unproven; Pyth SDK build unproven; no keypairs generated (Kunal's workstation decision); no DNS.
- **Commands that work:** `anchor --version`, `solana program show <id> --keypair <any>`, `bash scripts/copy-grep.sh landing/`, `node dump-idl.mjs <program>` (scratchpad), `python3 disc.py` (scratchpad).
- **Docs touched:** `docs/FACTS.md` (new), `docs/adr/ADR-001…006` (Proposed), `docs/BACKLOG.md` (new), this file.
- **Next:** Kunal answers D0–D4 (ADR-001…005) and the key/account questions; then `prompts/P08-paper-runner.md` starts the same day, and `prompts/P01-repo-bootstrap.md` runs with a green pre-flight.
