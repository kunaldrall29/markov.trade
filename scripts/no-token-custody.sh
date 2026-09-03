#!/usr/bin/env bash
# demo_perps is an accounting mock: it must never be able to move a token.
#
# The claim in SECURITY.md is "demo_perps holds no token custody", and a claim
# like that is worth a check rather than a promise. Two independent passes:
# the source must not reference a token CPI, and the built binary must not
# contain the SPL Token program id.
set -uo pipefail

SRC=programs/demo-perps/src
SO=target/deploy/demo_perps.so
TOKEN_PROGRAM=TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
fail=0

echo "== source =="
# anchor_spl::token, spl_token, or any transfer/mint/burn CPI
if grep -rnE 'anchor_spl|spl_token|token::(transfer|mint_to|burn|close_account)|TokenAccount|Transfer *\{' "$SRC" 2>/dev/null; then
  echo "FAIL  demo_perps source references a token program or a token CPI"
  fail=1
else
  echo "ok    no token program, no token account type, no transfer CPI in $SRC"
fi

echo "== built binary =="
if [ -f "$SO" ]; then
  if strings "$SO" 2>/dev/null | grep -qF "$TOKEN_PROGRAM"; then
    echo "FAIL  the SPL Token program id is embedded in $SO"
    fail=1
  else
    echo "ok    $SO does not contain the SPL Token program id"
  fi
else
  echo "skip  $SO not built (run anchor build); source check alone is not enough"
  fail=1
fi

echo "== instruction list =="
if [ -f target/idl/demo_perps.json ]; then
  ixs=$(python3 -c '
import json
idl = json.load(open("target/idl/demo_perps.json"))
print(" ".join(i["name"] for i in idl["instructions"]))')
  echo "      $ixs"
  if printf '%s' "$ixs" | grep -qE 'transfer|withdraw|deposit|mint|burn|sweep'; then
    echo "FAIL  an instruction name suggests it moves value"
    fail=1
  else
    echo "ok    no instruction moves value"
  fi
else
  echo "skip  no IDL yet"
fi

[ "$fail" -eq 0 ] && echo "no-token-custody: clean"
exit "$fail"
