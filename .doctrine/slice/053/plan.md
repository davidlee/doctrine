# Implementation Plan SL-053: Terminal output polish: comfy-table listings + owo_colors

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

The slice adds visual polish to every CLI listing surface without forking the
rendering path. Both new capabilities land *behind* the existing shared
chokepoint in `src/listing.rs` — comfy-table behind `render_table` (layout),
owo_colors behind `render_columns` (colour). The asset is that single seam; the
risk is the inverse of duplication — letting table or colour logic leak out to
per-kind call sites. The plan is therefore organised to keep each capability
contained to its own layer and to land the two riskiest properties — output
determinism and the exact line shape — first, before any colour exists to
confuse a regression diff.

## Sequencing & Rationale

Two code phases plus a capture phase, in strict order. They are **sequential,
not file-disjoint**: PHASE-01 and PHASE-02 both rewrite `src/listing.rs`, so
under `/dispatch` they run serial (one worker per phase), never a concurrent
batch. Each phase's EX/VT is written to be self-contained so a worker can verify
it without cross-phase context.

**Why PHASE-01 (renderer) before PHASE-02 (colour).** Colour is the cheap,
reversible layer; the load-bearing risk is comfy-table's behaviour, not the
paint. Two properties must hold before anything else is built on top:

- *Determinism.* comfy-table's default arrangement consults terminal width, and
  the `custom_styling` feature we need (for ANSI-aware width measurement)
  transitively pulls the `tty` feature, whose content formatter reads
  `stdout().is_terminal()` at format time. Either path makes piped output
  terminal-dependent and flakes the black-box goldens. Both are neutralised, but
  only by a specific pair — `ContentArrangement::Disabled` **and**
  `force_no_tty()` — and neither alone suffices. The design treats this as a
  spike: prove the pair deterministic (byte-identical terminal-vs-pipe) before
  committing to the swap. If the spike fails, the decision to adopt comfy-table
  re-opens before any colour work is wasted on top of it.

- *Line shape.* The old hand-rolled renderer had two load-bearing properties —
  no leading space and no trailing whitespace — that comfy-table does not
  reproduce for free, and that the obvious `NOTHING` preset actively reverses
  (its border components are space-filled, and default column padding adds a
  trailing space). Edge-whitespace baked into goldens is fragile (CI/editors
  that strip trailing WS would corrupt them). PHASE-01 pins the shape explicitly
  and proves it byte-exact, so the re-baseline encodes the intended shape, not an
  accident.

Landing these in PHASE-01 — as a pure, monochrome shape change in its own commit,
with the golden re-baseline isolated from any logic change (RSK-1) — means the
shared-surface diff a reviewer sees is *only* separators and shape. No colour, no
content change can hide in it.

**Why the golden re-baseline is expected, not a smell.** The
`e2e_list_conformance` net exists precisely to force acknowledgment of any change
to the shared listing surface; tripping it is the mechanism working. The
behaviour-preservation gate in the house rules targets the entity engine, not
this deliberate output change — so the re-baseline *is* the acknowledgment, taken
in an isolated commit.

**PHASE-02 (colour) rides the proven renderer.** Capability detection
(`NO_COLOR` + isatty) is impure, so it is resolved once in the command shell and
injected as a `bool` — the established date/uid injection pattern. The pure layer
never reads env or tty (no `if_supports_color`); the bool is the single
authority. Colour is applied only inside `render_columns`, so piped output stays
byte-for-byte plain and the PHASE-01 goldens stay green *unchanged* — a diff
there in this phase would itself be the bug, signalling colour leaking into
non-tty output. The status-hue map is shared and singular, fed each row's
*semantic* status by a per-column extractor rather than the emitted cell, so the
surfaces that decorate their status cell still get coloured correctly.

**PHASE-03 (capture) closes the deferral loop.** The slice deliberately ships
auto-detection only and leaves two surfaces uncoloured (the ad-hoc `writeln!`
surfaces and priority). Deferred-but-needed work must be tracked before closeout,
so the follow-ups are authored as backlog items. This phase writes doctrine
entities, not code — it is run by the orchestrator directly, not fanned out to a
dispatch worker (entity authoring is not dispatchable).

## Notes

- Downstream conduct is `/dispatch` (user direction), serial. After the plan and
  phase sheets exist, re-handover with a dispatch framing: serial run, worker
  cadence, per-phase self-contained EX/VT.
- The canonical technical detail lives in `design.md` (D1–D7 and findings
  F-1…F-8); this plan does not restate it. PHASE-01 EX/VT encode D6 (force_no_tty),
  D7 (line shape), F-2 (rectangular-grid guard), and F-8 (cargo-group dup check).
