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
  ADR-001 gate stays green. This is the general
  [[mem.pattern.safety.resolve-every-ref-before-pure-compare]] principle (impure
  shell resolves git readings; pure leaf compares) applied to g3.

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

## PHASE-03 — g2 base-corpus freshness at setup (fail-closed) — DONE

Commits on the fork `slice/SL-166-corpus-loss-guards`: `d99afd53` (seam + 3
tests), `d14dde98` (gate + wiring + 5+2 tests). `just gate` green (clippy
`--workspace`, fmt, full test suite, build). NOT yet landed on edge.

**What shipped**
- `git::last_corpus_commit(root, refish, pathspec)` — tri-state corpus-tip seam
  (`git.rs`): `rev-parse --verify <refish>^{commit}` non-zero ⇒ **Err** (set-but-
  unresolvable = fail-closed, F-1); else `rev-list -1 <refish> -- <pathspec>` →
  empty ⇒ **Ok(None)** (resolves-no-corpus), else **Ok(Some(tip))**. Explicit
  exit-code (NOT `git_opt`); `pathspec` a PARAM (keeps git.rs leaf off corpus_guard).
- `worktree::coordinate::ensure_base_corpus_fresh(root, authoring_branch, base)`
  — the g2 shell check: None ⇒ inert; Ok(None) ⇒ inert; corpus tip not an
  ancestor of base ⇒ `Err(BASE_CORPUS_STALE)`. Called in `coordinate()` Create
  leg AFTER `base_has_slice_plan`, BEFORE `worktree add` with `base = trunk` —
  refuses before the fork, no worktree minted (EX-2/EX-3).
- Threading: `coordinate()`/`run_coordinate()` gained `authoring_branch: Option<&str>`.
  Resolved at command tier (`dispatch.rs` setup + `worktree/mod.rs` Coordinate
  dispatch, both `load_doctrine_toml(&root)?.dispatch.authoring_branch`).
  `coordinate.rs` imports NO `dtoml`/`dispatch_config` — VA-1 module-graph clean,
  layering gate green (worktree→dtoml is command→leaf, downward).
- Dropped `BASE_CORPUS_STALE`'s `#[expect(dead_code)]` (now consumed).

**Design-as-built (decisions, flag for audit)**
- **D-T2a single-value thread.** EX-4 says "resolved authoring_branch/**deliver_to**
  values"; g2's gate uses only `authoring_branch` + `base` (design §5.2 g2 snippet),
  so only `authoring_branch` crosses into `coordinate()`. `deliver_to` is consumed
  by g1 in the dispatch SHELL (PHASE-04), not coordinate. Minor EX-4 wording
  deviation — trivially extensible if audit wants the pair threaded.
- **D-T1a pathspec param.** EX-1 writes `last_corpus_commit(root, ref)` (2-arg);
  built 3-arg with `pathspec` to keep git.rs off `corpus_guard` — mirrors PHASE-02's
  `diff_doctrine_paths`. Same layering-faithful split as the g3 predicate
  ([[mem.pattern.safety.resolve-every-ref-before-pure-compare]]).
- **g2 vs g3 are absolute/relative (Model B, locked):** g2 here uses
  `is_ancestor(corpus_tip, base)` (absolute corpus floor); g3 uses
  `merge-base(new, cur)` (relative). They do NOT share a primitive — confirmed
  as-built, no accidental coupling.

**Config-validate ref check (design §5.2:178 "additionally") NOT in PHASE-03** —
no VT covers it; setup-time fail-closed (VT-3) is the PHASE-03 deliverable. The
validate-time resolution check would need git in the (currently pure) config
validate path — out of scope, flag only if a reviewer demands belt-and-braces.

**VT/EX coverage map** (`src/worktree/mod.rs` tests + `src/git.rs` tests)
- EX-1 → `last_corpus_commit_returns_tip_when_corpus_exists` /
  `_returns_none_when_ref_resolves_without_corpus` / `_errors_on_unresolvable_ref`.
- VT-1 → `ensure_base_corpus_fresh_refuses_when_base_predates_corpus` +
  `coordinate_refuses_create_when_base_predates_corpus` (wiring: no worktree dir).
- VT-2 → `_ok_when_base_carries_corpus`, `_noop_when_authoring_unset`,
  `_noop_when_no_corpus_yet`.
- VT-3 → `_refuses_when_authoring_unresolvable`.
- VA-1 → `grep` (coordinate.rs config-free) + `architecture_layering_gate` green.
- Behaviour-preservation: existing `coordinate_refuses_create_when_base_lacks_the_slice_plan`
  updated only for the new `None` 4th arg; still green.

**Phase order remaining:** PHASE-04 (g1, dispatch-shell verb guard) → PHASE-05
(enable posture + INV-2 parity + docs).

## PHASE-04 — g1 refuse trunk-mutating verbs on a buffer checkout — DONE

Commits on the fork `slice/SL-166-corpus-loss-guards` (see SHAs in the audit
handover). `just gate` green (clippy `--workspace`, fmt `--check`, full test
suite, build — exit 0, zero warnings).

**What shipped**
- `corpus_guard::on_integration_buffer(current, authoring, deliver_to) -> bool`
  — the pure g1 predicate (leaf): inert when `authoring` is `None` OR
  `authoring == deliver_to` (defensively inert on a misconfigured posture, EX-3);
  else refuses iff `current` is the short name of `deliver_to`.
