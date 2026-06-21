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
--max-warnings=0"`). The recipe uses `npx`, not bun. Fold the runner fix in while
touching `lint-js`. Options:

- **bunx** — `cd web/map && bun run lint` (or `bunx eslint`); uses the project's
  own toolchain/lockfile, but still needs `web/map/node_modules` present (same
  fork-absence bite as above).
- **flake dep** — pin eslint as a nix flake input so it resolves hermetically and
  offline (no `node_modules`, no registry fetch in the jail). Heavier setup, but
  removes both the npx-vs-bun and the gitignored-node_modules problems at once —
  and the jail already builds the frontend hermetically (justfile comment, recipe
  `web-build`).

(Also: the recipe runs `npx eslint web/map/` from the repo root, where there is no
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

## Scope: project, not platform

Keep the two altitudes separate when fixing this — do not conflate in skills /
strategies:

- **Platform (doctrine-as-platform).** Generic, every consumer inherits it: forks
  materialize only tracked files → gitignored deps absent; the gate should be
  fork-safe (skip-loud); `.worktreeinclude` is the provisioning mechanism. This is
  the only part that belongs in shipped skills (dispatch/worktree) and IDE-017.
- **Project (doctrine-as-project).** This repo's frontend only: `web/map/dist`,
  bun, flake-pinned eslint, the `lint-js` recipe. These land in *project* surfaces
  — `justfile`, this repo's `.worktreeinclude`, this repo's flake — and must NOT
  leak into platform skills/strategy/guidance as if every doctrine consumer had a
  bun frontend.

So: the *guard pattern* (skip-loud when a gitignored dep is absent) is platform; the
*target paths and runners* (`web/map`, `bunx`, flake eslint) are project config.

## Second instance — gitignored RustEmbed source (not just node_modules)

A dispatch worker fork threw 3 spurious `map_server::assets`/`routes` failures: the
fork lacks the **gitignored** `web/map/dist` RustEmbed source, so the embed is empty
→ runtime fail (compiles fine). Same root as the node_modules bite above — a fork
materializes only tracked files, so any gitignored build artifact a test/gate needs
is absent. Ruled environmental at the time: green on base B, delta touched no
`map_server`, gone on the coordination tree (which has the artifact). Captured in
`mem.pattern.dispatch.worker-fork-missing-gitignored-embed`.

Bearing on the fix: this is a **second** gitignored target, which weakens the
"node_modules is the lone heavy outlier" framing behind the provisional lean on A.
Two distinct gitignored deps (node_modules for lint, `web/map/dist` for map_server
tests) argue for a general provisioning answer over per-dep guards — see IDE-017
(orchestrator-addressable worker provisioning when `.worktreeinclude` grows divergent
untracked state), the mechanism this chore's option B/B′ feeds into.

## Origin

Surfaced by RV-118 / SL-128 audit (F-1). Related:
`mem.pattern.dispatch.pi-arm-worker-ops`. Sibling latent mechanism: IDE-017.
