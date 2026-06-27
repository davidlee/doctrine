# Design SL-166: Dispatch corpus-loss guards

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

A `/dispatch` drive can **silently delete or revert the authored `.doctrine`
corpus** with no conflict, abort, or diagnostic (ISS-056; witnessed on the SL-164
drive 2026-06-27 — 4816 authored files vanished). The mirror of ISS-038 (silent
*code* revert): same outcome shape, opposite half. Three mechanism-level guards
close the doctrine-verb-mediated paths, each holding **regardless of which branch
is checked out where** — the funnel's safety can no longer rest on the unenforced
"primary stays on the authoring branch / the integration ref is never checked
out" etiquette, which agents demonstrably violate (deepseek switched the primary
worktree across the integration ref four times on the SL-164 drive).

Scope is locked upstream in **RFC-005 §H6 OQ-7/8/9**; this design realizes it.
Governed by **ADR-012** (dispatch integration topology). Source: **ISS-056**.

## 2. Current State

- **Fork base / setup.** `worktree::coordinate::coordinate` (Create leg,
  `coordinate.rs:150`) forks the coordination worktree off `git::trunk_commit`
  (the ladder / `DOCTRINE_TRUNK_REF` resolver) and gates only
  `base_has_slice_plan` (`coordinate.rs:105,161`, ISS-036) — asserts the base
  carries the slice's *own plan*, nothing about the broader corpus.
- **Advance legs.** `integrate` (`dispatch.rs:1696`) → `advance_row` (`:1786`)
  branches on `worktree_for_ref`: `advance_pure_ref` (`:1821`, the not-checked-out
  integration ref — CAS-and-done post-SL-157) and `advance_checked_out` (`:1856`,
  the checked-out authoring ref — `merge --ff-only`, the safe atomic leg). Both are
  **FF-only and blind to `.doctrine/**` content regression**.
- **Projection.** Phase trees filter `.doctrine` entirely
  (`filter_tree([".doctrine"])`, `dispatch.rs:~2141`); the orchestrator authors
  `.doctrine` deltas separately (ADR-012 bucket routing). So in a healthy advance
  the projected tree's `.doctrine/**` equals the fork base's — corpus survival
  depends purely on the base being current and the advance not regressing it.
- **No checkout-posture guard.** No verb refuses running while HEAD is on the
  integration ref.
- **Config surface.** `dispatch_config.rs` `[dispatch]` table: `deliver_to`
  (default `refs/heads/main`, the integration/delivery ref dispatch advances to),
  `preferred_subprocess_harness`, `claude_force_subprocess_dispatch`. There is
  **no** authoring-branch concept — the edge/main split is ~1 week old, project-
  specific, passed ad-hoc via `--edge` at integrate.

## 3. Forces & Constraints

- **ADR-012 D4 — FF-only / CAS contract.** Guards are *additive refusals*; they do
  not relax FF-only, force-push, or auto-resolve. The non-FF auto-merge question
  stays in RFC-006.
- **POL-002 — platform independence from host conventions.** The edge/main split
  is *this repo's* convention, not a platform universal. g1/g2 must not bake in a
  branch name or assume the split; they activate only when the project *declares*
  the posture via config. A single-branch project (primary checked out on the
  integration ref, advancing it via the safe ff leg) must keep working unchanged.
- **STD-001 — no magic strings.** The new config key, refusal tokens, and the
  `.doctrine` pathspec are named constants.
- **Behaviour-preservation gate.** The existing dispatch suites (`e2e_dispatch_sync`,
  `e2e_dispatch_close`, the `git` unit suites) are the proof — they stay green
  unchanged except where a guard is the direct subject.
- **Pure/imperative split.** Guard *predicates* are pure over injected git
  readings; git I/O stays in the thin shell.
- **The raw-git boundary.** The actual SL-164 deletion was a *manual* `git merge` /
  `git fetch . edge:main`, not a doctrine verb. No guard here catches raw git
  directly; this slice **names** that boundary (§8), it does not close it.

## 4. Guiding Principles

1. **Mechanism, not etiquette.** Every guard holds regardless of which branch is
   checked out, who ran what raw git, or whether the promotion ritual was honored.
2. **Fail closed, escape explicit.** Refuse on doubt; the only way past is an
   operator naming the exact casualties, on the record — never a soft prompt, never
   an auto-derived allowlist.
3. **Natural reference per surface.** g2 (is the *fork base* fresh?) and g3 (does
   *this advance* shrink the corpus?) are different questions; each gets its own
   reference (Model B), not one forced primitive.
4. **Posture opt-in.** The buffered-trunk split is declared, never assumed; the
   universal guard (g3) needs no declaration.

## 5. Proposed Design

### 5.1 System Model

Three guards over two surfaces, gated by one new config field.

