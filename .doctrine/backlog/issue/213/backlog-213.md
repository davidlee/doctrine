# ISS-213: build_memory_key_map ignores shipped memories — key-form relation targets on shipped memories dangle

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Defect

`build_memory_key_map` (`src/catalog/hydrate.rs:466`) builds the memory
key→uid map by reading symlinks from **`.doctrine/memory/items/` only**
(`MEMORY_ITEMS_DIR`). The shipped corpus at `.doctrine/memory/shipped/`
(`MEMORY_SHIPPED_DIR`) is **not scanned**, and its dirs carry **no key
symlinks** (stored by UID dir only — 0 symlinks vs 284 in `items/`).

Consequence: when a memory authors a TOML `[[relation]]` whose `target` is a
readable **key** (`mem.signpost.doctrine.lifecycle-start`) and that key belongs
to a **shipped** memory, `classify_target` finds no entry in `mem_key_map` and
the edge resolves to `EdgeTarget::UnvalidatedText` — it **dangles**, drawing no
edge in the map graph and generating no clean diagnostic.

Only two things resolve today for a shipped target:
- a **UID** target (`mem_<hex>`) — resolved directly against `key_set`; or
- a **key** target that happens to also exist as a **local** `items/` memory.

## Evidence

Empirically confirmed during SL-200 `/design`: a shipped→shipped relation
authored by key on `mem.signpost.doctrine.overview` produced
`{"UnvalidatedText": {"raw": "mem.signpost.doctrine.lifecycle-start"}}`; the
same relation authored by UID produced `{"Resolved": "mem_019e9a11e833…"}` and
drew the edge. (Spike reverted; corpus clean.)

## Fix direction

Extend the key→uid map to cover shipped memories. Shipped dirs have no
symlinks, so the map cannot be built from symlinks alone — read each shipped
`memory.toml`'s `memory_key` field (or add key symlinks at sync time). Prefer
reading `memory_key` to avoid a sync-side write. Merge shipped + items keys into
one `mem_key_map` so `classify_target` resolves either corpus. Guard against
key collisions (a local override of a shipped key) — decide precedence.

## Relation to SL-200

SL-200 (author TOML relations on the shipped onboarding memories so they browse
in the map) is **blocked on ergonomics** by this: without the fix, onboarding
relations must be authored by opaque UID. SL-200 can proceed UID-first, but the
readable-key authoring path wants this. Relate SL-200 `needs`/`after` ISS-213 at
design lock, per whichever sequencing is chosen.

## Verification

- A shipped memory with a `[[relation]]` targeting another shipped memory **by
  key** resolves to `Resolved` in the catalog graph (map edge draws).
- Existing `items/`-key resolution stays green (behaviour-preservation).
- Collision precedence covered by a test.
