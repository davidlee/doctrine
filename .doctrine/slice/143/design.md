# SL-143 Design

## Current State

29 shipped memories (6 concept, 2 fact, 3 pattern, 18 signpost) under `memory/`,
all `trust_level = "medium"`, `verification_state = "unverified"`, `repo=""`,
`anchor_kind=none` (ADR-002 compliant). Authored in two batches (June 5–6 core,
June 15–16 entity-kind signposts). Unreviewed since.

### Key Symptoms

- **cli-command-map 45% stale** — 14 verbs added since last update, 1 dead
  verb (`claude`). 21 of 43 current CLI verbs never mentioned in any shipped
  memory.
- **1 orphan** — `mem.signpost.doctrine.overview` has zero inbound wikilinks.
- **Entity kind coverage gaps** — REC (reconciliation records), RFC (RFC
  artifacts), CM (concept maps) have zero shipped memory coverage.
- **11 memories last updated June 6** (18 days stale at slice start).
- **Skills reference zero shipped memories** — concept/fact/pattern tier is
  only discoverable via wikilink breadcrumbs from signposts.
- **No onboarding path** — agents must navigate a flat index of 18 signpost
  keys in the boot digest with no progression or decision guide.

## Target State

32 shipped memories. Overview rewritten as a self-contained PULL hub (one
retrieval after boot). All memories current against the CLI surface and entity
model. Complete wikilink web. Self-correction deferred to SL-147.

## Design Decisions

D1. **Overview is the single PULL hub.** The boot digest differentiates
    overview from other signposts — it is listed first or marked as the
    starting point (e.g. `mem.signpost.doctrine.overview ← start here`).
    One `retrieve-memory mem.signpost.doctrine.overview` after boot gives
    the agent: doctrine's four pillars, the essential mental model (TOML+MD
    tiers, read-via-show, authored/runtime/derived), key conventions (reference
    forms, CLI is truth, no guessing), and a **when-to-retrieve-what** index —
    a markdown table with columns "When you need to..." | "Retrieve..." that
    maps situations to shipped memory keys. Concept/fact/pattern memories stay
    as deep reference; overview inlines just enough that the agent can function
    without pulling them.

D2. **Phase order: audit → content → overview → reachability → gate.**
    PHASE-01 produces the findings ledger. PHASE-02 executes corpus structural
    changes and content fixes against that ledger (creates new signposts,
    deletes cli-command-map, promotes work-intake-membership, fixes currency).
    PHASE-03 writes the overview hub against the complete post-PHASE-02 corpus,
    so the when-to-retrieve-what table covers all 31 non-overview memories.
    PHASE-04 validates reachability. PHASE-05 re-embeds and gates.
    The overview must be written last among content phases because its table
    depends on the final corpus shape — writing it earlier would produce a
    stale table referencing deleted memories and missing new ones.

D3. **Self-correction deferred to IMP-163 (tracked in slice-143.toml).**
    IMP-163 depends on SL-147's domain-map mechanism for automated
    staleness detection. No new mechanism in SL-143. The `after` relationship
    in slice-143.toml points to SL-147; IMP-163 is listed under `tracked_by`
    so the close gate can verify it hasn't been abandoned.

