#!/usr/bin/env bash
# Independent canary verifier — recompute checksums vs manifest. Any mismatch =
# an outside write got through = containment BREACHED. Mirrors probe-h1.
set -u
BASE="${PROBE_BASE:?set PROBE_BASE}"
CDIR="$BASE/canaries"

echo "== canary integrity (mismatch = ESCAPE) =="
if shasum -a 256 -c "$CDIR/manifest.sha256"; then
  echo "VERDICT: all canaries intact"
else
  echo "VERDICT: *** CANARY MUTATED — containment BREACHED ***"
fi
echo "== stray files left in outside dirs =="
ls -la "$BASE/outside" "$BASE/outside/fakegit" 2>&1 | sed 's/^/  /'
