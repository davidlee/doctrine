# Implementation Plan SL-223: Publication seam

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases realise `design.md` for the first Contract B slice — the publication
seam. The design's decisions map cleanly: PHASE-01 is D-B/D3 (the neutral leaf
seam), PHASE-02 is D1/D6/D9/D-E plus the D-A production consumer's admission half,
PHASE-03 is D2/D-F (resolver + emit), PHASE-04 is the release-artifact hardening
that RV-287 F-2/F-3 forced. Each phase ends green and independently gate-clean.

## Sequencing & Rationale

**Why this order — the `deny(unused)` constraint drives it.** The crate's lint
posture (`warnings=deny`, `unused=deny`, `unreachable_pub=deny`) makes a
`pub(crate)` API a binary reaches only from `#[cfg(test)]` a *hard compile error*
(this is exactly codex RV-287 F-4, the finding that reversed D-A). The corollary
governs phase boundaries: **every phase must land the production consumer of the
API it introduces, in the same phase.** You cannot build the engine in one phase
and wire the command in the next — the intermediate phase would not compile. So:

- **PHASE-01 — neutral seam first, alone.** The `asset_source` extraction is a
  behaviour-preserving refactor with an unambiguous proof (install's existing
  suites stay green *unchanged*). Doing it first de-risks the foundation the whole
  slice rides on, and it has its own production consumers already (install's
  `asset_text`/`embedded_asset` delegate to it), so it is dead-code-clean without
  any publication code. Landing it separately keeps the refactor's blast radius
  auditable against the behaviour-preservation gate.

- **PHASE-02 — declaration + admission + the command that consumes them.** The
  admission API (`admit`, `load`, the enums, `LogicalAddress`) is introduced
  *together with* `publication validate`, which `load()`s the shipped manifest.
  That command is the production consumer keeping the API live. The `publication/`
  embed root and the templates-only manifest land here too, because `load()` needs
  real bytes and the command test needs a real pass. The non-projection regression
  test co-locates here — the moment the new embed root exists is the moment its
  leakage risk exists, so the guard lands with it.

- **PHASE-03 — resolver + emit, command extended to consume them.** `resolve`/
  `emit` are added and the command grows to `emit` every entry into `io::sink()`.
  Same discipline: the new engine surface gets its production caller in-phase.
  `Resolver<A: SourceAdapter>` is generic (RV-287 F-1) specifically so the
  storage-independence VT can drive a second in-memory adapter through the *same*
  API a concrete field could not express.

- **PHASE-04 — release-path last, host-gated.** The crane embed-strip + Cargo
  packaging + Nix probe are a distinct concern (the "asset ships hollow with no
  compile error" footgun family) and are host-only — nix is absent in the jail.
  Isolating them in a final phase keeps the host-only boundary explicit and stops
  it blocking jail-side execution of PHASE-01..03. The probe is only *possible*
  because PHASE-03 gave us a binary that observes the embedded bytes.

## Notes

**The dead-field corollary (PHASE-02).** `deny(unused)` flags never-read struct
fields, not just unused items. A serde-deserialized `PublicationEntry` field that
admission does not read is a hard error. So admission must *read* every field it
parses (title non-empty, backing non-empty, licence ∈ set, provenance/customization
known, address well-formed) or the command report must surface it. This is why the
admit validations in EX-2 are exhaustive over the field set — it is a compile
constraint, not just thoroughness.

**Command wiring sites (PHASE-02, all in `src/commands/`).** A new command touches
five places, mirroring `validate`: the handler `publication.rs::run_publication_validate`;
`mod.rs` module registration; `cli.rs` the `Command` variant *and* its dispatch arm
(~cli.rs:1332); a `PublicationCommand::Validate` subcommand enum; and `guard.rs`
read-only classification (a new command absent from the guard's read/mutate split
can trip its completeness check). Riding `run_validate`'s `writeln!(std::io::stdout())`
+ `anyhow::bail!` shape keeps us clear of the `print_stdout` clippy deny.

**ADR-001 layering registration (PHASE-01/02).** The `architecture_layering` gate
(`tests/architecture_layering.rs`, run by `just gate`) fails with
`Violation::Unclassified` for any import-graph module absent from
`.doctrine/adr/001/layering.toml [tiers]`. So each module-introducing phase must add
its row: `asset_source = "leaf"` (PHASE-01), `publication = "engine"` (PHASE-02).
`commands::publication` needs none — `commands = "command"` is classified wholesale.
All new edges are downward (install/publication → asset_source; commands →
publication → asset_source), so no new tangle-baseline is needed.

**Opportunity surfaced, out of scope.** `install = "command"`, and layering.toml
line ~178 baselines a pre-existing ADR-001 wart: engine/leaf callers reach *up* into
`install::asset_text`. Extracting the read into `asset_source` (leaf) makes it
*possible* to retire that wart later by redirecting those ~30 external callers to the
leaf (a downward edge). This slice deliberately keeps `install::asset_text` a
delegating shim and does NOT touch external callers (minimal blast radius,
behaviour-preservation). The cleanup is a separate backlog candidate, not SL-223 work.

**Verification-mode choices.** PHASE-01..03 are VT-heavy (engine + command are
jail-testable under `just gate`). PHASE-04 is VA — `just nix-build` and
`cargo package --list` are host/release-check gates, not per-commit tests (nix is
not on PATH in the jail), so they are agent/human-verified on the host rather than
marked VT and silently skipped. PHASE-01 VA-1 covers the behaviour-preservation
gate (no test-body edits), which a keyword match cannot assert.

**Coverage honesty (design §9, RV-287 F-5).** The plan does not attempt to *meet*
the user-facing PRD invariants (REQ-359/363/367/369) — their acceptance criteria
need the deferred `library` verbs. This slice delivers and proves the mechanism
(REQ-374/376/379/380 met; the PRD invariants partial/foundation), recorded at
reconcile.