```
[dispatch] authoring-branch = "<ref>"   (optional; presence ⇒ buffered-trunk posture)

setup   ── coordinate() ─────────▶ g2  base-corpus freshness   (iff authoring_branch set)
verb    ── integrate / candidate ─▶ g1  refuse-on-trunk-checkout
                                       (iff authoring_branch set ∧ ≠ deliver_to)
advance ── advance_row(): per leg ─▶ g3  3-way .doctrine/** clobber gate  (ALWAYS)
                                        └─ escape: --allow-corpus-clobber <path>…
```

`authoring_branch` is the project's source-of-truth authoring ref (this repo:
`refs/heads/edge`); its presence declares that `deliver_to` is a non-checked-out
integration buffer and the corpus lives on `authoring_branch` ahead of it. Unset
(default, single-branch projects) → g1 + g2 are no-ops; g3 still protects.

### 5.2 Interfaces & Contracts

**Config (`dispatch_config.rs`).**

```rust
/// The authoring branch — the source-of-truth ref where `.doctrine` content is
/// authored, ahead of `deliver_to`. Its presence declares the buffered-trunk
/// posture: `deliver_to` is a non-checked-out integration buffer, promoted from
/// this ref. Unset ⇒ single-branch posture; g1/g2 inert. NOT the fork-base
/// resolver (ADR-006 D3 ladder / DOCTRINE_TRUNK_REF).
#[serde(default)]
pub(crate) authoring_branch: Option<String>,   // TOML: authoring-branch
```

**g1 — `guard_not_on_integration_ref(root, cfg) -> Result<()>`** (new, dispatch
shell). Called at the head of the ref-advancing verbs: `sync --integrate` and the
trunk-advancing `candidate` paths (`create --role close_target`, `integrate`; see
OQ-3 on `admit`). Inert unless `authoring_branch` is set and differs from
`deliver_to`.

```
let Some(_) = cfg.authoring_branch else { return Ok(()) };          // posture off
let buffer = short_name(cfg.deliver_to);                            // e.g. "main"
if current_branch(root)? == Some(buffer) {
    bail!(REFUSE_ON_TRUNK, …)   // names the buffer ref + recovery
}
```

Refusal (constant `REFUSE_ON_TRUNK`): *"refused: HEAD is on the integration buffer
`<deliver_to>` — the primary must stay on `<authoring_branch>`. Restore (`git
checkout <authoring_branch>`) and promote via `git fetch . <authoring_branch>:<buffer>`,
never `checkout <buffer>`."*

**g2 — extend `coordinate()` Create leg.** After `base_has_slice_plan`, before the
`worktree add`:

```
if let Some(corpus_ref) = cfg.authoring_branch {
    if let Some(corpus_tip) = git::last_corpus_commit(root, &corpus_ref)? {   // rev-list -1 <ref> -- .doctrine
        anyhow::ensure!(
            git::is_ancestor(root, &corpus_tip, &base)?,                      // base carries the corpus
            BASE_CORPUS_STALE, …                                             // names corpus_tip, base, fix
        );
    }
    // corpus_ref unresolvable or no .doctrine corpus yet ⇒ degrade to no-op
}
```

`last_corpus_commit` (new `git.rs` seam): `rev-list -1 <ref> -- .doctrine`,
`Ok(None)` when the ref is unresolvable or touches no corpus — distinct from a
usage error (mirror `is_ancestor`'s explicit exit-code handling; do **not** route
through `git_opt`, which conflates exit 1 with 128 —
[[mem.pattern.dispatch.project-off-pinned-fork-base-not-live-trunk-tip]]).

**g3 — `corpus_clobber_check(root, base, new, cur, allow) -> Result<()>`** (new,
pure-ish over injected tree readings). Called in `advance_row` per leg, *before*
the CAS/ff mutation, for both the `--trunk` and `--edge` targets.

```
let changed = git::diff_doctrine_paths(root, base, cur)?;   // .doctrine/** where cur ≠ base
let clobbers = changed
    .filter(|p| blob_at(new, p) == blob_at(base, p))   // advance didn't touch p ⇒ would revert cur's change
    .filter(|p| !allow.contains(p));                    // minus operator allowlist
if !clobbers.is_empty() { bail!(CORPUS_CLOBBER, render(clobbers)) }     // each path + verdict (deleted|reverted)
```

`base = merge-base(new, cur)`. Escape: a repeated `--allow-corpus-clobber <path>`
CLI arg (sole-writer orchestrator), threaded into `allow` and recorded on the
integrate journal row. Fail-closed when absent.

### 5.3 Data, State & Ownership

- **Authored.** New `[dispatch] authoring-branch` in the project's `doctrine.toml`
  (set to `refs/heads/edge` here, in a separate commit — it *enables* the posture).
