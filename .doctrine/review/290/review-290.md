# Review RV-290 — reconciliation of SL-212

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Post-implementation reconciliation audit of SL-212 (operator-ingest of a
hand-resolved trunk merge). The slice ships **5/5 phases**; governance is
settled (REV-030 applied — amended ADR-012 D4: `merge_oid` validated by
*provenance + content*, not authorship; FF-only publication untouched). The
design cleared **three adversarial design passes** (codex) plus a code-review
ledger (**RV-289**, projection stress-test).

**Reviewed surface.** Solo, non-dispatched slice — audit runs against the
working tree on `edge` (HEAD `7a92457dd`), not a candidate interaction branch.

**Lines of attack / invariants the slice is held to:**

1. **Conformance algebra.** `slice conformance` undeclared/undelivered cells vs
   the six `design-target` selectors — is every source touch declared?
2. **The one-engine invariant (D2).** Create *materialises* merge-tree's own
   `T_c` (`read-tree --reset -u`), never a second `git merge` whose config
   (`branch.<n>.mergeOptions`) could perturb the tree. Ingest recomputes the
   *same* merge-tree. Regression: `..._immune_to_merge_config`.
3. **The predicate (D1).** `diff(R.tree, T_c) ⊆ C` byte-wise, `--no-renames` —
   ordered parents `[base, source]`, markers advisory. Arbitrary trees refused.
4. **Exact projection (RV-289 F-2).** `read-tree --reset -u T_c` removes
   cleanly-deleted paths so the operator's `git add` cannot inflate `D`.
5. **Coordination-root identity (RV-289 F-1).** Guard keys on the candidate
   subpath alone (a coordination tree is *itself* a linked worktree).
6. **Fail-closed guards.** >1 merge-base refuse; custom (non-built-in) merge
   driver refuse; `R == base` refuse; write-once pre-state gate (exactly-one
   Conflicted ∧ empty-`merge_oid` row).
7. **Durability (D7).** `ledger::store` atomic (temp+rename); durable row
   before the worktree; crash≡resume bounded to referenced follow-up IMP-305.
8. **Non-regression.** `admit`/`integrate` unchanged; clean/non-worktree
   suites green; new `CandidateRow` fields serde-default-compat with legacy rows.
9. **Close-gate readiness.** RV-289's F-1 (blocker) / F-2 (major) are
   `answered/design-revised` but non-terminal — must be resolved or the
   `audit→reconcile` transition is refused.

**Evidence run this session:** `doctrine check gate` clean (no clippy warnings;
corpus citation-lint noise is pre-existing, unrelated); `cargo test --test
e2e_dispatch_candidate` → **35/35 pass** (ingest, D2-immunity, F-2 clean-deletion
projection, ordered-merge, refusal taxonomy); pure `validate_ingest_provenance`
unit set present; atomic-store regression present.

## Synthesis

**Closure story.** SL-212 lands the operator-ingest verb clean. The
implementation tracks a design that was hardened across three codex design
passes and a projection-focused code review (RV-289); this audit's job was
confirmation, not discovery, and it found no divergence between the shipped code
and the locked design or its governing decisions.

Every load-bearing invariant was verified against the source and exercised by a
green test:

