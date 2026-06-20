# SL-127 Design — Dispatch base freshness

> Status: drafted, pending adversarial review.
> Foundations locked interactively: Q1=A (freshen the dispatch branch), Q2=(i)
> (operator verb + auto-surfaced drift, no auto-refresh), Q3=(c) (ancestor-dominant
> ladder + plan-presence gate). Governed by ADR-006, ADR-011, ADR-012; specs
> SPEC-021 (orchestrator process) / SPEC-012 (mechanism container); honours RV-030
> F-1 (pinned fork-point).

## 1. Model

The dispatch base is currently a **frozen snapshot** of trunk taken once at
`setup`. Nothing advances it during the drive and nothing inspects its drift until
candidate-create time. The two witnessed failures (ISS-036, RSK-010) are the two
faces of that one defect. This slice makes the base a **tracked reference**:
selected ancestor-correctly at setup (axis 1) and explicitly advanceable, with its
drift observable, across the drive (axis 2).

The seam both axes ride is the merge-base primitive family already in `git.rs`:
`is_ancestor` (954), `merge_base` (994), `merge_tree`/`commit_tree_merge`/
`update_ref_cas` (the candidate-create machinery). No new git idioms are
introduced; the slice composes existing ones.

**Invariant preserved (RV-030 F-1 / SPEC-021).** The dispatch projection pins its
fork-point to `merge-base(dispatch, trunk)` precisely so a *foreign commit landing
on trunk mid-run cannot silently reparent the projection*. `refresh-base` does not
violate this: it advances the base by an **explicit, recorded operator action**
that also regenerates the bundle — the same shape as the already-sanctioned
*self-basing* step (SPEC-021: "the orchestrator advances coordination HEAD to phase
N's integrated tip before spawning N+1"). The pin bars *silent* reparenting by the
live tip, not *explicit* advance of B.

## 2. Axis 1 — Ancestor-dominant trunk ladder + plan-presence gate

### 2.1 Current behaviour
`git::trunk_ladder` (git.rs:1030) resolves `DOCTRINE_TRUNK_REF → origin/HEAD →
main → master`, **first that peels wins**. In a commit-on-main local-first repo
`origin/HEAD` lags local `main` by tens of commits, so the ladder forks a stale
ancestor. When that ancestor predates the dispatched slice's own authored
`slice/NNN/plan.toml`, `worktree::coordinate → slice::run_phases` hard-aborts deep
in phase-sheet regen (`Plan for slice N not found at …`) and rolls back —
recoverable today only via a `DOCTRINE_TRUNK_REF=main` prefix on *every* dispatch
command.

### 2.2 Target behaviour
Two layered changes:

**(a) Ancestor-maximal selection in the ladder (generic, framework-neutral).**
Resolve every *implicit* candidate that peels, then choose the **descendant** when
one candidate is an ancestor of another (most-advanced wins). Genuinely-diverged
candidates (neither an ancestor of the other) keep the original ladder order —
preserving today's behaviour for the ambiguous case. An explicit
`DOCTRINE_TRUNK_REF` is unchanged: it still short-circuits at the top and a bad
explicit ref is still a hard error (the F4/X6 asymmetry stands).

This fixes every *witnessed* case: a lagging `origin/HEAD` is by definition an
ancestor of `main`, so dominance selects `main`. It is framework-neutral — it
expresses no opinion about *which named ref* is trunk, only "never fork a base that
an equally-valid candidate strictly dominates."

Signature (new helper, `git.rs`):
```rust
/// The ancestor-maximal commit among `candidates` (resolved shas): the unique
/// one that every other is an ancestor of. `None` when the set is empty or the
/// candidates diverge (no single maximum) — caller falls back to ladder order.
fn ancestor_maximal(root: &Path, candidates: &[String]) -> anyhow::Result<Option<String>>
```
`trunk_ladder`'s implicit-candidate arm becomes: peel each of
`["origin/HEAD","main","master"]`; collect the resolved set (de-duplicated by sha);
if `ancestor_maximal` returns `Some`, use it; else fall back to first-resolves.

**(b) Plan-presence refuse-gate at the dispatch layer (slice-specific).**
`worktree::coordinate` (~1700), after resolving `trunk` and *before* `run_phases`,
asserts the chosen base's tree carries the dispatched slice's plan:
`git ls-tree <trunk> -- .doctrine/slice/<NNN>/plan.toml`. Absent ⇒ `bail!` early
with the `DOCTRINE_TRUNK_REF` hint, converting the deep regen abort into a single
clear pre-flight refusal. This backstops the rare *diverged* case (force-pushed /
rebased `origin/HEAD`) that dominance cannot decide. It lives at the dispatch layer,
not in the generic ladder, because plan-presence is slice-specific.

### 2.3 Why both
Dominance is the primary fix and handles every witnessed case neutrally. The gate
is cheap insurance for the diverged edge and replaces a confusing deep abort with
an actionable early error. (a) helps all ladder callers (`trunk_entity_ids` etc.);
(b) is dispatch-only.

## 3. Axis 2 — `refresh-base` verb, drift surfacing, conflict classifier

### 3.1 Reuse — drift is already computed
`dispatch::run_status` (dispatch.rs:1819) already derives trunk drift:
`fork_point = merge_base(dispatch_tip, trunk_tip)`, `ahead = rev-list --count
fork_point..trunk_tip`, `trunk_state = stable|moved` (tested:
`dispatch_status_moved_trunk`). **No parallel drift implementation.** Extract:
```rust
struct Drift { fork_point: String, ahead: u32 }
fn trunk_drift(root: &Path, dispatch_tip: &str) -> anyhow::Result<Drift>
```
`run_status` is refactored to call it (behaviour-preserving — existing status tests
stay green); the new call-sites (verb + classifier) share it.

### 3.2 The verb — `doctrine dispatch refresh-base --slice N`
Single responsibility: **advance the dispatch branch's base past trunk drift.** It
does *not* regenerate the bundle — the operator re-runs `dispatch sync
--prepare-review` afterwards (the existing step), matching the two-step shape the
SL-122 manual fix used. (Bundling regen into the verb is a deferred OQ — §6 OQ-1.)

Mechanism (all primitives existing; object-db merge, never a dirty worktree):
1. Resolve `dispatch_tip = dispatch/<NNN>`, `trunk_tip = trunk_commit`.
2. `mb = merge_base(dispatch_tip, trunk_tip)`. If `is_ancestor(trunk_tip,
   dispatch_tip)` (no drift) ⇒ report "already fresh", exit 0, no write.
3. `merge_tree(mb, dispatch_tip, trunk_tip)`:
   - **Clean** ⇒ `commit_tree_merge(tree, dispatch_tip, trunk_tip, "refresh(NNN):
     merge trunk into dispatch/<NNN>")`, then `update_ref` the dispatch branch
     under CAS on its expected tip. Advances B; bundle now contains trunk's later
     changes, so the eventual candidate-create 3-way goes clean.
   - **Conflict** ⇒ **report the conflicting paths and halt; no ref/worktree
     mutation.** Mirrors `candidate_create`'s no-`--worktree` abort and SPEC-021
     stage-2 *report-never-auto-resolve*. Operator resolves the trunk merge by hand
     in the coordination worktree, then re-prepares.

Preconditions: dispatch branch exists; coordination context resolvable. No trunk
write (writes only `dispatch/<NNN>`) — consistent with stage-1's no-trunk-write.

### 3.3 Conflict classifier at candidate-create
`candidate_create`'s `MergeTree::Conflict` arm (dispatch.rs ~417) today emits a
generic content-conflict message. Target: consult `trunk_drift`; when `ahead > 0`,
the bail text **names the base-divergence** and directs the operator to
`refresh-base` (then re-prepare + re-create), rather than implying a content
conflict the operator cannot fix on the bundle. When `ahead == 0` the existing
message stands (a true content conflict).

### 3.4 Next-step guidance
`run_status`'s `next_guidance`: when all phases complete **and** trunk has moved
(`ahead > 0`) **and** the bundle has not been refreshed past the drift, surface a
`RefreshBase` next-step ahead of `PrepareReview`/candidate-create. Turns the
invisible terminal conflict into an up-front prompt.

## 4. Code impact summary

| Path | Change |
|---|---|
| `src/git.rs` | `ancestor_maximal` helper; `trunk_ladder` implicit arm → ancestor-maximal-then-order. |
| `src/worktree.rs` | `coordinate`: plan-presence refuse-gate before `run_phases`. |
| `src/dispatch.rs` | extract `trunk_drift`/`Drift`; new `refresh-base` verb (`run_refresh_base` + core); classifier in `candidate_create` conflict arm; `RefreshBase` guidance arm in `run_status`. |
| CLI wiring (dispatch subcommand enum) | register `refresh-base --slice`. |
| `plugins/doctrine/skills/dispatch*/SKILL.md` | route to `refresh-base`; retire the `DOCTRINE_TRUNK_REF=main` env-prefix ritual. |

## 5. Verification alignment

- **Ladder** (`git.rs` tests): ancestor pair → descendant selected; diverged pair →
  original order; explicit `DOCTRINE_TRUNK_REF` still wins; bad explicit still hard
  errors (regression-guard the existing `trunk_ladder_explicit_*` tests stay green).
- **Plan gate** (`worktree.rs`): base lacking `slice/NNN/plan.toml` ⇒ early `bail!`
  with the hint; base carrying it ⇒ proceeds. No env prefix in either.
- **`refresh-base`** (`dispatch.rs`): reproduce SL-122 — seed a drift where trunk
  advanced past the fork with a same-block rewrite; `refresh-base` merges clean and
  advances `dispatch/<NNN>`; a following `candidate create` admits. Green test.
- **`refresh-base` conflict**: a genuinely-conflicting trunk merge ⇒ report-and-halt,
  dispatch ref unmoved, no worktree mutation.
- **Classifier**: `candidate_create` conflict with `ahead > 0` ⇒ message names
  base-divergence + `refresh-base`; with `ahead == 0` ⇒ existing message.
- **Drift helper**: extend `dispatch_status_moved_trunk` / add a `trunk_drift` unit.
- **Gate**: `just gate` green; `DOCTRINE_TRUNK_REF=main` workaround removed from the
  dispatch skills.

## 6. Open questions

- **OQ-1 — verb scope: merge-only vs bundle-regen.** §3.2 ships merge-only
  (operator re-runs `prepare-review`), favouring single responsibility and matching
  the proven SL-122 two-step. Bundling regen into one operator action is friendlier
  but couples two concerns. Settle before plan; default merge-only.
- **OQ-2 — does ancestor-dominance fully subsume the plan gate?** Likely not (the
  diverged case); both retained. Confirm no redundancy under test.

## 7. Governance touches (reconcile → REV, ADR-013)

- **ADR-006 D3 amendment**: the ladder ordering refines from a literal
  `origin/HEAD`-first order to ancestor-maximal-then-order. Small amendment / DEC.
- **SPEC-012 / SPEC-021 new REQ**: `refresh-base` as a mechanism verb (SPEC-012)
  and a between-phase cadence option (SPEC-021). Requirement authored at reconcile,
  coverage reconciled not inferred.

## 8. Non-goals (carried from scope)

Integrate-side phantom (ISS-038 / IMP-122), `[dispatch] deliver_to` config
(IMP-124 / IMP-101), `import --allow-reanchor` (IMP-043), and auto-refresh
(Q2 deferred) are out — adjacent seams / a proven-verb follow-up.
