# SL-143 Implementation Plan — Rationale & Sequencing

## Why Five Phases

PHASE-01 through PHASE-05 decompose the design into dependency-ordered units.
Each phase produces a coherent, reviewable delta. The critical constraint:
**the overview (PHASE-03) must be written against the final corpus shape**,
which means PHASE-02 (content update, which creates/deletes/promotes memories)
must complete first.

### PHASE-01: Audit

The audit produces a single findings ledger. It is pure research — no memory edits.
This separates evidence-gathering from action, so later phases can cite the ledger
rather than rediscovering facts. The preflight research (`.doctrine/state/sl-143-preflight-*.md`)
already surfaced the critical gaps; PHASE-01 extends this to a per-memory systematic audit.

Key audit dimensions:
- **Currency**: body correctness against current CLI surface (`doctrine --help`) and entity model
- **Completeness**: `commands` scope in TOML covers all relevant verbs; entity kind references current
- **Wikilinks**: outbound links valid, no broken references; TOML `[[...]]` false-positives corpus-wide
- **D5 audit**: does the memory enumerate CLI verbs inline (vs pointing to `doctrine --help`)?
- **ADR-002**: evergreen compliance — no repo-specific detail, no stale anchors
- **POL-002**: platform independence — concrete signature checklist per D7 (commit-scoping patterns,
  branch names, build commands, jail/bwrap specifics)

Agent note: 29-memory read-only audit against CLI output is a strong candidate for
parallel scout delegation.

### PHASE-02: Content Update

The bulk of the corpus work. PHASE-02 takes the PHASE-01 ledger and executes per-memory
fixes: currency, completeness, D5 verb-enumeration cleanup, metadata updates.
It also handles corpus structural changes:
- Deletion of cli-command-map (redundant with D1 overview + D5 CLI-in-binary)
- Creation of REC, RFC, CM signposts (entity kind coverage gaps)
- Promotion of work-intake-membership per D9: create new shipped memory with
  `repo=""`, `anchor_kind=none`; retire the local original via `doctrine memory record --retire`
- POL-002 remediation: conventions memory `(SL-NNN)` → generic `(scope): …` examples
- D5 enforcement on surviving memories that enumerate verbs (reading-entities,
  skill-map, lifecycle-start, review, and others flagged in the ledger)

PHASE-02 gates on PHASE-01 (ledger). It is file-disjoint from PHASE-03 (overview)
but MUST complete before PHASE-03 because the overview's when-to-retrieve-what
table references the complete corpus shape.

Agent note: content update benefits from a librarian agent to verify CLI surface
accuracy when fixing stale verb references. The bulk-editing nature (28+ files)
may justify batch-submission via dispatch workers.

### PHASE-03: Overview Rewrite

The overview is the hub — every other memory links back to it. Written against
the **complete post-PHASE-02 corpus** so the when-to-retrieve-what table covers
all 31 non-overview memories (including the 3 new signposts and the promoted
work-intake-membership). No stale references to deleted memories. No missing rows.

The when-to-retrieve-what table is the core novelty — a markdown table mapping
action-oriented situations to shipped memory keys. This replaces the flat signpost
index in the current overview with a decision guide: an agent reads the overview,
checks the table for their situation, and retrieves exactly the memory they need.

Line budget: ≤ ~100 lines (relaxed from original ~60 to accommodate the
comprehensive table). The table is ~33 lines; other sections are concise pointers,
not deep reference.

### PHASE-04: Reachability

Verification-heavy phase. Validates the wikilink web, adds skill references per
D11 pre-assignment table, and normalises formatting. The ≤3-hop reachability
property ensures an agent starting from overview can navigate to any shipped
memory without dead ends.

Key additions:
- Each concept/fact/pattern memory gains a `[[mem.…]]` reference in its
  pre-assigned skill's SKILL.md (D11 mapping)
- Overview differentiated as starting point in boot digest
- `[[relation]]` TOML-table references in relating-entities normalised to
  backtick formatting

Agent note: ≤3-hop graph validation is parallelisable — researcher agents can
validate disjoint subgraphs. Skill reference additions are lightweight (one-line
per skill), file-disjoint from each other, and could run in parallel.

### PHASE-05: Re-embed & Gate

The final integration phase. Pre-checks verify tooling and directory structure;
then a single re-embed cycle avoids the build-tax of incremental edits.

- Pre-checks: verify `memory/` directory structure, `doctrine memory sync`
  availability, jail target-dir redirect
- `touch src/corpus.rs && cargo build` forces RustEmbed recompilation
- `doctrine memory sync` materialises the embedded corpus
- `doctrine claude install` refreshes skills with new memory references
- `just gate` must pass green
- Post-checks: verify 32 shipped memories, no stale UUID duplicates, overview
  body correct, cli-command-map absent, no POL-002 violations, D5 compliance

## Agent Strategy

Several phases benefit from sub-agent delegation. The plan does not mandate
agent use — solo execution or dispatch can employ these at the orchestrator's
discretion.

| Phase | Agent Type | Rationale |
|-------|-----------|-----------|
| PHASE-01 | scout / researcher (parallel) | 29 memories read-only against CLI output; all independent; parallel scouts produce per-memory findings faster and catch patterns a serial agent might miss |
| PHASE-02 | librarian | Verifying CLI surface accuracy when fixing stale verb references; fetching `--help` output for each verb to cross-check memory body claims |
| PHASE-04 | researcher (parallel) | ≤3-hop graph validation across 32 nodes; disjoint subgraphs can be validated independently |
| PHASE-04 | (skill edits) | Adding one-line `[[mem.…]]` references to skills per D11 — 12 skills, all file-disjoint, parallelisable |

Dispatch workers should use the `scout` or `researcher` agent types with the
`librarian` skill where available. For PHASE-01 in particular, fanning out per
memory (or per batch of 5-6 memories) would reduce wall-clock time significantly.

## Deferred

IMP-163 (:after SL-147) — self-correction gate via domain-map. Not a phase to
execute; tracked as a backlog dependency in slice-143.toml (`tracked_by`).

## Phase Dependencies

```
PHASE-01 (audit) ──→ PHASE-02 (content) ──→ PHASE-03 (overview) ──→ PHASE-04 (reachability) ──→ PHASE-05 (gate)
```

PHASE-02 gates on PHASE-01 (ledger informs every fix). PHASE-03 gates on PHASE-02
(overview must reflect the complete post-content-update corpus, including new
signposts and the promoted memory). PHASE-04 gates on PHASE-02 + PHASE-03
(complete corpus + overview as reachability root). PHASE-05 is the terminal gate.
