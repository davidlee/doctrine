# CHR-017: Worker gate gap: just check fails in forks (lint-js needs gitignored node_modules; npx vs bun)

## Problem

`just check` = `fmt lint lint-js test build`. `lint-js` is `npx eslint web/map/`,
which needs `web/map/node_modules` (~120 MB / 6921 files, **gitignored**). A
`git worktree` fork only materializes tracked files, so node_modules is absent in
every dispatch worker / `.worktrees/*` fork → `lint-js` can't resolve eslint →
`just check` is unrunnable in a fork.

**Consequence (the bite).** Dispatch drives sidestep `just check` entirely and
hand-roll a partial gate (`cargo clippy` + `cargo test`), which silently drops
`cargo fmt`. SL-128 shipped an un-rustfmt-clean bundle for exactly this reason;
RV-118 (audit) caught it and repaired on `review/128` @ `72527bbd`. The fix is
already prescribed in `mem.pattern.dispatch.pi-arm-worker-ops` ("scope verify +
rustfmt --check touched files"), but that's guidance-not-followed — the durable
fix is making the gate itself safe/correct in a fork.

## Secondary defect — wrong runner

`web/map` is a **bun** project (`bun.lock`, `package.json` with `"lint": "eslint
--max-warnings=0"`). The recipe uses `npx`, not bun. Should be `bunx eslint` or
`cd web/map && bun run lint` — fold this in while touching `lint-js`. (Also: the
recipe runs `npx eslint web/map/` from the repo root, where there is no
`node_modules`; the eslint binary lives at `web/map/node_modules/.bin/eslint`.)

## Options (decide at design)

| | fix | cost | JS lint in workers? |
|---|---|---|---|
| **A** | guard `lint-js` → skip-loud when `web/map/node_modules` absent | ~1 line | no (runs at close, main tree has node_modules) |
| **B** | add `web/map/node_modules` to `.worktreeinclude` | +120 MB / 6921-file **copy per fork** | yes |
| **B′** | symlink node_modules into forks | cheap | yes — but `.worktreeinclude` copies, not symlinks (needs `worktree.rs` change) |
| **C** | split gate: `check` = Rust-only, `check-all` adds JS | small | no (explicit) |

`.worktreeinclude` (`src/worktree.rs`, `enumerate_candidates` via `git ls-files
--others --ignored`) is purpose-built to provision **gitignored** deps into a
fork — node_modules is its canonical target, so B is mechanically supported today,
just heavy. ~All dispatch phases are Rust, so paying a 120 MB copy per fork for a
lint-js step the phase never exercises is poor; B′ (symlink) removes the copy cost
but needs code. Provisional lean: **A** (or A+C) — cheapest, fixes the exact
failure, keeps JS lint authoritative where node_modules actually lives. Revisit if
workers should genuinely lint JS.

## Origin

Surfaced by RV-118 / SL-128 audit (F-1). Related:
`mem.pattern.dispatch.pi-arm-worker-ops`.
