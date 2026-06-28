# SL-174 implementation notes

Durable findings harvested from phase execution. Phase sheets under
`.doctrine/state/.../phases/` are gitignored + disposable; anything that must
survive lives here.

## PHASE-01 — embed smoke-test gate

- `scripts/smoke.sh <binary>` is the embed-integrity gate (R1 mitigation):
  `--version`, the `install/` embed (asserts `.doctrine/templates/slice.toml`
  appears after `install --path <tmp> --yes`), and the `web/map/dist` map embed
  (boots `map serve --port 0`, parses the announced `http://127.0.0.1:<port>/`
  line, GETs `/`, requires HTTP 200 + non-empty body). Fails closed.
- `just smoke` runs it locally (web-build → `touch src/map_server/assets.rs` →
  cargo build → smoke). Same script CI reuses — no CI-only duplicate.
- Proven locally: with-dist binary PASS (exit 0); no-dist binary fails on the map
  check (curl status 000) → the gate catches a broken embed. shellcheck clean
  (host run).
- **F1 (design gap → resolved):** `doctrine` had **no `--version`** — clap
  `#[command]` never wired it though `Cargo.toml` is 0.8.1. Operator chose to WIRE
  it (`version` attr in `src/main.rs`) rather than weaken C1. Now reports
  `doctrine 0.8.1`. Design §5.2 assumed `--version` exists; it now does. Consider
  a §5.2 annotation at reconcile (no plan edit needed). `just check` green → no
  suite depended on its absence.

## PHASE-02 — release workflow + cross-link proof

- `.github/workflows/release.yml`: `v*` tag-triggered, `permissions: contents:
  write`, macos-14 arm64 runner, matrix over both apple-darwin triples,
  `just web-build` before cargo (embed), `scripts/smoke.sh` per artifact before
  upload, `doctrine-<triple>.tar.gz` + `.sha256` (single `doctrine` exe) per §5.2,
  `softprops/action-gh-release@v2`.
- Asset names carry the **triple only**, not the version (version = the release
  tag). install.sh / binstall (PHASE-03) are consumers of this contract — §5.2,
  R2: change names in one place, update consumers same commit.
- x86_64 is cross-compiled on the arm runner and smoke-tested under Rosetta
  (`softwareupdate --install-rosetta`). This re-enters the iconv link domain
  (R3-amended) — the riskiest step, unprovable in-jail. macos-13 native fallback
  leg is pre-armed (commented matrix entry) for a one-line flip.
- First CI in the repo. CI adoption is **slice-local** (design D1) — no ADR
  (F5 resolved).
- Runners lack `just` → `extractions/setup-just@v2`. dtolnay/rust-toolchain pins
  by channel `@stable` (its model); other actions pinned to major tags.
- VT-1 actionlint clean (host). VA-1 read passes. **VH-1 (real-tag x86 cross-link
  proof) is the gating verification — operator/GitHub, pending a tag push.**
