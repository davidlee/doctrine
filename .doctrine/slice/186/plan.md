# Implementation Plan SL-186: Prompt cascade: per-context instruction resolver

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-186 delivers the **inert resolver world** — the pure cascade engine, the
impure corpus loader, the `doctrine prompt` verbs, a seed corpus, and one
minimal live consumer (the install-time def↔hymn seam, D7). Delivery to live
session-start/spawn surfaces is SL-187; the two share the locked `prompt resolve`
contract and are dispatchable in parallel.

The plan follows the **ADR-001 layering axis** (leaf ← engine ← command), which
also happens to be the safest dependency order: the pure engine can be proven in
full on hand-built fixtures before any I/O exists, the loader can be proven
before any verb is wired, and the verbs + real corpus land last on top of both.
D7 (the def seam) rides last because it is the first real *caller* of `resolve`
and needs the seed `role/worker` snippet to exist.

## Sequencing & Rationale

**PHASE-01 — Pure engine (`src/hymns.rs`).** Everything load-bearing about the
model — band order, `band→specificity→provenance→alpha`, band-primary-axis
specificity (D3), `replaces` unique-most-specific (INV-3), seal disk-twin drop
(INV-6) — is decided here, against in-memory `Snippet` fixtures. No disk, no CLI.
This front-loads the design's hardest correctness surface where tests are
cheapest (pure, table-driven) and isolates it from I/O churn. The `SealSet` is a
*passed-in argument* so the engine stays pure; the loader populates it in
PHASE-02.

**PHASE-02 — Loader + seal + projector (impure shell).** The thin impurity:
walk embedded `install/hymns/**` ⊕ disk `.doctrine/hymns/**`, derive Slot+Selector
from path, overlay sidecar `.toml` per-axis, tag provenance by root, build the
embedded `SealSet` from `install/manifest.toml`, and drop disk twins of sealed
slots. Deliberately **not** `corpus::sync_corpus` — that verb is memory-specific
(`MEMORY_SHIPPED_DIR`, memory uids; finding 7), so hymns gets its own
embedded→disk projector. `HYMNS_ROOT` is a single const — no `doctrine.toml`
override (OQ-1); root-relative bands make a later knob trivial, so YAGNI now.
This phase is testable in isolation: the loader is a function returning
`(corpus, SealSet)`, verified by loader unit tests before any verb calls it.

**PHASE-03 — Verbs + wiring + seed corpus + E2E.** The thin command shell
(`src/commands/prompt.rs`) over engine+loader: `resolve` (assembled markdown to
**stdout, read-only** — SL-186 writes no disk; the boot-unstale/disk half is
explicitly SL-187), `model-keys`, `explain`, `check`. Wire the `prompt` command
in `main.rs`/`cli.rs`. Seed `install/hymns/**` with real snippets across
universal/harness/model/role bands (including the `role/worker` snippet PHASE-04
consumes) plus the authoring convention doc. This is the heaviest phase; its
runtime `/phase-plan` will break the four verbs, the seed corpus, and the E2E
golden into ordered tasks. The design's one resolved doc tension is pinned in
EX-1: **`resolve` is stdout-only and read-only in SL-186** — the interface
block's earlier "regenerate boot.md" line was superseded by the review that split
delivery into SL-187 (F8).

**PHASE-04 — Def↔hymn install-time seam (D7).** The minimal wiring the user
elected: put a `{{ prompt resolve --role worker }}` marker in the shipped
`dispatch-worker.md` defs and expand it — **role band only** — inside
`skills::install_agents_for`, on the existing canonical-def refresh (the "always
overwrite — derived" `fs::write`). A literal marker replace, no template engine
(no other `{{ }}` use exists in defs). This proves the seam end-to-end (edit a
`role/worker` hymn → reinstall → def carries it) rather than leaving it as
untested intent, and unblocks IMP-197. Per-spawn axes (model/arm/stage) reach the
worker via SL-187's spawn envelope, which **excludes** the role band (contract
note recorded on SL-187) so the two surfaces do not duplicate worker role
guidance.

## Notes

- **Module placement follows the design.** Pure engine at top-level `src/hymns.rs`
  (matches `boot.rs`/`dispatch.rs`); command shell + loader at
  `src/commands/prompt.rs` per the design's Code Impact. The loader lives with the
  verbs but is authored and tested first (PHASE-02) before the verb dispatch
  (PHASE-03) — same file, distinct functions.
- **Behaviour-preservation gate.** Existing boot/dispatch/install/skills suites
  stay green unchanged. PHASE-04's marker-free-def-byte-identical criterion (EX-2)
  is the explicit guard on the shared `install_agents_for` write path.
- **Serial dispatch.** Each phase depends on the prior (engine → loader → verbs →
  def seam), so phases are not file-disjoint enough to parallelise; run serial.
- **OQ-2 (stage-label vocabulary source)** stays open — a `check`-validator detail
  resolved when PHASE-03's `check` verb is phase-planned (const vs skills manifest).
