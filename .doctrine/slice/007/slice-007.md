# Memory anchoring & capture: record scope+git frame, verify, git seam

## Context

SL-005 landed memory v1's **write + read-by-id** half (`record` / `show` /
`list`), but `record` captures almost nothing the spec's retrieval depends on: it
writes *only tags* (+ workspace). `scope.paths` / `globs` / `commands`, `repo`,
and the entire `[git]` anchor render empty / `none`. The validated `Memory`
([src/memory.rs](../../../src/memory.rs)) discards `[git]` (`RawGit{}` is
fieldless) and never carries `reviewed`. And doctrine has **no git surface at all**
— `.git` appears only as a root marker (`src/root.rs`).

So before retrieval can be built, the **producer** must exist: a memory has to
*know where it lives* (scope) and *when it was last true* (a git anchor). This
slice is that producer — the write side of memory v1's retrieval story
([memory-spec](../../../doc/memory-spec.md) § Scope & anchoring, locked decision
6, interop constraint 4). It deliberately stops short of `find` / `retrieve`,
which ride this slice's output and are the sibling reader slice (SL-008).

Why this is its own slice, not folded into retrieval: it is a **distinct cohesive
capability** with a different risk profile — git/IO edge cases, subprocess
portability, the locked born-frame contract — and a clean, authored seam to the
reader (populated `memory.toml` scope + anchor). An adversarial review of the
combined design (the original SL-007) clustered its blocking findings in exactly
this half (incomplete git frame vs locked decision 6; unresolved repo identity;
`verified_sha` semantics), confirming the producer deserves a design pass *about*
anchoring. The producer is not consumer-less in the meantime: `show` already
displays the anchor (the SL-005 `anchor: none` placeholder), and `verify` writes
and reads it.

Three forces shape it:

- **The born frame is a locked contract.** Locked decision 6: a repo-scoped memory
  needs `repo + HEAD commit/tree/ref + dirty + checkout_state_id + base_commit`.
  `record` must construct it; an unborn/non-git context yields an `unanchored`
  memory, permitted only for unscoped memory (interop constraint 4).
- **`verified_sha` is the *verification* axis, not the *capture* axis.** The spec
  defines `verified_sha` as "SHA at last verification" and gates attested staleness
  on it. `record` verifies nothing, so it must **not** write `verified_sha` (doing
  so would make every memory falsely "attested" and turn the time-based staleness
  mode into dead code). A minimal `verify` verb stamps it — that is what makes the
  reader's attested mode real.
- **The git seam is impure; the schema change touches the parser.** Git lives in a
  thin `src/git.rs` shell (subprocess); the `Memory` widening is a real
  `RawGit` / `RawReview` parser change needing serde defaults for legacy-file
  compatibility, not a mere validated-layer addition.

## Scope & Objectives

- **Git IO seam (`src/git.rs`) — the producer's half.** doctrine's first git
  surface: `head_frame(repo_root) -> GitFrame` resolving the full locked-decision-6
  born frame — `repo`, HEAD `commit`, `tree`, `ref_name`, the `dirty` flag,
  `checkout_state_id` (dirty), `base_commit` — by shelling `git` (`rev-parse`,
  `symbolic-ref`, `status --porcelain`, `rev-parse HEAD^{tree}`). Plus **repo
  identity** derivation (locked here, § design): `origin` remote → normalized
  `host/owner/name`, else the first remote, else `--repo` required; non-git → empty
  `repo` (unscoped memory only). Every git failure (missing binary, non-git dir,
  unborn HEAD) maps to `anchor_kind = none`, never a panic. Impure shell only.
  (`commits_touching`, the staleness reachability query, is the *reader's* git need
  and lands in SL-008 — this slice builds only what the producer uses.)

- **`record` captures scope.** Repeatable `--path` / `--glob` / `--command` flags
  and `--repo` (defaulting to the resolved repo identity) populate `scope.paths` /
  `globs` / `commands` / `repo`. The `Draft` and `render_memory_toml` widen to
  carry them; the existing tag/title/summary capture is untouched. A bare `record`
  with no scope flags stays valid (an unscoped memory).

- **`record` builds the born frame.** At capture, `record` calls `head_frame` and
  writes the real `[git]` block: clean tree → `anchor_kind = commit` with `commit`
  / `tree` / `base_commit` / `ref_name`; dirty → `checkout_state` +
  `checkout_state_id`; unborn/non-git → `none`. `verified_sha` stays **empty**
  (record does not verify). A repo-scoped memory in an unborn/non-git context is a
  hard error (interop constraint 4).

