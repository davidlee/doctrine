# IMP-488: Promote doctrine search in agent-facing guidance and document its scope

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctrine search` (SL-141) is a BM25 index over the entity corpus and the
cheapest way for an agent to answer "where does this live / what is the right way
here?". It was named in exactly two shipped surfaces — a one-line entry in the
project `governance.md` *useful commands* block, and one passing mention in
`spec-coverage-assessment` — and in **no** shipped reference doc. Its **scope
appeared nowhere**, verified against source (`src/search.rs`, `src/kinds/mod.rs`
`SEARCH_DEFAULT`):

- the corpus is `scan_catalog` entities — **not** memories (no `MEM` prefix
  exists) and **not** the reference library (`install/**` is RustEmbed bytes
  behind a `reference/` logical address, reachable only via `doctrine library
  show`);
- the default kind set is `SL, ADR, PRD, SPEC, RFC, backlog, knowledge` — so
  `POL`, `STD`, `REQ`, `RV`, `REC`, `CM`, `REV` are **outside** the default
  scope and need `-k all` or `--with`;
- the discovery groups (`-k backlog|governance|specs|knowledge|all`) were named
  nowhere.

An agent assuming the bare verb is exhaustive silently misses policies and
standards (exactly what `/canon` needs) and concludes "not in the corpus" for
anything memory-shaped. Recovering those three facts cost a source read.

The missing corpora are already governed, not missing by oversight — hence
**guidance-only**:

- `IMP-154` seeds the widening (non-entity docs + path column);
- `PRD-017` `REQ-364/365/366` and `SPEC-026` `REQ-377/378` govern the provider
  federation that turns entity + memory + library into one entry point;
- `SPEC-026` **D10** deliberately keeps library *asset content* out of the index
  (metadata only) — a coherence question for federation planning.

## Delivered

Canonical home for the scope facts is `reference/using-doctrine.md`; every other
surface points there rather than restating it (one fact, one home).

- `install/using-doctrine.md` — a "find a thing by text" row in the verb map,
  plus a *Finding things by text — three corpora, three verbs* section: the three
  corpora, the entities-only limit, the narrow default kind set, the widening
  flags, and the named groups.
- `install/routing-process.md` — one mid-flight sentence in the routing digest
  (rides every session's boot prefix): search before assuming.
- `.doctrine/governance.md` — the *useful commands* block now names the scope and
  the memory corpus's separate index.
- Skills that do discovery but never reached for the verb: `preflight` (search
  order), `route` (route-unique rules), `canon` (with the `-k governance` jump),
  `retrieve-memory` (a *when the question is not memory-shaped* section).

Deliberately **not** done: no `glossary.md` entry (it owns vocabulary and ids;
the verb map owns intent → verb), and no `install/governance.md` seed line (the
seed is intentionally empty scaffolding, and `routing-process.md` already reaches
every client session).

## Sibling sweep — canonical verb

`doctrine memory find` is a *hidden alias* of `memory search` (the SL-184
rename), so guidance naming `find` still works but teaches a verb absent from
`doctrine memory --help`, and names the non-canonical one in the
discoverability surface. Aligned on `search` across:

- `plugins/doctrine-memory/skills/retrieve-memory/SKILL.md` (+ its
  `plugins/doctrine/skills/` hardlink), `plugins/doctrine/skills/record-memory`,
  `plugins/doctrine/skills/dreaming`;
- the shipped memory masters `memory/mem_019e9a12560d7972b29124e09f4de704`,
  `memory/mem_019ec92b0fff76e1935b222348938d7f`;
- the `memory retrieve` help description (`src/memory.rs`) and the help-snapshot
  assertion that pinned the old name (`src/main.rs`).

Delivery verified, per the shipped-corpus materialisation rule:
`cargo build` → `doctrine memory sync --dry-run` showed `0 new, 2 changed` →
real sync → re-check `0 new, 0 changed, 31 unchanged`; reads confirmed through
the render (`doctrine library show reference/using-doctrine.md`), not the source.
`doctrine validate`, `doctrine publication validate`, `doctrine doctor` (no new
finding) and `doctrine check gate` all green.

## Notes

Only `install/**`, `memory/**` and `plugins/*/skills/**` are tracked;
`.agents/`, `.pi/` and `.doctrine/skills/` are derived. `.doctrine/skills/`
refreshed from the embed; `.agents/skills/` is a mirror sourced from the
**published** repo and will not carry these edits until the commit is pushed —
expected, not a failed install (do not chase it via RustEmbed).
