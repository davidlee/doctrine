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

D1. **Overview is the single PULL hub.** The boot digest lists signpost keys
    (unchanged). One `retrieve-memory mem.signpost.doctrine.overview` after
    boot gives the agent: doctrine's four pillars, the essential mental model
    (TOML+MD tiers, read-via-show, authored/runtime/derived), key conventions
    (reference forms, CLI is truth, no guessing), and a **when-to-retrieve-what**
    index — a context-keyed decision guide: "about to relate entities? →
    relating-entities. Unsure which skill? → skill-map." Concept/fact/pattern
    memories stay as deep reference; overview inlines just enough that the
    agent can function without pulling them.

D2. **Phase order: overview-first, audit-inform rest.** Overview is the hub
    that everything links back to. Getting it right early gives a target for
    the remaining memories. Preflight findings already surfaced critical gaps;
    the formal PHASE-01 audit ledger confirms and extends them.

D3. **Self-correction deferred to SL-147 domain-map mechanism.** Filed as
    IMP-163 (:after SL-147). SL-147 creates a domain-map of changed files;
    the reconcile/close loop will use it to flag potentially stale shipped
    memories. No new mechanism in SL-143.

D4. **Corpus delta: +4, −1 = 32 memories.**
    - New signposts: `mem.signpost.doctrine.rec` (REC kind), `mem.signpost.doctrine.rfc` (RFC kind), `mem.signpost.doctrine.concept-map` (CM kind).
    - Promoted: `mem.concept.backlog.work-intake-membership` (currently project-local; body is normative, belongs in shipped corpus).
    - Deleted: `mem.signpost.doctrine.cli-command-map` (CLI verb enumeration lives in `--help` — a future slice enriches `doctrine --help` output; memory is a thin pointer at best, redundant with D1's overview hub).

D5. **CLI verb enumeration is a binary concern, not a memory concern.**
    The overview hub and `mem.fact.doctrine.cli-source-of-truth` both state
    "use `doctrine --help`." No shipped memory enumerates verbs, subcommands,
    or flags.

D6. **Wikilink rule: every memory has ≥1 inbound link.** Overview is the
    root — everything links back to it (directly or within 3 hops). Boot
    digest lists all signpost keys. Skills point to concept memories where
    their operations depend on the concept.

## Phase Plan

### PHASE-01 — Audit

Produce a per-memory findings ledger covering:
- Currency: is the body correct against the current CLI surface and entity model?
- Completeness: does the memory's `commands` scope cover all relevant verbs?
- Wikilinks: are outbound links valid? are there broken references?
- ADR-002 compliance: is the memory truly evergreen (no repo-specific detail, no stale anchors)?

Output: `.doctrine/state/sl-143-phase-01-ledger.md`.

### PHASE-02 — Overview Rewrite

Rewrite `mem.signpost.doctrine.overview` as the hub. Sections:
1. What doctrine is (four pillars)
2. Mental model (TOML+MD tiers, show, tiers)
3. When to retrieve what (context-keyed index of all shipped memories)
4. Key conventions (reference forms, CLI is truth, no guessing)
5. Quick-links to file-map, skill-map, lifecycle-start

### PHASE-03 — Content Update

For each remaining memory (27 existing + 4 new):
- Fix currency against CLI surface and entity model
- Update `updated` date to 2026-06-24
- Add inbound wikilinks to overview where missing
- Fix `commands` scope in TOML metadata
- Delete cli-command-map directory
- Create REC, RFC, CM signposts
- Promote work-intake-membership to shipped (set `repo=""`, `anchor_kind=none`, `created=2026-06-24`)

### PHASE-04 — Reachability

- Verify every memory has ≥1 inbound wikilink
- Verify overview reachable from boot digest
- Verify all memories reachable from overview within ≤3 hops
- Add skill references to concept/fact/pattern memories where appropriate
- Fix the `[[relation]]` TOML-table references in `relating-entities` and
  (former) `cli-command-map` — normalize to backtick formatting

### PHASE-05 — Re-embed & Gate

- `touch src/corpus.rs && cargo build` to re-embed
- `doctrine memory sync` to materialize changes
- `doctrine claude install` to refresh installed skills
- `just gate` must pass green

### Deferred (IMP-163, :after SL-147)

- Wire self-correction gate into reconcile/close via SL-147 domain-map

## Verification

| ID | Test |
|----|------|
| VT-01 | `doctrine memory find --glob 'shipped/*'` returns exactly 32 entries |
| VT-02 | `mem.signpost.doctrine.overview` body has sections: pillars, mental model, when-to-retrieve-what, conventions, links |
| VT-03 | Every shipped memory has ≥1 inbound wikilink from another shipped memory |
| VT-04 | Overview key listed in boot digest Memory section |
| VT-05 | `mem.signpost.doctrine.cli-command-map` key absent from shipped corpus |
| VT-06 | `mem.signpost.doctrine.{rec,rfc,concept-map}` keys exist with non-empty bodies |
| VT-07 | `mem.concept.backlog.work-intake-membership` has `repo=""`, `anchor_kind=none` |
| VT-08 | `just gate` passes after re-embed (zero warnings) |
| VT-09 | No shipped memory references `claude` as a CLI verb |
| VT-10 | `[[relation]]` in `relating-entities` uses backtick formatting, not wikilink syntax |

## Affected Surface

- `memory/` — 28 existing directories updated, 3 created, 1 deleted, 1 promoted
- `src/corpus.rs` — touch target for re-embed
- `.agents/skills/` — may gain retrieve-memory calls to concept memories
- `.doctrine/state/boot.md` — regenerated; signpost key list may shift

## Risks

- **Re-embed footgun.** Every memory edit requires `touch src/corpus.rs && cargo build`. Batch edits per phase, verify in one build cycle.
- **Corpus must stay evergreen.** Shipped memories carry `repo=""`, `anchor_kind=none`. Edits must not introduce repo-specific detail or stale anchors (ADR-002).
- **Overview bloat.** The overview must stay within ~60 lines — enough to orient, not enough to replace the deep reference. Review for compactness before lock.
- **Skill-amendment scope creep.** Adding retrieve-memory calls to skills touches the restate-line (ADR-005). Keep amendments minimal — add pointers only, no flag tables.
