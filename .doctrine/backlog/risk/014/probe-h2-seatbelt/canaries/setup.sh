#!/usr/bin/env bash
# Plant checksummed canaries OUTSIDE the probe worktree, record a manifest. Run
# before the battery. Mirrors probe-h1/canaries/setup.sh (sha256sum -> shasum -a
# 256 on macOS, which lacks coreutils sha256sum by default).
set -eu

# BASE = the probe sandbox root (gitignored scratch); WT lives under it.
BASE="${PROBE_BASE:?set PROBE_BASE to the probe scratch root}"
WT="$BASE/wt"
OUT="$BASE/outside"
CDIR="$BASE/canaries"
mkdir -p "$WT" "$OUT" "$CDIR" "$OUT/fakegit"

sha() { shasum -a 256 "$@"; }

# DUTMP = /var/folders/$USER/T (SL-183 F-E second temp surface). Canary paths
# under it prove the coarse DUTMP deny holds AND that the anchored xcrun_db
# re-allow does NOT leak to a deeper subpath (RV-203 F-3 anchoring boundary).
DUTMP="$(realpath "$(getconf DARWIN_USER_TEMP_DIR)")"
mkdir -p "$DUTMP/xcrun_db_x"   # deeper-subpath decoy dir (must stay DENIED under the anchored regex)

# Outside-wt canaries spanning the parity surface:
printf 'PARENT_CANARY_PRISTINE\n'  > "$OUT/parent_canary"          # parent-tree analog
printf 'GIT_CANARY_PRISTINE\n'     > "$OUT/fakegit/PROBE_CANARY"   # shared-.git analog
printf 'PTMP_CANARY_PRISTINE\n'    > /tmp/h2_ptmp_canary           # /private/tmp alias target
printf 'HOME_CANARY_PRISTINE\n'    > "$HOME/.h2_home_canary"       # $HOME target
printf 'DUTMP_CANARY_PRISTINE\n'   > "$DUTMP/h2_dutmp_canary"      # non-xcrun_db DUTMP path (must stay DENIED)
printf 'DUTMPDEEP_CANARY_PRISTINE\n' > "$DUTMP/xcrun_db_x/canary"  # deeper subpath past `xcrun_db` seg (anchor boundary; DENIED)

{
  sha "$OUT/parent_canary"
  sha "$OUT/fakegit/PROBE_CANARY"
  sha /tmp/h2_ptmp_canary
  sha "$HOME/.h2_home_canary"
  sha "$DUTMP/h2_dutmp_canary"
  sha "$DUTMP/xcrun_db_x/canary"
} > "$CDIR/manifest.sha256"

echo "planted canaries; manifest:"; cat "$CDIR/manifest.sha256"
echo "WT=$(realpath "$WT")"
echo "/tmp realpath = $(realpath /tmp)   (alias footgun: subpath matches resolved)"
echo "DUTMP realpath = $DUTMP"
