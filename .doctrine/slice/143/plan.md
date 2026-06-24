# SL-143 Implementation Plan — Rationale & Sequencing

## Why Five Phases

PHASE-01 through PHASE-05 decompose the design into file-disjoint, dependency-ordered
units. Each phase produces a coherent, reviewable delta.

### PHASE-01: Audit

The audit produces a single findings ledger. It is pure research — no memory edits.
This separates evidence-gathering from action, so later phases can cite the ledger
rather than rediscovering facts. The preflight research (`.doctrine/state/sl-143-preflight-*.md`)
already surfaced the critical gaps; PHASE-01 extends this to a per-memory systematic audit.

Key audit dimensions:
- **Currency**: body correctness against current CLI surface (`doctrine --help`) and entity model
- **Completeness**: `commands` scope in TOML covers all relevant verbs; entity kind references current
- **Wikilinks**: outbound links valid, no broken references
- **ADR-002**: evergreen compliance — no repo-specific detail, no stale anchors
- **POL-002**: platform independence — no host-convention references (commit scoping, branch names, build commands)

### PHASE-02: Overview Rewrite

The overview is the hub — every other memory links back to it. Getting it right first
(D2) gives a target for PHASE-03 inbound wikilinks and PHASE-04 reachability. The
overview depends on preflight research, not PHASE-01 ledger — the preflight already
surfaced the key corpus shape; the ledger confirms detail on the remaining 28 memories.

The when-to-retrieve-what table is the core novelty — a markdown table mapping
action-oriented situations to shipped memory keys. This replaces the flat signpost
index in the current overview with a decision guide: an agent reads the overview,
checks the table for their situation, and retrieves exactly the memory they need.

### PHASE-03: Content Update

The bulk of the corpus work. PHASE-03 takes the PHASE-01 ledger and executes per-memory
fixes: currency, completeness, metadata updates. It also handles the corpus structural
changes:
- Deletion of cli-command-map (redundant with D1 overview + D5 CLI-in-binary)
- Creation of REC, RFC, CM signposts (entity kind coverage gaps)
- Promotion of work-intake-membership (normative body, repo="" grade)

File-disjoint from PHASE-02 (overview is a different file). Could run in parallel
with PHASE-02 if desired, but sequenced after to ensure inbound wikilinks target
the final overview body.

### PHASE-04: Reachability

Verification-heavy phase. No content changes to memory bodies (barring minor
wikilink fixes) — it validates the wikilink web, adds skill references for
ADR-005 compliance, and normalises formatting. The ≤3-hop reachability property
ensures an agent starting from overview can navigate to any shipped memory
without dead ends.

### PHASE-05: Re-embed & Gate

The final integration phase. `touch src/corpus.rs && cargo build` forces
RustEmbed recompilation (the re-embed footgun). `doctrine memory sync`
materialises the embedded corpus. `doctrine claude install` refreshes skills.
`just gate` must pass green. This phase gates on PHASE-02/03/04 all being
complete — it is the single re-embed cycle that avoids the build-tax of
incremental edits.

## Deferred

IMP-163 (:after SL-147) — self-correction gate via domain-map. Not a phase to
execute; tracked as a backlog dependency.

## Phase Dependencies

```
PHASE-01 (audit) ─────────────────────┐
                                      ├── PHASE-03 (content) ──┐
PHASE-02 (overview) ──────────────────┘                        ├── PHASE-04 (reachability) ── PHASE-05 (gate)
                                                               │
                                          (file-disjoint,       │
                                           could parallelise     │
                                           if desired)          │
```

PHASE-02 is independent of PHASE-01 (depends on preflight). PHASE-03 depends on
both PHASE-01 (ledger) and PHASE-02 (overview as wikilink target). PHASE-04
depends on PHASE-02 + PHASE-03 (complete corpus). PHASE-05 is the terminal gate.