- `corpus_guard::short_branch_name(refish)` — strips the single-source
  `REFS_HEADS_PREFIX` (`refs/heads/`) const; the form `symbolic-ref --short HEAD`
  reports. Reused by the predicate and the shell refusal message (DRY, STD-001).
- `dispatch::guard_not_on_integration_ref(root, cfg)` — the shell guard: reads the
  worktree-local HEAD via the existing `current_branch(root)` (`symbolic-ref
  --quiet --short HEAD`, EX-2 — same seam the raw-evidence-ref guard at
  `dispatch.rs:1048` uses), runs the pure predicate, and `bail!`s with
  `REFUSE_ON_TRUNK` naming the buffer ref + the `git fetch . <authoring>:<buffer>`
  (not `checkout`) recovery.
- Wired at the head of `run_integrate` (the verb entry, EX-1 — earliest/cheapest
  per design §5.4), AFTER `root::find`, loading `cfg` via
  `crate::dtoml::load_doctrine_toml(&root)?.dispatch`. ONE call site covers BOTH
  the `--trunk`/`--edge` legs and the candidate-active legs (they all land in the
  single `integrate()`; `run_integrate` is its sole caller).
- Dropped `REFUSE_ON_TRUNK`'s `#[expect(dead_code)]` (now consumed) — same move g2
  did for `BASE_CORPUS_STALE`.

**g1 verb-set enumeration (EX-1 / VA-1, confirmed against `dispatch.rs`)**
- GUARDED — `sync --integrate` (`DispatchCommand::Sync { integrate, .. }`
  → `run_integrate` `dispatch.rs:587`): the guard fires before `integrate()`
  `:1874`, which is the SOLE landing for both the legacy `--trunk`/`--edge`
  advance legs (`plan_trunk_row`/`plan_edge_row`) AND the candidate-active legs
  (`plan_candidate_trunk_row`/`plan_candidate_edge_row`, inner branches at
  `:1909-1924`). `integrate()`'s only caller is `run_integrate`, so one guard call
  is the complete cover.
- EXCLUDED (correct — F-4 / OQ-3, advance no integration ref):
  - `candidate create` (`candidate_create` `dispatch.rs:1037`) — writes a
    `candidate/*` ref + row, not the buffer.
  - `candidate admit` (`run_candidate_admit` `dispatch.rs:1273`) — writes
    `candidates.toml`, advances no ref.
  - `sync --prepare-review`, `--show-journal-trunk-oid`, `setup`, `refresh-base`,
    `record-boundary`, `plan-next`, `status`, `deliver-to`, `arm-spawn` — none
    advance `deliver_to`/`edge`.
  The `g1_guards_only_the_integrate_verb_entry` test pins this (exactly one
  production call site; candidate fns assert-free) so the set can't silently drift.

**Design-as-built (decisions)**
- **D-P4a: pure predicate in the leaf.** Followed the PHASE-02/03 layering split
  ([[mem.pattern.safety.resolve-every-ref-before-pure-compare]]): the decision
  (`on_integration_buffer`) is a pure `corpus_guard` leaf; the shell does only the
  HEAD read + message interpolation. The predicate is small but earns the leaf for
  testability + DRY with the EX-3 inert conditions.
- **D-P4b: g1 inert on `authoring == deliver_to`** (beyond design §5.2's bare
  `Some(_)` check). `validate_posture` rejects that config, but g1 may run on
  unvalidated config, so it stays defensively inert (EX-3 literal: "unset OR ==
  deliver_to"). No deviation — strengthens the snippet.
- **Call site = `run_integrate`, not `integrate`.** Keeps `integrate()` posture-
  free so existing e2e/unit suites that call it with HEAD on `main` stay green
  unchanged (INV-2). Posture is unset in those fixtures ⇒ inert regardless; the
  placement is the clean seam. No deviation from "verb entry" — `run_integrate` IS
  the CLI verb entry.

**VT/EX coverage map**
- EX-1 (verb-set) → `dispatch::tests::g1_guards_only_the_integrate_verb_entry`.
- EX-2 (worktree-local HEAD) → `integrate_refused_when_head_on_buffer` /
  `integrate_allowed_on_authoring_branch` (drive the real `symbolic-ref` seam over
  a temp repo with HEAD on `main` vs a checked-out `edge`).
- EX-3 (inert conditions + refusal wording) → `corpus_guard::tests::g1_inert_*`
  (`_when_posture_unset`, `_when_authoring_equals_deliver_to`, `_on_detached_head`)
  + the refused test's message assertions (names buffer ref + fetch-not-checkout).
- VT-1 (refuse on buffer) → `corpus_guard::tests::g1_refuses_when_head_on_buffer`
  (unit) + `dispatch::tests::integrate_refused_when_head_on_buffer` (shell+git).
- VT-2 (allowed on authoring / inert unset) →
  `corpus_guard::tests::g1_allows_on_authoring_branch` +
  `dispatch::tests::{integrate_allowed_on_authoring_branch,g1_inert_when_posture_unset}`.
- VA-1 (verb-set audit) → `g1_guards_only_the_integrate_verb_entry` (a test, not
  just a grep — pins the enumeration in CI).
- Behaviour-preservation (INV-2): existing dispatch suites green unchanged; the
  guard short-circuits to `Ok` when `authoring-branch` is unset (fixtures' case).

**Phase order remaining:** PHASE-05 (enable posture in doctrine.toml + INV-2
parity re-run + operator docs).
