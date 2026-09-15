# keys/

Private key files are gitignored (`keys/*.json`). Public keys belong in `docs/FACTS.md`.

| File | Role |
| --- | --- |
| `markov-program.json` | Program keypair. Becomes `PROGRAM_ID` after `solana program show` confirms it on the target cluster. |
| `deployer.json` | Upgrade authority / `GlobalConfig.admin` on devnet. Mainnet admin is a Squads multisig. |

Rules: no private key in the repo, CI logs, screenshots, or client bundles. Rotation is `infra/runbooks/key-rotation.md`.
