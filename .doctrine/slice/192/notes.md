# Notes SL-192: Cascade trait-set selection

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 — complete (green, committed on fork, not yet landed)

Executed solo in worktree fork `w/SL-192-p01` (base `4c43b4d0`). Pure-engine
change per design §3/§4; no algebra reopened, no `/consult` needed.

**Delivered (EX-1..5, VT-1/2/3, VA-1 all met):**
- `ContextVector.model` / `Selector.model` → `BTreeSet<String>` (empty = prior
  `None` don't-care).
- Match: `segments_prefix_match` (single key) → `model_pattern_matches` (membership,
  ctx side) + `matches` model arm = conjunction (every pinned pattern hits some
  member; intersection targeting, D2).
- Specificity: `Spec = (Vec<(String,u32)>, u32)`; primary = `model_pairs` (raw
  sorted `(root,depth)` multiset, NOT root-collapsed — so distinct-subtree
  intersection outranks its factor); single-token band → `[("",0|1)]`; bandless →
  `[]`. `depth_of(Model)` = Σ pattern depths (Σ-other scalar only). Context-free.
  `PrecedenceKey`/`precedence_key` follow (D3).
- Shell kept single-valued this phase: `build_ctx` / `default_selector` /
  `overlay_selector` wrap single input → singleton set.

**Accepted boundary (design §4, INV):** two-root intersection can sort BELOW a
one-root alpha-earlier factor (`adherence/low ∧ capability/code/high` <
`capability/code/high`). Encoded + asserted in VT-3, not "fixed" — D3's mandated
lexicographic `(root,depth)` order.

**Only intended output change:** the `explain`/`Spec` byte-form, now
`spec=([root:depth,…],other)` (e.g. `[anthropic:2]`, bandless `[]`, single-token
`[:1]`). No e2e golden pinned the old form, so none was silently rewritten (VA-1).
PHASE-02 adds the e2e explain golden.

**Commits (fork):** `e8470d64` feat (src: hymns/prompt/install); `fc4eee72` chore
(lifecycle ready→started). Gate (`doctrine check gate`) exit 0 after last edit.
Conformance boundary (`code_start`=4c43b4d0, `code_end`=e8470d64) in runtime state.

**Next (PHASE-02, same fork — phases not file-disjoint):** repeatable `--model`
(`Vec<String>`→set in `build_ctx`); `Sidecar.model: Option<Vec<String>>`
(load-bearing Option: None=keep path pin, Some([])=unpin, Some(list)=replace);
e2e goldens over shipped snippets (`anthropic/claude-sonnet-4`, `deepseek/_default`).
Land the fork at slice completion (`worktree land`, `--no-ff`, preserve TDD
history) — never per-phase.

## PHASE-02 — complete (green, committed on fork, not yet landed)

Executed solo on the SAME fork `w/SL-192-p01`. Shell surface only (ADR-001
command/install layer); engine untouched — it already supported the set input.

**Delivered (EX-1..3, VT-1/2/3 all met):**
- `--model` repeatable on `resolve`+`explain` (`Vec<String>` → `BTreeSet` in
  `build_ctx`): absent = empty set (don't-care), one occurrence = singleton
  (CLI behaviour unchanged), many = the context trait set.
- `Sidecar.model: Option<Vec<String>>` — the load-bearing Option (§8/D4). `None`
  (omitted) keeps the path-derived pin; `Some([])` unpins; `Some(list)` replaces
  with the conjunctive set. `overlay_selector` honours the presence distinction
  (one-line body swap — PHASE-01 left the `if let Some(..)` scaffold in place).
- `default_selector` Model was ALREADY singleton-from-label (delivered PHASE-01);
  EX-2's default half needed no edit.
- VT-2 proves the presence semantics **through serde** (`toml::from_str`), at the
  boundary where a bare `Vec` would have collapsed omitted↔empty.

**Live explain multi-key trace (verified):**
`model/anthropic/claude-sonnet-4 → spec=([anthropic:2],0)`;
`model/deepseek/_default → spec=([deepseek:1],0)` (deepseek depth 1 — `_default`
is the uncounted wildcard segment). Both rank; `role/worker` wins.

**Commit (fork):** `fe6a68b9` feat (prompt.rs, install.rs, e2e golden; +132/-13).
Gate `doctrine check gate` exit 0. Conformance: 4/4 conformant, 0 undeclared/undelivered.

**REV-019 landing-adjacency (assessed disjoint):** REV-019 (exposed-slot override
via self-`replaces`) also edits `install.rs` and amends SPEC-023 suppression/precedence
prose — but it is engine-unchanged and touches the *provenance* term + the
seal/expose projection, NOT the *model axis* (matching/specificity) this slice
delivers. No design/plan impact. Expect only a textual `install.rs` merge when both land.

## Slice status: both phases complete → ready for `/audit`

Fork `w/SL-192-p01` carries 4 commits atop base `4c43b4d0`: `e8470d64` (P01 feat),
`fc4eee72` (lifecycle), `23445fb7` (P01 notes), `fe6a68b9` (P02 feat). NOT yet landed.
Remaining: `doctrine slice status SL-192 audit` → `/audit` → reconcile → close →
`worktree land --fork w/SL-192-p01 --no-ff` → `worktree gc`.
