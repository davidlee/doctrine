# SL-127 Design — Dispatch base freshness

> Status: drafted; internal pass (F1/F3/F4/F6) + external codex pass (C2–C8)
> integrated — see §9. **One open decision before lock: C1** (ladder neutrality —
> §2.2a residual). Pending user confirmation on C1, then lock → `/plan`.
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
/// Fold `candidates` (resolved shas, in ladder-preference order) toward the
/// freshest reachable base: start at the first, and step to a later candidate
/// only when it is a *descendant* of the current pick; a candidate that diverges
/// from (is not a descendant of) the current pick is skipped, never regressed to.
/// So a stale ancestor is always overtaken by a descendant later in the list,
/// while a diverged candidate cannot drag the pick backwards. NOT a global
/// maximum (codex C2) — "preferred-order, advance-to-descendant".
fn freshest_descendant(root: &Path, candidates: &[String]) -> anyhow::Result<Option<String>>
//   candidates.iter().try_fold(None, |acc, c| match acc {
//     None => Some(c),
//     Some(a) if is_ancestor(a, c)? => Some(c),   // c descends a → advance
//     Some(a) => Some(a),                          // diverged/older → keep a
//   })
```
`trunk_ladder`'s implicit-candidate arm becomes: peel each of
`["origin/HEAD","main","master"]` **in that order**; collect the resolved shas
(de-duplicated); fold with `freshest_descendant`. Order is the *preference* seed
(so genuinely-diverged candidates resolve to the most-preferred), and the fold only
ever advances to a strict descendant.

> **F1 (internal):** an earlier "unique global maximum, else fall to order"
> formulation reintroduced the bug for `{origin/HEAD(old), main(new),
> master(diverged)}` — no unique max ⇒ fall to order ⇒ stale `origin/HEAD`. The
> left-fold above picks `main` there.
>
> **C2 (codex):** the fold is **not** a true global maximum — it is
> "preferred-order, advance-only-to-a-descendant". A pathological set where the
> *most-preferred* candidate diverges from a fresher later one (e.g.
> `origin/HEAD` diverged, `main < master`) keeps `origin/HEAD`. We accept this:
> (i) it never *regresses* below the most-preferred resolvable ref, (ii) the
> realistic failure (origin/HEAD an *ancestor* of main) is handled, and (iii) the
> **plan-presence gate (§2.2b) is the backstop** — a diverged stale pick lacking
> the slice's `plan.toml` is refused with the hint, not silently forked. Naming
> reflects this: the helper is `freshest_descendant`, not `ancestor_maximal`.

**(b) Plan-presence refuse-gate at the dispatch layer (slice-specific).**
`worktree::coordinate` (~1700), after resolving `trunk` and **before the `git
worktree add -b` fork** (F6, not merely before `run_phases`), asserts the chosen
base's tree carries the dispatched slice's plan:
`git ls-tree <trunk> -- .doctrine/slice/<NNN>/plan.toml` (reuse the
`trunk_entity_ids` tree-read idiom — F2). Absent ⇒ `bail!` early with the
`DOCTRINE_TRUNK_REF` hint, converting the deep regen abort into a single clear
pre-flight refusal. This backstops the rare *diverged* case (force-pushed /
rebased `origin/HEAD`) that dominance cannot decide. It lives at the dispatch layer,
not in the generic ladder, because plan-presence is slice-specific.

> **F6 (internal):** gating *before* the worktree fork means a bad base never
> creates a worktree to roll back — which sidesteps RSK-010's "setup rollback
> reverted uncommitted session-`main` WIP" sharp edge **for the base-staleness
> cause** (the rollback path is never entered). The general rollback-safety
> remedy (RSK-010 candidate (e): rollback must never touch the session tree for
> *any* cause) stays out of scope — flagged as a follow-up.
>
> **C3 (codex) — gate scope is the `Create` fork only.** The gate guards
> `CoordAction::Create` (the off-trunk fork — the ISS-036 cause).
> `CoordAction::Resume` reattaches a *pre-existing* `dispatch/<NNN>` branch (not a
> fresh trunk fork), so a stale-Resume that deep-fails in `run_phases` is a
> *different* failure than base-selection and is **explicitly out of scope** here.
> A symmetric Resume preflight (assert the resumed tip carries `plan.toml`) is a
> recorded follow-up, not v1.

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
/// Drift of `tip` against current trunk: fork_point = merge_base(tip, trunk),
/// ahead = count(fork_point..trunk). Parameterized on `tip` (F4) so the
/// classifier can measure the *bundle/source*, not only the dispatch branch.
fn trunk_drift(root: &Path, tip: &str) -> anyhow::Result<Drift>
```
`run_status` is refactored to call `trunk_drift(root, dispatch_tip)`
(behaviour-preserving — existing status tests stay green); the verb and the
candidate-create classifier share it on their relevant tips.

