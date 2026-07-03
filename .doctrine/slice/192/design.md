# SL-192 design — Cascade trait-set selection

Conformance fix: make the delivered prompt-cascade engine (SL-186) match
SPEC-023's forward-intent — a **set-valued** model axis (agent as a set of trait
points) with **conjunctive selector pattern-sets** (intersection targeting) and
**root-wise normalized** model specificity. Bounded change in `src/hymns.rs`,
`src/commands/prompt.rs`, `src/install.rs`. Implements SPEC-023 FR-004, FR-005,
FR-007 (+ the FR-009 CLI-arity delta). Precursor to SL-191.

Upstream is locked: SPEC-023 D2 (set-valued context + conjunctive selector
pattern-set), D3 (root-wise `(root, depth)` specificity), D4 (conjunctive-only
axiom), D6 (bands closed, trait trees open); RFC-013 §2–§5. This design does not
re-open the algebra; it specifies the engine encoding.

## 1. Current vs target behaviour

| Surface | Delivered (SL-186) | Target (this slice) |
|---|---|---|
| `ContextVector.model` | `Option<String>` — one point | `BTreeSet<String>` — a set of points (membership) |
| `Selector.model` | `Option<String>` — one pattern | `BTreeSet<String>` — conjunctive pattern-set (empty = unpinned) |
| model match | `model_matches(pat, Option<&str>)` | membership (ctx side) + conjunction (selector side) |
| model specificity | `model_depth(pat): u32` folded into `(primary, Σ other)` | root-wise `(root, depth)` pairs as the primary component |
| `--model` CLI | single | repeatable |
| sidecar `model` | `Option<String>` | list |

Existing single-model behaviour degenerates to the singleton case: the resolver
produces byte-identical output for a singleton context/selector. Note the
**source-level** cost this hides — every delivered test that constructs
`Selector { model: Some(..) / None, .. }` or `ContextVector { model: .. }` (≈12
sites in `src/hymns.rs` + `src/commands/prompt.rs`) gets a *mechanical* migration
to `BTreeSet::from([..])` / `BTreeSet::new()`. Behaviour and assertions are
preserved; only the `explain` byte-golden (the `Spec` printed form) changes
intentionally. "Green unchanged" applies to *outcomes*, not to the struct-literal
call sites, which the type change forces.

## 2. Decisions

- **D1 — Set-valued both axes; empty = unpinned.** `ContextVector.model` and
  `Selector.model` are both `BTreeSet<String>`. An empty `Selector.model` is the
  exact `None`-equivalent don't-care — the unpinned path stays byte-identical to
  today. No `Option` wrapper; the empty set is the degenerate. (SPEC-023 D2 /
  FR-004 / FR-005.)

