# REV REV-029 — reconcile SL-223

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

Reconcile REV for SL-223 (publication seam), authored at close to resolve
status-lag drift the RV-288 F-3 coverage records surfaced.

SL-223 fully delivered and verified four SPEC-026 tech-spec mechanism
requirements (design §9 "Met", twice codex-reviewed — RV-286/RV-287 F-5). Their
authored status was still `pending` ("declared, not started"), now factually
stale against the shipped + verified implementation. This REV advances each to
`active` ("in force, verified"):

- **REQ-374 (FR-002)** — storage-independent `Resolver<A: SourceAdapter>` +
  storage-independence test (relocated backing resolves identically). Met.
- **REQ-376 (FR-004)** — neutral embedded byte-read seam (`asset_source` leaf),
  owned by neither projection nor publication. Met.
- **REQ-379 (FR-007)** — fail-closed {MIT, GPL} licence gate, staleness-proof,
  surfaced by `publication validate`. Met.
- **REQ-380 (NF-001)** — manifest independence / non-projection: manifest lives
  in a separate `publication/` embed root `build_plan` never walks. Met.

Scope boundary: the SPEC-026 foundation requirements REQ-373 (FR-001, embedded
covered / runtime-loaded pending OQ-4) and REQ-381 (NF-002, seam foundation), and
the PRD-017 invariants REQ-359/363/367/369, stay `pending` — their user-facing
outcome pends the deferred `library list|tree|show` verbs. Not advanced here.

Drives: RV-288 finding F-3. The verified coverage cells remain the implementation
evidence; this REV brings authored status into agreement. Residual stale-evidence
drift is discharged by the accept RECs authored alongside this REV at close.
