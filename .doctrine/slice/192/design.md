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

Existing single-model behaviour degenerates to the singleton case: current
goldens pass with singleton sets, modulo the `Spec` type's printed form.

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
  (`Vec<String>` → `BTreeSet`); the sidecar `model` field becomes a TOML list.
  No existing sidecar pins `model`, so the list carries zero back-compat cost.
  (SPEC-023 FR-009 delta.)

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

### 3.4 CLI + loader

- `src/commands/prompt.rs`: `--model` → `#[arg(long)] model: Vec<String>` on
  `resolve` and `explain`; `build_ctx` takes `Vec<String>` → `BTreeSet`. The
  `explain` line renders the primary pair-vec instead of `spec=(u32,u32)`.
- `src/install.rs`: `Sidecar.model: Vec<String>` (`#[serde(default)]`);
  `default_selector` for `Band::Model` seeds a singleton set from the path label;
  `overlay_selector` replaces the set when the sidecar declares one.

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
  before length). This is a direct, intended consequence of D3's root-alpha
  cross-tree rule (SPEC-023: *"more pinned roots at an equal shared prefix →
  the intersection is more specific"* — the guarantee is scoped to a shared
  prefix). Still a total order, deterministic, and stable under taxonomy
  deepening. Accepted as-is.
- **Edge — leading `_default` pattern** (`_default/foo`): root = literal
  `_default`, depth = 1 (non-`_default` count). Legal, rare; sorts under root
  `"_default"` (`_` < ASCII letters). Deterministic.
- **Edge — duplicate-root pins** (`adherence/low ∧ adherence/high`): legal at
  the engine level (satisfiable iff the agent's set carries both members); not
  special-cased. Sorted pairs `[(adherence,2),(adherence,2)]` order
  deterministically. Any "usually a mistake" diagnostic is `prompt check`
  territory → routed to SL-191 (SPEC-023 OQ-3).
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
- `src/install.rs` — `Sidecar.model: Vec<String>`; `default_selector` /
  `overlay_selector` set construction.
- `tests/e2e_prompt_resolve_golden.rs` — multi-key compose + `explain` trace
  goldens.

## 7. Non-goals

Per slice scope: no hymn content (SL-191), no new bands, no grammar-OR, no
classification/mapping machinery, no delivery changes beyond CLI arity. The
required-trait `prompt check` lint (SPEC-023 OQ-3) is routed to SL-191, where def
trait declarations give it something to check against.
