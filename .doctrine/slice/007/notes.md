# Notes SL-007: Memory anchoring & capture: record scope+git frame, verify, git seam

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Findings (durable)

### F1 — `canonical_bytes` returns `Result`, not `Vec<u8>` (PHASE-01)

plan.toml EX-2 prose says `canonical_bytes(&Value)->Vec<u8>` *and* "float-rejecting".
Incompatible: rejection under `panic="deny"`/`unwrap_used="deny"` can't be a bare
`Vec<u8>` return. Implemented as `Result<Vec<u8>, NonIntegerNumber>`, matching
forgettable's `to_canonical_bytes`. Internal callers (`checkout_state_id`) use
`.unwrap_or_default()` on all-string payloads that never error. **No further
action** — correctness-driven deviation from loose plan prose; criteria intent met.

### F2 — `CaptureError::Unborn` design/plan conflict — RESOLVE IN PHASE-02

`design.md` §5.2 lists `Unborn` as a `CaptureError` variant, but §5.5 + plan.toml
PHASE-02 EX-2/VT-1 say **unborn/non-repo → `Ok(Frame{anchor_kind:None})`** (a
captured None-anchor state, not an error). forgettable agrees: unborn → partial
non-writable frame (`Ok`); only non-repo is its own error there.

Resolution for PHASE-02: unborn **and** non-repo are the `AnchorKind::None` *Ok*
states (design §5.5 + constraint 4 — a repo-scoped record over a None frame is what
errors, at the `record` layer in PHASE-04). `CaptureError` carries only the
*unstable-frame* guards + git failures: `MultiRoot, Submodule, Symlink,
AmbiguousRemote, Git(String)`. Drop `Unborn` as an error variant. **Open
sub-question:** non-repo — `Ok(None)` (design §5.5) vs forgettable's
`NotARepository` error. design §5.5 wins for doctrine (None state, so a non-git
`record` can still write an unscoped memory; `verify` in non-git stamps the review
axis per Q-B). Confirm against §5.4 record flow before coding.
