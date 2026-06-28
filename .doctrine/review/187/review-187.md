# Review RV-187 — reconciliation of SL-174

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-174 (prebuilt binary distribution) against design.md,
plan.toml, and governance (SPEC-009, ADR-001, POL-002, STD-001). Reviewed the
authored surface on `edge` (not a dispatched slice — no candidate branch).

Lines of attack:
- **Path conformance** — `slice conformance SL-174`: every declared design-target
  delivered? Any undeclared touch (scope creep / missed design update)?
- **Verification evidence** — VT-1 (actionlint), VT-2 (cargo metadata), VA-1
  (asset-name contract), VH-1 (no-toolchain install). Mechanically proven, or
  asserted?
- **Spec coherence** — does binary *delivery* fit inside SPEC-009's embed/lay-down
  scope, or is it new evergreen surface (OQ-3)?
- **Invariants** — STD-001 single-sourcing of the asset-name contract across its
  three consumers; POL-002 host-independence of installer + workflow.

Build state: `just check` green (build + suites). Conformance: 6/6 design-targets
conformant, 0 undelivered.

## Synthesis

SL-174 ships prebuilt macOS doctrine binaries through a tag-triggered GitHub
Actions workflow, sidestepping the `cargo install` `-liconv` failure that
motivated the slice. The work is complete and coherent against its design.

**Closure story.** All three phases landed and verified: PHASE-01 the
embed-integrity smoke gate (`scripts/smoke.sh`, reused by CI — no duplicate),
PHASE-02 the release workflow (`release.yml`) with VH-1 proven end-to-end on
`v0.8.1-rc1` (both apple-darwin triples green, cross-link iconv resolved on the
arm runner under Rosetta, assets published), PHASE-03 the no-compile install
channels (`install.sh` curl|sh + `[package.metadata.binstall]`) and README
reorder. `just check` green; path conformance clean — all six declared
design-targets delivered, none undelivered.

**Conformance noise.** The `undeclared` cell carried foreign paths
(`.doctrine/review/186/*`, `.doctrine/slice/173/*`, `src/backlog.rs`,
`.doctrine/rfc/011/case-notes.md`) — concurrent-agent commits caught inside the
recorded source-delta oid ranges on the shared `edge` branch, not SL-174 scope
(`src/backlog.rs` last touched by `fe37354b review(173)`). Two genuine SL-174
touches surfaced as undeclared: the one-line clap `version` wiring in
`src/main.rs` (F-1) and the `install-test.sh` test seam (F-2). SL-174's own
slice governance files (`slice-174.toml`, `notes.md`) are authored state, not
code design-targets, and are expected outside the selectors.

**Standing risks.** The riskiest mechanism — x86_64-on-arm cross-compile through
the iconv link domain — is unprovable in-jail but was proven green on the rc; a
macos-13 native-runner fallback leg is pre-armed (commented matrix entry) for a
one-line flip if a future toolchain bump breaks cross-link. The asset-name
contract (`doctrine-<triple>.tar.gz` + `.sha256`) is single-sourced across three
consumers (release.yml, install.sh, binstall metadata) per STD-001/R2 — a rename
must edit all three in one commit; this coupling is documented in notes.md but
not mechanically enforced.

**Tradeoffs consciously accepted.** (1) `--version` was wired rather than
weakening the smoke gate's assertion (F-1) — the honest fix, documented for a
design annotation. (2) Slice-level VH-1's *default* (latest-resolving) channels
resolve only once a non-prerelease v0.8.1 is published; this is sequenced to
/close (F-4, STOP-2), the mechanism already proven on the rc. (3) Binary delivery
is new evergreen surface beyond SPEC-009 (F-3) — routed to reconcile to extend
the spec or record a boundary.

No unresolved blocker; ledger `done · await=none`.

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md §5.2 (F-1):** annotate that `doctrine --version` was *wired by this
  slice* (clap `version` attr in `src/main.rs`, PHASE-01) rather than
  pre-existing as §5.2 / EX-2a assumed. Documentary only — no plan edit
  (PHASE-NN / EN-/EX-/VT- ids immutable).

### Governance/spec (REV)
- **SPEC-009 (F-3, OQ-3):** binary *delivery* — the GitHub-release download
  channel (release.yml), the curl|sh installer (install.sh), and cargo-binstall
  metadata — is durable evergreen surface beyond SPEC-009's "embed + lay-down
  into a project" scope. Resolve via REV: either extend SPEC-009 to cover the
  delivery channel, or record an explicit scope boundary (delivery is out of
  SPEC-009 scope). Operator decision at reconcile.

### No write surface (carried to close)
- **VH-1 / v0.8.1 cut (F-4, STOP-2):** not a reconcile edit — an operator-only
  release action at /close. Cut a non-prerelease v0.8.1 (`just release` or manual
  `v0.8.1` tag, push) so the latest-resolving `curl|sh` + `cargo binstall`
  channels resolve, completing slice-level VH-1. Mechanism already proven on
  `v0.8.1-rc1`.