### 3.2 The verb — `doctrine dispatch refresh-base --slice N`
Single responsibility: **advance the dispatch branch's base past trunk drift.** It
does *not* regenerate the bundle — the operator re-runs `dispatch sync
--prepare-review` afterwards (the existing step), matching the two-step shape the
SL-122 manual fix used. (Bundling regen into the verb is a deferred OQ — §6 OQ-1.)

**Operates in the live coordination worktree (revised — codex C4).** The coord
worktree is checked out on `dispatch/<NNN>` and parked live for the whole drive
(SPEC-021), so `refresh-base` runs a **real `git merge`** there — not an object-db
`merge_tree`. This is deliberate: the verb's job is to advance the very branch the
operator is working on, and a conflict must leave a *materialised, resolvable*
state. (`candidate_create` uses object-db `merge_tree` precisely to avoid dirtying
a worktree it does not own; the opposite is correct here — the operator owns this
tree.) `merge_tree` was rejected because it returns only a `Conflict` token with no
index, paths, or merge state for the operator to resolve (codex C4).

Mechanism (in the coordination worktree, cwd on `dispatch/<NNN>`):
1. Resolve `trunk_tip = trunk_commit`; read the worktree's current `dispatch_tip`.
2. `mb = merge_base(dispatch_tip, trunk_tip)`. **`None` ⇒ refuse** (unrelated
   histories — codex C7), before any merge. Else if `is_ancestor(trunk_tip,
   dispatch_tip)` (trunk already contained) ⇒ report "already fresh", exit 0, no
   write.
3. `git merge --no-ff <trunk_tip>` in the coord worktree:
   - **Clean** ⇒ git commits the merge on `dispatch/<NNN>` (first parent
     `dispatch_tip`, second `trunk_tip`). Bundle now contains trunk's later changes,
     so the eventual candidate-create 3-way goes clean. After refresh,
     `merge_base(dispatch, trunk) == trunk_tip`, so a re-run `prepare-review`
     re-pins the projection to the fresh base (no gap).
   - **Conflict** ⇒ git leaves conflict markers + `MERGE_HEAD` in the coord
     worktree; the verb **reports the conflicting paths (real `git` output) and
     halts** (SPEC-021 stage-2 *report-never-auto-resolve*). The operator resolves
     in the live coord tree and commits, then re-prepares. (A future `--abort`
     convenience over `git merge --abort` is a follow-up, not v1.)

Preconditions: coordination worktree live on `dispatch/<NNN>` (refuse otherwise
with a `dispatch setup`/`resume` hint); clean coord worktree before the merge
(refuse a dirty tree — don't merge over WIP). No trunk write (advances only
`dispatch/<NNN>`).

### 3.3 Conflict classifier at candidate-create
`candidate_create`'s `MergeTree::Conflict` arm (dispatch.rs ~417) today emits a
generic content-conflict message. Target: a **diagnostic hint, not a cause
verdict** (codex C5). `candidate_create` merges an arbitrary `--base` with
`source`, so `ahead > 0` only proves *trunk moved relative to the source*, not that
this particular conflict **is** base-divergence — and `ahead == 0` does not prove
content-only. So the arm consults drift on the source (`trunk_drift(root,
source_oid)`) and, when `ahead > 0`, **appends** a hint to the existing message:
"trunk has advanced N commits past this source — the conflict may be base
divergence; try `dispatch refresh-base` then re-prepare + re-create." It never
*replaces* the content-conflict framing or asserts the cause. Diagnostic only — no
change to the abort's no-durable-state guarantee.

### 3.4 Next-step guidance
`run_status`'s `next_guidance`: surface a `RefreshBase` next-step when all phases
complete **and** the *prepared bundle* is stale past trunk. "Stale" is defined
concretely (codex C6 — not a vague flag): `trunk_drift(root, review_tip).ahead > 0`
where `review_tip = review/<NNN>` (or, pre-prepare, the dispatch tip). That is a
computed fact — trunk has commits the bundle's fork-point lacks — ordering
`RefreshBase` ahead of `PrepareReview`/candidate-create. When `review/<NNN>` is
already refreshed past trunk, the arm does not fire.

## 4. Code impact summary

| Path | Change |
|---|---|
| `src/git.rs` | `freshest_descendant` helper (left-fold); `trunk_ladder` implicit arm → preferred-order-advance-to-descendant. |
| `src/worktree.rs` | `coordinate`: plan-presence refuse-gate on `Create` **before the `worktree add -b` fork** (F6); Resume out of scope (C3). |
| `src/dispatch.rs` | extract `trunk_drift`/`Drift`; new `refresh-base` verb (real `git merge` in the live coord worktree — C4); diagnostic hint in `candidate_create` conflict arm (C5); `RefreshBase` guidance from `merge_base(review_tip,trunk)` (C6) in `run_status`. |
| CLI wiring (dispatch subcommand enum) | register `refresh-base --slice`. |
| `plugins/doctrine/skills/dispatch*/SKILL.md` | route to `refresh-base`; retire the `DOCTRINE_TRUNK_REF=main` env-prefix ritual. |

## 5. Verification alignment

