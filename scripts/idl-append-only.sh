#!/usr/bin/env bash
# BlockReason is append-only. Discriminants 0–10 are exactly what the
# predecessor program 5o8E… emitted on devnet (docs/FACTS.md, decoded from 20
# on-chain ActionRefused payloads); 11–16 were appended for Gate B by ADR-004.
# Renaming, reordering or removing any of them breaks every receipt ever
# emitted, so CI fails the build instead.
#
# usage: scripts/idl-append-only.sh [path/to/idl.json]
set -euo pipefail
IDL="${1:-target/idl/markov_mandate.json}"
[ -f "$IDL" ] || { echo "idl-append-only: $IDL not found (run anchor build)"; exit 1; }

FROZEN="OverTxCap OverDailyCap OverSpendCap OverSpendDailyCap ProgramNotAllowed \
TokenNotAllowed SlippageExceeded Expired Paused Revoked Unauthorized \
StaleOracle ActionNotAllowed DuplicateIntent GlobalHalt VenueRejected PostCheckFailed"

ACTUAL=$(python3 -c '
import json, sys
idl = json.load(open(sys.argv[1]))
for t in idl.get("types", []):
    if t["name"] == "BlockReason":
        print(" ".join(v["name"] for v in t["type"]["variants"]))
        break
else:
    sys.exit("BlockReason not found in the IDL")
' "$IDL")

read -r -a want <<< "$FROZEN"
read -r -a got <<< "$ACTUAL"

fail=0
for i in "${!want[@]}"; do
  if [ "${got[$i]:-<missing>}" != "${want[$i]}" ]; then
    printf 'FAIL  discriminant %-2s is %-20s expected %s\n' "$i" "${got[$i]:-<missing>}" "${want[$i]}"
    fail=1
  fi
done
if [ "${#got[@]}" -lt "${#want[@]}" ]; then
  echo "FAIL  ${#got[@]} variants, expected at least ${#want[@]}: a variant was removed"
  fail=1
fi
if [ "$fail" -eq 0 ]; then
  extra=$(( ${#got[@]} - ${#want[@]} ))
  echo "idl-append-only: clean (${#want[@]} frozen variants in order$([ "$extra" -gt 0 ] && echo ", $extra appended"))"
fi
exit "$fail"
