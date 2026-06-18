# Review RV-085 — reconciliation of SL-101

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-101 (Estimate & Value facets) against `design.md`, the
phase `VT-`/`EX-` criteria, ADR-001, and the SPEC-020/PRD-014 lineage. Candidate
surface reviewed: `refs/heads/candidate/101/review-001` (tip `700c6caa`) — the
no-ff 3-way merge of the impl bundle (`review/101`, `e459259f`) onto `main`
(`ec2de060`); the `notes.md` merge conflict was resolved (both sides kept).

Lines of attack (invariants this audit holds SL-101 to):

1. **ADR-001 leaf purity** — `estimate.rs`/`value.rs` import only external crates.
2. **FR-002 validation matrix** — present bounds required, finite, ordered; absent
   clean; no silent repair.
3. **NF-001 non-blocking** — no workflow predicate reads facet presence.
4. **NF-003 forward-compat** — unknown keys tolerated at parse, dropped at normalise.
5. **FR-004 round-trip** — valid facets survive parse→serialise→parse.
6. **Behaviour-preservation gate** — existing conduct/verification/slice suites
   stay green unchanged.
7. **Design §3.3 "No runtime effect in this slice"** — confidence bounds are
   "purely informational until consumed"; the slice must not give them a runtime
   effect that couples unrelated config reads.
8. **Design signature fidelity** — impl signatures match design §3.3/§4.3, or
   divergences are consciously dispositioned.

Evidence: 1767 bin tests pass on the candidate (0 failed, 1 ignored); `cargo fmt`
+ `cargo clippy` clean. Pre-existing main failure
(`dispatch_router_skill_is_shrunk`) is out of scope — SL-101's bundle does not
touch that SKILL.md.
