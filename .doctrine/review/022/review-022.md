# Review RV-022 — reconciliation of SL-062

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Post-implementation conformance audit (mode: conformance) of SL-062 across its
three phases, all orchestrator-verified green via the `/dispatch` funnel.

**Lines of attack:**
1. **Behaviour-preservation gate** — the existing gov/slice/backlog/requirement
   setter suites + slice FSM suites must stay green with assertion text unchanged
   (only import-path shifts + the EX-4 gov/requirement non-destructive F-1 reword).
2. **ADR-001 no-cycle** — `lifecycle.rs` must be a pure leaf (no kind module, no
   disk/clock import).
3. **P1 one mutation core** — `apply_string_append` and `dep_seq::append`'s `Needs`
   arm share `push_str_if_absent`; the `After {to,rank}` struct path untouched (R3).
4. **P2 transaction** — `supersede` parses both docs once, mutates held docs, writes
   NEW-then-OLD once each; no second write seam, no re-parse.
5. **D4 capability boundary** — `supersede_policy(kind)` is a hardcoded ADR-only
   match (`None` for POL/STD/slice), not GovKind data.
6. **No-op semantics** — `apply_status` no-op excludes the derived `updated` stamp,
   reproducing every donor's no-op identity.
7. **Atomicity tradeoff (R1/F-F)** — a torn supersede mid-write is detectable by
   `doctrine validate` (relation_graph), not the verb itself.

**Evidence base:** `just gate` + `just check` green on the combined tree at every
phase landing; `e2e_supersede` (7 tests) green; `supersede` verb ships in the fresh
jail-target binary; `lifecycle.rs` import scan clean.

**Standing concern (process, not code):** all three dispatch workers integrated
their commit directly onto shared `main` — `isolation: worktree` did not hold the
orchestrator-sole-writer split. Each delta was nonetheless clean (exact declared
files, R-5 clean, parent==B linear) and verified green POST-landing. Probed as a
delivery-integrity risk; SL-064 (foreign, in-flight) owns the systemic fix.

## Synthesis

**Closure story.** SL-062 lands as designed: the slice-trapped lifecycle FSM is now
a pure leaf (`src/lifecycle.rs`) beside `conduct.rs`, completing the ADR-009 two-leaf
pairing (D1); the byte-duplicated edit-preserving TOML write-core is collapsed to ONE
mutation seam in `src/dep_seq.rs` (`apply_status` / `apply_string_append` pure cores +
`set_authored_status` / `append_string_array` IO wrappers), with four status setters
retired onto it while each keeps its own gate in its shell (D3); and the top-level
`supersede <NEW> <OLD>` verb composes those cores in a parse-once / hold-both /
write-once transaction, ADR-first via a hardcoded `supersede_policy(kind)` capability
boundary (D4). All three phases were orchestrator-verified green (`just gate` +
`just check`); `e2e_supersede` (7 tests) passes; the verb ships in the fresh
jail-target binary. The behaviour-preservation gate held — existing setter and FSM
suites stayed green with assertion text unchanged except the EX-4 gov/requirement
non-destructive F-1 hint reword.

**Invariants confirmed.** ADR-001 no-cycle (`lifecycle.rs` imports no kind module /
disk / clock); P1 one mutation core (`push_str_if_absent` shared by
`apply_string_append` and `dep_seq::append`'s `Needs` arm; the `After {to,rank}`
struct path byte-untouched, R3 — SL-060 needs/after suites green); P2 single
transaction (no re-parse, no second write seam); ADR-004 carve-out (`superseded_by`
written only by the verb); no-op identity excludes the derived `updated` stamp,
reproducing every donor's semantics.

**Standing risks / consciously-accepted tradeoffs.**
- *Two-file non-atomicity (F-3, aligned).* The supersede transaction is not FS-atomic;
  R1 accepts this. NEW-then-OLD write ordering makes the F-D one-sided no-op sound, and
  a torn state is detected by `doctrine validate` (relation_graph), not the verb (VT-6).
- *Dispatch sole-writer bypass (F-1, tolerated).* A delivery-mechanism drift, not a
  code defect; every delta was clean + R-5 + verified. SL-064 owns the systemic fix.
- *Incomplete DRY collapse (F-2, follow-up).* The fifth byte-identical setter
  `knowledge::set_record_status` remains un-retired; IMP-061 carries the fold.

No unresolved blocker. Code and design are reconciled. F1 (destructive-verb carve-out),
F2 (POL/STD/slice supersession vocab), and F3 (SL-048 OD-3 unblocked) are CLOSE-time
mints, handled by `/close`.
