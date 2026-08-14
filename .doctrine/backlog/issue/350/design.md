# ISS-350 — informal design: one registry root per tree that owns one

Status: **approved** 2026-08-14 (owner), as the *tactical* fix. Scope: small
backlog item, no slice. The strategic question it does not settle — which tree
owns the registry at all — is tracked as `QUE-216` against `RFC-025` (see §8).
Companion to `backlog-350.md` (the defect report); that file states the problem,
this one states the fix and why it is the fix.

## 1. The one-line problem

`boundaries_path()` (`src/state.rs:943`) resolves the source-delta registry
against `git::primary_worktree(cwd)` for both read and write, while
`phases_dir()` (`:135`) resolves phase sheets against the local project root.
A tree that carries its own runtime state for a slice therefore reads its phases
locally and its registry from somewhere else, and `slice conformance` reports
`unavailable` while a complete registry sits in the tree the command was run
from.

## 2. Evidence that reshapes the issue's proposal

`backlog-350.md` proposes: *"read locally when the linked worktree holds its own
`.doctrine/state/slice/<NNN>/` for a slice the primary does not know."*

**That predicate does not fix the reported case.** Checked against the live
trees:

```
.doctrine/state/slice/254/            → design.toml, design-journal.toml, phases/, boundaries.toml
.worktrees/SL-254-audit/.../254/      → phases/, boundaries.toml
```

`SL-254` was *designed* in the primary and *built* in a capsule, so the primary
holds `design.toml`, `design-journal.toml` and a full set of phase sheets for
254. "The primary does not know this slice" is false, and the fallback fires
straight back to the broken behaviour.

The directory is the wrong witness because it is shared by four unrelated
artefacts (design snapshot, design journal, phase sheets, registry), each written
by a different subsystem at a different lifecycle stage. Only one of them is the
thing being resolved.

**The registry file is the right witness.** Its presence in a tree is a positive,
single-writer fact: `write_registry` is the only thing that creates it, reached
only through `record_source_delta` / `forget_source_delta`. A tree holds a
`boundaries.toml` iff a boundary was recorded into that tree or delivered with
it. Nothing else can put one there.

## 3. The rule

> **The registry resolves to the local tree when the local tree already holds
> one; otherwise to the primary worktree.**

One `exists()` check, applied identically to read, write, and evict — so there is
no read/write asymmetry to keep in sync, and no new concept ("adopted checkout",
"foreign provenance") enters the model.

## 4. Cases

| tree | local `boundaries.toml`? | resolves | why it is right |
|---|---|---|---|
| primary | n/a (local *is* primary) | primary | unchanged |
| dispatch worker fork | no — `.doctrine/state/**` is a **withheld** provision tier (`allowlist.rs:73`) | primary | the fork-family contract: N forks must not fragment the orchestrator's view |
| dispatch coordination tree | no — same withheld tier; its writes have always gone to the primary | primary | `prepare-review`'s completeness gate reads the primary (`dispatch.rs:3600`) |
| solo `/worktree` fork | no | primary | unchanged |
| capsule (a fresh clone in a microVM) | trivially — it is its own primary | local | `primary_worktree(cwd) == cwd` in a single-worktree clone; the pin is already a no-op there |
| capsule-delivered / hand-copied checkout | **yes** — the state tier arrived with the tree | **local** | the registry that describes this tree's history is the one it reads |

Verified against the live worktrees, which are a complete positive/negative
control set:

- `.worktrees/SL-231-p01`, `.worktrees/SL-186-p04` (worker forks) — no
  `.doctrine/state/` at all.
- `.dispatch/SL-209` (coordination tree) — has `state/slice/209/`, **no**
  `boundaries.toml`.
- `.worktrees/SL-248`, `.worktrees/SL-254-audit` — carry their own
  `boundaries.toml`, delivered with the tree.

### Why the fork family can never drift into local resolution

Induction on the only writer: a tree resolves local **only if** it already holds
a registry, and a fork/coord tree starts with none (withheld from provision), so
the base case never fires and no write can create the first local file. The
fork-family behaviour is preserved by construction, not by a guard that could be
forgotten.

## 5. Disclosure — the part that actually prevents the next misdiagnosis

The whole three-finding misdiagnosis in `RV-356` (`F-7`, `F-14`, superseded by
`F-15`) came from a *silent* root choice. Two advisories, both to stderr, both
following the established `warn_capture` / `advise_boundary_span` idiom:

1. **Cross-tree read/write.** When the resolved root is not the local root, say
   so: `note: source-delta registry resolved to <primary> (this tree is a linked
   worktree)`. Fires for forks and coord trees; silent in the primary and silent
   in a tree that owns its registry — i.e. zero noise in the common cases, and it
   fires exactly in the case that misled the audit.
