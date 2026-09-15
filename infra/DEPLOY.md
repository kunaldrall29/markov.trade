# Deploy

- Program: `anchor build && solana program deploy target/deploy/markov.so --program-id keys/markov-program.json` on the cluster in `MARKOV_ENV`. Do not write PROGRAM_ID_STATUS=deployed until `solana program show` / `getAccountInfo` executable is true.
- API / keeper: `render.yaml` web services, `HOST=0.0.0.0` `PORT` from the platform. Filesystem is ephemeral; set `MARKOV_DATA_DIR` on a disk if receipts must survive deploys. Postgres is still the production target.
- Terminal / landing: Render or Vercel. `NEXT_PUBLIC_ENV=devnet` for the test stage host. Terminal rewrites `/v1` to `API_INTERNAL_URL`.
- Bind HTTP to `0.0.0.0:$PORT`.
