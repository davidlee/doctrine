# Implementation Plan SL-116: Split worktree.rs into a submodule folder

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

A pure mechanical, behaviour-preserving cohesion split: lift `src/worktree.rs`
(3539 lines) into a `worktree/` folder of 12 files along its existing concern
seams, per the locked `design.md` (RV-131-scourged, exhaustive 42-item
item→home→visibility map). The behaviour-preservation gate — 46 tests green with
byte-unchanged bodies — is the whole correctness proof. No behaviour,
state-machine, or allowlist semantics change.

## Sequencing & Rationale

The design notes phasing is `/plan`'s call: "likely 1 mechanical phase, or split
by file-group if isolation wanted." This plan splits into **three phases**, not
because parallel isolation is wanted (conduct is self/auto, single agent), but
because the split carries two named execution hazards — **visibility drift**
(under-widen → E0603) and **de-interleaving** (land/coordinate/fork are
physically interleaved today) — that localise far better against a green
checkpoint per file-group than against one 3539-line commit. Each phase ends with
the gate green, so a break is bounded to the group just moved, and the history is
three reviewable conventional commits rather than one. AGENTS.md's "frequent
conventional commits" and doctrine's verifiable-increment posture both point the
same way.

The ordering is dictated by the `pub(super)` dependency direction, not by file
size:

- **PHASE-01 — foundation first.** `shared.rs` must exist before the machines
  that consume its four cross-machine helpers (`resolve_common_dir`,
  `resolve_commit`, `gather_tree_clean`, `gather_fork_worktree`). Extracting it,
  plus the other low-coupling concerns (`allowlist.rs` pure leaf, `marker.rs`,
  `test_helpers.rs`), establishes the widened seams up front. Everything else
  stays in `mod.rs` as a shrinking remainder that can still reach the new files.
  D4 is honoured by keeping the impure `read_allowlist`/`ALLOWLIST_FILE` behind in
  `mod.rs` this phase (they belong with provision, which lands in PHASE-02) so
  `allowlist.rs` is born pure.

- **PHASE-02 — the disk-lifecycle bulk.** `provision`/`import`/`land`/`gc`/`fork`
  + `subagent` (D7). This is where `read_allowlist`/`ALLOWLIST_FILE` move into
  `provision.rs` (D4) and where `fork`'s three cross-machine helpers widen to
  `pub(super)` — they must exist before `coordinate` (PHASE-03) consumes them, so
  fork precedes coordinate. `coordinate`'s body stays in `mod.rs` one more phase,
  reaching fork's now-`pub(super)` helpers across the boundary.

- **PHASE-03 — coordinate + closure proof.** `coordinate.rs` is extracted last:
  it is the command-tier file carrying the `slice::run_phases` upward edge
  (the slice's declared Non-Goal), and homing it last leaves `mod.rs` as a clean
  command surface — `WorktreeCommand`, `dispatch`, `root()`, and the 8-symbol
  `pub(crate) use` re-export checklist. The public-surface proof (8 caller files
  compile untouched) and the full behaviour-preservation gate seal the slice.

## Notes

- **ADR-001 layering obligation (binding, RV-131 F-3).** The split makes
  `worktree` a mixed umbrella. The `MixedUmbrella` assertion does not *force*
  sub-classification on its own here, because `worktree` sits at the top
  (`command`) tier and nothing reaches above it — but the design makes the
  sub-classification a binding obligation regardless, for truthfulness and because
  the identical omission burned SL-132 (RV-121) and SL-133 (RV-130 F-1). Entries
  are **extractor-generated** (`cargo test --test architecture_layering
  dump_real_graph -- --nocapture --ignored`), classified by actual imports, and
  added incrementally as each file-group lands. `worktree::allowlist=leaf`,
  `worktree::coordinate=command`; the rest engine iff imports stay inward.

- **Tangle-ratchet risk (not in design — surfaced in `/plan`).** Assertion 4 fails
  only when a tier's cyclic-edge count exceeds its baseline (`actual > baseline`;
  command=120, engine=0, leaf=0). Splitting one `command` unit into ~11 sub-units
  redistributes edges across tiers; the expectation is the command tangle shrinks
  or holds (engine files leave the command SCC) and engine stays 0. If the
  extractor shows engine tangle > 0 or command > 120 after a phase, that is a
  genuine partition signal — `/consult` before adjusting the baseline; a baseline
  edit is legitimate only if it reflects the true mechanical re-partition, never to
  paper over an accidental cycle.

- **No `git mv`.** Git sees a content split, not renames (design §Migration). The
  one exception is the module root: `worktree.rs` becomes `worktree/mod.rs` (same
  `mod worktree;` resolution), then concerns carve out of it.

- **Verification modes.** VT = the gate (`just gate`) and the `architecture_layering`
  / gitignore-classification tests. VA = agent diff of extracted items against the
  exhaustive §Target layout map (orphan / mis-home / over-widen). VH (PHASE-03
  only) = human sign-off that the partition matches design intent and the change is
  purely mechanical, before close.
