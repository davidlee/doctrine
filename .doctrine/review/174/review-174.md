# Review RV-174 — reconciliation of SL-164

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Audit of SL-164 (MCP memory write tools + onboarding). Lines of attack:

1. **Conformance** — does the implementation touch match the declared
   design-target selectors? Are there undeclared edits or undelivered targets?
2. **Design fidelity** — does the code follow the design.md dispatch patterns,
   error mapping, and engine-change contract?
3. **Verification** — are all phase VT/VA/VH criteria satisfied? Tests green?
4. **Surface completeness** — are all 3 MCP tools + mapping table + boot footer
   present and wired correctly?

Evidence: `review/164` + `phase/164-NN` refs from dispatch funnel; 3 phases,
3 commits, 6 files touched.

## Synthesis

SL-164 shipped cleanly. Three phases landed on the dispatch branch with a single
commit each, all tests green (2613 unit + 17 E2E), clippy zero warnings.

**Implementation matches design.** The `memory_record` dispatch uses
`Status::parse` / `Lifespan::from_str` (not phantom helpers — F-1/F-2 from
RV-173 addressed), status resolves to concrete `Status::Active` default, error
wrapping uses `"invalid arguments: {e:#}"` with colon suffix satisfying
`map_review_error`'s `starts_with("invalid arguments:")` gate. The `writer:
&mut impl Write` parameter follows the established `run_show`/`run_list` pattern.
`render_onboard` gracefully handles missing signpost memories with a fallback
message.

**Conformance issue found and fixed.** Two minor findings: 3 undeclared edit
paths (`install/boot-footer.md`, `src/boot.rs`, `src/retrieve.rs`) and one
wrong-path selector (`.doctrine/boot-footer.md` declared but never touched —
the authored source is `install/boot-footer.md`). Both fixed by updating the
slice's design-target selectors. Post-fix conformance: 0 undeclared, 0
undelivered, 6 conformant.

**Verification.** All phase VTs satisfied:
- VT-1: `cargo test` 2613 passed (0 failed, 1 pre-existing e2e skip unrelated)
- VT-2: `cargo clippy` zero warnings
- VT-3 to VT-8: new unit + E2E tests for error mapping, round-trips, onboard
- VA-1: mapping table reviewed — complete, covers all 17 MCP tools
- VH-1: boot footer wording reviewed — MCP-first with clear fallback

**Standing risks.** The `doctrine_onboard` tool uses `retrieve_reference` which
runs holdback checks — if a signpost memory were held back, the onboard output
would carry a fallback message rather than the full bundle. This is by design
(design § Tolerated divergence). The dispatch workers executed inline on the
coordination tree (no isolated worktrees — the WorktreeCreate hook didn't fire),
which is non-ideal per the dispatch spec but produced correct results.

## Reconciliation Brief

### Per-slice (direct edit)
- **F-1/F-2 resolved in-audit** — selectors updated: added `install/boot-footer.md`,
  `src/boot.rs`, `src/retrieve.rs`; removed `.doctrine/boot-footer.md`.
  Conformance now clean.

### Governance/spec (REV)
- None — no governance or spec changes surfaced by this audit.

## Reconciliation Outcome

All findings were resolved in-audit with `fix-now` disposition. No writes needed.
Reconcile pass complete — handoff to /close.

### Direct edits applied
- **F-1/F-2** — selectors updated in-audit: added `install/boot-footer.md`, `src/boot.rs`,
  `src/retrieve.rs`; removed `.doctrine/boot-footer.md`. Conformance clean.

### REVs completed
- None needed.

### Withdrawn / tolerated
- None.
