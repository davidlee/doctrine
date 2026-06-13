# Review RV-013 — reconciliation of SL-048

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciliation of SL-048 (structural cross-corpus relation edges) against its
locked `design.md`, `plan.toml` (PHASE-01..06 EX/VT), and governance (ADR-010 the
contract, ADR-004 outbound-only). The slice was driven via `/dispatch`: P01–P04 in
prior sessions, P05 by a dispatch worker (b62d527), P06 inline (4ce6d91).

**Subject surface.** Code: `src/relation.rs` (RELATION_RULES, read_block,
append_edge/remove_edge + EOF-defence, forward kind-check), `src/main.rs`
(link/unlink verbs, run_validate composition), `src/relation_graph.rs` (corpus-edge
walk, table-driven render_inbound), `src/integrity.rs`, `src/governance.rs`
(supersession_pair typed seam), `tests/e2e_link_unlink.rs`. Docs: ADR-010
amendment, SPEC-018, SPEC-005/006/016 rewire, IMP-032/IMP-006.

**Lines of attack / invariants held:**
1. **Report-only validation.** `validate` + supersession cross-check must NEVER
   rewrite a file (the reseat precedent) — only report + exit non-zero.
2. **Forward kind-check is NEW code (R2-M1).** `link SL-048 governed_by SL-003`
   must be refused on target-KIND even though `ensure_ref_resolves` passes;
   SameKind refuses cross-gov; Unvalidated accepts free-text.
3. **EOF-append defence (R2-m1/F1).** append never tail-inserts into a trailing
   `[[relation]]` array on a hand-edited file — re-home or refuse.
4. **Outbound-only by construction (ADR-004/ADR-010 D4/D5).** No inverse label
   (`superseded_by`) authorable via link; supersession pair excluded (OD-3).
5. **X5 inbound render.** `governed_by` derives "governs"; legacy
   `inbound_name == name()` keeps goldens byte-identical; `--json` keeps raw label.
6. **Storage rule (PHASE-06).** No queried/derived enumeration duplicated into
   prose; SPEC-005/006/016 reference SPEC-018, tell ONE consistent story with the
   shipped RELATION_RULES; whole-corpus `validate` clean.
7. **Layering (ADR-001).** validate edge-walk built without pulling relation_graph
   into the integrity leaf (the worker composed it in `main::run_validate`).

## Synthesis

SL-048 reaches close clean. The write path and its teeth (PHASE-05) and the
governance reconcile (PHASE-06) both land faithful to the locked design, with the
full `--workspace` gate green and whole-corpus `validate` clean on the migrated
corpus. Each invariant in the Brief was checked against the **landed code**, not the
worker's report:

- **Report-only validation (1)** holds — `append_edge`/`remove_edge` write only on
  an actual change (`Wrote`/`Removed`), and the entire `validate` path
  (`validate_relations` + `validate_supersession`) returns finding strings and never
  touches a file. The reseat precedent is honoured.
- **Forward kind-check (2)** is genuinely new code (`relation::check_target_kind`),
  wired in `run_link` *after* `ensure_ref_resolves` and *only* for non-`Unvalidated`
  targets — so `link SL-048 governed_by SL-003` is refused on kind, `SameKind`
  refuses cross-gov, and free-text `drift` skips both gates. Pinned by unit + e2e.
- **EOF defence (3)** refuses (does not silently corrupt) a trailing typed table via
  `trailing_typed_table_after_relation`, with the idempotent no-op guard ordered
  *first* so a re-link never inspects layout. Design allowed "re-home or refuse";
  refuse was chosen — acceptable.
- **Outbound-only (4)** is structural: `RELATION_RULES` admits no inverse label, gov
  `supersedes` is `LifecycleOnly` (refused by `link`, owning verb named), and the
  supersession pair stays typed (OD-3). `supersession_pair` reads `superseded_by`
  via the typed governance seam, exactly as R2-m2 requires.
- **X5 (5)** surfaced the one real defect (F-1): a *latent* PHASE-04 gap where
  `render_inbound` never moved to the new `inbound_name`, so `governed_by` would
  render its inbound backwards. No data exercised it, so goldens held — it would have
  failed EX-1's X5 had any `governed_by` edge existed. Caught and reconciled
  in-scope (PHASE-05) by making inbound render table-driven; goldens stayed
  byte-identical. Disposed **fix-now**, fix already in `b62d527`.
- **Storage rule (6)** holds — PHASE-06 added prose references to SPEC-018 from
  SPEC-005/006/016 and corrected the now-stale "`[relationships]` inert" descriptions
  (governance `related` migrated to `[[relation]]`); no enumeration is transcribed
  into prose, and the ADR-010 amendment, SPEC-018, and the three rewired specs tell
  one consistent story with the shipped table.
- **Layering (7)** holds — the `validate` edge-walk lives at the command layer
  (`main::run_validate`), composing `integrity` (id-scan) and `relation_graph`
  (edge-walk, which depends back on `integrity`) without a cycle.

**Standing risks / consciously accepted.** Two nits, both tolerated: a loose
"`[[relation]]` edge" message for a dangling *typed* gov `supersedes` (detection
sound, wording only — F-2), and `validate`'s three-pass disk read over the corpus
(cold command, immaterial at scale — F-3). Neither warrants follow-up work.

**Open follow-up (pre-existing, owned).** The OD-3 exclusion stands until the
transactional supersede verb exists — owned by **IMP-006** (recorded), with the
read-side guard (the cross-check) already shipped and IMP-032 reclassified to it. No
*new* deferral is created at this close.
