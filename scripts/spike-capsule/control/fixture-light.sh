#!/usr/bin/env bash
# fixture-light.sh — provision the LIGHT fixture (PHASE-01 T5, EX-4).
#
# `ledger`: a small TypeScript project, deliberately unlike this repo (D5) —
# trunk `mainline`, `[add] …` commit style, npm scripts, no Rust anywhere. Its
# job is to test the taxonomy's PORTABILITY: does a known trigger class have a
# TypeScript instance? A fixture conventioned like this repo would pass every
# row for reasons that say nothing about portability.
#
#   usage: fixture-light.sh [--force]
#   env:   SPIKE_CAPSULE_ROOT   capsule / fixture root (default: ~/capsules)
#
# Layout — the declaration is a SIBLING of the repo, never inside it (F-5):
#
#   $SPIKE_CAPSULE_ROOT/fixtures/light/
#     repo/                        the project, doctrine-installed
#     interpretation-surface.txt   copied from the rig's authored source
#
# No dependencies and NO `npm install`: Node strips TypeScript types natively
# and `tsc` comes from the environment. Provisioning therefore needs no network
# — one less thing to be flaky inside a capsule.
set -euo pipefail

RIG_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
# shellcheck source-path=SCRIPTDIR
# shellcheck source=../lib/common.sh
. "${RIG_DIR}/lib/common.sh"

force=0
while [ $# -gt 0 ]; do
  case "$1" in
    --force) force=1 ;;
    -h | --help)
      sed -n '2,22p' "${BASH_SOURCE[0]}"
      exit 0
      ;;
    *) rig_die "unknown argument: $1" ;;
  esac
  shift
done

# I6 — FIRST, before any provisioning.
rig_enter

src="${RIG_DIR}/fixtures/light"
fixture="${RIG_ROOT}/fixtures/light"
repo="${fixture}/repo"

[ -d "${src}" ] || rig_die "missing authored fixture source: ${src}"
guard_not_real_repo "${repo}"

if [ -e "${repo}" ]; then
  [ "${force}" -eq 1 ] || rig_die "fixture exists: ${repo} — pass --force to rebuild"
  rm -rf -- "${repo}"
fi
mkdir -p -- "${repo}"

# `git -C` everywhere, and identity pinned locally: the fixture's history must
# not depend on the host's git identity being configured, and must never adopt
# it silently.
git -C "${repo}" init --quiet --initial-branch=mainline
git -C "${repo}" config --local user.name "ledger fixture"
git -C "${repo}" config --local user.email "fixture@spike-capsule.invalid"
git -C "${repo}" config --local commit.gpgsign false

# The authored source carries `gitignore`, not `.gitignore`. A dotfile there
# would be a live ignore rule inside THIS repository's tree; it becomes a real
# `.gitignore` only once it is inside the fixture.
install -m 644 "${src}/gitignore" "${repo}/.gitignore"

commit() { git -C "${repo}" commit --quiet -m "[add] $1"; }

# ── commit 1 — scaffold ──────────────────────────────────────────────────────
install -m 644 "${src}/package.json" "${src}/tsconfig.json" "${src}/README.md" "${repo}/"
git -C "${repo}" add .gitignore package.json tsconfig.json README.md
commit "ledger scaffold — npm scripts, strict tsconfig"

# ── commit 2 — the failing test ──────────────────────────────────────────────
mkdir -p -- "${repo}/src"
install -m 644 "${src}/src/money.test.ts" "${repo}/src/"
git -C "${repo}" add src/money.test.ts
commit "failing test for cents conversion and rendering"

# RED, OBSERVED. A red→green claim backed by nothing but commit subjects is a
# claim about commit subjects. `money.ts` does not exist yet, so the import
# fails and `npm test` must be nonzero HERE.
printf 'observing RED at %s\n' "$(git -C "${repo}" rev-parse --short HEAD)"
if (cd "${repo}" && npm test) >/dev/null 2>&1; then
  rig_die "red step passed — the failing test does not fail (fixture is vacuous)"
