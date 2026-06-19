---
seq: 0003
scope: codebase
target: src/relation_graph.rs, `doctrine inspect` (src/main.rs:3187)
confidence: med
reversible: yes (proposal only; no code written — analysis is read-only)
---
## What
Doctrine has three relation-walking surfaces, and they disagree on reach. The
cross-kind authored relation graph — the strategic asset — is the shallowest:

- `doctrine inspect <ID>` renders **1-hop only**: `outbound` / `inbound` /
  `danglers` (`src/relation_graph.rs:646` `render_human`, `:674` `render_outbound`,
  `:702` `render_inbound`). `run_inspect` (`src/main.rs:3187`) takes no depth/
  transitive parameter at all.
- `doctrine blockers <ID> --transitive` **does** walk N-hop, but only over the
  dep/seq (`needs`/`after`) overlay — "walks both chains via `reachable`"
  (`src/priority/surface.rs:394`; `src/priority/mod.rs:74-85`).
- `doctrine retrieve … --expand <N>` **does** walk N-hop — "Expand graph by
  traversing relations N levels deep (retrieve only)" (`src/main.rs:239-241`) — but
  only over the memory relation graph.

So multi-hop traversal is a solved, shipped capability on two surfaces, and the
reachability primitive itself already lives in the shared graph crate
(`crates/cordage/src/query.rs` — `reachable`, flagged for triplication cleanup by
**IMP-020**). `relation_graph` already builds the full cordage `Graph` with one
overlay per relation label (`build_relation_graph_from`, `:219`; `OverlayMap`,
`:141`). The missing piece is purely **exposure**: there is no transitive
impact / blast-radius query over the governance & derivation overlays
(`GovernedBy`, `Implements`, `DescendsFrom`, `Supersedes`, `DecisionRef`).

Concretely, a product dev cannot ask the question the graph topology is built to
answer: *"If I change ADR-005 / REQ-094 / SPEC-001, what transitively depends on
it?"* — the single most load-bearing use of a governance graph for a team. The
data and the walk both exist; only the verb is absent.

## Options
1. **Add `--depth N` (and/or `--transitive`) to `inspect`.** Reuse the cordage
   `reachable` walk over the existing relation overlays; render the closure
   grouped by hop / by label. Tradeoff: smallest surface (one existing command
   grows a flag, mirrors `blockers --transitive` exactly), but conflates "the
   1-hop neighbourhood view" with "the transitive cone" on one verb — and `inspect`
   already concatenates the priority actionability block, so output grows busy.
2. **New first-class `doctrine impact <ID>` verb.** A dedicated upstream/downstream
   transitive-closure query (direction-selectable: what governs X vs. what X
   governs), label-filterable. Tradeoff: clearest mental model and best discovery
   ("impact analysis" is the named feature product teams want), but a new command
   + render path + tests — a slice, not a flag.
3. **Do nothing — direction the user via repeated `inspect`.** Tradeoff: zero cost,
   but the graph's flagship value (transitive impact) stays manual and effectively
   undiscoverable; a 4-hop governance chain is invisible.

## Recommendation
Option 2 (`doctrine impact <ID>`), scoped as a slice, but **gated on IMP-020 first**.
IMP-020 already records that `crates/cordage/src/query.rs` has three *diverged*
reachability walks (`reachable` / `spine_path` / `extend_chains`) that should be
unified. Building a new traversal consumer on top of a known-triplicated primitive
would add a fourth caller to an unstable seam — exactly the "no parallel
implementation" footgun. Sequence: consolidate the cordage walk (IMP-020), then
expose `impact` on the clean primitive. The reuse story is then airtight: one walk,
three consumers (`blockers`, `retrieve --expand`, `impact`).

Why this is the highest-leverage graph move available: every other relation feature
(inspect, blockers, retrieve) answers a *local* or *single-axis* question. Transitive
cross-kind impact is the one query that turns the corpus from a filing system into a
decision-support graph — "this is what your change touches" is the sentence that
makes the tool indispensable to a team.

Decisions deferred to YOU:
- (a) **flag-on-inspect (1) vs. new verb (2)** — surface shape and discoverability
  vs. minimalism.
- (b) **whether to gate on IMP-020** or accept a 4th `reachable` caller now and
  unify later (faster, but grows the debt IMP-020 names).
- (c) **default direction & label set** — does `impact X` mean "what X depends on"
  (downstream) or "what depends on X" (upstream)? For change-impact, upstream
  (dependents) is the intuitive default; worth confirming.
- (d) whether this is genuinely slice-worthy or a `backlog new improvement` capture
  for later sequencing behind IMP-020.

## Next doctrine move
```
# confirm the gap and the existing reach surfaces (read-only):
doctrine inspect SL-046            # 1-hop today
doctrine blockers SL-046 --transitive   # the N-hop precedent
doctrine backlog show IMP-020      # the cordage-walk-unification prerequisite

# then capture intent (NOT executed here — fence forbids backlog transition):
doctrine backlog new improvement "Transitive impact/blast-radius query over the \
  cross-kind relation graph (governance + derivation overlays) — `doctrine impact \
  <ID>` riding cordage reachable; sequence behind IMP-020 walk unification" \
  --tag area:relations --tag area:cli --tag cordage

# if pursued as design work rather than a backlog capture:
/route                             # → /slice (code-changing intent, no governing slice)
```

## Illustration (optional)
None. The change is design-shaped (a new query surface + render path), not a
mechanical edit — a speculative worker diff would imply more settled design than
exists, and the real first step (IMP-020 walk unification) is the actual prework.
The CLI shape under discussion is sketched in Option 2 above, deliberately in prose.
