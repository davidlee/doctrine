# REV REV-052 — Dispatch collapses onto one confined subprocess arm

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-052.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

`SL-254` collapsed dispatch from **two arms** onto **one**. The retired arm was
the in-session "claude arm": a worker spawned through Claude's `Agent` tool with
`isolation: worktree`, identified by a disk marker the orchestrator stamped into
its worktree, confined by a `PreToolUse` hook wall, and permitted to self-commit
through a gated `worker_commit` MCP tool. What ships now is a single subprocess
arm: **every** harness — claude included — is spawned by
`scripts/spawn-confined.sh <harness>` inside a kernel-level `bwrap` jail
(`sandbox-exec` on Darwin), identified by `DOCTRINE_WORKER` set by the same argv
that establishes the write floor, unable to commit, handing back an uncommitted
working tree the orchestrator imports.

That is not a refactor. It **inverts the load-bearing claim** four ADRs and four
specs were built on — that a harness-agnostic framework could not require a
harness-specific command or an environment seam, and that confinement was
therefore a codex/pi-only enhancement layered above a cooperative floor. The
shipped path requires `claude -p` (`scripts/spawn-confined.sh:220`) and enforces
confinement in the kernel on every harness, refusing by name when `bwrap` is
absent. What the incumbent governance called the ceiling is now the floor, and
what it called the agnostic floor — the disk marker — does not exist.

This revision corrects the authored corpus to describe what ships. It is a
**correction of falsified description, not a new decision**: every decision it
records was already taken and already landed under `SL-254`'s own `DEC-2xx` set.

### Scope boundary — three things this revision deliberately does NOT do

- **It does not touch `ADR-020`.** `ADR-020` (`accepted`, execution capsules as
  the dispatch authority boundary) carries a clause naming `ADR-006`, `ADR-008`,
  `ADR-011` and `ADR-012` as *"revised only at cutover through `REV-046`"*. That
  clause reads two ways — as a constraint binding other slices, or as a statement
  about `ADR-020`'s own reach. The owner ruled (2026-08-14) for the self-scoping
  reading, so this revision proceeds; `ADR-020` itself is read for consistency
  and left alone, and that assessment is carried to `/reconcile` (slice
  `notes.md`, `PHASE-08 EX-12`).
- **It does not supersede `ADR-011`.** `ADR-011` is the worst-hit entity — the
  falsification reaches its title's second half and its own `VA` verification
  criterion, which the shipped code now directly contradicts. Superseding it was
  considered and declined by owner ruling: it is **amended in place**, matching
  the treatment the other entities get and keeping the decision history in one
  document. The amendment says out loud that the ADR fails its own acceptance
  basis rather than quietly repairing the criterion.
- **It does not reconcile other slices' debt.** Pre-existing drift found while
  sweeping — `SPEC-012`'s five-vs-eight `Tier` variants, its `descends_from`
  contradiction, the `CARGO_TARGET_DIR` redirect retired by `SL-156` but still
  asserted in three entities, `REQ-252`'s per-worktree env contract dead since
  `SL-156` — is **recorded, not fixed**. Folding it in would make `SL-254` the
  reconciler of debt it did not create. Backlog candidates at reconcile.

## Method — how the target set was derived (`VH-2`, `DEC-218`, `R8`)

The design's `§3.1` table is explicitly a **floor**, not a target list: the survey
under-counted eleven times before this phase, always low. So the set was
re-derived from the corpus rather than read off the table, and the derivation is
recorded here because a revision that merely *asserts* it re-derived is not a
result.

**Sweep 1 — mechanism.** Literal tokens of the retired machinery across the
governance corpus: `pretooluse`, `nominat*`, `SubagentStart|Stop`,
`worker_commit`, `arm-spawn`, `claude-force-subprocess`, `dispatch-agent`,
`dispatch-subprocess`, `pi-spawn-confined`, `dispatch-orchestrator`,
`dispatch-probe`, `drive-slice`, `WorktreeCreate`, `marker`, `verify-worker`,
`in-session|claude arm|both arms`, `privileged.?agent`, `mode b`.

**Sweep 2 — consequence.** The vocabulary a falsified region uses when it does
**not** name the mechanism: `stamp`, `subagent`, `altitude`, `self-commit`,
`commit gate`, `isolation.?worktree`, `claim.?lock`, `strict-mcp`,
`DOCTRINE_WORKER`, `jail.?prefix`, `arming`, `write_class`, `hook.?mint`,
`degraded`, `in-session`, `nested agent`, `harness capabilit`. Sweep 2 is what
finds a region like `ADR-012` `D3`, which says *"the Claude `Agent` arm"* and
never names a deleted symbol at all.

**Every hit was then read in context**, and every entity in the set was read end
to end — a grep locates a candidate, it does not adjudicate one. Positive control:
unrelated senses of `marker` (boot-snapshot fallbacks, project-root detection,
status/drift display glyphs), `altitude` (C4 level, product-vs-technical),
`nominat` (ordinary English, and the `denominator` substring) and `arm` (match
arms) all hit and were correctly discriminated out.

**What the re-derivation found, against the floor.** Nine entities and ~191
regions, against a floor of six entities and ~46 — roughly four times the design's
count, and the eleventh consecutive under-count.

