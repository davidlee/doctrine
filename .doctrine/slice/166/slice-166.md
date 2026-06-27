# Dispatch corpus-loss guards

## Context

ISS-056: a `/dispatch` drive whose coordination branch forks from a base that
**predates the authored `.doctrine` corpus** produces phase commits carrying a
corpus-less tree. Merging/promoting that bundle onto a corpus-bearing trunk
**silently deletes the entire authored corpus** — no conflict, no abort, no
diagnostic. Witnessed live on the SL-164 drive (2026-06-27): 4816 authored files
vanished from `edge`/`main`. Fails **open and destructive** — the mirror of
ISS-038 (silent *code* revert via phantom index); this is the silent *corpus*
deletion half.

The existing guards do not fire on this chain: ISS-036's setup gate asserts only
the slice's **own plan** is on the base (one notch short of the corpus); the
FF-only / leg-aware integrate guards see a clean FF because `main` had been
promoted *back* to the stale base before integrate. The deepest cut — the
manual `git merge` onto the live `edge` tree — is raw git, not a doctrine verb.

A second, load-bearing finding from the same drive: the funnel's safety model
rests on the **unenforced behavioral invariant** "primary stays on `edge` / `main`
is never checked out." Agents violate it (deepseek switched the primary worktree
`main`↔`edge` four times). That is *also* what re-opened SL-157's R1 (its
`main`-never-checked-out premise). So every guard here must be **mechanism-level —
hold regardless of which branch is checked out where.**

Scope locked via **RFC-005 §H6 OQ-7/8/9** (the funnel-integrity survey, where this
hazard is recorded as the apex correctness item). Governed by **ADR-012** (dispatch
integration topology). Source: **ISS-056**.

## Scope & Objectives

Three mechanism-level guards, layered (defense-in-depth), sharing one primitive.

- **g2 — base-corpus freshness at setup (PRIMARY).** Extend the ISS-036 setup gate
  (`coordinate.rs`): assert the fork base contains the **current authored-corpus
  head**, not merely the slice's plan. Fail-closed *before* the fork, so a
  corpus-less bundle is never authored. Kills the witnessed chain at its root.
- **g3 — corpus-shrink refusal at the ref-advance.** Refuse a funnel advance that
  drops authored `.doctrine/**` paths the slice did not itself author. Applies to
  **both the `--edge` and `--trunk` advance legs** (the corpus lives on `edge`, so
  the edge leg is where it bites). **Relative 3-way reference (Model B, locked in
  design §7 D1):** a per-leg clobber check over `base = merge-base(new, cur)` —
  flag any `.doctrine/**` path where `new == base ∧ cur ≠ base` (the advance would
  revert/delete a change `cur` holds). *Not* an absolute corpus-tip comparison: g3
  asks "does *this advance* shrink the corpus?", a different question from g2's
  "is the *fork base* fresh?". Invariant: *the authored corpus is append-mostly; no
  funnel op may advance a ref to a tree that shrinks it.*
- **g1 — refuse trunk-mutating dispatch verbs on a trunk checkout.** If a
  trunk-mutating dispatch/integrate verb's own HEAD is on the trunk branch (`main`),
  **refuse** with a loud, instructive diagnostic (restore to `edge`; promote via
  `fetch`, not `checkout`). Converts the unenforced "stay on edge" etiquette into a
  hard mechanism refusal; closes the agent-induced R1 window at the verb. Verb set
  TBD in `/design` (OQ-9 steer: all trunk-mutating verbs, not just integrate).

**Reference model (OQ-8, RESOLVED → design §7 D1 Model B).** g2 and g3 take
*different* references — they ask different questions, so they do **not** share one
primitive. **g2 (absolute):** the **corpus tip** = the highest commit on the
`authoring-branch` lineage that touches `.doctrine/**` (`rev-list -1 <ref> --
.doctrine`), resolved at runtime — not `origin/edge` (network-dependent, may lag),
not a hand-maintained marker ref (drifts). **g3 (relative):** `merge-base(new, cur)`
per advance leg. Model A (one corpus-tip for both) was rejected — it false-positives
on a buffer that legitimately lags the authoring branch and couples g3 to the
promotion ritual.

**Closure intent.** Each guard fail-closes on a constructed corpus-shrinking /
stale-base / trunk-checkout scenario, proven by integration tests that replay the
SL-164 shape (corpus-less bundle, gutted-trunk FF, HEAD-on-main). Behaviour-
preservation gate: the existing dispatch suites stay green unchanged. No corpus
deletion survives any funnel path the slice guards.

## Non-Goals

- **g4 — promotion guard on `edge:main`.** Deferred. `git fetch . edge:main` is raw
  git; guarding it needs a doctrine promotion *verb* or a preflight hook — separate
  scope. Safe to defer: g2 makes a stale `main` harmless (only dangerous if a
  dispatch forks from it, which g2 refuses). → Follow-up.
- **Raw destructive git on a live tree.** The actual SL-164 deletion was a hand-run
  `git merge` — **not a doctrine surface.** No guard here catches raw git directly;
  g2 neutralizes it *indirectly*. A pre-merge/pre-commit hook or policy is a
  separate concern. This slice **names** the boundary, it does not close it. →
  Follow-up.
- Reworking the integrate mechanism (SL-157 shipped that). This slice adds guards
  on top of the existing advance legs; it does not re-architect them.
- Changing ADR-012's FF-only / CAS contract. Guards are additive refusals; the
  non-FF auto-merge question stays in RFC-006.

## Summary

Three layered, mechanism-level guards (g2 setup base-freshness, g3 3-way
corpus-clobber refusal, g1 refuse-on-trunk-checkout) under a **split reference
model (Model B, §Scope / design §7 D1):** g2 is absolute (the corpus tip), g3 is
relative (a per-leg 3-way over `merge-base(new, cur)`) — they ask different
questions, so they do **not** share one primitive. Together they close the
doctrine-verb-mediated paths by which a stale-base dispatch bundle silently
deletes the authored corpus. Raw-git and promotion-verb gaps are explicitly
deferred.

## Follow-Ups

- g4 promotion guard / promotion-as-doctrine-verb (raw `edge:main` ritual).
- Raw-destructive-git pre-merge/pre-commit hook or policy (out of mechanism scope).
- OQ-9: confirm the exact trunk-mutating verb set g1 covers (in `/design`).