- **No new runtime/derived state.** Guards are read-only over git objects/refs plus
  the CLI allow-list arg; the allowlist is recorded on the existing journal row,
  not a new store.
- **Ownership.** g1/g2 read config + refs; g3 reads three trees. All git reads via
  existing/new `git.rs` seams (sole impurity site). Predicates are pure.

### 5.4 Lifecycle, Operations & Dynamics

- **g1** fires at verb entry — earliest, cheapest, before any ref work.
- **g2** fires at `coordinate()` Create — before the fork, so a stale base never
  spawns a corpus-less bundle (fail-closed, no rollback path entered, like ISS-036).
- **g3** fires per advance leg, before the mutation. A true `merge --ff-only`
  advance can never clobber (new descends cur ⇒ new already carries cur's
  `.doctrine` changes), so g3 is **inert on FF advances** and only bites on a
  non-FF / tree-replacing advance — exactly where silent loss lives (a hand merge,
  or RFC-006's future non-FF integrate). Defense-in-depth today, load-bearing then.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1 (append-mostly corpus).** No funnel advance may delete or revert a
  `.doctrine/**` path the target ref holds, unless the advance authored that change
  (`new ≠ base`) or the operator allowlisted it. g3 is the enforcement.
- **INV-2 (posture parity).** With `authoring_branch` unset, behaviour is byte-
  identical to today (g1/g2 short-circuit to `Ok`). Single-branch dispatch
  unaffected.
- **Edge: renames.** `A→B` ⇒ A deleted (but A existed in history; B is `new≠base`)
  → not a clobber. Correct.
- **Edge: concurrent authored add on the ref under FF.** new descends cur ⇒ new
  carries it ⇒ no clobber. Under a non-FF replace, dropping it *is* flagged →
  operator allowlists or re-merges. Correct.
- **Edge: g2 vs legitimately-lagging buffer.** Per the pre-dispatch ritual the base
  == promoted authoring branch, so `is-ancestor(corpus_tip, base)` holds; it trips
  precisely when promotion landed the buffer behind the corpus (ISS-056).
- **Assumption.** g3's `cur` is the *live* target ref tree at advance time; its base
  is `merge-base(new, cur)` — not the pinned fork base (those differ when the ref
  moved). Verified against the leg's actual CAS expected-old.

## 6. Open Questions & Unknowns

- **OQ-1 (deferred, not blocking).** g4 promotion guard — make
  `<authoring>:<buffer>` promotion a doctrine verb / preflight. Raw git today; out
  of scope (§8).
- **OQ-2 (deferred).** Raw-destructive-git pre-merge/pre-commit hook class — the
  only thing on the manual-merge step; separate concern (§8).
- **OQ-3 (design-time, resolve in /plan).** Does the trunk-advancing `candidate`
  set for g1 include `admit` (which advances no ref, only records an OID)? Lean:
  guard only the ref-mutating members; confirm against `dispatch.rs` candidate arms
  during phase-planning.

## 7. Decisions, Rationale & Alternatives

- **D1 — Model B (split reference).** g2 absolute (corpus floor), g3 relative
  (3-way over the advancing ref's own base). *Alt:* Model A (one corpus-tip for
  both) — rejected: it false-positives on a buffer that legitimately lags the
  authoring branch, and couples g3 to the promotion ritual.
- **D2 — g3 = 3-way clobber, not deletion-only.** `new==base ∧ cur≠base` covers
  deletion and stale-revert uniformly; "authored" = `new≠base`. *Alt:* deletion-
  only provenance (`rev-list` existence) — rejected: misses stale edits to
  surviving files; the 3-way model subsumes it and is simpler.
- **D3 — escape = explicit per-path allowlist.** *Alts rejected:* (a) ledger auto-
  reports the actual delta → records the catastrophe's deletions as "intended",
  self-defeating; (b) soft "orchestrator confirms" prompt → too weak for the blast
  radius; (c) heuristic threshold → a hack, only if forced.
- **D4 — posture gated on `authoring-branch` config (POL-002).** *Alt:* hardcode
  edge/main or always-on g1/g2 — rejected: assumes this repo's week-old convention
  and breaks single-branch dispatch.
- **D5 — g1 keys off `deliver_to`, not a literal.** The integration ref is already
  configurable; reuse it. g1's job is "don't sit on the buffer."
- **D6 — additive refusals only (ADR-012 D4).** No FF/CAS contract change → no
  ADR-012 Revision. (Confirm in adversarial pass.)

## 8. Risks & Mitigations

- **R1 — raw-git gap.** The manual `git merge`/`fetch` that did the SL-164 damage
  is not a doctrine surface; g1–g3 don't catch it directly. *Mitigation:* g2
  neutralizes it indirectly (a stale base can't enter the funnel); the residual is
  named, deferred to OQ-2 (a hook/policy), and called out — not pretended closed.
- **R2 — g3 cost on the catastrophe path.** A 4816-path clobber set means 4816 blob
  comparisons. *Mitigation:* batch via one `diff --name-only`; short-circuit/cap the
  rendered list with a logged "+N more". The expensive case is the one we refuse
  anyway; the normal path (empty delta) is ~free.
- **R3 — g2 false-positive under unusual promotion topologies.** A project whose
  authoring branch and buffer share no `.doctrine` history. *Mitigation:* degrade to
  no-op on `Ok(None)`; document the posture contract; g3 still backstops.
- **R4 — posture misconfiguration.** `authoring-branch` set wrong (e.g. == the
  buffer). *Mitigation:* g1's `≠ deliver_to` guard; a `config validate` check that
  the two differ when the posture is on.

## 9. Quality Engineering & Validation

TDD red/green/refactor; behaviour-preservation gate on the existing suites.

- **g1.** `integrate_refused_when_head_on_buffer` (posture on, HEAD==deliver_to →
  refuse); `integrate_allowed_on_authoring_branch` (HEAD==authoring → ok, the safe
  leg); `g1_inert_when_posture_unset` (single-branch parity).
- **g2.** `setup_refused_when_base_predates_corpus` (the SL-164 shape: base lacks
  corpus the authoring ref has → refuse, no worktree created);
  `setup_ok_when_base_carries_corpus`; `g2_noop_when_authoring_unset`;
  `g2_noop_when_no_corpus_yet`.
- **g3.** `advance_refused_on_phantom_corpus_deletion` (new=absent, base=absent,
  cur=present → clobber); `advance_refused_on_stale_revert` (new=old, base=old,
  cur=edit → clobber); `ff_advance_never_clobbers` (new descends cur → ok);
  `authored_doctrine_delta_allowed` (new≠base → ok); `allowlist_lets_named_path_through`;
  `unnamed_path_still_refused_with_partial_allowlist`.
- **Parity.** Re-run `e2e_dispatch_sync` / `e2e_dispatch_close` unchanged with
  `authoring-branch` unset → green (INV-2).

## 10. Review Notes

### Internal adversarial pass (2026-06-27)

- **A — g3's present value is contingent (RESOLVED: ship).** Both advance legs are
  FF-only today (`advance_pure_ref` CAS-checks `is_ancestor`; `advance_checked_out`
  is `merge --ff-only`), and a true FF advance cannot clobber (new descends cur).
  So g3 catches nothing the existing FF guards don't already block — its present
  value is forward-insurance for RFC-006's non-FF integrate plus making INV-1
  explicit + tested. **Decision: ship g3 in this slice** (User, 2026-06-27) — the
  ~1-phase cost is a decent payoff to have the corpus safety net *land before* the
  non-FF capability it protects, rather than as an easily-forgotten prerequisite.
- **B — g2-strict enforces promote-before-setup (RESOLVED: accept).**
  `is-ancestor(corpus_tip, base)` refuses setup whenever the authoring branch holds
  an un-promoted `.doctrine` commit, i.e. it mandates the `fetch <authoring>:<buffer>`
  promotion before every `dispatch setup`. **Accepted** (User) as deliberate ritual
  enforcement — skipping that promotion *is* the ISS-056 precondition. Document the
  constraint in the refusal text; the degrade-to-no-op valve (unset posture / no
  corpus) bounds it.
- **g1 reads the invoking worktree's HEAD.** `current_branch(root)` must resolve the
  *cwd worktree's* HEAD (as the existing raw-evidence-ref guard at `dispatch.rs:1067`
  already does), not the common dir — verify in /plan.
- **D6 — ADR-012 Revision check.** g3 *adds* a refusal to integrate; it does not
  change D4's FF-only/CAS contract (still FF-only, still 3-arg CAS, no force, no
  auto-resolve). Lean: additive gate, no Revision. **Flag for the external pass to
  confirm** — if a hostile reviewer reads g3 as altering the integration contract,
  route a mechanism-only ADR-012 Revision at reconcile.
- **ADR-001 layering — g2 threading `DispatchConfig` into `coordinate()`.**
  `worktree::coordinate` must reach `dtoml`/`dispatch_config` without forming a
  cycle (engine→config). Confirm the edge direction in /plan; if it would close a
  cycle, pass the resolved `authoring_branch`/`deliver_to` *values* in rather than
  the loader.
- **`--allow-corpus-clobber` is global across both integrate legs.** One allowlist
  applies to the `--trunk` and `--edge` advances in a single integrate call. Accepted
  as a minor imprecision (a path is rarely a legit clobber on one ref but not the
  other); note in the verb help.

(External `/inquisition` to follow before lock.)
