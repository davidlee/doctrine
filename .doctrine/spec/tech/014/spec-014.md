# SPEC-014: Slice surface

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

The slice surface is the slice kind: the primary unit of intentional change in
Doctrine — a declarative change bundle answering *what changes, why, what it
touches, what risks, and what done looks like* before code moves, realising
**PRD-001**. It is a component of the entity engine (SPEC-004): a slice is an
`entity::Kind` over the kind-blind engine, and all shared mechanism — identity,
the atomic claim, id allocation, the scaffold/render pipeline, and the
edit-preserving status transition — lives in the parent container and is used
here unchanged. This spec carries only what is specific to the slice kind: its
reserved fileset plus the non-reserved design/plan/notes siblings, the nine-state
status vocabulary, the lifecycle FSM and its closure seam, the orthogonal conduct
axis, and the read-time join of the gitignored phase rollup. The phase-rollup's
runtime tracking is itself the entity engine's runtime-state boundary; this
component owns only the join and the divergence verdict over it.

## Responsibilities

Mirrors the structured `responsibilities` list: the slice `Kind` and its sibling
scaffolds; the status vocabulary; the lifecycle FSM and closure seam; the three
divergent status predicates; the conduct axis; and the phase-rollup join with its
markers.

### Slice kind and sibling scaffolds

The slice is a top-level reserved kind whose scaffold materialises three artefacts
under the slice directory — the `slice-<id>.toml` sister TOML, the `slice-<id>.md`
prose body, and an `<id>-<slug>` symlink alias. Three further siblings ride the
same materialiser as **non-reserved** kinds added into an existing slice
directory: a single-file `design.md`, the implementation plan (`plan.toml` +
`plan.md`), and an on-demand `notes.md`. A design or plan sibling has no id or slug
of its own — its template inherits the parent slice's canonical id and title by
token substitution. The fileset shapes and the materialisation path are the
parent container's; only the descriptors and templates are slice-specific.

### Status vocabulary

Slice status is one of nine states — **proposed · design · plan · ready · started ·
audit · reconcile · done · abandoned**. This `&[&str]` const is the single
authority: it is both the known-set that `slice list --status` validates the
filter against and the vocabulary the `slice status` write verb checks its target
against. The vocabulary is purely additive over the original six, so existing
slices need no migration. Write-time enforcement of the *stored* status is
deliberately deferred — a hand-edited `slice-<id>.toml` may carry an out-of-vocab
status, which is tolerated on disk, never hidden, and surfaced with a trailing
drift marker rather than rejected.

### Lifecycle FSM and the closure seam

The `slice status <id> <state>` verb classifies a from→to move against the
lifecycle FSM (axis A) before any disk write. `classify` is a pure, total
edge-table function — explicitly not index arithmetic, since `abandoned` is last
in the const but is not "after `done`" in the chain — producing one of seven
`Transition` verdicts. Precedence is fixed: no-op → from-terminal → closure-seam
(by target) → abandon → forward/back edges → skip. The **closure seam** is gated
by the target edge: `→ reconcile` is legitimate only from `audit`, `→ done` only
from `reconcile`; any other source is a structural `SeamBreach`, refused — so a
blessed skip-to-`done` cannot author itself unflagged. Leaving a terminal source
is `FromTerminal`, also refused (reopening is deferred). Advances and the named
back-edges (`audit`/`reconcile` → `design`, etc.) are written; an unclassified
move is a `Skip` — written and surfaced, not blocked. The transition itself is the
parent's edit-preserving seam (status + `updated`, clock injected by the shell);
this component fixes only the FSM and the seam.

### The three divergent status predicates

Three slice-status predicates over the same vocabulary are kept deliberately
distinct, each feeding one surface: `is_terminal_status` is `{done}` only and
feeds the divergence verdict; `is_hidden` is `{done, abandoned}` and feeds the
`slice list` default hide-set; `is_transition_terminal` is `{done, abandoned}` and
feeds the write verb's reopen refusal. The sets diverge by design — adding
`abandoned` to the terminal set would false-flag `⚠` on an abandoned-incomplete
slice, which is *consistent* (work was dropped), not divergent.

