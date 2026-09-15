# Markov program — account layout and instructions

See `prompts` in the documentation bundle and `programs/markov`. Seeds and reason codes are in `docs/FACTS.md`.

Enforcement split:

- Pacifica is off-chain matching. `record_decision` for `venue_id = PACIFICA` does **not** require an adjacent venue instruction; the owner signs a Pacifica JSON message. That is a documented limitation, not a workaround.
- Drift / Phoenix / Jupiter owner-signed allows require the next instruction’s program id to match `GlobalConfig.venue_program[venue_id]`.
- Invest keeper: `invest_execute` then Subscriptions `collect` then Jupiter (or the named `devnet_swap_stub` on devnet only).
