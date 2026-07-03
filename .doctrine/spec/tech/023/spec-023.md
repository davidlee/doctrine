# SPEC-023: Prompt cascade

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

The prompt cascade is the container that composes per-context agent instruction
from a snippet corpus, CSS-style: snippets declare *where they go* (band) and
*when they apply* (selector); one resolver matches them against a context vector
and concatenates the survivors in precedence order. It exists so that guidance
lives **once** and is *matched*, never copied per combination — and so that
orchestrators stop hand-assembling worker context in prose (ADR-011: mechanism
in a CLI verb, harness-identical).

It sits beneath the whole-system root (SPEC-003), realising PRD-007's
per-context session-instruction intent. It rides the install container
(SPEC-009) for its embedded assets and install-time projection, and is consumed
by the boot snapshot (SPEC-011) and dispatch (SPEC-012/SPEC-021) delivery
surfaces. This spec owns the corpus model, the selection algebra, the
precedence/specificity order, the classification layer, and the `prompt` verb
contract. It does **not** own where output is delivered (boot/dispatch
contracts) nor how an agent comes to be classified into the trait taxonomy (see
§ Boundaries and OQ-1).

Provenance: distilled from RFC-013 (selection algebra, disjunction-via-
classification, trait classification) and the SL-186/SL-187 locked designs.
Postures are mixed and marked: the corpus/precedence/seal machinery below is
**delivered**; the set-valued classification extensions are **forward-intent**
(§ Delivered vs target).

## Responsibilities

Mirrors the structured `responsibilities` list: load the two-root layered
corpus; enforce seal at resolution; match by a purely conjunctive selector
algebra; carry the model axis as a set-valued classification space; order by
the total-order precedence key with root-wise-normalized specificity; suppress
only via validated `replaces`; front the pure engine with the `prompt` verbs;
and hold the delivery/classification boundary.

### The corpus — two roots, one cascade

A **snippet** is one `.md` file of prose. Its **slot** is `<band>/<label>`
(band = first path segment under the corpus root; label = the rest); its
**selector** is a set of axis→pattern constraints, derived from the path and
superseded **per-axis** by an optional sidecar `<file>.toml`. Two roots layer:

```
install/hymns/     compile-embedded (rust_embed), framework-authored, the superset
.doctrine/hymns/   on-disk, user-authored + projected editable starters
```

The loader (the one impurity, in `src/install.rs`) unions both roots at read
time; **provenance is derived from the source root**, never stored. The
**seal set** (embedded, from the install manifest) is enforced at resolution:
a user-provenance snippet whose slot is sealed is dropped *before matching* —
sealed framework content wins by active exclusion of disk twins.

**Bands** are a closed registry in fixed order — position, not identity:
`preamble · harness · model · role · stage · project`. A band decides *where in
the output* a snippet lands; the selector decides *whether it applies*. A
selector may pin **any** axis regardless of the snippet's band (a `role/worker`
snippet may pin `model=adherence/low`); the band never enters matching, only
ordering.

### The selection algebra

Matching is **purely conjunctive across axes**: every pinned axis must match
the context; an unpinned axis is don't-care; non-match is absence, never
suppression. Within the model axis, a pattern prefix-matches the context key
left-to-right with `_default` as a per-level wildcard — so a shallow pattern
matches its whole descendant subtree (a *contiguous-subtree OR*, pre-baked by
tree shape).

The geometry (RFC-013 §2): one selector expresses **(subtree-OR on axis₁) ∧
(subtree-OR on axis₂) ∧ … — an axis-aligned box** in the taxonomy product
space; the corpus is a **union of boxes** (every matching snippet concatenates).
What one selector cannot express: non-sibling OR within an axis, or any
cross-axis OR — those cost extra snippets (boxes), and a well-shaped taxonomy
keeps box-count low.

**Set-valued classification (forward-intent, RFC-013 §3 + F-A).** The context's
model axis carries a **set** of trait keys — the agent is a set of points, one
per declared trait (`adherence/low`, `capability/code/high`, …). Two matching
modes compose:

- **Membership (context side):** a single model pattern matches if it matches
  *any* member of the context set. This is what lets all of an agent's trait
  guidance fire at once (union-of-traits delivery).
- **Intersection (selector side):** a selector may pin a conjunctive **set** of
  model patterns; *every* pinned pattern must match some context key. This is
  what targets trait *intersections* — "smart AND loose only" — without which
  `adherence/low` content cannot be scoped away from dumb-loose agents. Each
  pinned pattern is still a plain prefix pattern; the selector remains a
  conjunction, so the ordering algebra below is untouched.

### Precedence and specificity

Matched snippets order by the total-order key, ascending (last word wins):

```
band → specificity → provenance(framework < user) → alpha(full slot path)
```

Specificity dominates provenance: a framework exact-trait snippet outranks a
user broad one; the user wins only the *same-slot* tiebreak (the legitimate
customisation). Within a band, specificity leads with the **band's namesake
axis**, then the sum of other pinned axis depths — lexicographic
`(band_axis_depth, Σ other)` — so axis-count can never bury an exact primary-
axis match.