- **`Memory` / parser widening.** Give `RawGit` real fields (the `[git]` block) and
  `RawReview` its `reviewed` / `review_by` fields, all `#[serde(default)]` so
  legacy `memory.toml` files (no `[git]` data, no `reviewed`) still parse. Validated
  `Memory` carries an `Anchor` and the `reviewed` date. Additive: every existing
  field and reader is untouched.

- **`verify` verb.** `doctrine memory verify <uid|key>` — confirm a memory still
  holds and stamp the verification axis: `verified_sha` = current HEAD,
  `reviewed` = today, `verification_state` = `verified`. Edit-preserving `toml_edit`
  mutation of the authored committed `memory.toml` (the `adr status` /
  `state::set_phase_status` pattern). This is the producer of the `verified_sha`
  the reader's attested staleness mode consumes; without it, attested mode is dead.

- **`show` displays the real anchor.** Replace `render_show`'s hardcoded
  `anchor: none` with the memory's actual anchor (kind + commit/ref + verified_sha
  presence). The first consumer of the populated anchor.

End state: a recorded memory carries its scope and a born git anchor; `verify`
advances the verification axis; `show` surfaces both. The store now holds the data
the reader (SL-008) ranks and ages — built behind the locked born-frame contract,
in a design that is *about* anchoring.

## Non-Goals

- **`find` / `retrieve` and all ranking.** Scope matching, the deterministic 9-key
  sort, staleness computation, and the two query verbs are the **reader slice
  (SL-008)** — they ride this slice's populated scope + anchor. Out of scope here.

- **`commits_touching` / reachability.** The staleness commit-count query is the
  reader's only git need; it lands in SL-008's extension of `src/git.rs`. Building
  it here would be a producer-side function with no producer-side caller.

- **Heavier lifecycle / re-stamp verbs.** `reanchor` (rebind to a new commit),
  `supersede`, `retract`, `promote` advance lifecycle and belong with the reserved
  ledger seam (every mutation is also an event, interop constraint 1). v1 ships only
  the minimal `verify` (the one verb the reader's attested mode requires).

- **Lifecycle ledger / events.** `events.toml`, NDJSON interchange, the event-store
  backend adapter — all deferred (spec § reserved seam). `verify` edits current
  state only; the paired event is the ledger seam's job when it lands.

- **Git library dependency.** doctrine shells `git` (subprocess); it does not
  vendor `git2`/`gix` for three plumbing reads.

- **Engine change.** This slice adds `src/git.rs`, widens the `src/memory.rs`
  parser + `record` + `show`, adds the `verify` verb, and adds CLI arms; it does
  **not** touch `src/entity.rs`. The entity / slice / state suites stay green
  unchanged. `record`'s rendered output *does* change (it gains a real anchor +
  scope) — an intentional change to an SL-005 verb whose own tests update; the
  engine contract does not.

## Summary

The producer half of memory v1's retrieval story: make a recorded memory know
where it lives and when it was true. `record` gains scope-capture flags
(`--path`/`--glob`/`--command`/`--repo`) and builds the locked-decision-6 git born
frame via doctrine's first git seam (`src/git.rs` `head_frame` + repo-identity
derivation, subprocess, every failure → `anchor_kind = none`). The `src/memory.rs`
parser widens (`RawGit` gains the `[git]` block, `RawReview` gains `reviewed`, both
`serde(default)` for legacy compat) and validated `Memory` carries an `Anchor` +
`reviewed`. A minimal `verify` verb stamps the *verification* axis
(`verified_sha`/`reviewed`/`verification_state`) edit-preservingly — distinct from
the capture axis, and the producer of the `verified_sha` SL-008's attested
staleness consumes. `show` surfaces the real anchor.

The full `GitFrame` shape, the repo-identity derivation rule, the
`verified_sha`-vs-`verification_state` axis split, the parser legacy-compat
contract, and the `record` error semantics for unanchorable repo-scoped memory
live in the design doc ([design.md](design.md)) — revised from the adversarial
review of the original combined design, pending re-review before `slice plan`.

## Follow-Ups

- **SL-008 — memory retrieval (the reader).** `match_scope`, the 9-key
  deterministic `Ord`, `staleness` (the three modes; adds `commits_touching` to
  `src/git.rs`), `find` (ranked rows), `retrieve` (the security agent-context
  block). Rides this slice's scope + anchor.
- **F1 — heavier re-stamp / lifecycle verbs.** `reanchor`, `supersede`, `retract`,
  `promote` — the mutation half that turns on the reserved `events.toml` ledger
  seam (each mutation is also an event).
- **CLAUDE.md.** Add `doctrine memory verify` (and, when SL-008 lands,
  `find`/`retrieve`) to the CLI surface; note the `src/git.rs` seam in the layout.
