# Research Spec — lazyspec as a read-only front-end for doctrine

**Status:** assignment (placeholder; results pending)
**Target repo:** `../lazyspec` (Rust TUI spec framework — https://github.com/jkaloger/lazyspec)
**Consumer:** whoever researches lazyspec, then authors the *integration brief* that
pieces 2–4 build from.
**Date authored:** 2026-06-08

---

## 0. Objective

De-risk and characterise lazyspec well enough to:
1. add a **doctrine read-only backend** to lazyspec, consuming **doctrine CLI JSON**
   (never doctrine's TOML/MD files);
2. project doctrine's **lazily-composed specs** into lazyspec's `virtual_doc`
   `DocMeta` views;
3. ship a **doctrine-compat `.lazyspec.toml` preset** so the two snap together
   reproducibly with no per-project hand-fiddling.

Write side is **out of scope for v1** — lazyspec is a viewer. Mutations get
selectively re-enabled later as doctrine grows mutation verbs.

---

## 1. Already established (do NOT re-derive)

The architectural shape is decided; research validates and fills it in, it does not
relitigate it.

- **The core mismatch.** lazyspec is *document-centric*: a spec **is** a file
  (`DocMeta` = one markdown file + YAML frontmatter, `engine/document.rs:188`).
  doctrine is *composition-centric*: a spec is **lazily assembled** from requirement
  entities + outbound edges at display time — there is no spec file to read.
- **The boundary.** lazyspec talks to the `doctrine` CLI's **JSON output**, not its
  files. Keeps lazyspec ignorant of doctrine's storage rule (TOML sister-files,
  queried-data-never-in-prose); keeps doctrine ignorant of YAML/Ratatui.
- **Adapter lives in lazyspec, not doctrine.** lazyspec has no plugin loader;
  backends are a compile-time `StoreBackend` enum (`engine/config.rs:112`). So the
  mapping code is a new lazyspec backend. doctrine gains at most one **leaf**
  command emitting the composed-spec projection as JSON (guaranteed no internal
  dependents by ADR-001 layering).
- **RO-first.** The backend returns `Err(ReadOnly)` from all four `DocumentStore`
  mutation methods. No doctrine mutation verbs required for v1.
- **doctrine is the brain.** doctrine owns ids, edges, sequencing, lifecycle,
  validation. lazyspec's own validation rules get disabled in the preset so two
  validators never disagree.

### Key lazyspec seams (carry these into the read)

| Component | File:line | Note |
|---|---|---|
| `DocMeta` struct | `engine/document.rs:188` | the target shape |
| `Relation` / `RelationType` | `engine/document.rs:116` | Implements/Supersedes/Blocks/RelatedTo |
| `DocumentStore` trait (write seam) | `engine/store_dispatch.rs:38` | 4 methods to stub as `Err` |
| `StoreBackend` enum | `engine/config.rs:112` | add a 4th variant here |
| `Store::load_with_fs` (read path) | `engine/store.rs:27` | dispatch a doctrine read here |
| `FileSystem` trait | `engine/fs.rs:4` | I/O abstraction — may assume real paths |
| `Config` | `engine/config.rs:227` | preset surface |
| `TypeDef` | `engine/config.rs:133` | doctrine-kind → lazyspec-type mapping |
| `ValidationRule` enum | `engine/config.rs:14` | must be emptiable |
| graph traversal | `tui/state/graph.rs:8` | shallow — cycle-detect + status-propagate |

*(Anchors from a prior exploration pass; confirm they haven't drifted.)*

---

## 2. Research questions (the assignment)

### A. Write-refusal / RO tolerance — **load-bearing, answer first**
- Does the TUI **degrade gracefully** when every `DocumentStore` mutation returns
  `Err`? Greyed-out create/edit keys, or error popups?
- What code paths **assume the store is writable** (create flows, status edits,
  provenance edits)? Enumerate them.
- Is there an existing notion of a read-only or remote backend whose UX we can copy?
- **Verify empirically:** wire a stub `DocumentStore` that errors on all four
  methods, run the TUI, observe.

### B. Backend extensibility cost
- Exact, ordered steps to add a 4th `StoreBackend` and dispatch its read path.
- Is it truly fork-only, or is there a cleaner seam (trait object, registry) worth
  upstreaming to lazyspec instead of forking?
- How invasive is the change to `Store::load_with_fs` — does it assume a directory
  walk, or can a backend hand back a pre-built `DocMeta` set?

### C. `DocMeta` fidelity — gates the wire format (piece 2)
- Can doctrine entities (slice, adr, plan, **composed spec**, requirement) map onto
  `DocMeta` **without lossy compromise**? Produce a field-by-field mapping table.
- Where does doctrine's richer model have **no home** in `DocMeta` (edge types
  beyond the 4 `RelationType`s, phases, EN/EX/VT criteria, membership labels,
  lifecycle status set)? List the gaps — these decide whether `DocMeta` needs
  optional extension fields or whether we accept lossy projection in v1.
- **`virtual_doc` semantics:** what does setting `virtual_doc: true` actually
  enable/restrict in the TUI? Does it render, navigate, search, graph like a real
  doc? Does anything assume a real on-disk `path`?

### D. Load model
- Eager whole-tree load (`Store` holds everything in a `HashMap`): what's the cost,
  what triggers reload, does it watch the filesystem?
- If "files" are CLI-sourced virtuals, does the loader/`FileSystem` trait assume
  **real filesystem paths**? Where would that break?
- Is there any incremental/lazy load path, or must every composed spec be assembled
  up front (fights doctrine's lazy-assembly grain)?

### E. Validation / relation ownership
- Can lazyspec's `ValidationRule` set be **emptied** in config? Does anything break
  with zero rules?
- How are relations consumed for the **graph view** — must edges be expressed as
  `DocMeta.related` to light it up, or can lazyspec render an **externally-supplied
  ordering/DAG** from doctrine? (Determines whether doctrine's sequencing survives
  the projection or gets recomputed by lazyspec's shallow traversal.)

### F. Config / preset surface
- Full `.lazyspec.toml` schema needed for the doctrine preset: type mapping, store
  dispatch, UI.
- **Numbering:** confirm `numbering = Reserved`/external actually prevents lazyspec
  from **minting ids** (doctrine owns ids absolutely — double-minting = corruption).
- Is there a presets/`init`-template mechanism, or do we ship a static
  `.lazyspec.toml`?

---

## 3. Deliverable — the integration brief

A single document containing:
1. **RO-degradation verdict** (A) — confirmed TUI behaviour + any lazyspec patch
   needed before RO is truly free.
2. **`DocMeta` ↔ doctrine-entity mapping table** (C) — field by field, with the gap
   list and a recommendation (extend `DocMeta` vs accept lossy v1).
3. **Proposed wire-format schema** — the JSON doctrine must emit, driven by (2).
   This is what piece 2 locks and tests.
4. **Backend-add recipe** (B) — ordered steps, fork-vs-upstream call.
5. **Draft doctrine-compat `.lazyspec.toml`** (F) — the ready-to-run preset.
6. **Graph/sequencing verdict** (E) — does doctrine's ordering survive, or must
   edges be flattened into `related`.

---

## 4. Out of scope

- Writing the doctrine backend (piece 4).
- Implementing/locking doctrine's wire format + tests (piece 2 — but its schema is
  *specified* by this research's deliverable #3).
- doctrine's lazyspec-brief emitter (piece 3).
- Any forking or upstream PR.

---

## 5. Method hints

- Build & run lazyspec against its own sample data first — learn the happy path.
- Carry the §1 anchor table; confirm line drift.
- For (A): the erroring-stub `DocumentStore` is the fastest empirical answer.
- For (E/F): try emptying `rules` and setting numbering external in a scratch
  `.lazyspec.toml`, observe.
- Confirm ambiguous areas against the lazyspec source directly; the config system
  and split-file (frontmatter/body) support are the likely soft spots to probe.

---

## 6. Piece ordering (context)

```
1. research (this spec)
        │  produces the brief (§3)
        ▼
2. doctrine wire-format lock + tests + thin generic adapter   ┐ schema specified
3. doctrine emits lazyspec brief (the projection JSON)        ┘ by brief #3
        │
        ▼
4. lazyspec fork + doctrine backend (built off brief)
   + doctrine-compat ready-to-run .lazyspec.toml
```

Research gates 2 and 3 (the wire format can't lock until `DocMeta` field needs are
known). 2 and 3 can then proceed in parallel; 4 consumes all.
