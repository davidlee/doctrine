# IMP-120: Transitive impact query on relation graph

## Context

Doctrine has three relation-walking surfaces, and they disagree on reach. The
cross-kind authored relation graph — the strategic asset — is the shallowest:

- `doctrine inspect <ID>` renders **1-hop only** — outbound / inbound / danglers.
- `doctrine blockers <ID> --transitive` walks N-hop, but only over the dep/seq
  (`needs`/`after`) overlay.
- `doctrine retrieve … --expand <N>` walks N-hop, but only over the memory relation
  graph.

Multi-hop traversal is a solved, shipped capability on two surfaces, and the
reachability primitive itself already lives in the shared graph crate
(`crates/cordage/src/query.rs` — `reachable`, flagged for triplication cleanup by
IMP-020). The missing piece is purely **exposure**: there is no transitive impact /
blast-radius query over the governance and derivation overlays (GovernedBy,
Implements, DescendsFrom, Supersedes, DecisionRef).

## Recommendation

New `doctrine impact <ID>` verb — a dedicated upstream/downstream transitive-closure
query, direction-selectable, label-filterable. The data and the walk both exist;
only the verb is absent.

**Gated on IMP-020 first.** IMP-020 records three diverged reachability walks in
`crates/cordage/src/query.rs` that should be unified. Building a new traversal
consumer on top of a known-triplicated primitive would add a fourth caller to an
unstable seam. Sequence: consolidate the cordage walk (IMP-020), then expose
`impact` on the clean primitive.

Concrete query this answers: *"If I change ADR-005 / REQ-094 / SPEC-001, what
transitively depends on it?"* — the single most load-bearing use of a governance
graph for a team.

## Decisions deferred

- default direction and label set — does `impact X` mean downstream (what X depends
  on) or upstream (what depends on X)? For change-impact, upstream is the intuitive
  default.
- slice scoping vs. backlog improvement capture.

_Source: proposal 0003 (loop/proposals-2026-06-20), 2026-06-20._
