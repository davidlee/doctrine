# SL-022 implementation notes

Durable implementation notes — durable findings/decisions taken during execution.
Runtime task progress lives under `.doctrine/state/` (gitignored); this is the
committed record.

## PHASE-02 — Registry edges, product set & FK / subject-kind checks

**Shape.** `Registry` gained `product_specs: BTreeSet`, `parents: Vec<ParentEdge>`,
`descents: Vec<DescentEdge>` (each edge carries `on_product`). Two new pure HARD
checks `descent_findings` / `parent_findings`, both scope-aware, both 3-way
(invalid-kind on product subject / clean / invalid-kind on wrong-kind target /
dangling). `dangling_interaction_targets` rewritten to split invalid-kind (product
target) from dangling. `validate` extends both new checks; signature unchanged.

**Behaviour change (disclosed, not a gate breach).** REQ-084 / PRD-012 §6: an
interaction target that is a product spec is now *invalid kind*, not *dangling*.
The SL-015 test `non_tech_interaction_target_is_flagged_tech_only` asserted the old
contract → rewritten as `product_interaction_target_is_invalid_kind_not_dangling`.
This is the ONE intended behaviour change in an otherwise behaviour-preserving
phase. Carry to audit.

**Charge I — new fallible parse + widened error surface.** `build_registry` now
parses each `spec-NNN.toml` (it parsed none before — only `members.toml` /
`interactions.toml`). Consequence: a malformed `spec-NNN.toml` that `validate`
never opened before now fails the build. Intended widening, proven by Layer C
`build_registry_surfaces_a_malformed_spec_toml`, NOT assumed from the hand-built
unit suites (they bypass this seam). Both arms parse + harvest both tech-only
fields so a product carrying one is seen and flagged, not dropped (codex F5b).

**DRY.** Added `canonicalize_spec_ref` (free fn in `spec.rs`) — the single
canonicalisation path for every outbound spec→spec ref; the interaction-target
block now reuses it too.

**Seam left for PHASE-03 (clean, additive).** PHASE-02's parse `Err` arm just
propagates `"Failed to parse"` via `?`. PHASE-03 inserts the `second_parent`
classification branch *before* the `?` and adds the `build_findings` carrier —
no PHASE-02 rework required.

