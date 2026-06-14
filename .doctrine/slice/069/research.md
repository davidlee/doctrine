# SL-069 research: shipped memory corpus audit

## 1. Catalogue of shipped memories

The shipped orientation corpus consists of 14 memories under `memory/` (repo root),
embedded via RustEmbed, materialised by `doctrine memory sync` into the gitignored
`.doctrine/memory/shipped/`. All carry the ADR-002 class signature: `repo=""`,
`anchor_kind=none`, scoped (≥1 path/glob/command), evergreen (non-decaying
`reference` staleness). Types span signpost, concept, pattern, and fact — the full
type surface except `system` and `thread`, which are project-local by convention.

### Signposts (5) — Navigation pointers

| # | key | title |
|---|-----|-------|
| 1 | `mem.signpost.doctrine.overview` | Four pillars: slice lifecycle, governance, memory, entity engine |
| 2 | `mem.signpost.doctrine.file-map` | Layout: `.doctrine/slice/`, `adr/`, `memory/`, `state/`, `doc/`, `install/`, `src/` |
| 3 | `mem.signpost.doctrine.lifecycle-start` | route → slice → design → plan → phase → audit → close |
| 4 | `mem.signpost.doctrine.skill-map` | Routing table: When → Skill, plus mid-flight skills (consult, notes, record-memory) |
| 5 | `mem.signpost.doctrine.cli-command-map` | CLI verb surface: install, slice, memory, adr, spec, backlog, boot |

### Concepts (4) — Durable mental models

| # | key | title |
|---|-----|-------|
| 6 | `mem.concept.doctrine.storage-model` | Authored vs runtime vs derived; the storage rule (TOML for data, MD for prose) |
| 7 | `mem.concept.doctrine.entity-engine` | Shared engine for all authored artifacts; behaviour-preservation gate |
| 8 | `mem.concept.doctrine.memory-model` | Two faces: local-captured (scoped+anchored) and shipped orientation (global+unanchored) |
| 9 | `mem.concept.doctrine.routing-gate` | Route-before-you-act: mandatory gate, stricter-skill-when-unsure, no-code-without-plan |

### Patterns (3) — Reusable workflows and rules

| # | key | title |
|---|-----|-------|
| 10 | `mem.pattern.doctrine.core-loop` | `/route → /slice → /design → /inquisition → /plan → /phase-plan → /execute → /audit → /close` |
| 11 | `mem.pattern.doctrine.conventions` | Conventional commits, pure/imperative split, immutable ids, no parallel implementation |
| 12 | `mem.pattern.doctrine.tdd-loop` | Red → green → REFACTOR per phase; test behaviour not implementation |

### Facts (2) — Non-negotiable truths

| # | key | title |
|---|-----|-------|
| 13 | `mem.fact.doctrine.cli-source-of-truth` | CLI is source of truth for command shapes; guessed flags are stale flags |
| 14 | `mem.fact.doctrine.storage-tiers` | Cheat sheet: authored (committed), runtime state (gitignored, disposable), derived |

### Summary statistics

- **Types:** 5 signpost, 4 concept, 3 pattern, 2 fact (no system, no thread)
- **Total body size:** ~4,500 words across all 14 `.md` files
- **Scopes:** broad `.doctrine/` paths, `doctrine` command; one memory
  (`mem.pattern.doctrine.tdd-loop`) additionally scoped to `src/`
- **Coverage:** the corpus covers the *what* and *how* of doctrine's internal
  machinery (storage, entities, lifecycle, routing, conventions) but not the
  *why* of client operations (install, boot, memory recording, backlog, ADR,
  review, reconciliation)

---

## 2. Non-shipped memories with client-relevant content

Of 250 project-local memories under `.doctrine/memory/items/`, the vast majority
are implementation patterns for building doctrine itself (dispatch internals, lint
rules, testing gotchas, build conventions, render quirks). The following contain
durable knowledge that would materially improve correctness for a client agent
driving doctrine:

