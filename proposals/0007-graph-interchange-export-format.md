---
seq: 0007
scope: capture
target: discovered — `doctrine export` (src/main.rs), map_server `/api/graph`
confidence: med
reversible: yes (proposal only; nothing authored or built — read-only capture)
---
## What
`doctrine export` presents itself as a **format-extensible interchange surface** —
"Export the doctrine corpus to an external interchange format", a clap subcommand
*group* (`export <COMMAND>`) — but ships exactly one format: `lazyspec` ("Emit the
corpus as a single lazyspec Brief (JSON)", SL-026). That one format is
**agent-facing** (the lazyspec Brief is built for LLM/agent consumption), not
team-tooling-facing.

Meanwhile the full corpus graph is **already serialized internally**: the map
explorer's `/api/graph` route (`src/map_server/routes.rs:86`, handler `:165`)
hands the whole node+edge set to the web UI. So the graph data, nodes, and edges
are computed and shaped; they are simply not exposed on the CLI in any format the
*standard graph ecosystem* eats — GraphML, DOT, Cypher/`CREATE`, or a CSV edge
list. A product team that wants doctrine's topology in Gephi, Neo4j, Mermaid-live,
Observable, or a BI dashboard has no path that isn't "scrape the web API" or
"write a lazyspec→graph adapter themselves."

This is a *capture*, not a proposed build: the observation is that `export` was
deliberately structured for multiple formats and the highest-leverage missing one
— a graph interchange that plugs doctrine into tools teams already run — is absent,
even though the serialization seam (`/api/graph`) and the relation graph
(`relation_graph::build_relation_graph`) already exist. It directly serves the
standing focus ("capitalize on the graph topology to become indispensable to
teams"): the topology becomes useful to a team exactly when it leaves doctrine and
lands in the tools the team already trusts.

## Options
1. **`doctrine export graph --format {graphml|dot|cypher|csv}`** — one new export
   subcommand, fed by the same serialization behind `/api/graph` (reuse, no
   parallel graph build). Tradeoff: small, rides two existing seams; the only real
   work is per-format writers + choosing the initial format set.
2. **Single format first (DOT or GraphML).** Ship one well-chosen interchange
   format, defer the rest. DOT = instant Graphviz/Mermaid; GraphML = Gephi/yEd/
   networkx. Tradeoff: minimal surface, proves the seam; picks a winner under
   uncertainty about what teams actually use.
3. **Don't add CLI export; document the `/api/graph` shape as the integration
   point.** Tradeoff: zero build, but pins integrators to an HTTP server + an
   unversioned internal JSON, and excludes scripted/CI pipelines that want a file.

## Recommendation
Capture as a backlog **idea/improvement** now (it's an opportunity, not a defect),
scoped as Option 1 with Option 2 as the first increment: ship `export graph
--format dot` first (highest "see it immediately" payoff, trivial writer), then
GraphML (the broadest analyst toolchain), then Cypher/CSV on demand. Reuse the
`/api/graph` serialization rather than a second graph walk — same anti-parallel
discipline as the other graph proposals (0003/0004/0006).

Decisions deferred to YOU:
- (a) **is this wanted at all**, or is lazyspec-for-agents the intended *sole*
  consumer of `export` (i.e. teams are expected to use the web map, not files)?
- (b) **format set & order** — DOT-first vs GraphML-first; which of
  GraphML/DOT/Cypher/CSV make the cut.
- (c) **node/edge fidelity** — full attributes (status, facets, ranks) or a thin
  id+label+relation edge list? Richer export is more useful but couples the
  interchange schema to internal facets (a stability surface).

## Next doctrine move
```
# confirm the single-format surface and the existing serialization (read-only):
doctrine export --help
sed -n '165,230p' src/map_server/routes.rs        # the /api/graph shape to reuse

# capture the opportunity (NOT executed — fence forbids backlog transition):
doctrine backlog new improvement "doctrine export graph --format {dot|graphml|\
  cypher|csv}: expose the relation graph to the standard graph ecosystem (Gephi/\
  Neo4j/Graphviz), reusing the /api/graph serialization — extends the export \
  subcommand group beyond lazyspec" --tag area:cli --tag area:relations --tag export
```
(Verbs described, NOT executed.)

## Illustration (optional)
None. A capture: the value is in naming the opportunity and the reuse seam
(`/api/graph` serialization → CLI), not in pre-committing a format writer.