- **Ladder** (`git.rs` tests): origin/HEAD-ancestor-of-main → main selected;
  diverged pair → most-preferred kept; explicit `DOCTRINE_TRUNK_REF` still wins; bad
  explicit still hard errors (existing `trunk_ladder_explicit_*` stay green).
- **Minting fallout (codex C8)**: a mixed `origin/HEAD`(behind)/`main`(ahead) repo
  mints ids off the *ahead* ref — assert `trunk_entity_ids` now sees the local-only
  ids (collision-safer), and that no existing `e2e_trunk_minting` expectation
  silently regresses (update with rationale if it does).
- **Plan gate** (`worktree.rs`): on `Create`, base lacking `slice/NNN/plan.toml` ⇒
  early `bail!` with the hint *before the fork* (no worktree created); base carrying
  it ⇒ proceeds. No env prefix in either.
- **`refresh-base` clean** (`dispatch.rs`): reproduce SL-122 — seed a drift where
  trunk advanced past the fork with a same-block rewrite; `refresh-base` `git merge`
  succeeds and advances `dispatch/<NNN>`; a following `candidate create` admits.
- **`refresh-base` conflict**: a genuinely-conflicting trunk merge ⇒ report paths +
  halt, leaving the coord worktree in a resolvable merge state (`MERGE_HEAD`
  present); `dispatch/<NNN>` ref not yet advanced.
- **`refresh-base` guards**: unrelated histories (`merge_base` None) ⇒ refuse, no
  merge (C7); already-fresh (`trunk` ancestor of dispatch) ⇒ no-op exit 0; dirty
  coord worktree ⇒ refuse before merge; no live coord worktree ⇒ refuse with hint.
- **Classifier hint (C5)**: `candidate_create` conflict with source-drift `ahead>0`
  ⇒ message *appends* the base-divergence hint; `ahead==0` ⇒ unchanged message. The
  hint never asserts cause.
- **Guidance (C6)**: `RefreshBase` fires iff `trunk_drift(review_tip).ahead>0`;
  not when the bundle is already fresh.
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
  `origin/HEAD`-first order to preferred-order-advance-to-descendant. Small
  amendment / DEC. (Bears on codex C1 — the amendment is where the neutrality
  stance is recorded normatively.)
- **SPEC-012 / SPEC-021 new REQ**: `refresh-base` as a mechanism verb (SPEC-012)
  and a between-phase cadence option (SPEC-021). Requirement authored at reconcile,
  coverage reconciled not inferred.

## 8. Non-goals (carried from scope)

Integrate-side phantom (ISS-038 / IMP-122), `[dispatch] deliver_to` config
(IMP-124 / IMP-101), `import --allow-reanchor` (IMP-043), and auto-refresh
(Q2 deferred) are out — adjacent seams / a proven-verb follow-up.

**New follow-ups surfaced by the adversarial passes:**
- RSK-010 candidate (e) — `coordinate`'s rollback must never mutate the session
  working tree for *any* cause. F6 neutralizes it for the base-staleness cause only
  (the gate fires before the fork, so rollback is never entered); general remedy is
  a separate rollback-safety fix.
- Resume-preflight (codex C3) — a symmetric plan-presence check on
  `CoordAction::Resume`'s reattached tip.
- `refresh-base --abort` convenience (over `git merge --abort`).

## 9. External adversarial pass — codex (GPT-5.5), 2026-06-20

8 findings; dispositions:

| # | Sev | Disposition |
|---|---|---|
| C1 | high | **OPEN** — ladder neutrality. Dominance-default + explicit-override + documented residual (§2.2a). Codex's "refuse ambiguous" alt rejected: it reinstates the `DOCTRINE_TRUNK_REF` ritual on every local-first dispatch (the pain we kill). Awaiting user confirm. |
| C2 | med | Accepted — fold is "preferred-order advance-to-descendant", renamed `freshest_descendant`; plan gate backstops a diverged stale pick (§2.2a). |
| C3 | high | Accepted (scope) — gate is `Create`-only; Resume out of scope, preflight → follow-up (§2.2b). |
| C4 | high | **Accepted — major revise.** `refresh-base` uses a real `git merge` in the live coord worktree, not object-db `merge_tree` (which yields no resolvable conflict state). Conflict leaves markers + `MERGE_HEAD` (§3.2). |
| C5 | high | Accepted — classifier is a **diagnostic hint appended**, never a cause verdict (§3.3). |
| C6 | med | Accepted — `RefreshBase` guidance from `trunk_drift(review_tip).ahead`, a computed fact, not a vague flag (§3.4). |
| C7 | med | Accepted — explicit `merge_base` None refusal before any merge (§3.2). |
| C8 | low | Accepted — mixed origin/main minting test; ladder change is collision-*safer* for `trunk_entity_ids` (§5). |

Sound-as-written (codex): explicit `DOCTRINE_TRUNK_REF` short-circuit, no-trunk
`Ok(None)` preservation, clean-refresh parent ordering.