2. **Sibling registry on write.** When recording, if another live worktree also
   holds a registry for this slice, name it: `warning: <path> also holds a
   source-delta registry for SL-NNN`. This is the shadow-write hazard, and it is
   the half the resolver does **not** fix — an operator running `record-delta`
   from the primary for a slice built elsewhere still writes locally, because in
   the primary the local root *is* the primary. `git::list_worktrees()`
   (`git.rs:1798`) already enumerates; no new git surface.

Advisory only. Neither refuses; a status transition and a manual correction must
both remain possible (the existing D5 degrade rule).

## 6. Changes

All in `src/state.rs` unless noted.

1. New resolver, the single decision point:

   ```rust
   enum RegistryRoot {
     /// The local tree owns the registry (the primary, or a tree that carries one).
     Local(PathBuf),
     /// Resolved across trees to the primary — this tree holds none of its own.
     Primary(PathBuf),
   }
   fn resolve_registry_root(local: &Path, slice_id: u32) -> anyhow::Result<RegistryRoot>
   ```

   Canonicalise `local` before comparing with `primary_worktree()`'s canonical
   answer — the same `fs::canonicalize(...).unwrap_or_else(...)` idiom
   `mirror_completion_into_primary` uses (`:583`).

2. `boundaries_path()` becomes a thin call through the resolver. Its `cwd`
   parameter is documented as "any path in the repo" but every caller passes a
   project root; tighten the name to `local` and the doc to match, since the
   local-registry probe genuinely needs a root.

3. Both advisories, emitted from the two command shells (`run_record_delta`,
   `conformance_outcome`) rather than from the resolver, so the pure-ish resolver
   stays quiet and testable.

