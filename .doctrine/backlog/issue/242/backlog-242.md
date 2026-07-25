# ISS-242: Concept-map kind is ungoverned on both the product and tech axes

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observation

`src/concept_map.rs` is **4,104 loc** — the largest single dark module in the
repo — plus nine CLI verbs (`concept-map new|list|show|check|add|remove|
rename-node|export|…`). Nothing governs it on either axis:

- **Tech axis.** No tech spec anchors it. SPEC-025 (Web explorer) governs the
  concept map's *HTTP read/mutate surface* and explicitly pushes the kind's own
  CLI verbs out of scope; nothing catches them. The DSL model itself
  (`parse_dsl` / `check` / `get_dsl` / `set_dsl` and the edit algebra) is
  described in SPEC-025's prose only where the server touches it.
- **Product axis.** No PRD claims the concept map *as a capability*. PRD-016
  §2 claims the **concept-map view** — rendering and structural editing inside
  the web explorer — but not the kind: its authoring workflow, its DSL, its
  diagnostics, or its export.

It is the only entity kind in the corpus with no kind-surface spec. Every peer
has one: SPEC-005 (ADR), SPEC-014 (slice), SPEC-015 (backlog), SPEC-016
(POL/STD), SPEC-019 (knowledge).

## Why this is a capture, not a defect to fix now

Ranked second in the CHR-046 spec-coverage census (behind the graph projection,
now SPEC-027) and **deliberately excluded from that round by operator
decision** — recorded here as a known dark zone rather than an oversight.

It ranked below the graph gap despite being twice the size because it is
*self-contained*: nothing else depends on it, so its darkness does not
propagate. Its cost is latent, not compounding.

## Why it cannot be closed by authoring a tech spec alone

The blocking question is at the **product** altitude, and it is a real fork:

- **(a) Own capability.** The concept map is a human-authored narrative
  artefact with its own DSL, lifecycle, and diagnostics — arguably a capability
  beside graph exploration, not inside it. This reading wants a new PRD, then a
  component tech spec descending from it.
- **(b) A channel of PRD-016.** The concept map is one of the three views
  PRD-016 already names, and the kind is simply that view's storage and CLI. This
  reading wants PRD-016's scope widened and a tech spec descending from it.

Authoring a tech spec before settling this would guess at `descends_from` — the
exact failure mode [[ISS-239]] records for the three specs that guessed by
omitting it.

The tie-break evidence worth gathering: whether the DSL and its diagnostics are
meaningfully usable *without* the explorer (reading (a)) or only ever read
through it (reading (b)); and whether PRD-016's OQ-001 (should the concept map
cross-validate against the live relation graph?) resolves toward a bridge
capability, which would make the kind's independent standing clearer.

## Suggested path

1. Settle the product-altitude call above — `/spec-coverage-assessment` scoped
   to the concept-map surface, or a `/knowledge` decision record if the call can
   be made from what is already known.
2. Then author the tech spec: component level, anchoring `src/concept_map.rs`
   and its CLI module, riding SPEC-025's boundary language (the explorer
   *consumes* the DSL seam and says so) rather than restating it.

Related: [[CHR-046]] (the census that surfaced it), [[SPEC-027]] (the rank-1 gap
from the same census, now closed), [[ISS-239]] (unplaced descent, same corpus
sweep), [[IDE-015]] (concept-map ↔ relation-graph bridge).
