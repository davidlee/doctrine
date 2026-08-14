# REV REV-053 — reconcile SL-254

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

`SL-254` collapsed dispatch onto one worker-spawn shape and landed its governance
delta as `REV-052` (nine entities, ~191 regions, re-derived rather than re-cited).
Its implementation audit `RV-356` then found two governance regions `REV-052` did
not reach. Both are **descriptions that stopped being true**, not decisions that
need revisiting: no decision in either ADR changes here.

This REV is the reconcile pass's governance surface. Per-slice artefacts
(`design.md`, `notes.md`, the selector registry) and knowledge-record corrections
(`DEC-204`, `DEC-210`, `DEC-213`) are direct-edit surfaces and are **not** rows
here — they land under `SL-254`'s own commits.

## Reconcile narrative (SL-254)

### `ADR-001` — `layering.toml:138` (`RV-356` `F-6`, `modify`, primary)

`DEC-206` / `PHASE-01` moved four jail primitives into `src/worktree/jail.rs`.
Two are impure and **not** behind the module's injected seam: `have_bwrap` reads
the environment and stats the filesystem (`jail.rs:152`), and
`write_seatbelt_profile` calls `std::fs::write` (`jail.rs:507`). Both statements
of the module's contract still deny it.

*Before* — `.doctrine/adr/001/layering.toml:138`:

    "worktree::jail" = "leaf"  # SL-182: pure jail core — no disk/git/clock/rng;
                               # topology+capability+policy-read done by the
                               # shell, passed in as data (R1)

*After* — restate as the module's **genuine** invariant, which has been
*impurity behind the injected `ResolveEnv` seam* since `SL-183`, not an absence of
I/O. `impl ResolveEnv for RealEnv` in the same module already shells `getconf` and
makes several `std::fs` calls (`jail.rs:729-771`), so the "no disk" wording was
already partly false before `SL-254`; what `SL-254` added is impurity that is not
behind that seam, which is the part that needs naming.

**The twin must land in the same change or the two will disagree.** `jail.rs:5`
("PURE leaf: no clock / git / disk / rng") and `jail.rs:19` ("## Purity contract
(leaf, ADR-001 — no clock/git/disk/rng)") are a source edit, and they are the copy
a reader of the module actually meets.

Nothing mechanical catches this: `tests/architecture_layering.rs` parses
crate-module `use` edges and checks tier direction — it has no notion of
`std::fs`, so the gate is green either way. `SL-254` diagnosed it itself
(`notes.md`, "Follow-up: `jail.rs`'s ADR-001 posture") and nominated it as
`PHASE-08`'s preferred route; `PHASE-08` did open `ADR-001` and corrected the two
deleted-module rows and `marker.rs`'s comment, but this region fell between the
two passes.

### `ADR-020` — Context, `:5-10` and `:8-9` (`RV-356` `F-11`, `modify`)

`ADR-020` (accepted — adopt execution capsules as the dispatch authority
boundary) motivates capsules from **four** incumbent brittlenesses. `SL-254`
discharged two of them:

- worker identity is no longer a **cooperative marker** — it is `DOCTRINE_WORKER`,
  set by the same confining argv that establishes the kernel write floor;
- there is no longer a **harness-specific spawn path** — one confined subprocess,
  every harness.

The two that survive remain live motivation: a shared Git object store between
coordination and untrusted work, and a trusted orchestrator replaying a large
worktree/import choreography.

**The decision is unaffected.** The amendment acknowledges the two discharged
defects so the capsule case does not read stronger than it now is. `:86-88`'s
authority clause — *"ADR-011 and the incumbent worktree dispatch remain
authoritative until REV-046's cutover gates are met"* — stays **literally true**
and needs no edit: `SL-254` amended `ADR-011` in place rather than superseding it.
But a reader who takes that clause as a promise the incumbent is *unchanged* will
be wrong, and the Context amendment is where that is made visible.

`PHASE-08` escalated before authoring anything, because `ADR-020:145-146` reserves
`ADR-006`, `ADR-008`, `ADR-011` and `ADR-012` to be revised "only at cutover
through `REV-046`". The owner ruling of 2026-08-14 was route **(C)** — proceed on
the self-scoping reading, do not edit `ADR-020` in that phase — with a standing
follow-up explicitly addressed to `/reconcile`. This is that follow-up. `ADR-020`
is not itself among the four entities it reserves.

### Not a row here: `REV-046`

`RV-356` `F-11`'s second residual is that `REV-046` (`proposed · approval=none`)
enumerates precisely the mechanisms `SL-254` has now **deleted** — "worktree
marker identity, `DOCTRINE_WORKER`, SubagentStart stamping, base-by-placement, the
gated `worker_commit` exception, per-harness arm routing and altitude" — and
describes them as unimplemented *target* work awaiting a cutover. Either its
rationale needs restating against the post-`SL-254` baseline, or its gates do.

That is a live question requiring separate debate, not a description fix. Per the
split rule it is deliberately **not** a row in this REV, so a stuck row cannot
block `SL-254`'s close. Its disposition is recorded in `RV-356`'s
`## Reconciliation Outcome`.