4. **Simplification, in scope:** collapse
   `registry_completeness(cwd, project_root, slice_id)` to a single root. No
   caller has ever passed two different values — `slice.rs:2907` passes
   `(root, root)`, `dispatch.rs:3600` passes `(&primary, &primary)`, every test
   passes `(&repo, &repo)`. The two-parameter split advertises a divergence that
   only ever happened *inside* `boundaries_path`, and its doc comment ("they
   coincide in the primary worktree") is exactly the confusion this issue is
   about.

Untouched on purpose: `phases_dir()` stays local-rooted, and the ISS-212
primary-completion mirror stays as it is. Making phase sheets share the resolver
would change dispatch's completion semantics — a much larger blast radius for no
gain here.

## 7. Tests

New, in `state.rs`'s existing `init_repo` + `git worktree add` fixture style
(the shape is already there in `record_from_linked_worktree_targets_primary_tree`,
`:2656`):

- **VT-A** adopted tree, local registry present → `read_source_deltas` returns
  the local rows and a record from that tree lands locally; the primary's file is
  not created.
- **VT-B** adopted tree, local *phase sheets and design snapshot* present but no
  local registry → resolves primary. This is the case the issue's original
  predicate got wrong; it is the regression test for §2.
- **VT-C** `registry_completeness` from a tree with local sheets + local registry
  reports `Complete` — the `unavailable` regression from `RV-356` `F-15`, at the
  level that actually failed.
- **VT-D** the existing `record_from_linked_worktree_targets_primary_tree` stays
  green, unedited. It is the fork-family contract and the behaviour-preservation
  proof.

## 8. Residuals — explicitly not fixed here

- **Runtime state still does not travel with a branch.** A tree that reconciles a
  slice built elsewhere still needs the state copied across. Unchanged; it is the
  argument for promoting the registry to the authored tier (`backlog-350.md` fix
  direction 2), which stays out of scope — putting derived evidence in the
  committed tier deserves its own storage-rule argument, not a side effect of a
  bug fix.

  **Tracked as `QUE-216`** — *which tree owns a slice's source-delta registry* —
  shaping `RFC-025` (the capsule programme) and this issue. Two facts push it
  past a residual mention once capsule-build-then-adopt is the common case
  rather than the exotic one. First, this fix is a strict **no-op in the
  primary**: local *is* primary there, so conformance run from the control plane
  for a capsule-built slice whose state sits in a sibling worktree still reports
  `unavailable`. Second, the fix trades *one wrongly-rooted source of truth* for
  *two correctly-rooted ones that can drift* — pre-fix every tree read one file
  and so could not disagree. Deliberately not decided here: the surviving tree
  taxonomy is mid-change (§10), and the two candidate answers cost
  asymmetrically — adoption-into-primary as a ritual is reversible, promoting the
  tier is a migration.
- **A pre-existing shadow keeps winning in its own tree.** Both trees now hold a
  registry for 254 (byte-identical, so harmless here); each reads its own. The
  sibling advisory surfaces the divergence rather than resolving it, which is
  right — doctrine cannot know which history is authoritative.
- **`record-delta` from the primary for foreign work** still writes locally. The
  sibling advisory names it; nothing refuses.

## 9. Alternatives considered

- **Issue's directory-existence predicate** — rejected in §2: false on the
  reported case.
- **Registry follows the phase sheets** (one root for the whole
  `state/slice/<NNN>/` dir) — this is the coherence invariant the issue reaches
  for, and it is right in principle, but the coordination tree has local phase
  sheets *and* must share the primary's registry, so it inverts dispatch. Rescuing
  it needs a live-`dispatch/<NNN>`-coord probe as an exception — a second
  discriminator to keep in sync with the two that already key on it. More moving
  parts for the same outcome.
- **Drop the primary pin for reads only** (local-if-present read, always-primary
  write) — the same behaviour as the chosen rule in every case that matters, but
  it splits one resolution into two rules. The single rule subsumes it.
- **Delete the pin outright now** (always local), on the grounds that §10 kills it
  anyway — rejected on timing, not principle. `dispatch.rs:3572` resolves the
  primary *itself* and passes it explicitly, so prepare-review's read survives;
  what breaks is the **write** — the coordination tree's `conclude_phase` would
  land its row locally and the primary's completeness gate would then see a gap.
  It becomes correct the moment dispatch is deleted, and is one line then (§10).

## 10. Direction of travel — why the fallback has a death date

The claude dispatch arm goes when `SL-254` lands, and dispatch itself is likely
next. The parallelisation primitive becomes the **capsule**: a fresh clone in a
microVM whose gitignored state returns over a git sideband and is adopted into a
worktree here (oubliette `NOTES item 32`). The surviving tree taxonomy is the
primary (control plane), a worktree for sequential execution, and an audit/review
worktree — the last two carrying their own slice-scoped runtime state.

In that taxonomy the resolver's `Primary` arm is **never taken**: a capsule is
its own primary; an audit/review worktree and a state-carrying execution worktree
both resolve `Local`; the primary is `Local` trivially. The arm fires only for a
linked worktree with no state of its own — the dispatch fork and coordination
tree, deleted with dispatch. So the rule degrades to "always local" by
subtraction, and the invariant `backlog-350.md` reaches for (one root per
`state/slice/<NNN>/`) arrives without a second migration.

Ordered path, each step independently safe:

1. this fix;
2. the claude arm goes with `SL-254`;
3. dispatch goes → delete the `Primary` arm, and the resolver with it;
4. the state-provisioning policy (`.doctrine/state/**` as a **withheld**
   provision tier, `allowlist.rs:73`) is decided on its own merits. This fix is
   deliberately independent of it — which is the reason it keys on the registry
   file rather than on a provisioning rule.

### External dependency: adoption must be slice-scoped

Local-first resolution is only as narrow as what an adoption puts in the tree.
The sideband collects a **host-declared path list**, and that list is repo-wide
today — `.doctrine/state/slice`, `.doctrine/dispatch`, every unit the checkout
ever held. A broad adoption would give an audit worktree a point-in-time snapshot
of registries for slices it has nothing to do with, and this rule would then
serve them in preference to the primary's current ones. Silently.

Empirically the hazard is latent, not present: the `SL-254` adoption was narrowed
by hand at extraction and landed `state/slice/254` and nothing else. The
invariant that must replace those fingers is now recorded in `NOTES item 32` —
*a collect brings back the out-of-band state of the work the capsule was
assigned, and none that is not* — to be enforced by parameterising the declared
allowlist with an opaque unit token supplied at assignment, so the project never
names its own perimeter (oubliette `NOTES item 25`).

With that invariant held, this fix composes exactly right: an audit worktree
resolves locally for the slice it was adopted for and falls back to the primary
for every other slice. The §5 advisories are doctrine's backstop; the invariant
is the actual fix, and it lives in the other repo.

### The mirror of that invariant on this side

`.doctrine/state/boot.md` has been dropped from the sideband's declared paths on
the rule that **state a consumer regenerates per checkout does not travel** — a
foreign copy landing in a live tree is stale authority the next tool reads as its
own. Its positive half is that regeneration belongs to *provisioning*.

Doctrine has the same gap in its own provisioner, and it is wider than the boot
snapshot: `worktree fork` copies, withholds, and stamps, but regenerates nothing,
so a fresh worktree has no skills and no shipped memory corpus either — while
`DERIVED_RUNTIME` (`allowlist.rs:99`) already asserts they are "regenerated by
`doctrine install` in the fork". The snapshot alone is partly covered by accident
of harness: the claude `SessionStart` hook runs `prompt resolve`, which
regenerates it in whatever tree the session opens in — late, and only for that
harness. Filed as `ISS-352`; not this one.
