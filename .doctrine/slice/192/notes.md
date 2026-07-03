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