- **One-engine (D2)** — create materialises merge-tree's own `T_c` via
  `read-tree --reset -u`; no `git merge` invocation exists for config to
  perturb. Proven by `..._immune_to_merge_config` (the D2 regression:
  `branch.<n>.mergeOptions=-Xours` in the fixture, create still projects
  merge-tree's tree).
- **Predicate (D1)** — `validate_ingest_provenance` enforces ordered parents
  `[base, source]` and `diff(R.tree, T_c) ⊆ C` on byte paths (`--no-renames`),
  markers advisory (fails open). Pure unit set + arbitrary-tree/reversed-parent
  refusal e2e.
- **Exact projection (RV-289 F-2)** — `read-tree --reset -u` removes
  cleanly-deleted paths; `..._projects_clean_deletion` guards the regression.
- **Coordination-root identity (RV-289 F-1)** — guard keys on
  `CANDIDATE_WORKTREE_SUBPATH` alone (a coordination tree is itself a linked
  worktree, so the `git-dir != common-dir` test could not discriminate);
  refusal + positive-acceptance tests both present.
- **Fail-closed guards** — >1 merge-base, custom (non-built-in) merge driver,
  `R == base`, and the exactly-one-Conflicted write-once pre-state gate all
  refuse; taxonomy cells cover rename/rename, modify/delete, add/add, binary,
  mode/symlink/gitlink, non-UTF-8 paths.
- **Durability (D7)** — `ledger::store` routes through `fsutil::write_atomic`
  (temp+rename); `store_is_atomic_and_leaves_no_temp` proves it; the Conflicted
  row is written before the worktree.
- **Non-regression** — `admit`/`integrate` unchanged (FF-only intact); new
  `CandidateRow` fields (`ingested_at`, `merge_provenance`) are
  `#[serde(default)]` with a legacy-row compat test.

**Conformance.** Six declared `design-target` selectors, all conformant; zero
undelivered. Two undeclared cells, both dispositioned non-blocking: a benign
enum-ripple in `src/mcp_server/dispatch.rs` (F-1 → reconcile, selector add) and
the slice's own authored TOML churn (F-2 → aligned, inherent noise).

**Governance.** Settled before implementation — REV-030 amended ADR-012 D4
(provenance + content, not authorship) and the slice ships within that bound
(clean-merge bar; IMP-303 exact-OID audit is a *related* follow-up, D4/R-2, not
a gate). ADR-012 §Verification's operator-ingest cases are realised as VTs
(arbitrary tree rejected, reversed parents rejected, genuine 3-way accepted,
FF-integrate by the same contract). No governance write is owed by this audit.

**Standing risks (disclosed, accepted).**
- **R-1** — a trusted operator may over-edit clean regions *within* a conflicted
  file; governance binds at path granularity (D3), bounded by trust + FF-only +
  admit + audit. Hunk-level is stricter than governance — out of scope.
- **R-4 / IMP-305** — crash between branch-create and row-write leaves an orphan
  ref (pre-existing in `create` today); atomic-store closes manifest corruption,
  full crash≡resume is a referenced follow-up (IMP-305, PRD-015), not absorbed.
- **R-2 / IMP-303** — inspectable ≠ inspected; ships at the clean-merge bar per
  D4. `related` link recorded.

None of these is new drift; each was consciously accepted in design and remains
correctly out of this slice's scope.

**Ledger closure.** RV-289's F-1 (blocker) and F-2 (major) were
`answered/design-revised` — design revisions that this session confirmed as
*implemented and tested*, then verified terminal (cooperative raiser role). With
RV-289 and RV-290 both `done · await=none`, no blocker gates
`audit→reconcile`.

## Reconciliation Brief

The audit surfaced **one** non-aligned finding touching a per-slice surface, and
**no** governance/spec write. Governance is already settled (REV-030 applied).

### Per-slice (direct edit)

- **RV-290 F-1 — selector registry gap.** `src/mcp_server/dispatch.rs` is a real
  SL-212 source touch (a `MergeTree::Conflict { .. }` destructure forced by the
  PHASE-01 enum widening) but is not in the `design-target` selector set, so
  `slice conformance` stays red. Load-bearing remediation is the **selector
  registry**, not prose:

  ```
  doctrine slice selector add SL-212 src/mcp_server/dispatch.rs --intent design-target
  ```

  Re-run `doctrine slice conformance SL-212` to confirm the undeclared cell
  clears (the two remaining `.doctrine/slice/212/*.toml` entries are the slice's
  own authored artefacts — F-2, aligned, expected to persist).

### Governance/spec (REV)

- **None.** REV-030 already amended ADR-012 D4 and is applied; ADR-012
  §Verification's operator-ingest cases are realised as VTs. No REV owed.

## Reconciliation Outcome

### Direct edits applied
- **RV-290 F-1** — selector registry: `doctrine slice selector add SL-212
  src/mcp_server/dispatch.rs --intent design-target`. `slice conformance` re-run
  confirms the path is now **conformant** (7 conformant, 0 undelivered). The
  undeclared cell now holds only the slice's own `plan.toml` / `slice-212.toml`
  (F-2, aligned — inherent, expected to persist).

### REVs completed
- **None.** The brief's Governance/spec section was empty — REV-030 already
  amended ADR-012 D4 and is applied; no governance/spec write was owed.

### Withdrawn / tolerated
- **RV-290 F-2** — aligned (no write): the slice's own authored TOML is inherent
  conformance noise, not drift; rationale in the finding disposition.

Reconcile pass complete — every brief item resolved. Handoff to /close.
