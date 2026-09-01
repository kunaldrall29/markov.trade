# Markov Book — Document Pack
**Date:** 31 August 2026 · thesis v0.2  
**Status:** Planning freeze for the pivot. Not a litepaper. Not a grant form.  
**Rule:** claims in these files are design intent. Nothing here is a live product claim.

| # | File | Purpose |
|---|---|---|
| 01 | `01-PRODUCT-THESIS.md` | The complete idea in plain language |
| 02 | `02-TECHNICAL-DECISIONS.md` | Binding ADRs. Read before writing code |
| 03 | `03-MVP-DEVNET.md` | What ships on Solana devnet and what “done” means |
| 04 | `04-BUILD-PLAN.md` | How we build it, week by week, who does what |
| 05 | `05-ROADMAP-AFTER-MVP.md` | Guarded mainnet → open book → marketplace |
| 06 | `06-PMF-REVENUE-STRATEGY.md` | Pre-PMF diagnosis, revenue model, go-to-market |
| 07–17 | `engineering-pack/` | Gate B architecture, specs, prompts. v0.2. Start at `engineering-pack/CORRECTIONS.md` |

**One-line product:** Deposit USDC. A house agent runs a bounded book on Solana perps. You can withdraw. Every fill and every refusal is a public receipt. Later books reuse the same lock on other listed names (including tokenized-equity perps) only after a venue passes the checklist.

**What this pack is not:** a promise of APY, a token plan, or a statement that an LLM “is” the market maker.
