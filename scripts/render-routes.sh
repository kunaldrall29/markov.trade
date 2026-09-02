#!/usr/bin/env bash
# Serve a built TanStack Start app with `vite preview` and write each route's
# rendered HTML to <outdir>/<route>.html so copy-grep can scan BUILT HTML.
# usage: scripts/render-routes.sh <app-dir> <outdir> <route>...
set -euo pipefail
APP=$1; OUT=$2; shift 2
mkdir -p "$OUT"
( cd "$APP" && npx vite preview --port 4173 --strictPort --host 127.0.0.1 >"$OUT/preview.log" 2>&1 ) & PID=$!
for i in $(seq 1 60); do curl -sf -o /dev/null -m 3 http://127.0.0.1:4173/ && break; sleep 1; done
for r in "$@"; do n=$([ "$r" = "/" ] && echo index || echo "${r#/}"); curl -sf -m 30 -o "$OUT/$n.html" "http://127.0.0.1:4173$r"; echo "rendered $r -> $OUT/$n.html ($(wc -c <"$OUT/$n.html") bytes)"; done
kill "$PID" 2>/dev/null || true; pkill -P "$PID" 2>/dev/null || true