fi
printf '  ok    npm test fails before the implementation lands\n'

# ── commit 3 — the implementation ────────────────────────────────────────────
install -m 644 "${src}/src/money.ts" "${repo}/src/"
mkdir -p -- "${repo}/tools"
install -m 644 "${src}/tools/format.mjs" "${repo}/tools/"
git -C "${repo}" add src/money.ts tools/format.mjs
commit "cents conversion, rendering, and a formatter"

printf 'observing GREEN at %s\n' "$(git -C "${repo}" rev-parse --short HEAD)"
(cd "${repo}" && npm test) >/dev/null 2>&1 ||
  rig_die "green step failed — HEAD does not pass its own suite"
printf '  ok    npm test passes at HEAD\n'

# ── doctrine ─────────────────────────────────────────────────────────────────
# A-install: `doctrine install` works in non-Rust projects (operator ruling), so
# a failure here is a real POL-002 finding, not an expected snag.
doctrine_bin="${DOCTRINE_BIN:-$(rig_repo_root)/target/debug/doctrine}"
[ -x "${doctrine_bin}" ] || rig_die "no doctrine binary at ${doctrine_bin}"

"${doctrine_bin}" install --path "${repo}" --yes >/dev/null ||
  rig_die "doctrine install failed on a non-Rust project — POL-002 finding, STOP and consult"

"${doctrine_bin}" slice new "Ledger rounding" --path "${repo}" >/dev/null
"${doctrine_bin}" slice selector add 1 'src/**' \
  --intent design-target --path "${repo}" >/dev/null

git -C "${repo}" add -A
commit "doctrine install, scratch slice SL-001, design-target selectors"

# ── declaration, OUTSIDE the repo ────────────────────────────────────────────
install -m 644 "${src}/interpretation-surface.txt" "${fixture}/interpretation-surface.txt"

# ── assertions (EX-4, EX-5) ──────────────────────────────────────────────────
printf 'asserting light-fixture invariants (EX-4, EX-5)\n'
rig_assert_eq "trunk is 'mainline', not this repo's convention" \
  "mainline" "$(git -C "${repo}" branch --show-current)"
rig_assert_eq "every commit uses the '[add] …' style" \
  "0" "$(git -C "${repo}" log --format=%s | command grep -cv '^\[add\] ' || true)"
rig_assert_eq "history is exactly the red→green→install sequence" \
  "4" "$(git -C "${repo}" rev-list --count HEAD)"

# Each script is RUN, not merely read out of package.json. Asserting that a key
# exists proves the fixture was authored; asserting it exits zero proves the
# fixture works — and the two came apart here (`build`/`lint` were declared and
# broken, because `tsc` cannot resolve `node:test` without @types/node).
# `clean` runs last: it deletes what `build` produced.
for script in build lint format test clean; do
  rig_assert "npm run ${script} succeeds" \
    npm --prefix "${repo}" run --silent "${script}"
done

rig_assert "doctrine is installed (.doctrine/ present)" test -d "${repo}/.doctrine"
rig_assert_eq "the scratch slice declares src/** as design-target" \
  "src/** design-target" \
  "$("${doctrine_bin}" slice selector list 1 --path "${repo}" --color never |
    tr -s ' ' | sed 's/ *-$//;s/ *$//')"
rig_assert "working tree is clean after provisioning" \
  test -z "$(git -C "${repo}" status --porcelain)"

rig_assert "declaration is present, as a sibling of the repo" \
  test -f "${fixture}/interpretation-surface.txt"
rig_assert "declaration is OUTSIDE the repo (F-5 non-live in the base fixture)" \
  test ! -e "${repo}/interpretation-surface.txt"

rig_assert_done "fixture-light"
printf 'light fixture ready: %s at %s\n' "${repo}" "$(git -C "${repo}" rev-parse HEAD)"