| entity | floor | re-derived |
|---|---|---|
| `ADR-011` | 11 | ~56 |
| `SPEC-012` | 10 | ~44 |
| `ADR-008` | 7 | ~35 |
| `ADR-006` | 9 | ~23 |
| `SPEC-021` | 6 | ~16 |
| `ADR-012` | 3 | ~7 |
| `ADR-001` | *not in set* | 3 |
| `PRD-015` | *not in set* | ~4 |
| `SPEC-022` | *anchor-only* | ~2 |

**And what a second, independent sweep found on top of that — under-count #12.**
The re-derivation above was cross-checked at authoring time by a second sweep run
without sight of it. It reproduced the nine-entity set and the anchor count
exactly, and found two things beyond it:

- **`SPEC-028`** — a marker-based worker-identity claim in the observation-capture
  contract that no prior survey had opened.
- **The `.doctrine/requirement/` corpus, missed by construction.** `EX-1` names
  the sweep's scope as `.doctrine/adr`, `.doctrine/spec`, `.doctrine/policy` and
  `.doctrine/standard`. Requirement **prose bodies do not live in any of those** —
  they live under `.doctrine/requirement/NNN/`. So every earlier sweep, including
  the re-derivation, was structurally incapable of seeing them. This is not a
  miscount; it is a **defect in the criterion's own scope**, and it is the reason
  the count moved again after a derivation that had already quadrupled it.

The durable lesson, recorded because it will outlive this slice: **a sweep scope
expressed as a directory list is a claim about where truth lives, and that claim
needs its own check.** The entity kinds a corpus contains are not the directories
a governance sweep names.

## The amendment instrument

Two forms, both already native to this corpus, chosen per region rather than
uniformly:

1. **Dated section-head blockquote** — `> **AMENDED — FALSIFIED (SL-254,
   2026-08-14).** …` — for a recorded **decision**, hypothesis, concern or open
   question. The historical body is **retained**; the blockquote says what is now
   false and what is true instead. This is how `ADR-011` `D5`/`D6`/`D7` and
   `ADR-006` `D2a`/`D9` already carry their `SL-056`/`SL-064`/`SL-181` amendments,
   and the visible chain of who-changed-what-when is the point.
2. **Rewrite in place** — for present-tense **description of shipped mechanism**,
   which is most of a tech spec and most of a requirement statement. Layering an
   amendment note over a false description leaves the false description as the
   thing a reader reads first. Structured registry data (a `layering.toml` row
   naming a module that no longer exists) is likewise data, not history, and is
   deleted rather than annotated.

**No entity is retitled.** `ADR-011`'s title names a "per-harness capability
altitude" that no longer exists, but no ADR in this repo has ever been retitled on
amendment — every title in the corpus was set once at creation — so the dead
title-half is stated in the amendment banner instead of being edited away. This is
a deliberate call, recorded so `/reconcile` can overturn it cheaply if the owner
prefers.

## Anchor sweep — the leg nothing mechanical performs

`doctrine spec validate` does **not** check that a `[[source]]` anchor resolves to
a file that exists. It reported `corpus clean` throughout. So every live
`identifier` row in every `spec-*.toml` was checked against disk by hand.

Exactly **two** dangling anchors existed, both in `spec-021.toml`, both resolving
by deletion into the merged skill `PHASE-07` shipped:

| | before | after |
|---|---|---|
| `spec-021.toml:28-30` | `plugins/doctrine/skills/dispatch-agent/SKILL.md` | *(row removed)* |
| `spec-021.toml:32-34` | `plugins/doctrine/skills/dispatch-subprocess/SKILL.md` | *(row removed)* |
| new | — | `plugins/doctrine/skills/dispatch-spawn/SKILL.md` |

Net: two rows out, one in. Nothing else in the corpus dangles — the
`doctrine/cli` hits are inside commented template blocks, not live rows.

Separately, two `[[source]]` **sibling comments** — not live rows, so nothing
dangled, but both misleading a reader — named the deleted `pretooluse.rs` and
`subagent.rs` while omitting the live `claim_lock.rs`: `spec-012.toml:28-30` and
`spec-022.toml:43-45`. Both corrected against `src/worktree/` as it actually
stands.

## Corrections this revision carries from earlier phases

Two findings that earlier phases escalated rather than absorbed, folded in here so
the corpus does not keep repeating a premise that was checked and found false:

1. **The import belt never enforced the configurable forbidden-writes list.**
   `DEC-204`/`DEC-213` recorded `classify_import` (`src/worktree/import.rs:120`)
   as `worker_commit`'s surviving replacement for enforcing
   `DispatchConfig::worker_forbidden_writes`. `PHASE-06` verified directly that it
   never read that config at all — it only ever enforced two hard-coded floors,
   `.doctrine/**` and `.claude/**`. `worker_commit` was the key's only production
   reader, so its configurable tail now enforces nothing. No config in this repo
   sets it, so there is no live exposure; the gap is carried to `SL-255` as
   `IDE-051` (enforce it with `bwrap` read-only binds at spawn, not a post-import
   belt — kernel, not cooperation). Wherever the corpus repeated the false
   premise, it now states the two floors plainly.
2. **`land`'s fork-role substitute is `is_dispatch_fork_branch`.** Design `§5.2.3`
   prescribed `classify_worktree_role` returning `"fork"`. That was wrong as
   written; `src/worktree/shared.rs::is_dispatch_fork_branch` was implemented
   instead. The corrected substitute is what the corpus now names.

## Before / after — by entity

<!-- Populated per entity as each amendment lands. -->