D4. **Corpus delta: +4, −1 = 32 memories.**
    - New signposts: `mem.signpost.doctrine.rec` (REC kind), `mem.signpost.doctrine.rfc` (RFC kind), `mem.signpost.doctrine.concept-map` (CM kind).
    - Promoted: `mem.concept.backlog.work-intake-membership` (currently project-local; body is normative, belongs in shipped corpus).
    - Deleted: `mem.signpost.doctrine.cli-command-map` (CLI verb enumeration lives in `--help` — a future slice enriches `doctrine --help` output; memory is a thin pointer at best, redundant with D1's overview hub).

D5. **CLI verb enumeration is a binary concern, not a memory concern.**
    The overview hub and `mem.fact.doctrine.cli-source-of-truth` both state
    "use `doctrine --help`." No shipped memory enumerates verbs, subcommands,
    or flags.

D6. **Wikilink reachability, not universal inbound links.** Overview is the
    root — every shipped memory is reachable from overview within ≤3 hops.
    Overview itself has ≥1 inbound wikilink from another shipped memory (the
    orphan fix). Not every leaf needs an explicit backlink — the reachability
    property ensures navigability without forced bi-directional links. Boot
    digest lists all signpost keys. Per ADR-005, every pull-reference memory
    must be pointed-at by at least one skill or the boot digest; concept/fact/pattern
    memories gain skill pointers in PHASE-04 where their content directly
    informs the skill's operation.

D7. **POL-002 platform independence.** Shipped memories are part of the shipped
    product. They must not reference host-project conventions or local state.
    PHASE-01 audit checks every memory for: commit-message style conventions
    (e.g. `fix(SL-NNN):`), project-local branch names (`edge/main`), build
    commands (`just gate`, `just check`), jail/bwrap specifics, or any other
    host-local convention. Findings: remediate or gate. The preflight found one
    candidate: `mem.pattern.doctrine.conventions` references `(SL-NNN)` commit
    scoping — this is this repo's convention, not a doctrine-owned contract.

D8. **Local corpus promotion considered.** Two local patterns
    (`mem.pattern.distribution.shipped-memory-authoring`,
    `mem.pattern.distribution.shipped-not-reachable`) describe doctrine's
    distribution architecture but are developer-facing, not end-user-facing —
    not promoted. The canonical-change-loop concept overlaps with already-shipped
    `core-loop` + `lifecycle-start` — not promoted. No other ship-worthy
    candidates in the local corpus.

D9. **Promotion mechanics: move-and-rewrite, not edit-in-place.** To promote
    `work-intake-membership` from local (`repo` is non-empty) to shipped
    (`repo=""`, `anchor_kind=none`): create a new shipped memory under `memory/`
    with the same body, appropriate metadata, and the shipped-class signature;
    then `doctrine memory record --retire <uid>` the local original.
    This avoids violating entity-engine consistency (the local memory has a
    real git anchor and `verified_sha` that don't transfer).

D10. **Agent delegation candidates.** PHASE-01 (29-memory audit against CLI
    surface) and PHASE-04 (≤3-hop wikilink graph validation) are parallelisable
    research tasks well-suited to scout or researcher sub-agents. PHASE-02
    (content update) benefits from a librarian agent to verify CLI surface
    accuracy when fixing stale verb references. The plan notes these as
    opportunities; dispatch or solo execution can use them at the
    orchestrator's discretion. No phase mandates sub-agent use.

D11. **ADR-005 skill-to-concept/fact/pattern pre-assignment.** Every
    concept/fact/pattern memory must be pointed-at by ≥1 skill or the boot
    digest (ADR-005 reachability). The pre-assigned mapping (executed in
    PHASE-04, verified in PHASE-05):

    | Memory | Pointed-at by |
    |---|---|
    | `mem.concept.doctrine.boot-snapshot` | boot digest (already referenced) |
    | `mem.concept.doctrine.entity-engine` | `/spec-tech` skill |
    | `mem.concept.doctrine.memory-model` | `/retrieve-memory` skill |
    | `mem.concept.doctrine.reading-entities` | `/inquisition` skill |
    | `mem.concept.doctrine.routing-gate` | `/route` skill |
    | `mem.concept.doctrine.storage-model` | `/canon` skill |
    | `mem.fact.doctrine.cli-source-of-truth` | `/canon` skill |
    | `mem.fact.doctrine.storage-tiers` | `/canon` skill |
    | `mem.pattern.doctrine.conventions` | `/canon` skill |
    | `mem.pattern.doctrine.core-loop` | `/route` skill |
    | `mem.pattern.doctrine.tdd-loop` | `/execute` skill |
    | `mem.concept.backlog.work-intake-membership` | `/backlog` skill |

## Phase Plan

### PHASE-01 — Audit

Produce a per-memory findings ledger covering:
- Currency: is the body correct against the current CLI surface and entity model?
- Completeness: does the memory's `commands` scope cover all relevant verbs?
- Wikilinks: are outbound links valid? are there broken references?
- TOML false-positives: corpus-wide scan for `[[...]]` patterns that could
  be misinterpreted as entity wikilinks (e.g. `[[relation]]`, `[[mem.…]]` as
  TOML-table or code references, not entity links).
- ADR-002 compliance: is the memory truly evergreen (no repo-specific detail,
  no stale anchors)?
- POL-002 compliance: does the memory reference host-project conventions?
  **Concrete signature checklist:**
  - `(SL-NNN)` or similar commit-scoping patterns
  - `edge/main` branch-pair references
  - `just gate`, `just check` build commands
  - `jail`, `bwrap`, `CARGO_TARGET_DIR`, `~/.cargo/doctrine-target-jail`
    build-environment specifics
  - `fix(SL-`, `doc(SL-`, `plan(SL-` commit-message patterns
  - Any other host-local convention referenced by name
- D5 audit: does the memory enumerate CLI verbs, subcommands, or flags inline
  (as opposed to pointing to `doctrine --help`)? Flag every memory that does.

Output: `.doctrine/state/sl-143-phase-01-ledger.md`.

Agent note: PHASE-01 is a strong candidate for parallel scout delegation —
29 memories read-only against `doctrine --help` output, all independent.

### PHASE-02 — Content Update

For each remaining memory (27 existing + 4 new):
- Fix currency against CLI surface and entity model per PHASE-01 ledger
- Apply D5 to surviving memories: remove inline CLI verb enumerations;
  replace with pointers to `doctrine --help`. The surviving memories that
  currently enumerate verbs (reading-entities, skill-map, lifecycle-start,
  review, and others flagged in the ledger) must be cleaned.
- Update `updated` date to 2026-06-24
- Add inbound wikilinks to overview where missing (overview key is stable;
  body will be rewritten in PHASE-03)
- Fix `commands` scope in TOML metadata
- Delete cli-command-map directory
- Create REC, RFC, CM signposts
- Promote work-intake-membership per D9: create new shipped memory under
  `memory/` with `repo=""`, `anchor_kind=none`, `created=2026-06-24`;
  retire the local original via `doctrine memory record --retire <uid>`
- Remediate POL-002 violations from PHASE-01 ledger (incl. conventions memory
  commit-scoping — conventional commits are a *doctrine-endorsed practice*, so
  replace `(SL-NNN)` with generic `(scope): …` examples, not host-local scoping)

Agent note: content update benefits from a librarian agent to verify CLI
surface accuracy when fixing stale verb references.

### PHASE-03 — Overview Rewrite

Rewrite `mem.signpost.doctrine.overview` as the hub, written against the
complete post-PHASE-02 corpus. Sections:
1. What doctrine is (four pillars)
2. Mental model (TOML+MD tiers, show, tiers)
3. When to retrieve what — a markdown table with one row per shipped memory
   (excluding overview itself, so 31 rows):
   ```
   | When you need to... | Retrieve... |
   |---|---|
   | Understand where files live | `mem.signpost.doctrine.file-map` |
   | Choose the right skill | `mem.signpost.doctrine.skill-map` |
   | Understand the core workflow | `mem.signpost.doctrine.lifecycle-start` |
   | ... | ... |
   ```
   Situations are action-oriented ("when you need to X"), not entity-kind-oriented.
   The table covers all 31 non-overview memories including the new signposts
   (rec, rfc, concept-map) and the promoted work-intake-membership.
4. Key conventions (reference forms, CLI is truth, no guessing)
5. Quick-links to file-map, skill-map, lifecycle-start

Line budget: ≤ ~100 lines. The when-to-retrieve-what table is the primary
content (~33 lines including header). Other sections must be concise — pillars
and mental model in 10-15 lines each, conventions in 5-8 lines, quick-links
in 3-4 lines. The overview does not replace deep reference; it orients.

Agent note: overview depends on PHASE-02 completing all corpus structural
changes so the table is correct on first write.

### PHASE-04 — Reachability

- Verify every memory has ≥1 inbound wikilink
- Verify overview reachable from boot digest (differentiate as starting point,
  D1)
- Verify all memories reachable from overview within ≤3 hops
- Add skill references per D11 pre-assignment table: each concept/fact/pattern
  memory gains a `[[mem.…]]` reference in its assigned skill's SKILL.md
  (or note in skill body if the skill already references it implicitly)
- Fix `[[relation]]` TOML-table references in `relating-entities` — normalize
  to backtick formatting
- Verify new signposts (rec, rfc, concept-map) linked into wikilink web
- Verify work-intake-membership linked from backlog signpost

Agent note: ≤3-hop graph validation is parallelisable — researcher agents
can validate disjoint subgraphs.

### PHASE-05 — Re-embed & Gate

- Pre-check: verify `memory/` source directory structure (human-readable
  key-named dirs, not symlinks; 32 expected)
- Pre-check: verify `doctrine memory sync` is available on the current binary
- `touch src/corpus.rs && cargo build` to re-embed (note: in the jail,
  CARGO_TARGET_DIR → ~/.cargo/doctrine-target-jail)
- `doctrine memory sync` to materialise changes
- `doctrine claude install` to refresh installed skills
- Verify: `doctrine memory find --glob 'shipped/*'` returns exactly 32 entries
- Verify: overview body contains rewritten sections (VT-02)
- Verify: cli-command-map key absent (VT-05)
- Verify: `just gate` passes green (zero warnings)
- Verify: no stale UUID-duplicate directories under `memory/` after sync

### Deferred (IMP-163, :after SL-147)

- Wire self-correction gate into reconcile/close via SL-147 domain-map

## Verification

| ID | Test |
|----|------|
| VT-01 | `doctrine memory find --glob 'shipped/*'` returns exactly 32 entries |
| VT-02 | `mem.signpost.doctrine.overview` body has sections: pillars, mental model, when-to-retrieve-what, conventions, links |
| VT-03 | Every shipped memory reachable from overview within ≤3 hops; overview has ≥1 inbound wikilink |
| VT-04 | Overview key listed in boot digest Memory section and differentiated as starting point |
| VT-05 | `mem.signpost.doctrine.cli-command-map` key absent from shipped corpus |
| VT-06 | `mem.signpost.doctrine.{rec,rfc,concept-map}` keys exist with non-empty bodies |
| VT-07 | `mem.concept.backlog.work-intake-membership` has `repo=""`, `anchor_kind=none` |
| VT-08 | `just gate` passes after re-embed (zero warnings) |
| VT-09 | No shipped memory references `claude` as a CLI verb |
| VT-10 | `[[relation]]` in `relating-entities` uses backtick formatting, not wikilink syntax |
| VT-11 | No surviving shipped memory body enumerates CLI verbs, subcommands, or flags inline — all point to `doctrine --help` (D5) |
| VT-12 | No shipped memory contains a POL-002 violation signature: `(SL-NNN)`, `edge/main`, `just gate`, `just check`, `jail`, `bwrap`, `CARGO_TARGET_DIR` |
| VT-13 | Every concept/fact/pattern memory pointed-at by its pre-assigned skill or boot digest per D11 |
| VT-14 | No stale UUID-duplicate directories under `memory/` after sync; exactly 32 human-readable key-named dirs |

## Affected Surface

- `memory/` — 28 existing directories updated, 3 created, 1 deleted, 1 promoted
- `src/corpus.rs` — touch target for re-embed
- `.agents/skills/` — may gain retrieve-memory calls to concept memories
- `.doctrine/state/boot.md` — regenerated; signpost key list may shift

## Risks

- **Re-embed footgun.** Every memory edit requires `touch src/corpus.rs && cargo build`. Batch edits per phase, verify in one build cycle. In the jail, `CARGO_TARGET_DIR` redirects to `~/.cargo/doctrine-target-jail`.
- **Corpus must stay evergreen.** Shipped memories carry `repo=""`, `anchor_kind=none`. Edits must not introduce repo-specific detail or stale anchors (ADR-002).
- **Overview bloat.** The overview must stay within ~100 lines — enough to orient, not enough to replace the deep reference. The when-to-retrieve-what table (~33 lines) is the bulk; other sections must be concise. Review for compactness before lock.
- **POL-002 platform independence.** Shipped memories must not reference host-project conventions. Preflight found one candidate: `mem.pattern.doctrine.conventions` references `(SL-NNN)` commit scoping. PHASE-01 audit must flag any other violations per the concrete signature checklist.
- **Promotion integrity.** The work-intake-membership promotion must use the move-and-rewrite pattern (D9), not an edit-in-place that would violate entity-engine consistency. Verify the local original is retired after promotion.
- **Stale UUID duplicates.** The sync cycle creates UUID-named directories alongside human-readable ones. After PHASE-05 sync, verify no stale duplicates remain — exactly 32 human-readable key-named dirs under `memory/`.
