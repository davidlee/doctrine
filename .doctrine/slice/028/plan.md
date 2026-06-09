# Implementation Plan SL-028: Enact ADR-003 reconcile seam and lifecycle states

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

This plan executes the **lifecycle-FSM vertical** the design cut to (D1). The
**canon half** — author + accept ADR-009 and apply the staged ADR-003 amendment
(vehicle C, D6) — was completed during `/design`, because F7 makes
canon-acceptance the gate the code phases sit behind ("build ahead of accepted
canon" is the failure it guards). So the plan that remains is **code-only**:
three phases mapping the three orthogonal structures of design §5.1, in dependency
order.

- **PHASE-01 — the spine.** The slice FSM vocabulary + transition verb. This is
  the headline: the inert lifecycle (`status` hand-edited, no verb) becomes
  advanceable, closing the CLAUDE.md "no slice lifecycle transition" gap.
- **PHASE-02 — conduct.** The orthogonal `actor × autonomy` axis (advisory),
  folded into the verb's output once the verb exists.
- **PHASE-03 — the enums.** The two-enum requirement/coverage truth model as
  vocabulary stubs — the lowest-coupling leaf, deferred engine and all.

## Sequencing & Rationale

**Why PHASE-01 first, and why it carries the spec + boot edits.** The FSM
vocabulary is foundational — both other phases assume it, and the verb is the
slice's reason to exist. The vocabulary change cannot land alone: the spec
canary `slice_statuses_matches_the_spec_vocabulary` pins `SLICE_STATUSES`
against `slices-spec.md` § Lifecycle, so the spec edit and the canary update are
**one atomic move** (F1) — splitting them would land a red canary. The boot
Core-process prose edit naming `… audit → reconcile → close` rides here too: the
`reconcile` *state* becomes real in this phase, so the prose naming it is factual
exactly when the state ships. The routing-table *skill* row is deliberately left
alone — pointing it at the deferred `/reconcile` skill would be
shipped-not-reachable (F2/F14); that row moves with the reconcile-skill slice.

**The verb is classify-don't-jail, except the closure seam.** PHASE-01's two
hard structural refusals — the closure seam (F12) and from-terminal (F13) — are
the load-bearing correctness work. The seam protects the FSM *topology* ADR-003
§7/§8 depend on (a blessed `started → done` writer would author an unreconciled
"closed" slice no read-side detector surfaces); from-terminal needs a *third*
predicate distinct from the two existing ones, because reusing `is_terminal_status`
would false-flag `⚠` on abandoned slices. Everything else writes and surfaces its
nature, matching the house read-surface posture. The verb reuses the
edit-preserving status-transition seam (`adr::set_adr_status` is the precedent)
rather than parallel-implementing a writer.

**Why PHASE-02 follows, not leads.** Conduct surfacing has no home until the
`slice status` verb exists — it enriches that verb's output (the source state's
exit posture, F19) rather than standing up its own surface. Conduct is otherwise
orthogonal to the FSM (design §4), so it cleanly slots second. It is **advisory
in v1**: parsed and surfaced, never enforced — and invoker-blind, so `actor` is
declared config, never attributed (F15). The vocabulary and the two baked gate
defaults (`plan`, `reconcile`) are fixed now so enforcement is later additive,
not a restructuring.

**Why PHASE-03 last.** The enums are the lowest-coupling work — vocabulary with
no producer (the derivation engine, registry, and coverage blocks are all
deferred non-goals). `CoverageStatus` lands behind a self-clearing
`expect(dead_code)` ahead of its consumers. The one invariant to hold is
*absence*: no `ReqStatus = f(coverage)` derivation — that absence **is** the
doctrine differentiator (explicit reconcile vs derive-by-precedence). Doing this
phase last keeps the risky spine and its review surface uncluttered by leaf
vocabulary.

## Notes

- **Behaviour-preservation gate.** The slice vocabulary change is purely additive,
  so the existing `slice list` / rollup / divergence / `is_drifted` / `is_hidden`
  behaviour suites are the proof and must stay green **unchanged**. The single
  intended diff is the spec-lockstep canary, which updates with the slices-spec
  edit by design (cf. `adr_known_set_matches_variants`) — that is the canary
  working, not a regression.
- **Pure/imperative split.** `classify` and `resolve` are pure; the date is
  shell-injected (`clock::today()`); disk + TOML parse live in the thin shell.
- **Lint posture.** `BTreeMap` not `HashMap`; suppress with `expect(reason=…)`
  never bare `allow`; `just check` (plain `cargo clippy`, bins/lib) zero warnings
  before every commit.
- **Open questions** stay in `design.md` §6 (OQ-1 `--force`, OQ-2 `retired`-setter,
  OQ-3 staleness detection, OQ-4 per-run conduct override, OQ-5 spec-edit scope) —
  all deferred to follow-on slices, none lock-blocking for this plan.
