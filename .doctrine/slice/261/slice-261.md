# Design adopt verb

## Context

A design run guards `design.md` with an **authored watermark** — the
fingerprint of the document as Doctrine last left it. A hand-edit makes every
ordinary mutating verb refuse. The only way across is re-adoption, specified by
`DEC-092` rule 2 and carried forward unchanged through `DEC-105` into the
operative `DEC-100`: *"a protocol, not a bypass"*.

Today that protocol is the `adopt_authored` payload key on `design apply`:

```json
{"adopt_authored": {
  "fingerprint": "<sha256 hex of the whole design.md>",
  "sections":    {"sec-N": "<sha256 hex of that section's body>", "…": "…"}
}}
```

It is close to unusable, and has been reported three times:

- `ISS-320` (2026-08-07) — the section map is something no verb emits.
- `ISS-348` (2026-08-12) — no verb reports the section fingerprints it demands.
- `IMP-390`'s friction report from a client repo (2026-09-24) — about eight
  attempts to discover it wants a full sha256 of each section body.

The failures, as inventoried on 2026-09-24:

| # | gap |
|---|---|
| A1 | the payload contract types both values as `text` — no digest, no algorithm, no byte boundary |
| A2 | no read surface emits the document's *current* per-section digests; only `authored_sections` (`src/commands/design.rs`) computes them, internally |
| A3 | the envelope's `fingerprint=` section rows are the run's *held* (pre-edit) digests, clipped to 12 hex — wrong on both counts, and the obvious thing to paste |
| A4 | what bytes constitute a "section body" is defined only by the `design_run::document` parser |
| A5 | `AdoptionMarkersInvalid` reports counts only — no ids, no expected values |
| A6 | the divergence refusal names the document fingerprint and never mentions the `sections` map |
| A7 | the hymn and `drafting.md` call adoption "the only lawful crossing" without saying how to perform it |

**The section map buys nothing.** Given a whole-document fingerprint that
matches, every section digest is a deterministic function of those same bytes —
the map carries zero additional information, and computing it proves no reading
occurred. Rule 2's real protections are all engine-side: the fingerprint CAS
(compare-and-swap against a stated value), complete marker validation, `DEC-066`
evidence invalidation with no inherited clearance, and re-baselining only after
the candidate validates. None needs the caller to supply digests.

## Scope & Objectives

Replace the payload protocol with a verb in which **the engine derives and the
caller confirms**:

```
doctrine design adopt SL-N [--expect <fingerprint>] [--dry-run] [--diff]
```

1. **New verb `design adopt`.** Reads `design.md`, validates markers (the same
   decomposition `authored_sections` runs today), derives the section map,
   applies `DEC-066` invalidation, re-baselines the watermark. Keeps every
   engine-side protection of rule 2 and the pre-write re-check. Supplies its
   own admission inputs; an already-aligned document is a no-op (`DEC-279`).
   Rides the one apply pipeline, split at the parse boundary with a crossing
   mode (`DEC-279`, inq-6). Refuses on a locked run, naming the regression
   (`DEC-279`, inq-7). Reads `design.md` once; fingerprint and sections come
   from the same bytes (`RV-374` `F-1`).
2. **Report what the crossing did.** Output names each changed / unchanged /
   reordered section and every act and review attestation invalidated. (No
   "added": `document::parse` refuses unknown and missing markers.) This is the
   information the current protocol never surfaces. `--diff` adds a per-section
   unified diff for changed sections via the `similar` crate (`DEC-279`).
3. **`--expect <fingerprint>`** — optional CAS against the bytes the caller
   reviewed; the divergence refusal already prints the value to paste. Absent,
   the fingerprint read at entry is the basis and the existing pre-write
   re-check covers the window, as for every other verb.
4. **`--dry-run`** — the same report with no write. Also replaces the
   *parser-readout* testing technique (`mem.pattern.design-run.adoption-is-the-parser-readout`)
   that currently hand-computes digests to probe the parser.
