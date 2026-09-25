# Shipped-corpus conformance

## Context

POL-002 says the product must not load-bear on host-project conventions or state.
Doctrine's largest client-facing surface is its shipped corpus, in three
sub-corpora that are all materialised into arbitrary client repos and read there
by agents with no doctrine corpus of their own: `install/` (published reference
docs, hymns, design-prompts, templates, projected integration assets), the shipped
memory corpus `memory/`, and the skills under `plugins/`.

An audit swept that corpus on 2026-09-26 (evidence and site-by-site ledger in
ISS-309, CHR-080, IMP-484) and found it fails that reader in three separable ways.

1. **It cites this repo.** Per-repo entity ids and repo-private paths are cited as
   if resolvable. They do not *dangle* — a client repo mints its own `ADR-007`, so
   the citation silently resolves to *the client's unrelated record*, with no
   error and no signal. 239 sites across the three sub-corpora, plus private paths
   (`src/relation.rs`, `install/<file>.md`) and private function names.
2. **Some claims about the CLI are stale.** The shipped knowledge signpost
   describes six kinds where the CLI mints seven, names the wrong default statuses
   (`held`, not `pending`; `proposed`, not `pending`), omits `settle` — the
   resolving verb — from its verb list, and its own syntax example (`EVD-1`) breaks
   the zero-pad rule and would be refused. The id-vocabulary table in
   `glossary.md` has no row for the `concept`/`CPT` kind at all.
3. **Whole CLI surfaces have no shipped orientation.** The corpus instructs actions
   it does not document — the sharpest being friction observation, which the
   shipped boot snapshot tells *every* agent to record while no shipped doc
   explains the ledger. Also unoriented: the corpus-health surface (`doctor`,
   `validate`, `check`, `publication`), the reports group (`next` et al.), `config`,
   and the facets group.

**Why one change and not three.** The acceptance test is a single reader. Each axis
is a separate way that reader is failed, and all three rest on one design question —
*what may a shipped claim be grounded on, and at which tier* — which a single answer
settles. Splitting by axis would force that decision three times.

**Scope extension recorded.** `plugins/**` (the skills) was outside the original
audit's two sub-corpora and is included here on evidence: 74 further sites across 14
skill files, all genuine rather than illustrations, and in places *worse* than a
collision — `spec-product/SKILL.md` instructs the reader to mirror `PRD-001` as "the
canonical shape", which in a client repo is the client's own PRD-001. It is also the
most context-resident shipped surface. A POL-002 rule already scopes to `plugins/**`
(`mem.pattern.doctrine.shipped-skill-platform-independence`), so this sub-corpus is a
known rule being violated — evidence for the drift-gate follow-up, not a reason to
exclude it. **Confirmed in-scope by the user on 2026-09-26.**

## Scope & Objectives

**Affected surface:** `install/**`, `memory/**`, `plugins/**`. No `src/**`.

1. **Citation conformance.** Resolve every repo-private id and path citation in the
   three sub-corpora. Per site, decide: inline the fact, drop it, or replace it with
   something a client can reach — a published `reference/<name>.md` address, or
   prose that stands without an id. Ledger in ISS-309.

2. **Accuracy.** Verify every shipped claim about CLI behaviour against the CLI and
   correct the mismatches. Confirmed instances in CHR-080; the pass must cover the
   whole corpus, not only the known-bad items.

3. **Sufficiency.** Disposition every gap in IMP-484 — each either gains shipped
   orientation, or is explicitly recorded as correctly out of scope with its reason.

4. **Governance.** Record the permissible-grounding rule settled in design (`DEC-311`)
   as a new ADR descending from ADR-005 / ADR-019, and deliver the rule in the shipped
   corpus so future authoring is bound by it. The ADR and the one new published
   reference doc the hard cases need are deliverables of this slice.

**Verification / closure intent.** Done is judged by a client-repo read test, not by
a diff: read each published reference doc, shipped memory, and skill as an agent with
no doctrine corpus, and confirm every citation either resolves to a published address
or stands without an id. Corpus gates must stay green (`doctrine doctor`,
`doctrine check`).

Two disciplines apply, both already learned in this repo:

- **The replacement resolving is the evidence — not the id being gone.** A
  grep-confirmed absence proves the deletion ran and nothing more
  (`mem.pattern.doctrine.reseat-renumbers-does-not-retarget`). "The old id no longer
  appears" and "the reader is now correctly served" are different claims, and only
  the second is the deliverable.
- **Do not sweep the illustrations.** Roughly 80 sites look like hits and are
  correct: the reference-form tables that *define* id shape (`glossary.md`, the
  `templates/*.md` reference-forms headers), client-structure references
  (`.doctrine/spec/`, `adr-nnn.md`), and fill-in-the-blank scaffolding. Classified
  site-by-site in ISS-309 and
  `mem.pattern.install.shipped-corpus-citation-illustrations`. A careless sweep
  corrupts the id-vocabulary docs and every projected template.

## Non-Goals

- **The drift gate.** A check that refuses repo-private citations in the shipped
  corpus so it cannot re-drift. Engine code, different risk profile, and an
  unsettled design question: which existing seam to ride (IMP-163's self-correction
  gate, IMP-411's inert-anchor conformance, or `doctor`'s existing unresolved-citation
  check), and how to settle the duplicate-rule problem — two POL-002 memories, one
  sub-corpus each, no single home. This slice's follow-up; ISS-309's part 2.
- **The boot-index defect (ISS-215).** Three shipped signposts exist on disk and are
  absent from the boot snapshot index. Confirmed an engine indexing defect, not a
  stale snapshot: regeneration does not add them and `boot --check` reports clean.
  Engine code, independent of corpus prose.
- **Distilling project-local memories into shipped** (CHR-036) — mid-flight, with its
  own source corpus.
- **Local memory health** (`.doctrine/memory/items/**` staleness, scope globs). A
  different corpus, with its own existing items.

## Summary

Bring the shipped corpus — `install/`, `memory/`, and the `plugins/` skills — into
conformance with POL-002 as a client-facing artifact: no citation a client cannot
resolve, no claim the CLI does not support, and no CLI surface that agents are told
to use but cannot read about. One change, one reader, one acceptance test; the drift
gate that would keep it conformant is a separate follow-up.

## Follow-Ups

- The drift-gate slice — ISS-309 part 2, with IMP-163 and IMP-411 as candidate seams.
  It must settle the duplicate-rule problem rather than adding a third rule.
- ISS-215 — the `gather_assets` key-named-dir skip that hides three shipped signposts.
- If the permissible grounding for a shipped claim turns out to be a project-global
  decision rather than a slice-local one, it belongs in an ADR descending from
  ADR-005 / ADR-019, not in prose here.
