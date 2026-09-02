#!/usr/bin/env bash
# Prove an RPC endpoint is Solana devnet before anything trusts it.
#
# A mainnet URL in a devnet config is the worst possible configuration error:
# every read succeeds, every number is wrong, and the page lies confidently.
# The genesis hash is the only answer that cannot be faked by a proxy that
# merely forwards requests, so that is what this checks.
#
# usage: scripts/rpc-check.sh <url> [<url> ...]
set -uo pipefail

DEVNET_GENESIS="EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG"
MAINNET_GENESIS="5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d"
TESTNET_GENESIS="4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY"

fail=0
for url in "$@"; do
  body='{"jsonrpc":"2.0","id":1,"method":"getGenesisHash"}'
  out=$(curl -s -m 20 -X POST -H 'content-type: application/json' -d "$body" "$url" 2>/dev/null)
  hash=$(printf '%s' "$out" | python3 -c 'import sys,json
try: print(json.load(sys.stdin).get("result",""))
except Exception: print("")' 2>/dev/null)
  case "$hash" in
    "$DEVNET_GENESIS")
      slot=$(curl -s -m 20 -X POST -H 'content-type: application/json' \
        -d '{"jsonrpc":"2.0","id":1,"method":"getSlot","params":[{"commitment":"confirmed"}]}' "$url" 2>/dev/null |
        python3 -c 'import sys,json
try: print(json.load(sys.stdin).get("result","?"))
except Exception: print("?")')
      printf 'ok       %-52s devnet, slot %s\n' "$url" "$slot" ;;
    "$MAINNET_GENESIS")
      printf 'FAIL     %-52s MAINNET — refusing\n' "$url"; fail=1 ;;
    "$TESTNET_GENESIS")
      printf 'FAIL     %-52s testnet — refusing\n' "$url"; fail=1 ;;
    "")
      printf 'FAIL     %-52s no genesis hash (%s)\n' "$url" "$(printf '%s' "$out" | head -c 80)"; fail=1 ;;
    *)
      printf 'FAIL     %-52s unknown cluster %s\n' "$url" "$hash"; fail=1 ;;
  esac
done
exit "$fail"
