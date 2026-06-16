# Review RV-041 — reconciliation of SL-076

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciliation audit of SL-076 (concept-map-web). All 7 phases completed;
1539 tests green, just gate clean. This review probes:

1. **Catalog integration** — CM entities in KINDS, outbound_for, sidebar list,
   kind filter, search. Verify CM kind pill colour (#16A085) and filter
   checkbox.
2. **Route design conformance** — GET/POST /api/concept-map/:id: response shape
   (nodes, edges, diagnostics, dsl_hash, description), error mapping (400/404/
   409/500), stale-write guard (base_hash). Thin-wrapper pattern: no DSL
   semantics in routes.
3. **DSL mutation correctness** — add/remove/rename_edge pure functions: input
   trimming, duplicate detection, edge-not-found, key-based collision, TOML
   preservation, comment preservation. No parallel write path.
4. **Frontend model + rendering** — conceptMapCache, normalizeConceptMap,
   isConceptMap, cmGraphToDot with escapeStringContent, cmNeighbourhood BFS
   filtering, cmFocusNode with URL hash, depth sharing, old-render guard
   (graphRenderSeq).
5. **Authoring UI** — add-edge form with autocomplete, edge table with remove
   buttons, inline rename, Edit/Done toggle, stale-write handling (base_hash
   sent, 409 auto-refetch).
6. **Docs + tradeoffs** — set_dsl inline comment loss documented in rustdoc;
   sha2 and hex deps verified.

Invariants: catalogue parity with other entity kinds; TOML preservation;
structured data from Rust, dumb JS; stale-render guard on both paths.

## Synthesis

SL-076 is a clean, well-tested slice. 7 phases delivered 1539 passing tests
across 115 unit tests in `concept_map.rs`, 43 route integration tests,
and comprehensive JS coverage. The design was followed closely.

**What checked out:**

- **Catalog integration** is complete and correct: CM in `integrity::KINDS`
  with stem `concept-map` and no `state_dir`; `outbound_for("CM")` returns
  `Ok(Vec::new())` alongside REQ; CM filter checkbox with `#16A085` pill
  colour; `kindOrder.CM = 20` for proper sort position in the entity list.

- **Route design** holds the thin-wrapper contract: routes call
  `concept_map::parse_ref`, `read_concept_map`, `get_dsl`, `parse_dsl`,
  `check`, `set_dsl` — zero DSL parsing in routes. GET returns correct
  shape (nodes, edges, diagnostics, dsl_hash, description). POST supports
  all three actions with proper error discrimination (400/404/409/500).
  Stale-write guard (`base_hash`) works: optional, 409 on mismatch,
  last-write-wins without it.

- **DSL mutations** are pure and well-tested: input trimming, duplicate
  edge detection, key-based node identity for collision checks, edge-not-found
  reporting, TOML field preservation (all non-dsl keys survive), DSL comment
  preservation (inner-# comments, blank lines). No parallel write path —
  the CLI shell verbs and web routes call the same `add_edge_to_dsl`/
  `remove_edge_from_dsl`/`rename_node_in_dsl` functions.

- **Frontend data layer** is correct: `conceptMapCache` clears on refresh
  and evicts stale CM entries on focus change; `normalizeConceptMap` parses
  the structured Rust response; `isConceptMap` uses `kindPrefix === 'CM'`
  from the catalog graph; `cmGraphToDot` uses `escapeStringContent` (bare
  escape, no surrounding quotes) and produces valid DOT with record-shaped
  nodes, `#f8f9fa` fill, `#4A90D9` border/edges, and `penwidth=3.0` on the
  focal node.

- **Stale-render guard** covers both paths: `renderConceptMap` increments
  `graphRenderSeq` before each DOT render and discards stale SVG responses
  (both success and error paths). The existing entity-graph guard is
  unchanged and covers that path independently.

- **Authoring UI** is fully wired: add-edge form with `<datalist>`
  autocomplete (rebuilt from cache after mutations); edge table with `[✕]`
  remove buttons; inline rename via click-to-edit (edit-mode SVG click
  triggers `startRenameNode`, view-mode click toggles `cmFocusNode`);
  `base_hash` sent with every POST mutation; 409 `stale_concept_map`
  auto-refetches.

- **CM focal node** with depth filtering works: BFS neighbourhood from
  `cmFocusNode` up to `state.depth` hops; `cm_focus=<key>` in URL hash;
  depth selector shared between entity and CM views.

- **Error surface** is complete: all states from the design's error table
  are handled — not-found (404), parse failure (500), duplicate edge (409
  inline warning), stale map (409 auto-refetch), empty field (400), node
  collision (409 inline warning), edge not found (404).

**What was found (4 findings, all resolved):**

| Finding | Severity | Disposition |
|---|---|---|
| F-1: `set_dsl` missing rustdoc re: dsl-key inline comment loss | minor | fix-now |
| F-2: `EdgeNotFound` missing source/rel/target fields | minor | tolerated |
| F-3: Mutation function signatures deviate from design spec | nit | design-wrong |
| F-4: `EmptyField` uses String instead of `&'static str` | nit | tolerated |

F-1 was fixed in this audit (rustdoc added to `set_dsl`). F-2 and F-4 are
conscious tolerations — the bare `EdgeNotFound` is functionally correct,
and the `String` allocation in `EmptyField` is negligible for a single-user
loopback tool. F-3 is a design-spec drift: the mutation functions take DSL
text (not full TOML) and the routes orchestrate `get_dsl`/mutate/`set_dsl`.
This improves testability and is functionally equivalent; `design.md` should
be updated to match.

**Standing risks:**

- The `dsl`-key inline comment loss in `set_dsl` is a documented tradeoff.
  If hand-edited CM TOML files carry inline comments on the `dsl` key line,
  they will be lost on first web mutation. Mitigation: `doctrine concept-map
  new` produces no such comments; the tradeoff is explicit.

- `remove_edge` removes only the first matching line when duplicates exist.
  This is intentional (legacy hand-edited files may have duplicates) but
  could surprise a user who added the same edge twice via the CLI.

- The CLI `run_rename_node` uses case-insensitive label matching while the
  web `rename_node_in_dsl` uses derived-key identity. This semantic
  difference is intentional per the design (CLI to gain key-based matching
  in a follow-up) but means CLI and web rename have different collision
  behaviour.

**Verdict: ready for close.** No blockers remain. The slice is implemented
as designed, thoroughly tested, and clean against `just gate`.
