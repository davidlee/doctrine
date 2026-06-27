# Notes SL-166: Dispatch corpus-loss guards

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-02 — g3 3-way corpus-clobber gate (always-on) — DONE

Landed on edge: `774c1401` (predicate + seams), `92c928b8` (wiring + e2e),
merged `--no-ff` at `ddbdf853`. Scope-doc Model B fix `96e7676d`. `just gate`
green (clippy `--workspace`, fmt, all tests) at the fork base; re-verified on the
merged edge (g3 + seam + layering tests pass with concurrent work integrated).

**What shipped**
- `corpus_guard::corpus_clobber_check` — pure predicate over injected 3-way blob
  readings (`new==base ∧ cur≠base`, minus allowlist); `render_clobbers` capped at
  `CLOBBER_RENDER_CAP=20` (EX-5). corpus_guard stays a **pure leaf** (out=0,
  layering gate 17/0).
- `git::diff_doctrine_paths` (batched changed-set) + `git::blob_oid_at` (oid
  compare via `ls-tree`) — both **explicit exit-code handling, not `git_opt`**
  (EX-1, fail-closed on a bad tree-ish).
- `advance_row(root, row, allow)` runs g3 before the per-leg mutation; inert on a
  creation (`current==ZERO_OID`) or FF advance (`base==cur`); fail-closed on an
  unallowed clobber. `integrate` threads repeatable `--allow-corpus-clobber`
  onto a call-global allowlist, recorded on the committed `Journal.allowed_clobbers`.

**Design-as-built (decision, carries to PHASE-03/04)**
- Design §5.2's `corpus_clobber_check(root, base, new, cur, allow)` pseudo does
  the git I/O inline. **Built the layering-faithful split instead:** the predicate
  is a pure leaf over injected readings (EX-2 literal + the corpus_guard module
  doc), and the shell `dispatch::corpus_clobber_refusal` does the merge-base /
  diff / blob reads. g2 (PHASE-03) and g1 (PHASE-04) likewise have pure predicates
  — keep their I/O in the shell, predicates in `corpus_guard` (leaf), so the
  ADR-001 gate stays green. See [[mem.pattern.dispatch.g3-pure-predicate-shell-io]].

**EX-4 minor deviation** — design says "recorded on the integrate journal **row**";
the allowlist is call-global across both legs (§10), so it is recorded once on the
`Journal` manifest (`allowed_clobbers`), not per-row. Flag for audit if per-row is
wanted; trivially movable.

**VT/EX coverage map**
- VT-1 phantom deletion → `corpus_guard::phantom_deletion_is_clobber` +
  `integrate_edge_refuses_corpus_clobbering_advance` (the deletion shape end-to-end).
- VT-2 stale revert → `stale_revert_is_clobber`.
- VT-3 non-ff edge advance → `integrate_edge_refuses_corpus_clobbering_advance`
  (edge = review-bundle + extra `.doctrine` file; advancing back drops it).
- VT-4 ff never clobbers → `integrate_edge_fast_forward_advance_is_unaffected_by_g3`
  + `empty_changed_set_is_inert`.
- VT-5 authored/allowlist → `authored_delta_is_not_clobber`,
  `allowlist_lets_named_path_through`, `unnamed_path_still_refused_with_partial_allowlist`.
- EX-1 seams → `git::tests::diff_doctrine_paths_*` / `blob_oid_at_*`.
- EX-5 render → `render_clobbers_*`.
- INV-2 parity → pre-existing `integrate_edge_is_opt_in_and_aggregates_the_review_bundle`
  still green (a normal edge advance authors deltas, never clobbers).

**Process gotcha applied** — the `--no-ff` land left HEAD on the merge commit, so
the `completed` flip's auto-binding refused (F-6: boundary must be a non-merge
tip). Bound manually: `slice record-delta --start 239eb88e --end 92c928b8`.
This is the known [[mem.pattern.audit.fork-land-unbound-source-delta]].

**Phase order remaining:** PHASE-03 (g2, PRIMARY) → PHASE-04 (g1) → PHASE-05
(enable posture + INV-2 parity). g3 was sequenced first because it is
posture-independent and load-bearing today on the un-gated `--edge` leg
([[mem.fact.dispatch.edge-advance-leg-not-ff-gated]]).
