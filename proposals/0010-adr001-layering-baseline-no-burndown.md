---
seq: 0010
scope: codebase
target: ADR-001 layering baseline (.doctrine/adr/001/layering.toml, tests/architecture_layering.rs)
confidence: med
reversible: yes (proposal only; analysis read-only; adjacent to an accepted GO decision)
---
## What
ADR-001 layering (leaf ← engine ← command, no cycles) is enforced by a real
whole-crate fitness test (`tests/architecture_layering.rs`, syn-AST edge
extraction, SL-112). The gate's verdict is **GO** with a baseline:
**10 accepted upward violations** pinned in `.doctrine/adr/001/layering.toml`, and a
**command tangle of 120 cyclic edges**. Crucially, the gate is explicitly
**"ratcheted (may not grow), not resolved"** (`tests/architecture_layering.rs:22`).

That ratchet is good engineering — it stops decay. But there is **no burn-down
path**: a `backlog list` scan for layering/ratchet/burn-down/leaf finds nothing.
A ratchet with no shrink plan is a permanent debt floor — the 10 violations and the
120-edge SCC are frozen forever by default, and "may not grow" silently becomes
"never shrinks."

The lowest-hanging fruit is named in the gate's own header
(`tests/architecture_layering.rs:15-16`): **`coverage*→requirement` (6 of the 10
edges)** — "requirement is the entity-kind command module; coverage engine modules
import its **types** — a classic ADR-001 wart." This is the textbook extract-a-leaf
case, and the project already has the pattern in spades: the leaf tier is **23
modules**, several of them pure type-only leaves (`kinds` = "entity-kind vocabulary
— pure constants", `dep_seq` = "dependency-sequence data types — pure", `estimate`,
`plan`, `projection`). Pulling `requirement`'s shared *types* into a `requirement_*`
leaf (mirroring `kinds`/`dep_seq`) lets the coverage engine import the types without
reaching up into the command module — clearing **6 of 10 violations (60%)** with a
mechanical, pattern-conformant move and **zero behaviour change** (the
behaviour-preservation gate: existing suites stay green).

## Options
1. **Keep ratcheting (status quo).** Tradeoff: zero effort, decay already
   prevented; accepts a permanent 10-violation / 120-edge floor and the "never
   shrinks" drift.
2. **Targeted burn-down of the coverage→requirement wart.** Extract requirement's
   shared types to a leaf, drop those 6 baseline entries. Tradeoff: high ratio
   (−60% of upward violations) for a mechanical leaf-extraction on a well-worn
   pattern; leaves the 120-edge command SCC and the other 4 warts (state→install,
   backlog_order→backlog, supersede→knowledge, dtoml→verify) for later.
3. **Full burn-down program** (the 4 remaining warts + chip at the command SCC).
   Tradeoff: maximal architectural cleanliness, but the command-tangle is "expected
   for a ~26-module CLI" and fighting it has steeply diminishing returns — likely
   not worth it.

## Recommendation
Option 2: a single targeted burn-down of the coverage→requirement wart, captured as
a chore. It is the only one of the 10 that (a) is a pure type-coupling, (b) maps
1:1 onto the dominant leaf-extraction pattern, and (c) clears the majority of the
baseline in one move. Explicitly NOT Option 3 — the command SCC is a known,
reasoned tradeoff and chasing it is low-value. The deferred meta-question is whether
the ratchet should ever shrink at all; if the answer is "no, the baseline is the
permanent contract," that is a legitimate choice — but it should be a *decision*,
not the default-by-absence it currently is.

Decisions deferred to YOU:
- (a) **burn down or keep ratcheting** — is the baseline a permanent contract, or a
  debt floor with an intended (if unscheduled) shrink?
- (b) if burning down, **scope** — just the coverage→requirement wart (Option 2),
  or the four type-coupling warts together (they share the extract-types-to-leaf
  shape).
- (c) **leaf shape** — a new `requirement_types` leaf vs. folding the shared types
  into an existing leaf (`kinds`?) — a cohesion call.

## Next doctrine move
```
# read the baseline + the wart (read-only):
sed -n '1,25p' tests/architecture_layering.rs       # the GO verdict + wart list
cat .doctrine/adr/001/layering.toml                  # the 10 baselined edges
grep -rn 'crate::requirement' src/coverage*.rs       # the 6 offending imports

# capture the burn-down (NOT executed — fence forbids backlog transition):
doctrine backlog new chore "Burn down ADR-001 coverage→requirement layering wart \
  (6 of 10 baselined edges): extract requirement shared types to a leaf (mirror \
  kinds/dep_seq), drop the baseline entries — behaviour-preserving" \
  --tag area:coverage --tag area:requirement --tag architecture
```
(Verbs described, NOT executed.)

## Illustration (optional)
None — the move is a known refactor shape (extract pure types to a leaf, already
done 23×); the value is identifying it as the 60%-reduction lever and surfacing the
absent burn-down decision, not a diff.