5. **Retire `adopt_authored` from `ApplyRequest`.** One crossing, not two
   (no parallel implementation). Retired via the first retired wire-key roster
   `(type, key, remedy)`, consulted by `refuse_unknown_keys` (`DEC-278`).
6. **Refusals name the verb.** The divergence refusal becomes *"run `doctrine
   design adopt SL-N`"*; `AdoptionStale` and `AdoptionMarkersInvalid` are
   re-expressed for the verb, and the markers refusal names the offending ids.
7. **Governance.** `DEC-279` supersedes `DEC-100`'s carried-forward rule 2
   (declaring caller → deriving engine); `DEC-278` refines `DEC-243`; a revision to `SPEC-029` (design
   run engine) wherever it specifies the payload crossing.
8. **Guidance.** Hymn (`install/hymns/stage/design.md`), `drafting.md`, the
   regenerated payload contract, and the memories that teach the payload
   (`mem.pattern.design-run.correcting-a-locked-run`, the parser-readout memory)
   updated to the verb.

## Non-Goals

- `IMP-390`'s other faces — `next_obligation` has no writer; the envelope does
  not render the next stage's unmet conditions. Separate slice.
- Adding field *semantics* (meaning / source) to the generated payload contract
  generally. The adoption fields leave the contract here; the general column is
  its own change (capture as backlog if not already).
- The envelope's abbreviated held-fingerprint rows (A3) stay as they are — they
  correctly report held state; the verb removes the reason anyone would paste
  them.
- The locked-run lifecycle for any verb but `adopt`. `adopt` refuses at
  `locked` (`DEC-279`); ordinary `apply` mutations at `locked` are a separate
  backlog item.
- A guard refusing a design-run regression once the slice is audited. No loss
  path needs it (`DEC-279`, inq-7); backlog idea.
- Strengthening `DEC-100`'s tolerated materialise lost-update window.

## Affected surface

- `src/commands/design.rs` — new subcommand; `authored_sections`,
  `refuse_authored_divergence`, `PreWriteBasis::AdmittedAt`
- `src/design_run/run.rs` — `adopt_authored` core
- `src/design_run/submission.rs` — `AdoptAuthored`, `ApplyRequest`
- `src/design_run/refusal.rs` — adoption refusals
- `src/design_run/payload_contract.rs`, `install/design-payload-contract.md` — retired wire-key roster
- `src/design_run/contract_check.rs` — roster consult, retired-key refusal
- `Cargo.toml` — `similar`
- `install/hymns/stage/design.md`, `install/design-prompts/drafting.md`
- `tests/e2e_design_*.rs`

## Risks, assumptions, open questions

- **Assumption (held, research ✓):** the caller-declared section map adds no
  protection beyond the whole-document fingerprint (`run.rs:877-895` only
  compares; `document::parse` enforces completeness).
- **Settled:** bare `adopt` adopts what is on disk at entry; the pre-write
  re-check covers the window; reviewed path is `--dry-run --diff` then
  `--expect` (`DEC-279`).
- **Settled:** the key is wire-only. Receipts store a digest and journals store
  recovery intents; no stored state carries `adopt_authored`.
- **Risk:** adoption is used in e2e tests as a parser probe; those tests move to
  the verb.

## Verification / closure intent

- e2e: hand-edit → any mutating verb refuses naming `design adopt` → `adopt`
  succeeds in one call, reporting changed sections and invalidated evidence.
- e2e: `--expect` mismatch refuses; `--dry-run` writes nothing.
- e2e: `adopt` on an aligned document writes nothing; on a locked run refuses
  naming the regression; `--diff` shows only changed sections.
- A submitted `adopt_authored` key is refused with a remedy naming the verb.
- Existing watermark / invalidation suites stay green with only the crossing's
  spelling changed.
- Closes `ISS-320`, `ISS-348`; recorded as partially fulfilling `IMP-390`.

## Summary

## Follow-Ups
