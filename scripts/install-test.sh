#!/usr/bin/env sh
# install-test.sh — unit test for install.sh's uname→triple mapping, the fragile
# unit (design §9). Sources install.sh in lib-only mode (no network, no install)
# and asserts the mapping for both supported arches + an unsupported one. Runs
# anywhere (pure string mapping) — exercised in-jail on Linux. SL-174 PHASE-03.
set -eu

here="$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)"
# shellcheck source=/dev/null
DOCTRINE_INSTALL_LIB_ONLY=1 . "$here/install.sh"

fail=0
check() {
  # check <arch-input> <expected-triple>
  got="$(triple_for_arch "$1")" || got="<error>"
  if [ "$got" = "$2" ]; then
    echo "ok: $1 -> $got"
  else
    echo "FAIL: $1 -> expected '$2', got '$got'" >&2
    fail=1
  fi
}

check arm64 aarch64-apple-darwin
check x86_64 x86_64-apple-darwin

# Unsupported arch must exit non-zero.
if triple_for_arch ppc64 >/dev/null 2>&1; then
  echo "FAIL: ppc64 should be unsupported (non-zero)" >&2
  fail=1
else
  echo "ok: ppc64 rejected"
fi

if [ "$fail" -eq 0 ]; then
  echo "install-test: PASS"
else
  echo "install-test: FAIL" >&2
  exit 1
fi
