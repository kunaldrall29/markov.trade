# Deploy

- Program: `anchor build && solana program deploy target/deploy/markov.so --program-id keys/markov-program.json` on the cluster in `MARKOV_ENV`.
- API / keeper: containers, `HOST=0.0.0.0` `PORT` from the platform. Filesystem is ephemeral; receipts in Postgres in production.
- Terminal / landing: Vercel. `NEXT_PUBLIC_ENV=devnet` for the test stage host.
- Bind HTTP to `0.0.0.0:$PORT`.