### Usage & distribution — foundational principles

| uid | key | relevance |
|-----|-----|-----------|
| `mem_019ea473...` | `mem.system.governance.ship-usage-guidance-to-clients` | **Critical.** Documents the worked failure: agent judged a requirement "hollow" by reading its `.md` (empty by design) because the *reading* consequence of the storage rule was absent from client surfaces. Establishes the push-vs-pull distribution split (boot asset for always-there, skill for on-demand). The origin of SL-069's premise. |
| `mem_019ea4f1...` | `mem.pattern.distribution.shipped-not-reachable` | **Critical.** Formalises the ADR-005 lesson: installing a `.md` into `.doctrine/` does not make it reachable; it must be pointed-at by the boot snapshot or a skill. This directly shapes which gaps matter — a memory that ships but has no pointer is invisible. |

### Core workflow concepts — client decision-making

| uid | key | relevance |
|-----|-----|-----------|
| `mem_019ea298...` | `mem.concept.workflow.canonical-change-loop` | ADR-003's change loop concept — distinct from the shipped `mem.pattern.doctrine.core-loop` in that it explains the "why" (four pillars, the reconciliation seam), not just the "what". |
| `mem_019ea25a...` | `mem.concept.backlog.work-intake-membership` | The work-intake membership test: what qualifies for backlog (`open→resolved→closed` lifecycle) vs knowledge record vs ADR. Directly governs the "which home for which record" decision clients make frequently. |
| `mem_019eb56e...` | `mem.system.lifecycle.defer-needs-backlog-before-close` | Deferred-but-needed-later items must be captured in a slice or backlog before closeout. A lifecycle invariant every client slice closure must honour. |

### Entity & memory mechanics — practical correctness

| uid | key | relevance |
|-----|-----|-----------|
| `mem_019ea4e4...` | `mem.pattern.entity.edit-preserving-status-transition` | Edit-preserving authored-TOML status transition — the invariant clients must uphold when hand-editing TOML fields no verb yet owns. |
| `mem_019ec14b...` | `mem.pattern.relation.relate-via-link-not-hand-authored-rows` | Relate entities via `doctrine link`, not hand-authored `[[relation]]` rows — hand-rows drift malformed and skip the legality check. A common client footgun. |
| `mem_019eafa1...` | `mem.pattern.memory.thread-hidden-until-verified` | Thread-type memories are hidden from `find`/`retrieve` until verified — a retrieval behaviour surprise that affects how clients capture and surface thread memories. |
| `mem_019eafc2...` | `mem.pattern.entity.kind-is-data-not-trait` | Entity-engine Kind is data, not a trait — fundamental to understanding how entity types are registered and why new types follow a standard pattern. |

### Reference docs awareness

| uid | key | relevance |
|-----|-----|-----------|
| — | (none) | No non-shipped memory carries a "these reference docs exist" signpost. However, the shipped push digest (`install/routing-process.md`) names `glossary.md` and `using-doctrine.md`. No concept memory introduces either — a client without the push digest (non-session-start context) has no pointer to them. |

---

## 3. Capability gap analysis

Doctrine's client-facing capabilities mapped against shipped memory coverage.
**Bold** entries are gaps where no shipped memory exists.

### Core lifecycle (well-covered)

| Capability | Shipped memory |
|---|---|
| Slice lifecycle ordering | ✓ `signpost.doctrine.lifecycle-start`, `pattern.doctrine.core-loop` |
| Routing gate | ✓ `concept.doctrine.routing-gate` |
| Design → plan → execute | ✓ `pattern.doctrine.core-loop` (covered as stages) |
| TDD discipline | ✓ `pattern.doctrine.tdd-loop` |
| Conventions (commits, ids, pure split) | ✓ `pattern.doctrine.conventions` |

### Storage & reading (covered but the critical failure lesson is missing)

