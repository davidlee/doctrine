# Shipped orientation memory corpus

## Context

Doctrine's memory store (`.doctrine/memory/items/`, SL-005/007/008) is a
**project-local capture** surface: rich TOML+MD entities, git-anchored to *the
client's* repo, with staleness tracking and a trust holdback. It answers "what
did an agent learn working in *this* codebase." It is authored at runtime, not
shipped.

There is a second, distinct need it does not serve: **framework orientation** —
durable knowledge about *how to drive doctrine itself* (the slice→design→plan→
phase loop, the storage rule, the routing gate, sharp edges in the artifacts and
CLI). Today that orientation is spread across the boot snapshot, the skills, and
`doc/*`, and an agent dropped into a fresh client repo has no doctrine-authored
memory corpus to retrieve against.

Sibling project **spec-driver** already does this: it ships ~86 flat
`.md`-with-frontmatter memories (in its repo-root `memory/`, force-included into
the wheel) that install to every client and orient agents driving spec-driver.
Its corpus shows the *shape* of a useful orientation set — `signpost`, `concept`,
`pattern`, `fact`, `reference` types covering overview, file-map, core loop,
ceremony, and per-subsystem gotchas.

This slice brings the same capability to doctrine: a doctrine-authored
orientation corpus plus an install path that delivers it to clients. spec-driver's
corpus is the **template for topic coverage**, not a literal port — most of its
86 entries are spec-driver-internal (`mem.concept.spec-driver.*`, `mem.fact.tui.*`)
and do not apply. "Reinterpret" means: author doctrine's *own* orientation
memories, using spec-driver's corpus to decide *what topics* an orientation set
should cover.

## Scope & Objectives

1. **Triage** spec-driver's 86 shipped memories into: (a) directly transferable
   (rewrite for doctrine), (b) topic-applicable (doctrine needs a memory on this
   subject, authored fresh), (c) inapplicable (spec-driver-internal / stack-
   specific — Python/Typer/Textual/pylint — dropped). Record the disposition.
2. **Author** the doctrine orientation corpus from (a)+(b): the doctrine core
   loop, file-map / storage model, routing gate, artifact conventions, CLI
   shape, and the durable sharp edges already known (several live in the current
   project-local memory store and may be promoted/generalised).
3. **Wire the install + refresh path** — masters at repo-root `memory/`, embedded,
   materialized by `doctrine memory sync` into a gitignored `.doctrine/memory/
   shipped/`; **M1**: a SessionStart hook auto-runs sync (idempotent) so clients
   self-heal on binary upgrade.

## Non-Goals

- **No literal port** of spec-driver's corpus, and nothing spec-driver-internal
  or Python/TUI-stack-specific.
- **No change to the runtime capture pipeline** (`record`/`find`/`retrieve`/
  scope ranking / git anchoring / trust holdback — SL-007/008). This slice adds
  a shipped corpus; it does not alter how project-local memories are captured or
  queried. The behaviour-preservation gate applies: existing memory suites stay
  green unchanged.
- **No re-litigation of the boot snapshot / skills / `doc/*` remit.** Where a
  topic is already projected by boot.md or a skill, the corpus links/points to
  it rather than duplicating (no parallel implementation). Drawing that boundary
  is design work.
- **No staleness-reaction hooks (M2)** — reacting to code changes under a
  captured memory's scoped paths. Deferred to a follow-up slice + behaviour-hooks
  ADR.
- **No override/suppress of shipped memories (M3)** — local shadowing by
  `memory_key`. Deferred; the `collect_all` uid-dedup seam is left open for it.
  Local *additive* orientation already works (an `items/` memory, repo=<client>,
  unscoped/tag-scoped, unanchored) and is **not** stored in shipped/, so sync
  never touches it (D8).
- **No `memory sync --check` sentry** in v1 — sync is idempotent, so the M1 hook
  needs none; a dedicated sentry is deferred.

## Affected surface

- `memory/` (new, repo-root) — the authored masters tree (committed).
- `src/memory.rs` — new `sync` verb (+ `sync install` hook wiring) + pure
  idempotent materializer; a `collect_all` composite over the unchanged
  `collect_memories` leaf (gate); `read_body` cross-root fallback.
- `src/retrieve.rs` — candidate collection + `list_rows` switch to `collect_all`.
- hook wiring (SessionStart → `memory sync`), reusing `boot install`'s mechanism
  (settings.json / agent hook surface).