### The conduct axis

Conduct (axis B) is orthogonal to the lifecycle FSM: *what* a change is (the
state) is separate from *how it is conducted* (`actor × autonomy`), declared
per state. The pure `conduct::resolve` folds a `ConductConfig` parsed from the
project-root `doctrine.toml [conduct]` table — read by the impure shell, absent
file yielding the baked defaults — and a queried state into the effective posture.
The posture is **advisory, never enforced**: `slice status` resolves the *source*
state's exit posture (`resolve(from)`) and renders it as one line; an unknown or
drifted state resolves to defaults rather than erroring.

### Phase-rollup join and markers

`slice list` joins the gitignored runtime `PhaseRollup` (the entity engine's
runtime-state tier, derived from the phase tracking sheets) onto each row *after*
the shared filter. From it the slice derives the `phases` cell —
`completed/total`, with appended `!N` blocked and `?N` anomaly markers, or `—`
when untracked — and decorates the status cell with two independent markers: a
trailing `?` when the stored status is out of vocabulary (drift), and a `⚠` when
the authored status and the rollup *disagree* (divergence). Divergence is
conservative: suppressed under anomalous or untracked tracking, and keyed on
`is_terminal_status` — terminal-with-work-outstanding, or fully-complete-but-not-
terminal. The list is read-only over the rollup: it *reveals* divergence but never
reconciles it.

## Concerns

- **Three-predicate drift.** The terminal, hide, and reopen-refusal sets are three
  near-identical predicates that must not collapse — a careless merge re-introduces
  the abandoned-incomplete false-flag they were split to avoid.
- **Vocabulary lockstep.** The `&[&str]` const, the `SliceStatus` clap `ValueEnum`,
  and the FSM edge table must mirror; a drift canary test pins the const and the
  enum together.
- **Read-only divergence.** The rollup join reveals lifecycle divergence but owns
  no transition that reconciles it; closing the gap is a separate, deferred verb.
- **Conduct is config, not control.** The posture is declared intent surfaced for a
  human; nothing in the verb gates on it, so it must never read as enforcement.

## Hypotheses

- **The FSM and conduct are orthogonal axes, not one enum.** Modelling *what a
  change is* (lifecycle state) separately from *how it is conducted*
  (`actor × autonomy`) is preferred over folding conduct into status, so gating can
  attach additively without reshaping the state set.
- **Divergence is surfaced, not auto-reconciled.** Deriving the rollup verdict and
  flagging `⚠` read-only — rather than mutating the authored status to match — is
  preferred, because the authored status is committed truth and the reconciliation
  is a human-gated lifecycle act.
- **A drifted stored status is tolerated, not rejected.** Deferring write-time
  vocabulary enforcement and instead surfacing a `?` marker keeps a hand-edited
  slice readable rather than unparseable, the read surface guarding its own
  coherence.

## Decisions

- **D1 — the slice rides the engine, not its own materialiser.** Identity, the
  atomic claim, the scaffold/render pipeline, and the edit-preserving transition
  are the parent container's; this component restates none of them and owns only
  the slice kind's surface.
- **D2 — the status vocabulary is a single const authority.** One `&[&str]` serves
  the list filter known-set and the write-verb target check; the clap `ValueEnum`
  mirrors it and a canary pins the two in lockstep.
- **D3 — the closure seam is gated by the target edge, structurally.**
  `→ reconcile` only from `audit` and `→ done` only from `reconcile`; every other
  source to those targets is a `SeamBreach`, so the ADR-003 closure spine cannot be
  skipped unflagged.
- **D4 — three terminal-ish predicates stay distinct.** `{done}` for divergence,
  `{done, abandoned}` for the hide-set, `{done, abandoned}` for reopen refusal;
  unifying them would false-flag an abandoned-incomplete slice as divergent.
- **D5 — conduct is advisory and resolved on the source state.** The verb renders
  `resolve(from)` as exit posture, never enforces it, and falls back to baked
  defaults on an absent config or unknown state.