**Root-wise normalization (forward-intent, F-B).** Once the model band hosts
orthogonal trait trees, raw depth comparison across trees is a taxonomy-shape
accident (`capability/code/high`, depth 3, would outrank `adherence/low`,
depth 2, for no semantic reason). The model-axis specificity is therefore
compared **root-wise**: a selector's pinned model patterns are keyed by their
top-level segment and compared as an ordered sequence of `(root, depth)` pairs,
lexicographically. Consequences, in order of intent:

- same root → deeper pattern is more specific (unchanged from delivered);
- more pinned roots at an equal shared prefix → the intersection is more
  specific than any of its factors (semantically true: a smaller box);
- different roots → ordering falls to root-name alpha — arbitrary but
  **deterministic and stable under taxonomy deepening**: reshaping or
  deepening one trait tree can no longer flip cross-tree ordering.

Specificity remains a context-free function of the snippet alone, so the key
stays a total order and `replaces`, seal, and last-word all stand.

### Suppression — `replaces`

Concatenation is the rule; `replaces` is the only suppression, and it is legal
**only on the unique most-specific active snippet of its own slot**. Two active
replacers targeting one slot, a non-top replacer, and any cycle are authoring
errors surfaced by `prompt check`/`validate` — never silently alpha-ordered. A
user replacer may suppress framework; a framework replacer can never reach a
user snippet (user is never lower in the order).

### The classification layer

The model band is a **user-definable classification vocabulary**, not a model
registry. Model identity is a leaky proxy for a trait tuple: it does not reuse
to the next loose model, fuses identity with trait, and mis-fires when a
model's real traits diverge from its vendor path. The honest axes are traits —
`adherence/*`, `capability/code/*`, `capability/reasoning/*` — orthogonal
groupings that cannot all be subtrees of one identity tree, which is exactly
why they are separate trees within the band.

- **Per-def declaration, no registry.** A worker def declares its trait-key set
  in frontmatter; the spawner may override or augment at spawn where it knows
  more. There is no central `models.toml` — a per-def trait set is small, slow
  (model-generation cadence), and locally owned: a *different artifact* from
  the churny id→spawn-param list the design fences (SL-186 P4/P6).
- **Adherence is assigned, never self-asserted.** A loose agent is exactly the
  one that will mis-declare its own adherence. Capability self-identification
  is tolerable; adherence classification must come from the def, the spawner,
  or the user's invocation.
- **Taxonomy shape is a design-time commitment.** One tree per trait axis bakes
  one OR-decomposition; shape each tree so the disjunctions you will want are
  subtrees (`adherence/needs-help/{low,med}` buys "low ∨ med" in one box).
- **Keys are opaque to the engine.** The resolver treats every taxonomy key as
  an uninterpreted path; meaning lives entirely in the authored corpus.

### The axiom — disjunction lives in classification, never in selector grammar

Locked invariant (RFC-013 §4): the selector grammar stays conjunctive.
Grounding: **specificity is a total order only over conjunctions** — a
conjunction has a per-axis depth; an OR-formula does not, and that total order
is what `replaces`, seal, and last-word ordering rest on. Grammar-OR would not
merely complicate the parser; it would dissolve the ordering algebra.