- a new `#[derive(RustEmbed)] #[folder = "memory/"]` (parallel to `install/`,
  `plugins/`).
- `install/manifest.toml` — `[gitignore].entries += ".doctrine/memory/shipped/"`
  (client denylist surface).
- this repo's `.gitignore` — add `.doctrine/memory/shipped/` beside the
  `index|embeddings|state/*` re-ignores (the authored-entity-wiring trap,
  inverted: derived subtree under the `!.doctrine/memory/` negation is
  committed-by-default here).
- the new ADR + `doc/memory-spec.md` amendment.
- `doc/install-spec.md` / a memory-sync note if the verb warrants spec text.

## Resolved design decisions (see design.md)

- **OQ-1 (format & home) — RESOLVED: native, repo-global, derived/gitignored.**
  Ship native memory entities (the existing `memory.toml`+`memory.md` schema), so
  they retrieve through `doctrine memory retrieve` and list in the boot snapshot —
  the scoped-retrieval payoff a flat format can't give. They carry `repo = ""`
  (the *global* class — admitted in every partition; repo-id is the cross-repo
  filter, so a real repo-id would self-exclude from clients) and
  `anchor_kind = none`. They live in a **gitignored derived tree**
  (`.doctrine/memory/shipped/`), NOT committed `items/`, so the committed
  capture tree and its scoped⇒anchored invariant are untouched.
- **OQ-2 (altitude) — RESOLVED: spawns an ADR + a memory-spec amendment.** Ship
  defines a new memory *class* (global / unanchored / path-scoped / derived) and
  a second indexer scan root — architectural. ADR blesses the class; memory-spec
  §295-308 (anchoring) + §326-368 (retrieval) gain the carve-out.
- **OQ-3 (overlap) — boundary held.** Corpus points *toward* boot/skills/`doc/*`,
  never restates them. The corpus's edge is *scoped* retrieval (per-path), which
  the static boot snapshot cannot do.
- **OQ-4 (re-install / upgrade) — DISSOLVED by the derived tier.** Gitignored +
  derived ⇒ `doctrine memory sync` overwrites to match source (ownership-safe,
  per skills' `classify_link`); no committed user edits to clobber, no merge.

## Architecture (locked)

- **Masters** at repo-root `memory/` (parallel to `install/`, `plugins/`; mirrors
  spec-driver's own layout). NOT under `install/` — the install scaffolder's embed
  would wrongly write them to the committed `items/` tier. A separate
  `#[derive(RustEmbed)] #[folder = "memory/"]` embeds them.
- **`doctrine memory sync`** (new verb, `memory` command family) materializes the
  embedded masters → `.doctrine/memory/shipped/<uid>/` (file-copy, overwrite,
  ownership-classified). Refresh-on-upgrade = re-run. First-time = after
  `doctrine install`. A `--check` sentry is deferred to v2.
- **Indexer second scan root** — `collect_memories` gains the shipped tree
  alongside `items/`, merged into the candidate set.
- **Gitignore** — manifest adds `.doctrine/memory/shipped/` (narrow, derived).
- **Layering (ADR-001)** — new code respects leaf ← engine ← command; the sync
  verb is a thin command shell over a pure materializer.

## Verification / closure intent

- Triage table exists with a disposition for all 86 spec-driver memories.
- A doctrine orientation corpus is authored and committed under the chosen
  masters location.
- A fresh `doctrine install` into a clean client lands the corpus at the chosen
  client path; verified end-to-end.
- An agent can retrieve/read the corpus through the chosen surface.
- Existing memory suites green unchanged (behaviour-preservation gate).
- `just check` clean; corpus + install wiring covered by tests at the level the
  design specifies.

## Summary

Give doctrine a shipped, doctrine-authored orientation memory corpus and an
install path to clients, modelled on (not ported from) spec-driver's shipped
memories. Triage → author → wire install. The format/home of shipped memories
and its altitude (slice vs ADR) are the open design questions.

## Follow-Ups

- **New slice + behaviour-hooks ADR**: M2 (staleness-reaction hooks — react when
  code changes under a captured memory's scoped paths; builds on SL-008's existing
  detection) and M3 (override/suppress a shipped memory by `memory_key` —
  key-precedence in `collect_all`; possibly relaxing `record`'s gate to permit
  unscoped+unanchored with a repo present, for local convention memories).
- Periodic corpus refresh as doctrine's own workflow evolves (corpus is
  evergreen-ish; needs an owner/cadence).