| Capability | Shipped memory |
|---|---|
| Storage tiers (authored/runtime/derived) | ✓ `concept.doctrine.storage-model`, `fact.doctrine.storage-tiers` |
| Storage rule (TOML for data, MD for prose) | ✓ `concept.doctrine.storage-model` |
| **Reading entities via `show` (not raw files)** | ✗ — Guardrail exists in push digest only; the worked failure lesson (agent judged requirement "hollow" from empty `.md`) is in non-shipped `mem.system.governance.ship-usage-guidance-to-clients`. No shipped memory carries the *reading* consequence of the storage rule. |

### Governance & installation (largely absent)

| Capability | Shipped memory |
|---|---|
| **Installation** | ✗ — No shipped memory. `doctrine install` is mentioned in `fact.doctrine.cli-source-of-truth` as a verb to ask about, but there is no signpost/concept/pattern explaining what install does, what it ships, or how a client gets doctrine into their repo. |
| **Boot snapshot lifecycle** | ✗ — No dedicated shipped memory. The file map signpost mentions `boot.md` as a file; the CLI command map lists `boot` as a verb. No memory explains: what the boot snapshot IS, how it's regenerated (`doctrine boot`/`boot install`), the disk sentry (`boot --check`), or the freshen-now ritual (`boot` then `/clear`). |
| **Governance customisation** | ✗ — `install/governance.md` ships to clients but its remit section says "keep it narrow / pointers only." No shipped memory explains the governance surface model or how to populate `.doctrine/governance.md`. |
| **Reference docs pointer** | ✗ — `glossary.md` and `using-doctrine.md` ship to clients and are named in the push digest, but no concept/signpost memory introduces either. A client agent that hasn't ingested the push digest has no pointer. |

### Entity engine (covered)

| Capability | Shipped memory |
|---|---|
| Entity engine model | ✓ `concept.doctrine.entity-engine` |
| File layout / where things live | ✓ `signpost.doctrine.file-map` |
| **Relation/linking** | ✗ — `doctrine link`/`unlink` is the validated seam for connecting entities, but no shipped memory covers it. The non-shipped `mem.pattern.relation.relate-via-link-not-hand-authored-rows` captures the footgun. |

### Memory system (partially covered)

| Capability | Shipped memory |
|---|---|
| Memory model (two faces) | ✓ `concept.doctrine.memory-model` |
| Shipped corpus mechanism | ✓ `concept.doctrine.memory-model` (describes the shipped class) |
| **Recording client memories** | ✗ — No shipped memory covers `doctrine memory record` (the local-capture path), the born-frame/claim mechanism, scope selection, or `record --global` (that knowledge is in non-shipped `mem.system.memory.global-master-authoring`). |
| **Memory verification** | ✗ — `doctrine memory verify` exists but has no shipped memory coverage. |
| **Trust/severity model** | ✗ — The trust holdback (`retrieve` suppresses low-trust/high-severity memories) is documented in code but has no shipped memory. |
| **Thread-type visibility** | ✗ — Thread memories hidden until verified (`mem.pattern.memory.thread-hidden-until-verified`, non-shipped) — a retrieval surprise no shipped memory warns about. |

### Backlog, ADR, specs (absent)

| Capability | Shipped memory |
|---|---|
| **Backlog work intake** | ✗ — CLI command map mentions backlog verbs. No shipped memory covers the membership test, the promotion path (backlog → slice), or the kinds (`issue`/`improvement`/`chore`/`risk`/`idea`). |
| **ADR authoring** | ✗ — No shipped memory. The lifecycle start signpost implies ADRs exist but no concept/pattern explains when to author one, how to transition status, or the `proposed→accepted→superseded` lifecycle. |
| **Spec authoring (product/tech)** | ✗ — No shipped memory. Product and technical specs are major entity kinds with no orientation. |
| **Knowledge records** | ✗ — `doctrine knowledge` exists as a verb; no shipped memory covers the knowledge-record kinds (`assumption`/`decision`/`question`/`constraint`). |