- **D2 — Two composing match modes.** *Membership* (context side): a single
  model pattern matches if it prefix-matches **any** member of the context set —
  this fires all of an agent's declared trait guidance in one resolve.
  *Conjunction* (selector side): every pinned pattern in `Selector.model` must
  match some context member — this targets trait *intersections* ("smart AND
  loose") without grammar-OR. Each pinned pattern is still a plain prefix pattern
  with `_default` as per-level wildcard; the selector remains a conjunction, so
  the ordering algebra is untouched.

- **D3 — Specificity generalizes only the primary component.** Keep the
  delivered `(primary, Σ other)` two-part shape. The primary component becomes a
  `Vec<(String, u32)>` of `(root, depth)` pairs sorted by root, so
  `Vec: Ord`'s lexicographic-with-prefix-tiebreak *is* the root-wise ordering.
  For non-model single-token bands the vec degenerates to one `("", 0|1)` pair
  (orders by depth as before); preamble/project → empty vec. Model as a
  **secondary** axis stays a scalar in `Σ other` (`Σ` of its pattern depths;
  singleton = today's `model_depth`) — the spec's root-wise clause governs the
  namesake-axis comparison only. Specificity remains a **context-free** function
  of the selector alone; the precedence key stays a total order. (SPEC-023 D3 /
  FR-007.)

- **D4 — CLI/loader are mechanical.** `--model` becomes repeatable
  (`Vec<String>` → `BTreeSet`). The sidecar `model` field becomes
  `Option<Vec<String>>` — the `Option` is load-bearing: it preserves the
  presence semantics the delivered `Option<String>` had. `None` (field omitted) =
  **not declared**, so a model-band snippet keeps its path-derived singleton pin;
  `Some([])` = explicit unpin (don't-care); `Some([p, ..])` = replace with the
  conjunctive set. A bare `Vec` with `#[serde(default)]` cannot distinguish
  "omitted" from "empty" and would silently clear every path pin — rejected. No
  existing sidecar pins `model`, so no migration needed. (SPEC-023 FR-009 delta.)

## 3. Proposed design

### 3.1 Types (`src/hymns.rs`)

```rust
struct Selector {
    harness: Option<String>,
    model: BTreeSet<String>,   // conjunctive pattern-set; empty = don't-care
    role: Option<Role>,
    arm: Option<Arm>,
    stage: Option<String>,
    replaces: Option<Slot>,
}

struct ContextVector {
    role: Role,
    harness: Option<String>,
    model: BTreeSet<String>,   // the agent's declared trait keys (a set of points)
    arm: Option<Arm>,
    stage: Option<String>,
    bands: BandFilter,
}
```

### 3.2 Matching

```rust
/// Membership: does a single model pattern prefix-match ANY context member?
/// (existing left-to-right segment logic with `_default` per-level wildcard,
/// applied per member).
fn model_pattern_matches(pat: &str, ctx: &BTreeSet<String>) -> bool {
    ctx.iter().any(|key| segments_prefix_match(pat, key))
}

// inside matches(): the model axis is a conjunction over pinned patterns —
// every pinned pattern must land on some context member.
if !sel.model.is_empty()
    && !sel.model.iter().all(|p| model_pattern_matches(p, &ctx.model))
{
    return false;
}
```

`segments_prefix_match` is the delivered `model_matches` body (pattern segments
are a left-to-right prefix of the key; `_default` matches any segment; a pattern
longer than the key cannot match), lifted to take a single key. An empty
`sel.model` skips the axis (don't-care). A singleton `sel.model` over a singleton
`ctx.model` reproduces today exactly.

### 3.3 Specificity

```rust
type Spec = (Vec<(String, u32)>, u32);   // (primary root-wise pairs, Σ other scalar)

/// `(root, depth)` for one model pattern: root = first segment (literal,
/// including a leading `_default`); depth = non-`_default` segment count.
fn model_root_pair(pat: &str) -> (String, u32) {
    let root = pat.split('/').next().unwrap_or("").to_string();
    (root, model_depth(pat))
}

/// The model axis as an ordered sequence of `(root, depth)` pairs, sorted by
/// root then depth — the canonical form compared lexicographically across
/// selectors.
fn model_pairs(set: &BTreeSet<String>) -> Vec<(String, u32)> {
    let mut v: Vec<(String, u32)> = set.iter().map(|p| model_root_pair(p)).collect();
    v.sort();
    v
}

fn specificity(band: Band, sel: &Selector) -> Spec {
    let primary: Vec<(String, u32)> = match band.primary_axis() {
        Some(Axis::Model) => model_pairs(&sel.model),
        Some(ax) => vec![(String::new(), sel.depth_of(ax))],   // single-token: 0|1
        None => vec![],                                        // preamble/project
    };
    let other: u32 = ALL_AXES
        .iter()
        .filter(|&&ax| Some(ax) != band.primary_axis())
        .map(|&ax| sel.depth_of(ax))
        .sum();
    (primary, other)
}
```

`depth_of(Axis::Model)` generalizes to `sel.model.iter().map(|p| model_depth(p)).sum()`
(the secondary-axis scalar). It is used only in the `Σ other` sum; the model-band
primary uses `model_pairs`, never `depth_of`.

**Why an ordered sequence (not a root-keyed map).** SPEC-023 D3 mandates
"compared as an ordered sequence of `(root, depth)` pairs" — a sequence, sorted by
root, *not* one entry per root. This is load-bearing for a real case:
`capability/code/high ∧ capability/reasoning/high` is a legitimate same-root
intersection (RFC-013's `capability/code/*` and `capability/reasoning/*` are
distinct sub-trees). It must outrank its factor `capability/code/high`. As a
sequence `[(capability,3),(capability,3)]` vs the factor `[(capability,3)]`, the
prefix rule delivers exactly that. Collapsing same-root pins to one entry would
flatten the intersection to equal its factor — wrong. So the primary is a raw
multiset sorted by `(root, depth)`.

### 3.4 CLI + loader

- `src/commands/prompt.rs`: `--model` → `#[arg(long)] model: Vec<String>` on
  `resolve` and `explain`; `build_ctx` takes `Vec<String>` → `BTreeSet`. The
  `explain` line renders the primary pair-vec instead of `spec=(u32,u32)`.
- `src/install.rs`: `Sidecar.model: Option<Vec<String>>` (`#[serde(default)]` →
  `None` when omitted); `default_selector` for `Band::Model` seeds a singleton set
  from the path label; `overlay_selector` overrides the set **only when the
  sidecar declares the field** (`Some(list)` → replace; `Some([])` → unpin;
  `None` → keep path pin) — preserving the delivered `if let Some(..)` presence
  semantics.

The `explain` render of the primary pair-vec is a **cosmetic** detail (byte-golden
only, no algebra) — its exact string is pinned at execution, not an open design
question.

## 4. Invariants & edge cases

- **INV — specificity is context-free.** `specificity` reads the selector only,
  never the context set. Load-bearing: it is why the precedence key stays a total
  order that `replaces`, seal, and last-word rest on. (SL-186 INV-2/3/6 stand.)
- **INV — empty `Selector.model` ≡ unpinned.** Identical resolution to the
  delivered `None` don't-care.
- **INV — intersection outranks a factor only when the factor is a prefix.**
  A two-root intersection outranks a factor that is a prefix of its sorted pair
  sequence (`adherence/low` < `adherence/low ∧ capability/code/high`). Across
  **different** roots, ordering falls to root-name alpha, which can rank a
  two-root intersection **below** a one-root factor on an alpha-earlier root
  (`adherence/low ∧ capability/code/high` sorts **below** bare
  `capability/code/high`, because leading `adherence` < `capability` decides it
  before length). This is a direct, intended consequence of D3's mandated
  *lexicographic `(root, depth)` sequence* mechanism. The SPEC-023 D3 **body**
  states it precisely — *"more pinned roots at an equal shared prefix → the
  intersection is more specific ... different roots → ordering falls to root-name
  alpha — arbitrary but deterministic"*. The FR-007 one-line summary
  ("intersections outrank their factors") is the compressed form and reads
  unqualified in isolation; under a lexicographic order, unqualified
  intersection-dominance and cross-tree alpha-stability cannot both hold, and D3
  resolves the tension by scoping the guarantee to a shared prefix. This design
  follows the D3 mechanism. Still a total order, deterministic, and stable under
  taxonomy deepening. Accepted as-is. (The FR-007 summary wording is a
  spec-clarity nit, not a design defect — surfaced for the open SPEC-023
  inquisition.)
- **Edge — leading `_default` pattern** (`_default/foo`): root = literal
  `_default`, depth = 1 (non-`_default` count). Legal, rare; sorts under root
  `"_default"` (`_` < ASCII letters). Deterministic.
- **Edge — same-root multi-pins.** Two shapes, both engine-legal and
  deterministic:
  - *Legitimate distinct-subtree intersection* (`capability/code/high ∧
    capability/reasoning/high`): a real target (two sub-trees of one root).
    Sequence `[(capability,3),(capability,3)]` correctly outranks either factor
    by the prefix rule. Supported, not flagged.
  - *Redundant-nested* (`adherence ∧ adherence/low`) or *contradictory*
    (`adherence/low ∧ adherence/high`) pins: authoring anti-patterns. Matching
    is still the full-set conjunction (a redundant broader pin is always
    satisfied when the narrower is; a contradictory pair matches only an agent
    whose set carries both). Specificity is deterministic but can be
    counter-intuitive (a redundant broader pin sorts the selector *below* the
    narrower one alone). Not special-cased in the engine; the "usually a mistake"
    diagnostic is `prompt check` territory → routed to SL-191 (SPEC-023 OQ-3).
- **Edge — different root counts** across selectors: handled by `Vec: Ord`'s
  prefix rule (a shorter sequence that is a prefix sorts less → fewer roots less
  specific). Total and stable.

## 5. Verification

### Engine goldens (`src/hymns.rs` tests)
- **Membership:** multi-key context, single-pattern selector — the selector fires
  when any context member matches.
- **Intersection:** multi-pattern selector matches only an agent whose set
  carries **all** pinned patterns; misses an agent carrying a proper subset.
- **Specificity table:** same-root deeper wins; prefix-intersection outranks its
  factor; cross-root alpha ordering; the boundary case (two-root intersection
  below a one-root alpha-earlier factor); stability under deepening (deepening one
  tree does not flip a cross-tree pair).
- **Singleton degeneration:** existing resolver goldens pass with singleton sets.

### E2E (`tests/e2e_prompt_resolve_golden.rs`)
- `prompt resolve --role worker --model adherence/low --model capability/code/high`
  composes both trait snippets.
- `prompt explain` traces the multi-key match and renders the pair-vec primary.

### Behaviour-preservation
- Existing resolver/loader/e2e suites green with singleton contexts. Any golden
  churn is confined to the `Spec` printed form in `explain` (the one test that
  byte-asserts `spec=(…)`), reviewed by intent.

## 6. Code impact (design-target touch-set)

- `src/hymns.rs` — `Selector.model` / `ContextVector.model` set-valued;
  `model_pattern_matches` (membership) + `matches` conjunction; `Spec` type +
  `model_pairs` / `model_root_pair`; `specificity` + `depth_of(Model)`
  generalization; `precedence_key` type follows.
- `src/commands/prompt.rs` — repeatable `--model`; `build_ctx` set-valued;
  `explain` render.
- `src/install.rs` — `Sidecar.model: Option<Vec<String>>`; `default_selector` /
  `overlay_selector` set construction (presence-preserving).
- `tests/e2e_prompt_resolve_golden.rs` — multi-key compose + `explain` trace
  goldens.
- Delivered unit tests in `src/hymns.rs` + `src/commands/prompt.rs` — mechanical
  `Option<String>` → `BTreeSet` migration of ≈12 struct-literal construction
  sites (outcomes unchanged; `explain` byte-golden updated by intent).

## 7. Non-goals & follow-ups

Per slice scope: no hymn content (SL-191), no new bands, no grammar-OR, no
classification/mapping machinery, no delivery changes beyond CLI arity. The
required-trait `prompt check` lint (SPEC-023 OQ-3) is routed to SL-191, where def
trait declarations give it something to check against.

Tracked follow-ups (not this slice):
- **Onboard `--model` copy (SL-187 delivery surface).** `doctrine_onboard`'s
  model-band guidance (`PROMPT_RESOLVE_MODEL_CMD` in `src/mcp_server/tools.rs`,
  `"… --model <id>"`) is phrased single-valued. Once `--model` is repeatable it
  *understates* the trait-set contract but is not wrong — the engine change does
  not touch it (non-goal: onboard wiring is SL-187's). Update the agent-facing
  usage copy as an SL-187/delivery follow-up. `prompt model-keys` needs no change:
  it reflects model-band authored labels (the taxonomy), which is exactly its
  contract.
- **SPEC-023 FR-007 summary wording** vs the D3 body (see the boundary INV in §4)
  — a spec-clarity nit for the open SPEC-023 inquisition, not a code change here.

## 8. Adversarial review record

Internal pass + external codex (GPT-5.5, read-only, pre-plan) pass run before
lock. Dispositions:

- **[fixed] Sidecar presence.** `Sidecar.model` is `Option<Vec<String>>`, not a
  bare defaulted `Vec` — preserves omitted-vs-empty (D4, §3.4).
- **[fixed] Singleton-degeneration wording.** Behaviour preserved, but ≈12 test
  construction sites take a mechanical `Option`→set migration; only the `explain`
  byte-golden changes (§1, §5, §6).
- **[clarified, no change] Same-root multi-pins.** The reviewer proposed
  rejecting/collapsing duplicate roots; that would break the legitimate
  `capability/code/high ∧ capability/reasoning/high` distinct-subtree
  intersection, which the mandated ordered-sequence form (SPEC-023 D3) correctly
  ranks above its factors. Design keeps the sequence; anti-pattern same-root pins
  are lint-deferred to SL-191 (§3.3, §4).
- **[defended] Boundary vs FR-007.** The cross-root alpha boundary is the D3
  body's explicit rule; the FR-007 summary is its compressed form. Conformant to
  the locked mechanism; wording nit surfaced upstream (§4, §7).
- **[follow-up] Onboard copy / model-keys.** Fenced non-goal (SL-187); tracked
  (§7).
