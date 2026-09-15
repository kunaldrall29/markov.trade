# Key rotation

1. Generate a new keeper keypair in KMS. Never export.
2. Owner `set_permission` the new actor with `invest:execute` only.
3. Drain: wait for in-flight cycles (`request_id` unique).
4. `revoke_permission` the old actor.
5. Destroy the old key.
6. Record the pubkeys in FACTS.
