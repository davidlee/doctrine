# QUE-216: Which tree owns a slice's source-delta registry

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## How it arose

`ISS-350` reports a resolver defect: `boundaries_path()` (`src/state.rs:943`)
resolves the source-delta registry against `git::primary_worktree(cwd)` for both
read and write, while `phases_dir()` (`:135`) resolves phase sheets against the
local project root. A tree carrying its own runtime state therefore reports its
phases complete and its registry absent at the same time, and
`slice conformance` degrades to `unavailable`. That is a bug with a tactical fix
(read locally when the local tree already holds a `boundaries.toml`, else
primary) and it is being taken.

The question here is what the fix *cannot* settle, and it only became visible
once the capsule workflow was named as the common case rather than the exotic
one: **design happens in the primary, implementation happens in a capsule, and
the runtime state comes back over a sideband to be adopted into some tree here.**

## Why the tactical fix does not answer it

Traced against `SL-254`, the case that surfaced this:

| step | tree | holds a registry | resolves | after the fix |
|---|---|---|---|---|
| design, plan | primary | no | primary (= self) | unchanged |
| build | capsule | yes — solo rows land as phases complete | local; a capsule is its own primary | unchanged |
| adopt state | linked worktree | yes — arrived with the tree | **local** | **fixed** |
| conformance from the primary | primary | only if copied back | primary (= self) | **still `unavailable`** |

In the primary the local root *is* the primary, so both arms of the resolver
give the same answer and the fix is a strict no-op there. If the capsule's state
sits in a sibling worktree, the primary — the control plane, and the tree that
holds the design — still sees nothing. Nothing looks sideways at other
worktrees, by design.

## What the fix trades away

Before the fix every tree read one file: the root was wrong, but it was
singular, so no two trees could disagree. After it, the capsule workflow
routinely produces two copies — the adopted one and any copy-back — and each
tree reads its own.

Live evidence at time of capture (2026-08-14): both
`.doctrine/state/slice/254/boundaries.toml` and
`.worktrees/SL-254-audit/.doctrine/state/slice/254/boundaries.toml` exist, each
with ten `provenance = "solo"` rows. They agree today because `RV-356`'s audit
restored the authentic registry into the primary by hand after the shadow-write
incident. Nothing keeps them agreeing. `ISS-350`'s sibling-registry advisory
surfaces the divergence; it does not resolve it, and it cannot — doctrine has no
way to know which history is authoritative.

So the fix converts *one wrongly-rooted source of truth* into *two
correctly-rooted ones that can drift*. That is a net improvement at the tree
where audits actually run, and it is the reason this question needs an owner
rather than a leftover-residual mention.

## The two candidate answers

1. **Adoption into the primary is the ritual.** One root, no divergence, and the
   primary answers conformance for every slice. Local-first resolution then
   demotes to a safety net for the window before copy-back. Cheap, but it keeps
   runtime state hand-carried, and the hand-carrying is exactly what produced the
   `RV-356` misdiagnosis.
2. **Promote the registry to the authored tier** (`ISS-350` fix direction 2).
   Rows land with the branch, adoption stops being a separate act, and the
   which-copy-wins question dissolves. Larger, and it needs the storage-rule
   argument on its own merits — this puts derived evidence in the committed tier,
   which the `ISS-350` design deliberately declined to argue as a side effect of
   a bug fix.

Neither is a free choice, and the cost of picking wrong is not symmetric: (1) is
reversible, (2) is a tier migration.

## Scope of the question

Named on the registry because that is where the evidence is, but the general
form is wider: **which tree owns a slice's gitignored runtime state once a
capsule produced it elsewhere, and does any of it need to travel with the
branch?** Two adjacent facts bound it:

- Adoption must be slice-scoped, or local-first resolution serves a
  point-in-time snapshot of registries for slices the tree has nothing to do
  with. Empirically latent, not present — the `SL-254` adoption was narrowed by
  hand at extraction. The invariant that must replace those fingers lives in the
  spike repo's notes, not here.
- Provisioning regenerates nothing (`ISS-351`), so a fresh worktree has no
  skills, no shipped memory corpus, and no boot snapshot either — while
  `DERIVED_RUNTIME` (`allowlist.rs:99`) already asserts `doctrine install`
  regenerates them in the fork. Whatever answers this question should not
  contradict that one.

## Why it is too soon to decide

The tree taxonomy is mid-change. The claude dispatch arm goes with `SL-254` and
dispatch itself is likely next; when it does, `ISS-350`'s `Primary` arm is
never taken and the resolver degrades to "always local" by subtraction. Deciding
registry ownership before the surviving taxonomy is fixed would settle it
against trees that are about to stop existing.
