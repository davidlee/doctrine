# Implementation Plan SL-020: Backlog entity v1: work-intake items (one kind + item_kind facet)

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See doc/glossary.md § reference forms. -->

## Overview

Six phases turn the locked design into the v1 backlog. The spine is the
substrate-first ordering the engine forces: build the typed model + scaffold
once, then layer the four verbs (`new`, `list`, `show`, `edit`) over it, then —
strictly last — wire the skills that prompt their use. This mirrors how ADR
(SL-006) and spec (SL-015) arrived: a single new caller on the unchanged
`entity.rs`, exercised at the model level before any CLI, then verb by verb.

The design (§9) suggested this cut; the plan adopts it, with one deliberate
adjustment — `new` and the install wiring (manifest + gitignore negation) are a
single phase, because a `new` you cannot `git add` is not done. The
authored-entity wiring trap is part of "capture works", not a separate chore.

## Sequencing & Rationale

- **PHASE-01 (model + templates + scaffold) is the foundation.** Everything else
  reads or writes `BacklogItem`, so the discriminator, the closed enums, the
  three-layer parse model, the `"" → None` seam, and the three templates land
  first — exercised purely at the round-trip / scaffold level, no CLI. The
  load-bearing claim "engine unchanged" (R6) is proven here by the
  behaviour-preservation suite staying green; if PHASE-01 needed an `entity.rs`
  edit, the whole single-caller premise would be wrong, and we want that signal
  before any verb is built on it. The all-five-kinds-seed-mutable-keys check
  (VT-3) is non-negotiable here, not deferred to `edit` — a kind missing the
  seeded keys would make PHASE-05's `edit` refuse it as malformed.

- **PHASE-02 (`new` + wiring) is the first verb and the first commit-able
  artifact.** It is sequenced before the read verbs because `list`/`show` need
  real items to read, and because the wiring trap (R5: re-including
  `.doctrine/backlog/` under the ignored `.doctrine/`) is best closed the moment
  the first item exists, with a real `git add` assertion. Independent per-kind
  counters are an emergent property of the per-kind `dir`s (D4), verified here.

- **PHASE-03 (`list`) before PHASE-04 (`show`)** — survey before inspect: `list`
  exercises the cross-kind read and the missing-dir tolerance (a kind with no
  reservation dir is the empty set, not an error — the total-function read the
  inquisition demanded, C2), which `show` then relies on implicitly. The
  visibility matrix (hide-terminal default; `--all` / explicit `--status`
  reveal; promoted falls out by the terminal rule) is the substantive logic
  here; the kind-then-id order is a deterministic grouping, not a priority claim
  (R7 — priority is PRD-011, wholly deferred).

- **PHASE-04 (`show`)** is the pure local-reassembly read: prefix auto-detect
  (last-`-` split, `u32` tail, `ISS-7`/`ISS-007` tolerated — R3), outbound
  relations only (inbound is the deferred registry surface, ADR-004). Small and
  self-contained once the parse model and read path exist.

- **PHASE-05 (`edit`) is the only mutating verb and carries the subtlest
  invariant** — the `resolution ⟺ terminal` coupling (both directions), the D9
  re-open auto-clear, and the edit-preserving `toml_edit`-in-place mutation that
  the PHASE-01 seeded keys exist to permit (a tail-insert would corrupt a
  subtable, the adr F-1 failure). The OQ-003 origin-edge boundary is honoured by
  doing nothing across the tree: a backlog-side re-open touches the item only;
  the slice-authored promotion-origin edge is the deferred registry reverse
  scan's to reconcile, never this verb's.

- **PHASE-06 (skill-wiring) is last by gate, not by importance.** It edits the
  shared skill/boot surface under the behaviour-preservation gate, so it must
  follow the verbs it references (`new`/`list`/`show`) — wiring agents to consult
  a `backlog list` that does not yet exist would be a dangling instruction. It
  closes the PRD-009 §5 "intake stops leaking" measure by mapping consult /
  capture / harvest onto the loop points and shipping the work / knowledge_record
  / ADR / memory boundary text (arbitrated by the membership test) to the client
  surfaces — not the build-repo CLAUDE.md. Verified by an agent check (VA-1) on
  the edited routing, plus the behaviour gate (VT-1).

## Notes

- **No `entity.rs` change anywhere** (R6). Every phase is a backlog-local
  addition plus `meta`/`toml_edit`/engine `Fresh` reuse; the behaviour-
  preservation suite is the standing proof, re-asserted in PHASE-01 and PHASE-06.
- **Deferred layers attach without reshaping the item** (design §5.4): authored
  priority (REQ-054 / PRD-011), the `--from-backlog` promote bridge (REQ-055),
  the registry reverse scan + derived priority (REQ-056 / PRD-011), `sync`, a
  `link` verb. None is in v1; the model already carries the seams they need.
- **`Status::is_terminal` is backlog-local** (R4) — deliberately NOT
  `slice::is_terminal_status` (different vocab). Flagged so a reviewer does not
  "DRY" two correct predicates into one wrong set.
- **rust-embed re-embed footgun** (PHASE-01 templates): a template-only edit is
  invisible until the embedding crate recompiles; tests assert rendered output,
  not template bytes.
