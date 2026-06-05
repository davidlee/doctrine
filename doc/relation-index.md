# Relation index — design note

**Status: deferred. No action now.** This records *why* nothing is built, and the
threshold that would change that.

## Context

Doc-relation queries — context chains (`slice → spec → requirement`), coverage,
validation — need a registry of parsed relationships across all entities.
spec-driver builds this by parsing YAML frontmatter across every file on each
invocation, which gets slow in large projects. The question was whether
doctrine needs a cache (sqlite, or a binary snapshot) to stay fast.

## Decision

**No cache now.** At the current and near-term scale (dozens to a few thousand
docs) plain TOML parsing is fast enough, provided the registry is not loaded
eagerly. The practical need for advance engineering here is near-nil.

The single thing to protect is already in place: relations live in small,
typed, cheaply-parseable sister files (`slice-NNN.toml`), separate from prose.
That keeps the index source isolated and a future cache a pure drop-in.

## Why TOML is enough at a few thousand

Startup is per-command, and most commands never touch the graph. Load only what
the command needs:

- **id allocation** (`reserve`, `slice new`) — needs only the max id → `readdir`
  of directory *names*, **zero parse**.
- **`slice list`** — metadata per doc → parse the small tomls, fields only.
- **graph queries** (context / coverage / validate) — the *only* commands that
  parse the full set.

Three levers keep even the full parse fast:

1. **Lazy, command-scoped loading** — never build the registry unless a graph
   query asks for it.
2. **Sister-toml isolation** — indexing reads `slice-NNN.toml` (~hundreds of
   bytes), never the markdown body. The split chosen for authoring doubles as the
   index boundary.
3. **Parallel typed deserialize** — `rayon` across cores; `serde`-derive straight
   into the metadata struct, skipping the generic `toml::Value` tree.

**Numbers.** A few thousand small sister tomls: I/O syscall overhead dominates,
not parsing (~5–30µs/file into a struct). Serial ≈ 100–150ms; **parallel ≈
20–50ms warm**. Acceptable for the graph commands; the id/list commands are
faster still. Comparable to what git eats on large repos.

**Count in files, not entities.** The spec decomposition (spec-entity-spec) is
the precondition this note protects, and it explodes the ratio. A spec itself is
light — ~3–4 sister files (identity + prose + `members.toml`, plus tech
`interactions.toml`) — but each requirement it members is its **own peer entity
directory** (`requirement/NNN/`, 2 files), so a spec with N requirements spans
~`4 + 2N` files across trees. "A few thousand specs" is still tens of thousands of
files — past the ~10k revisit line below. The parallel-parse numbers still hold
(the files stay tiny and independent), but the budget must be tracked in
**files**, not docs, and the revisit trigger restated accordingly.

## Staged path (only if scale demands it)

1. **Now (≤ ~few thousand docs):** plain toml + lazy load + parallel typed parse.
   No cache.
2. **If past ~10k docs** and graph-query startup is felt: a **binary snapshot** —
   `bincode` of the parsed registry, keyed by a `(path, content-hash)` manifest;
   re-parse only changed docs; query in memory. Written temp-then-`rename`
   (atomic). Disposable, gitignored, per-clone.
3. **sqlite:** only if the graph outgrows memory — which doc relations
   (~thousands of edges, single-digit MB) will not. Even then WAL + `busy_timeout`
   serve multiple agents with no daemon.

## Invariants that make this safe to defer

- **Source of truth stays TOML** at every stage. A cache, if it ever exists, is a
  disposable derived layer and never changes the authoring format.
- **Disposability dissolves the concurrency problem.** Because the cache is
  rebuildable, it needs no transactional integrity and no shared mutable writer:
  an immutable snapshot with atomic replace is correct under concurrent agents
  (last-writer-wins on identical derived bytes). The multi-agent / client-server
  worry that motivates sqlite never arises. (And even sqlite would not need a
  daemon — WAL + `busy_timeout` on a local file suffices.)
- **The cache is uncoordinated** — local, per-clone, gitignored. It is not shared
  state and carries no leases; coordination is the reservation layer's job
  (reservation-spec), kept cleanly separate.

## Two purposes, two triggers

This note conflated *cache* with *registry*; they separate cleanly:

- **The in-memory parsed graph** — built by lazy full-parse for graph queries,
  including **referential-integrity validation** (`doctrine validate`: every FK
  resolves to an existing entity). This needs **no cache**. Its trigger is
  **not** scale — it is *the first foreign key authored* (the moment dangling refs
  become possible, per spec-entity-spec § Diagnosis). Because the spec entity is
  what *introduces* the cross-entity edge tables (`members` spec→requirement,
  `interactions` spec→spec), that pass **co-lands in the spec entity's own slice**
  — it shipped as `spec validate` (SL-015); it is
  part of the minimum spec bundle, not a later deliverable (spec-entity-spec
  § Known risks, integrity). FK validation is the registry's headline value and
  it is **not** gated on the cache decision below.
- **A persistent cache** (snapshot/sqlite) — purely a *speed* optimization for the
  full parse, deferred until it is felt.

## Trigger to revisit (the cache only)

Graph-query commands feeling slow at real scale (order ~10k **files**, per the
count-in-files note above). Until then, the only "work" is *not* eagerly loading
the registry. The FK-validation pass (above) is a separate, earlier deliverable
and does not wait for this.