**Existing-test edits (mechanical, non-behavioural).** Only the `clean()`
`Registry` literal gained `..Default::default()` (covers PHASE-02's three fields
AND PHASE-03's future `build_findings` in one edit). No existing assertion changed
value, except the deliberate REQ-084 test rewrite above.

**Lint.** `map(..).unwrap_or_else(..)` on a `Result` trips `clippy::map_unwrap_or`
(pedantic) — use `map_or_else(default, f)`.

## PHASE-03 — Decomposition integrity (REQ-087)

**Shape.** Three new HARD checks + a scan-time carrier:
- `self_parent(scope)` (registry.rs) — sole reporter of A→A (tech subject).
- `parent_cycle(scope)` (registry.rs) — ephemeral child→parent `BTreeMap`
  (skips `on_product` + self-loops), walked from each node with an **ordered
  `Vec` path + first-seen index `BTreeMap`**. On revisit, the cycle SLICE =
  `path.get(first..)`; emit ONE finding only when `start == slice.min()` (least
  id). Dedups correctly for a tail feeding a ring (`T→A→B→A`: slice `{A,B}`, only
  A's walk emits). Terminates at root / dangling parent.
- `BuildFinding { spec, message }` + `Registry.build_findings` (codex F1 carrier).
  `validate(scope)` aggregates it scope-filtered (by `spec`), alongside the two
  new pure checks.

**Second-parent = parse-error classifier, NOT a line-scan (codex F2).**
`build_registry`'s `toml::from_str::<Spec>` Err arm is now a `match` with a guard
`Err(e) if is_second_parent(&e, &spec_text)` → push a named `BuildFinding`,
`continue`; any other Err still propagates `"Failed to parse"` via `?`. The
classifier (`is_second_parent`, spec.rs) attributes the error to the `parent` key
via the **error span's enclosing source line** (`enclosing_line`), then confirms
the shape by message text. Span attribution is the F2 guarantee made structural:
the parser already ignored comments, so a scaffold's `# parent = …` can NEVER be
the span — proven by `scaffold_commented_parent_does_not_trip_second_parent`.

**R2 — the toml-error match is version-fragile (toml 0.8.23).** No stable
error-kind enum. The match is: span's enclosing line key == `"parent"` AND
`message().contains("duplicate key")` (dup) OR `"invalid type: sequence"` (array).
Observed shapes (pinned by `second_parent_classifier_*`): dup →
``duplicate key `parent` in document root``; array → `invalid type: sequence,
expected a string` (note: the array message does NOT name the key — span
attribution is REQUIRED, message alone would false-hit `slug = []`). A plain
wrong-type (`parent = 5`) intentionally falls through to `"Failed to parse"` —
not a "second parent". On any match miss: degraded message, still non-zero exit,
never a silent pass (R2 mitigation holds).

**Scope handling for cycles (decision, runtime-sheet D-cycle-scope).**
`parent_cycle` always builds the corpus map and dedups by least-id; when `scope`
is `Some` it keeps only cycles whose slice contains the scope node. Keeps corpus
dedup correct while a scoped run still reports a cycle the scoped spec is in.

**Lint.** `clippy::indexing-slicing` is DENY here — `&path[first..]` and string
slicing (`enclosing_line`) both tripped it. Use `.get(range).unwrap_or_default()`
/ `.get(..).unwrap_or("")`.

**Existing-test edits.** NONE needed beyond PHASE-02's `..Default::default()` in
`clean()` (it already absorbed `build_findings`, codex F6b). No assertion changed.

**Scope left for PHASE-04.** PHASE-03 proved second-parent end-to-end (VT-2). The
self-parent / cycle / FK / subject-kind cases ride the existing `run_validate`
non-zero bail; the full crafted-corpus CLI sweep over every violation is PHASE-04.

## PHASE-04 — Cross-cutting validation sweep & closure

**Stale stub corrected.** The runtime sheet stub described a dropped model
("severity tier", `descent_on_product` warn, a `warnings()` sibling). Codex F5
DROPPED the tier (design §5.2, D5, §10 F5): `validate` is hard-only, signature
unchanged; a tech-only field on a product is a HARD invalid-kind finding. Sheet
rewritten before execution. **No production code this phase** — every check landed
PHASE-02/03; PHASE-04 is the end-to-end gate + closure.

**Shape — `run_validate`-level sweep, NOT a spawned binary.** Design §9: no spec
e2e harness exists. The sweep drives `run_validate(Some(root), None)` (the function
backing `doctrine spec validate`) over crafted temp corpora — the exact level
PHASE-03 proved second-parent at. A parametrized helper
`assert_validate_flags(build, expect_substr)` asserts BOTH the non-zero exit AND
the specific finding text (proves the RIGHT check fired, not merely that some error
did). One corpus PER violation — VT-1 reads "each crafted hard violation →
non-zero", so per-violation granularity attributes each exit to its check (a
mega-corpus only proves the aggregate). 11 cases: descent ×3 (dangling / tech-target
invalid-kind / product-subject), parent ×3 (dangling / product-target invalid-kind
/ product-subject), self-parent, cycle, interaction ×2 (dangling / product-target),
clean→zero. Second-parent referenced from PHASE-03, not re-proven.

**Integration confirmed (A2 STOP never fired).** Every violation independently
trips `run_validate`'s bail — PHASE-03's `validate` aggregation reaches every check.
The sweep's overlap with the Layer A pure-check tests is INTENDED: Layer A proves
the check; the sweep proves the same violation rides the CLI exit (the integration
the hand-built-registry unit tests bypass).

**VA-1 (rust-embed re-embed).** Fresh `doctrine spec new tech` scaffold emits the
`# descends_from` / `# parent` comment lines after the normal recompile — footgun
cleared, no `cargo clean` needed.

**VA-2 (REQ-082 AC3, satisfied by construction).** `Spec.descends_from` /
`Spec.parent` are `Option<String>` — id-only, no prose field to restate product
intent. Review check, not a code gate.

**EX-3 storage rule.** No derived data (children, reverse view) persisted; the
cycle inversion is ephemeral inside `parent_cycle`. Tests-only phase, no disk
writes added.