Disjunction therefore lives where it cannot corrupt ordering: in tree shape
(subtree-OR) and set membership (the context's trait set). The constraint is
generative — a new OR-need means *give that axis a declarable, set-valued
classification and shape its tree so the wanted disjuncts share a parent*.
Known fray edges (stress-tested, RFC-013 §5): T2 negation over an open axis
(silent-miss on an undeclared trait — see OQ-3); T3/T4 OR over
non-classifiable or cross-owner axes. The pressure valve is DNF-by-duplication:
bounded cost, non-corrupting for prose. Grammar-OR stays out unless a real T4
case with intolerable duplication appears — and the first question even then is
whether the offending axis can be made classifiable.

### The verb contract

- `prompt resolve --role <r> [--harness --model … --arm --stage --band …]` —
  stdout composition for the context; regenerates the **universal** on-disk
  boot artifact axis-invariantly (no flag ever alters the disk artifact);
  idempotent and deterministic. `--model` becomes repeatable with set-valued
  classification (one occurrence per trait key).
- `prompt model-keys [--harness]` — the full relative taxonomy keys that
  *exist in the corpus*, one per line: a reflection of authored vocabulary,
  never an enumeration of models. Empty ⇒ don't ask.
- `prompt explain` — the precedence trace: per slot, which snippets matched,
  who won, why. The cascade's debugger.
- `prompt check` — corpus integrity: sealed slots present and unshadowed,
  selectors parse, sidecars name real bands, `replaces` unique-most-specific.
  The verb is delivered; its *integration into the `doctrine check` cadence*
  (a `[verification]` entry, so `check quick/commit/gate` runs it) is
  forward-intent, not yet wired.

The engine (`src/hymns.rs`) is pure — no disk, clock, or env; the seal set is
passed in — and deterministic: the same `(corpus, context, seal)` yields
byte-identical output. Downstream cache-hold (SPEC-011's session-start
injection) depends on that determinism.

## Concerns

- **Cache split rides the boundary.** Stable model-agnostic content is composed
  into the cache-stable session-start sector; model/trait content rides
  cache-busting paths (`doctrine_onboard`, spawn-time bake). The cascade only
  guarantees determinism; the split itself is the delivery containers' contract
  (SL-187 D1).
- **No correctness invariant rests on the model band.** Trait guidance is
  fine-tuning; a missing or stale trait key degrades gracefully to broader
  `_default`/universal content. Delivery may be best-effort (floor +
  supplement), and staleness is bounded by construction.
- **Double-emit at box intersections.** An agent inside two duplicated boxes
  receives the shared body twice — wasteful, never incorrect. Accepted as the
  duplication valve's cost.
- **Authoring legibility.** The box/union model asks authors to reason about
  taxonomy shape. `prompt explain` is the mitigation: precedence must always be
  reconstructible from a trace, not folklore.

## Hypotheses

- **H1 — Trait vocabulary stays small and slow.** Per-def trait sets change at
  model-generation cadence; the taxonomy is sparse and self-pruning. If the
  vocabulary churns weekly, the classification layer has re-created the fenced
  registry and the split has failed.
- **H2 — Conjunctive-only holds for this domain.** Prose guidance to agents,
  where we own classification. The named fray edges (T2/T3/T4) stay tolerable
  via the duplication valve; a real intolerable T4 falsifies this.
- **H3 — Prefix-OR covers everyday disjunction.** Most wanted ORs are sibling
  groups reachable by shaping the tree; box-count stays low in practice.

## Decisions

- **D1 — Classification over identity on the model axis.** Trait trees
  (`adherence/*`, `capability/*`) are the intended vocabulary; a
  `vendor/name` identity key remains *expressible* (it is just another path)
  but carries no privileged semantics. From RFC-013 position 1.
- **D2 — Set-valued context + conjunctive selector pattern-set (F-A in).**
  Membership matching on the context side delivers union-of-traits;
  a per-selector conjunctive pattern set delivers intersection targeting.
  Both keep every selector a conjunction. Rejected: context-only set-valuing
  (leaves the motivating `{smart × loose}` intersection inexpressible);
  grammar-OR (dissolves the specificity total order).
- **D3 — Root-wise normalized model specificity (F-B).** `(root, depth)` pairs,
  lexicographic; cross-tree ordering is stable under taxonomy deepening;
  intersections outrank their factors. Rejected: raw depth sum (taxonomy-shape
  accident); declaring cross-tree ordering unspecified (precedence must be a
  total order).
- **D4 — Disjunction-via-classification is a locked invariant** (§ axiom).
  ADR-altitude candidate; recorded here until graduated.
- **D5 — Classification happens outside the cascade.** The engine receives an
  already-classified context; def frontmatter, spawner override, and
  user-nominated invocation are the assignment points. The cascade never maps
  a harness's model id to taxonomy keys (see OQ-1).
- **D6 — Bands closed, trait trees open.** Trait categories are labels *within*
  the model band; extending the vocabulary never touches the band registry.

## Open questions

- **OQ-1 — Model-mapping ownership.** Harnesses hold their own model ids (pi:
  `models: deepseek/deepseek-v4-pro`; env markers like `CLAUDE_*` are
  heuristically sniffable). Is there a shared mapping from
  harness-understood / self-identified / env-sniffed model identity back to the
  user's trait taxonomy — and who owns it? Not the cascade (D5); probably not
  solely a harness adapter. Orchestrator invocation is easy (user-nominated);
  the open case is unattended classification of a spawned or resumed agent.
- **OQ-2 — Band naming.** Whether `model` remains the band name for a trait
  space, or is renamed/re-homed, is an authoring-clarity question; the algebra
  is indifferent.
- **OQ-3 — Required-trait lint (T2 mitigation).** A def that forgets to declare
  a trait silently misses its guidance. Candidate: a `prompt check` diagnostic
  warning when a worker def declares no key under a required trait root (e.g.
  no `adherence/*`).

## Delivered vs target

Delivered (SL-186/SL-187, verified in `src/hymns.rs`): two-root corpus, seal
enforcement, conjunctive matching with prefix/`_default` model patterns,
band-primary specificity, `replaces` validation, the four `prompt` verbs
(the engine-facing behaviour; see below for the `check`-cadence caveat),
orchestrator session-start delivery, role-band def bake.

Forward-intent (this spec's target, from RFC-013 positions 4 + F-A/F-B):
set-valued context model axis with membership matching; conjunctive selector
pattern-set (intersection targeting); root-wise normalized model specificity;
repeatable `--model`; and wiring `prompt check` into the `doctrine check`
cadence (the verb runs today, but no `[verification]` entry feeds it into
`check quick/commit/gate`). Framed as SL-186 under-delivering its own
composable-category design; the change lands via a conformance Revision or
precursor slice (sequencing open at SL-191).
