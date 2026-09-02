#!/usr/bin/env bash
# Fails if any tracked file contains keypair material in the shapes that
# actually leak from this repo's tooling:
#   - a 64-element JSON byte array (solana-keygen / keys/*.json format)
#   - a "secretKey"/"private_key" field with a long value
#   - a PEM private-key block
# A base58 length check is deliberately NOT used: a base58 secret key and a
# transaction signature are both 64 bytes, so they are indistinguishable by
# length and FACTS legitimately lists signatures.
set -uo pipefail
hits=0
while IFS= read -r f; do
  case "$f" in *.png|*.jpg|*.mp4|*.pdf|*.zip|*.lock|package-lock.json|pnpm-lock.yaml) continue;; esac
  if grep -Eq '\[\s*([0-9]{1,3}\s*,\s*){63}[0-9]{1,3}\s*\]' "$f" 2>/dev/null; then echo "keypair-shaped byte array in $f"; hits=1; fi
  if grep -Eiq '"?(secretKey|secret_key|private_key|privateKey)"?\s*[:=]\s*"?[A-Za-z0-9+/=_-]{40,}' "$f" 2>/dev/null; then echo "secret-key field in $f"; hits=1; fi
  if grep -Eq 'BEGIN (RSA |EC |OPENSSH |)PRIVATE KEY' "$f" 2>/dev/null; then echo "PEM private key in $f"; hits=1; fi
done < <(git ls-files)
[ "$hits" -eq 0 ] && echo "no-keys-in-tree: clean ($(git ls-files | wc -l) tracked files)"
exit "$hits"
