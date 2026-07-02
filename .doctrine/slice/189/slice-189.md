# Pi-arm boundary recording scopes to imported code commit, not funnel span

## Context

Origin: **IMP-231** (surfaced by the SL-186 audit, RV-213 F-3, tolerated).

On the pi/codex (subprocess) dispatch arm, the funnel's **Record** beat writes the
per-phase conformance boundary with `slice record-delta <SL> PHASE-NN --start <B>
--end <B+1>`. `B → B+1` is a *commit span*, so it captures every commit the
orchestrator lands between the two boundaries — the trailed knowledge commits
(memories, backlog, authored `slice-NNN.toml` / `adr/**`) written *after* the code
commit, plus any interleaved `refresh-base` trunk merge — not just the imported
source diff. SL-186 showed 17 `undeclared` conformance paths against a projected
candidate that was a clean 19-file delta.

The claude arm does not have this problem: it code-scopes its boundary at the
imported code commit. Two arms, divergent boundary fidelity.

Structurally identical span-vs-single-commit pollution exists in the **solo**
auto-binding path (`capture_phase_boundary`, `src/state.rs`), tracked separately as
**IMP-175** — *out of scope here* (different code path, different trigger). This
slice is the dispatch/subprocess-arm cut only. The two share a root shape and
should be designed with an eye to a common primitive, but IMP-175's solo-execute
trigger (edge advancing between in_progress and completed) is not this slice's.

## Scope & Objectives

**Objective:** the pi/subprocess arm records a per-phase source-delta scoped to the
single non-merge imported code commit's own tree-diff — never a funnel span that
brackets trailed knowledge or refresh-base merges. Conformance on a pi-driven slice
matches the projected candidate; no hand-diffing against trunk to separate noise
from real scope creep.

In scope:
- The pi/codex arm's boundary-recording invocation at the funnel Record beat
  (`dispatch-subprocess` SKILL.md — the prose that picks `B`/`B+1`).
- The recording mechanism itself if the design lands a code-side fix: whether
  `record-delta` (or a new/adjacent primitive) should record the import commit's
  own single-commit tree-diff rather than a two-dot span the operator must scope
  correctly by hand. Guard already enforces non-merge `end`, but a merge *inside*
  `start..end` still pollutes the two-dot diff — so an operator passing a tight
  range is not sufficient protection; the design should weigh making single-commit
  scope structural.
- Design decision (deferred to `/design`, flagged here): **unify the pi arm onto
  the claude arm's code-scoped `record-boundary`** vs **keep the pi arm on
  `record-delta` with a tighter/structural single-commit scope**. The former kills
  the divergence at the root; note `record-boundary` writes the coord-tree
  `boundaries.toml` while `record-delta` writes the PRIMARY arm-neutral registry,
  and the codex/pi symmetric ledger *derive* is deferred (D6 / IMP-171) — so a
  naive "just call record-boundary" may not land in the arm-neutral registry the
  conformance gate reads. This coupling is the crux the design must resolve.

Out of scope:
- IMP-175 (solo auto-binding `capture_phase_boundary` pollution).
- The deferred codex/pi symmetric ledger derive (D6 / IMP-171) as a *feature* —
  referenced only insofar as it constrains the unify-vs-tighten choice.
- Any change to the claude arm's boundary recording.

## Affected surface (coarse — `/design` refines)

- `plugins/doctrine/skills/dispatch-subprocess/SKILL.md` — where the pi arm picks
  the `--start`/`--end` range (start here per handoff).
- `.doctrine/skills/dispatch-subprocess/SKILL.md` — the second home of this skill
  (mem: "dispatch-subprocess skill lives in two places"); keep in sync.
- `src/dispatch.rs` — funnel Record beat, boundary-recording surface, and the
  auto phase-binding referenced by record-delta's help.
- `src/slice.rs` — `run_record_delta` (2282–2322); stores oids only.
- `src/dispatch.rs` — `run_record_boundary` (712–744, dual-write ledger + registry),
  and the **conformance read seam**: the record verbs store start/end oids only —
  the over-attribution lives in the *reader*. Conformance reads a two-dot
  `git diff start..end` span, whereas the `phase/<slice>-NN` path (2501/2511)
  does a *chained single-commit tree cut* filtering `.doctrine/`. The clean-vs-noisy
  divergence is the read, not the record — design must decide where the single-commit
  scope is enforced (tighter stored oids on the pi arm vs routing its rows through
  the chained-cut read).
- `src/ledger.rs` (`record_boundary`, 554–566), `src/state.rs` (`record_source_delta`
  UPSERT, 668–711) — registry write path.
- `src/state.rs` — read-only reference (`capture_phase_boundary`, 495–568); NOT edited here.

## Risks / Assumptions / Open Questions

- **R1** — Behaviour-preservation gate: `record-delta` doubles as the manual escape
  hatch (correct-a-range / bootstrap) and its `Manual` provenance must not regress
  (D12: never clears an existing funnel/legacy halt). Any code change must keep the
  manual verb's contract green.
- **R2** — Registry-target divergence: `record-boundary` (coord `boundaries.toml`)
  vs `record-delta` (PRIMARY arm-neutral registry). Unifying arms must not orphan
  the conformance gate's read path (`dispatch sync --prepare-review` completeness
  gate `bail!`s on a missing row — ISS-052 failure mode).
- **A1** — The funnel produces exactly **one** non-merge import commit per *batch*
  (`S^==B`, Delta-check); single-commit scope is well-defined.
- **A2** — **Phase atomicity**: a phase's code lands in a single (batch) commit.
  Holds for serial (one worker/phase) and parallel batches; a phase split across
  multiple commits (mid-phase re-dispatch) is NOT single-commit-capturable and uses
  the retained `--start/--end` escape hatch. Named limitation, not silent (design
  §5.5 A2 / §8 R4).
- **OQ-1** — Prose-only fix (pass `feat^..feat` in the skill) vs structural
  code-side single-commit scope. Prose is cheapest but leaves the two-dot footgun
  for the manual/escape-hatch caller; code makes it safe by construction.
- **OQ-2** — Does resolving IMP-231 subsume or merely parallel IMP-175 — is a
  shared single-commit primitive worth extracting now, or does that overreach this
  slice?

## Verification / Closure intent

- Conformance on a pi/subprocess-arm-driven slice reports zero *funnel-noise*
  undeclared paths — the recorded range equals the imported code commit's own
  tree-diff (trailed knowledge + refresh-base merges excluded).
- `record-delta`'s manual escape-hatch contract stays green (guards, upsert,
  Manual-provenance non-clobber).
- Behaviour-preservation: existing dispatch / conformance / ledger suites stay
  green.

## Follow-Ups

- **IMP-175** (solo `capture_phase_boundary`) — adopt `single_commit_boundary` at
  the Completed flip (record `[feat^, feat]`); this slice builds the primitive.
- **Claude funnel** — adopt the primitive on `dispatch record-boundary` (kills the
  arm divergence at the root).
- **IDE-026** — auto-wire the commit boundary via harness hooks (SubagentStart /
  bracketing; bwrap/seatbelt wrap), so recording is a property of the spawn, not an
  orchestrator beat. Depends on this slice's primitive.
- D6 / IMP-171 (codex/pi symmetric ledger derive) — the deferred derive that would
  give the pi arm the clean `phase/<N>` projection the claude arm has.
