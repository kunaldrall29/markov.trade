# keys/ — Gate B keypairs (private files are gitignored)

Generated 2026-09-02 10:50 UTC on Kunal's workstation with `solana-keygen new --no-bip39-passphrase`, mode 600. `*.json` here is ignored by git (`.gitignore: keys/*.json`); only this README is tracked. Public keys are recorded in `docs/FACTS.md`.

| File | Pubkey | May do | Where the private key is allowed to live |
|---|---|---|---|
| `deployer.json` | `8wuYJD6bZjSA115mwXgguPoUzSqEP3dc3GxBpCu4M3mn` | deploy/upgrade the successor program, create the Gate B mints, pay rent | this directory only; CLI default signer on this box; never in a service env |
| `owner-demo.json` | `5RPxDN9hxG3YaBSZo6TWfDx1CUGpJKFsuP3hmjUkd1Hv` | fund, amend, pause, unpause, revoke, withdraw on the demo mandate; sign the tape | this directory; demo scripts only; regenerate after the tape |
| `operator.json` | `EU8a73vNg3Ti4DXtnXF41JLhc78um17er9LDZgsWCbNY` | propose actions inside policy | the `book-one` service env only |
| `emergency.json` | `A67Gw8VZbYx6qEvFeDdq4eGzpz33L3BkqyhpFrCN7JxB` | pause, revoke | the `bot` service env only, never the agent's |
| `mark-poster.json` | `2Sx9UFGkkHf1sQ2xmPQyHgNcP7HiVnxoQRg8btzhM5Bv` | write the house `MarkAccount` (P04 fallback) | the mark-poster job env only |

Rules (`docs/14-SECURITY-AND-KEYS.md` §5): no key in a repo, an issue, a screenshot, a doc, a chat message or CI. A keypair-shaped string in a tracked file fails the build once P14's grep lands. Rotation drill for the operator key runs before Gate B closes.

Funding status at generation: all five at 0 SOL; the devnet faucet refused every request from this box. Send devnet SOL to the deployer first.