### Review, audit, reconciliation (absent)

| Capability | Shipped memory |
|---|---|
| **Review/audit lifecycle** | ✗ — No shipped memory. The audit phase appears in the core loop and lifecycle signpost but has no concept memory. `doctrine review new`/`raise`/`dispose`/`verify`/`contest` has no shipped orientation. |
| **Requirements & reconciliation** | ✗ — No shipped memory. `doctrine coverage show`/`record`/`verify`, `doctrine reconcile` — all unrepresented. |
| **Standards & policies** | ✗ — No shipped memory. `doctrine policy new`, `doctrine standard new` exist but have no shipped orientation. |

### Advanced workflow (largely absent)

| Capability | Shipped memory |
|---|---|
| **Worktree/dispatch** | ✗ — Significant capability area with no shipped memory. |
| **Revision change-axis** | ✗ — `doctrine revision new` — ADR-013 kind, no shipped memory. |

---

## 4. Gap summary & prioritisation

### Tier 1 — Critical onboarding gaps (every client hits these day one)

1. **Installation** — no shipped memory. A client's first action with doctrine.
2. **Boot snapshot & governance surface** — the engine of the session prefix, invisible without explanation.
3. **Reading entities via `show`** — the worked failure lesson from `mem.system.governance.ship-usage-guidance-to-clients`; the reading consequence of the storage rule currently lives only in the push digest and `using-doctrine.md`.
4. **Reference docs pointer** — a signpost memory introducing `glossary.md` and `using-doctrine.md` so they're discoverable via `find`/`retrieve`.

### Tier 2 — High-value operational gaps (clients encounter in first slice)

5. **Relation/linking** — `doctrine link` as the validated seam; hand-row footgun.
6. **Memory record/verify workflow** — beyond the two-faces concept, into "how do I capture my own?"
7. **Backlog** — work intake model, membership test, promotion path.
8. **ADR authoring** — when, how, lifecycle states.

### Tier 3 — Expanding coverage (less frequent, still important)

9. **Review/audit lifecycle** — the RV kind, turn-based ledger, baton handoff.
10. **Requirements & reconciliation** — coverage, reconcile, observed tier.
11. **Spec authoring** — product/tech spec entity kinds.
12. **Standards & policies** — `doctrine policy new`/`standard new`.
13. **Knowledge records** — the epistemic record kinds.

### Tier 4 — Advanced (rare client operations, mostly doctrine-internal)

14. Worktree/dispatch, revision change-axis, `mem.pattern.memory.thread-hidden-until-verified` (corner case).

---

## 5. Tensions and open considerations

- **The push digest already carries some of this.** `install/routing-process.md` ships the "read entities via `show`" guardrail and names `glossary.md`/`using-doctrine.md`. The question is whether that's enough — the push digest's audience assumption is "the boot snapshot fires at session start," but an agent mid-session querying `memory find` will not see it. Shipped memories fill the *retrieval* gap the push digest leaves.
- **Some non-shipped memories are client-relevant but not canon-ready.** `mem.system.governance.ship-usage-guidance-to-clients` is a system memory with an open follow-up ("audit CLAUDE.md end-to-end") — promoting it as-is to shipped status would carry a stale caveat. The content is right; the framing may need updating.
- **The `using-doctrine.md` shipped doc covers the same ground as several proposed gaps** (which-verb-for-which-intent, reading-entities, relating-entities, backlog-vs-memory-vs-ADR boundary). A shipped signpost memory pointing to it may be more effective than duplicating its content into memories.
- **ADR-002 restricts shipped memory to `signpost` for reference-grade content** (no `reference` enum member). Most of the proposed Tier 1–2 gaps would be `signpost` or `concept` types, which is appropriate.
