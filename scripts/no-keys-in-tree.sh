#!/usr/bin/env bash
set -euo pipefail
# Fail if a Solana keypair JSON (64-byte array) is tracked.
if git ls-files -z | xargs -0 grep -lE '\[[[:space:]]*[0-9]+([[:space:]]*,[[:space:]]*[0-9]+){20,}' | grep -v 'package-lock' | grep -q .; then
  echo "possible key material in tracked files"
  git ls-files -z | xargs -0 grep -lE '\[[[:space:]]*[0-9]+([[:space:]]*,[[:space:]]*[0-9]+){20,}' || true
  exit 1
fi
echo "no-keys-in-tree ok"
