# SL-046 — implementation notes

Durable notes for the cross-kind relation graph spine. Phase-by-phase landings,
the load-bearing decisions, and the gotchas worth a future reader's time. Runtime
progress lives in the gitignored phase sheets; this is the keep.

## What shipped

`doctrine inspect <ID> [--json]` — a `show`-like read surface over a cross-kind
relation graph. Outbound = the entity's own authored relations grouped by label;
inbound = derived from the graph's reverse index (`in_edges`) per overlay;
danglers = unresolved / free-text / no-overlay outbound targets.

- **PHASE-01** `5bedbe1` — `Projection<K: Copy+Ord>` leaf (`src/projection.rs`):
  id↔NodeId bimap, mint-or-get `intern` in caller call-order. `backlog_order`
  swapped onto it (byte-exact golden held — the mint-sequence C4 tripwire).
- **PHASE-02** `ff1745c` — relation vocabulary leaf (`src/relation.rs`:
  `RelationLabel` + `RelationEdge`) + 6 per-kind `relation_edges` accessors +
  `outbound_for` data-dispatch (`src/relation_graph.rs`).
- **PHASE-03** `c86eea5` — `build_relation_graph` (all-kind scan → overlays →
  graph) + `inspect` query.
- **PHASE-04** `c316d6f` — `inspect` CLI command + human render + `--json`.

## Load-bearing decisions

- **`EntityKey { prefix: &'static str, id: u32 }`, NOT `{ kind: entity::Kind }`.**
  `entity::Kind` carries a fn-ptr (`scaffold`) so it is not `Ord` — it cannot be a
  `Projection` key. Key on the `&'static str` prefix (Copy + Ord); render the
  canonical ref via `listing::canonical_id(prefix, id)`. The design's literal
  `kind: entity::Kind` was illustrative.
- **13 `RelationLabel` = 11 overlay-backed + `Drift`/`DecisionRef`** (ADR-010 D2).
  The two free-text labels get NO cordage overlay — their targets have no kind in
  `integrity::KINDS`, never resolve, and always surface as danglers (visibility
  preserved, not dropped). `overlay_for(label)` returns `None` for them. The
  design's "~9"/"~11 overlays" is the approximate overlay count; the authoritative
  split is 11 overlay-backed labels.
- **Reference overlays are `Reject` + `Unbounded`** (I1). `Reject` removes no edges;
  `Unbounded` exempts arity eviction. So `in_edges` enumerates exactly the authored
  *unique* inbound set. `EdgeAttrs::new(0,0)` on every reference edge ⇒ two authored
  rows sharing `(label,src,dst)` collapse to one in cordage's `BTreeSet<Edge>` (C3
  dedupe — benign, tested).
- **Inbound is derived, never stored** (ADR-004 §3). `supersedes`-overlay inbound
  renders "superseded by" by *section*, reading no `superseded_by` field. A lone
  stored `superseded_by` with no reciprocal `supersedes` yields no inbound (C8/R3).
- **Scan order is `KINDS` table order, ids sorted ascending** (C5). `scan_ids`
  returns unsorted `read_dir` order; the explicit `sort()` makes mint + render
  filesystem-independent (REQ-077).
- **`inspect` never reads `graph.provenance()`** (C7) — a benign symmetric
  `related` 2-cycle emits a `Reject` `CycleDiagnostic` that must not leak into the
  relation view; diagnostics are a validate/SL-048 concern.
- **`--json` hand-builds `serde_json::json!`** (the `spec::show_json` precedent) —
  no `Serialize` derive on the domain enums. Interaction free-text `type` is a
  human-render annotation only (re-read at render from `interactions.toml`, C2);
  `--json` serializes the plain 4-field `InspectView`.
- **`build_relation_graph` is `pub(crate)`** (factored, not inlined in `inspect`)
  so SL-047 reuses the scan rather than re-forking it.
- **PHASE-04 retired the PHASE-02/03 `not(test)` `dead_code` expects.** Once
  `inspect` is CLI-reachable, the vocabulary leaf, per-kind accessors, scan and
  `inspect` are all live — their self-clearing expects became unfulfilled
  (strict-`expect` errors) and were removed across relation/backlog/governance/rec/
  review/slice/spec.rs + the module attr. Mechanical, behaviour-preserving; the
  existing suites stayed green unchanged (behaviour-preservation gate).

## Audit (RV-006, `730a277`) — both findings terminal, non-blocking

- **F-1 → follow-up (IMP-036):** the full-corpus scan aborts every `inspect` if any
  single entity is unparseable. Per design (validation scoped to validate/SL-048),
  but a real fragility. IMP-036 owns the harden: skip/note a malformed sibling,
  hard-fail only the queried id, optional `--strict`.
- **F-2 → fix-now (`6eb5796`):** pre-existing pre-canonical-ref bare-int relation
  data (SL-003 `supersedes=[2]`, ADR-002 `related=[1]`) the scan made fatal. Fixed
  to canonical-ref strings; repaired the latent `slice show 3` / `adr show 2`
  breakages as a by-product. Not a SL-046 defect.

## Dispatch gotchas (this drive was run via /dispatch, serial)

- **Shared-target false-green:** the jail shares `CARGO_TARGET_DIR` across
  worktrees, so a worker's first `just check` can read a stale compile (green
  without running new tests). `touch` the edited files + re-run; confirm the new
  test names actually executed. Recorded to memory.
- **3-way import onto a moving shared `main`:** `git diff B..S | git apply --3way
  --index` imports the worker's net diff onto a HEAD that moved under you; stage
  only the delta (`--index`) and commit WITHOUT `-a` so foreign untracked/dirty
  files never get swept in. Re-capture `B = git rev-parse HEAD` immediately
  pre-spawn; fork the worker from explicit `B`, never session HEAD. Recorded to
  memory.
