#!/usr/bin/env bash
# Fails if markov-guard's dependency tree (normal deps, all features) contains a
# crate that can do I/O, read a clock, or talk to an RPC. docs/09 §2.
set -euo pipefail
BANNED='^(tokio|async-std|reqwest|hyper|solana-client|solana-rpc-client|chrono|time)( |$)'
tree=$(cargo tree -p markov-guard -e normal --all-features --prefix none | awk '{print $1}' | sort -u)
if printf '%s\n' "$tree" | grep -Eq "$BANNED"; then
  echo "guard-purity: FAIL — markov-guard depends on:"; printf '%s\n' "$tree" | grep -E "$BANNED"; exit 1
fi
echo "guard-purity: clean ($(printf '%s\n' "$tree" | wc -l) crates in tree)"
