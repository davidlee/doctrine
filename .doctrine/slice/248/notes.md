# Notes SL-248: Capsule provisioning and Linux backend

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-06 · pre-design (research round complete) · 17860ef3

### Produced

- `DEC-153` (accepted) — answers `QUE-207`; two binaries split at canonical mutation.
- `IMP-404` — `SL-112`'s deferred engine/leaf crate extraction, captured.
- `ISS-319` — fresh-id allocation fails open when the trunk ref is unreachable.
- `QUE-208` — capsule-side entity id allocation; parked (see Open).
- `QUE-207` → answered.
- RFC-025 § State of play next-action 2 — rewritten to carry the five-slice
  decomposition, the unminted-until-predecessor rule, and the cross-cutting
  requirement warning.
- Research artefact — `research/research.md` + three `research/raw/*.md`
  (runtime tier, gitignored).

### Learned

- `POL-001` bans "load-bearing" plus tired physical metaphors — *plumbing, wiring,
  scaffolding, lever, seam, substrate, cut, sharp, spine* — across all English,
  including code comments and entity names. Three occurrences were corrected in
  authored records at `b33f2ce6`. It constrains this slice's module, error, and
  comment naming more than most.
- `dtoml.rs` is leaf-tier (`.doctrine/adr/001/layering.toml:32`) and deliberately
  tolerant. `reserve.rs:78-97` is the precedent for a validated typed projection
  over the shared file reader without an upward layering edge.
- `jail.rs:302-393` already implements REQ-449's refusal shape for jail policy.
- `jail.rs:806` — bwrap network is default-open; the capsule posture must invert it.
- REQ-461 is net-new: no free-space probe in `src/`.
- Governance sweep found **no revision candidates** for this slice.

### Open

- `QUE-208` — capsule-side entity id allocation. **Parked deliberately** 2026-08-06;
  does not block this slice. Live for the ingestion slice, unavoidable by the
  recovery slice. Its own first settling condition — whether v0 permits capsules to
  mint entities at all — is upstream of every option in it.
- `ISS-319` — separable defect; fixable independently of `QUE-208`.
- `SL-248` `OQ-3` — the shape of the cutover point (flag day / dual-run / atomic
  release). Belongs to the cutover slice, not this one.
- `SL-248` `OQ-4` — what replaces `review/*` and `phase/*` refs; REV-046 § ADR-012
  leaves it a target-design question.
- REQ-448, REQ-450, REQ-460 close in no single slice. This slice's share: REQ-448's
  denial half (proven by REQ-459's suite) and REQ-450 criterion 1.
- `IMP-397` / `QUE-204` — egress allowlist and non-Git build inputs, out of scope
  but adjacent to REQ-459's network-posture control.
