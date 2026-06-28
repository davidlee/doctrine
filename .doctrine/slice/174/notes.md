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
- VT-1 actionlint clean (host). VA-1 read passes. **VH-1 PROVEN** on `v0.8.1-rc1`:
  aarch64 + x86_64 both green end-to-end (cross-link resolved iconv on the arm
  runner, smoke passed both arches, assets published). macos-13 fallback unused.

## PHASE-03 — install channels + docs

- `install.sh` (repo root, curl|sh): POSIX `set -eu`, Darwin-only guard,
  `uname -m`→triple, version resolve (`DOCTRINE_VERSION` else latest GH release),
  download asset + `.sha256`, `shasum -a 256 -c`, best-effort quarantine strip,
  install to `${DOCTRINE_BIN_DIR:-$HOME/.local/bin}` + PATH hint. `REPO`/`BIN`
  single-sourced (STD-001).
- **Testable seam:** the `triple_for_arch` map is a pure function; the file is
  sourceable lib-only via `DOCTRINE_INSTALL_LIB_ONLY=1` so `scripts/install-test.sh`
  unit-tests the mapping in-jail (arm64/x86_64/reject) without running the installer.
- `Cargo.toml` `[package.metadata.binstall]` (pkg-url/pkg-fmt=tgz/bin-dir) →
  `cargo binstall doctrine` fetches the prebuilt asset. `cargo metadata` parses.
- README install reorder: curl|sh (rolling main + v0.8.1 tag pin) → cargo binstall
  → cargo install (with the `-liconv` toolchain caveat). justfile release recipe
  notes the v* tag → release.yml trigger.
- **Asset-name contract (§5.2)** is single-sourced across THREE consumers —
  release.yml, install.sh, `[metadata.binstall]`. Names: `doctrine-<triple>.tar.gz`
  + `.sha256`, tarball = one `doctrine` exe. Rename = edit all three together (R2).
- **STOP-2 (close sequencing):** GitHub `/releases/latest` excludes prereleases,
  so the default `curl|sh` (latest) + `cargo binstall` only resolve once a
  NON-prerelease **v0.8.1** is published. Decision: verify install.sh against the
  rc (`DOCTRINE_VERSION=v0.8.1-rc1`); cut the real v0.8.1 at /close to complete
  VH-1 (fresh-mac default one-liner + binstall, no toolchain).
- F5/governance: CI adoption is slice-local (design D1) — no ADR.
