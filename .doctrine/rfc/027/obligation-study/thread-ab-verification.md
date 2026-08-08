# Thread A/B — verification note

Verification of `raw/thread-ab-incumbent.md` (pi-research, 2026-08-08) against
primary sources. The raw file is kept verbatim; corrections live here.

Study context: brief 03 of the external design/proof pack
(`scratch/2026-08-08/03-obligation-graph-integration-brief.md`), investigating
whether Doctrine should model phase-level obligations with `needs`/`after` edges.

## Verdict

Sound on the actionability map and the dep/seq inventory. **One material error**,
plus minor line-cite drift. Corrected below.

## Material error — per-phase requirement links are NOT unused

The report claims (A.3, B.3.5) that the per-phase `specs` / `requirements` link
tables are *"empty in every existing slice's plan — they are scaffold-only"*.

**False.** Ten plan files populate them. Verified:

```
.doctrine/slice/020-backlog-entity-v1/plan.toml:68   requirements = ["REQ-053", "REQ-058"]
.doctrine/slice/057-formal-vt-verification/plan.toml:53  specs = ["SPEC-002"]
.doctrine/slice/057-formal-vt-verification/plan.toml:54  requirements = ["REQ-254"]
.doctrine/slice/043/plan.toml:72                     specs = ["SPEC-001"]
```

Top-level `[requirements].targets` is likewise populated in SL-043, SL-057 and
SL-167.

**The agent inherited the error from a stale code comment.** `src/plan.rs:14-18`
asserts the tables *"exist in the file but are empty (no registry yet) and are
not modelled"*. The second clause is true; the first is not.

**The corrected fact is sharper than the agent's version.** `Plan` deserializes
only `phases` (`src/plan.rs:20-23`), and `PlanPhase` carries no `specs` /
`requirements` fields (`src/plan.rs:31-43`). So authors have been writing
requirement→phase links into `plan.toml` for ten slices and **the reader silently
discards them**. This is authored-but-dark data, not absent data.

Bears directly on `REQ-439` AC-2 — *"Every phase states a stable phase identity
and objective plus its entry, exit, and verification criteria and any applicable
canonical spec or requirement links"* — a `pending` PRD-001 requirement the
current model does not satisfy despite the corpus already carrying the data.

## Minor cite drift (facts hold)

- `objective: String` is `src/plan.rs:37`, not `:63`.
- The per-criterion runtime-tier comment is `src/state.rs:346`, not `:348`.
- `is_work_like` admitting REV as both source and target is **confirmed**
  (`src/commands/dep_seq.rs:7-19`) despite a stale refusal message on
  `resolve_dep_seq_src_path` naming only slices and backlog items.

## Confirmed findings worth carrying forward

1. **`dep_seq` is kind-neutral by construction.** `src/dep_seq.rs` is an ADR-001
   leaf (imports only `toml_edit`/`anyhow`/`std`); its doc comment states the
   `DepSeq` schema "carries only the two relation axes, kind-neutral". `needs` =
   hard payload-free prerequisite; `after` = soft sequence with per-edge `rank`.
   Semantics match the brief's proposal exactly.

2. **The authoring seam is entity-bound, but the graph seam is not.** `dep_seq`
   reads an entity's `<stem>-NNN.toml` `[relationships]` table, and phases and
   criteria are rows inside `plan.toml`, not entities. This note originally
   framed that as a binary fork — promote obligations to entities, or build a
   doc-local analogue. **That framing was wrong**; see § Direction below.

3. **Phase sequencing is entirely unauthored.** Plan array order is the sole
   authority (`PHASE-NN` lexicographic sort, `src/dispatch.rs`); readiness comes
   from gitignored runtime sheets; `compute_next_phases()` encodes the batching
   rule in Rust, not in any plan file. There is no phase-level `needs`/`after`.

4. **The runtime tier already anticipates this.** `src/state.rs:346`: *"Richer
   per-criterion/task rows graduate to TOML when a consumer lands (D5/Q2)."*

5. **EX criteria are the prime duplication suspect.** An EX row states a required
   state of affairs — the brief's own definition of an obligation — and already
   carries an immutable id, authored order, and (under `REQ-442`) evolution
   lineage covering split/merge/relocation. The gap the agent identifies is that
   **no authored field links a VT row to the EX row it proves**; the mapping
   exists only in prose.

## Direction — grafting, not promoting (user, 2026-08-08)

The binary fork above was a false dilemma. The stated near-invariants:

- reuse cordage;
- **graft** the obligation graph onto the existing corpus actionability graph —
  at least whatever part of it is appropriate to load for a given slice;
- this must **not** imply splitting every in-slice node into its own entity;
- instead: a different **loader**, and possibly storage, strategy for a slice,
  starting from where we are — `plan.toml` as it stands;
- and it must **not** imply doing this for every slice on a corpus-wide load —
  only the slice being worked in detail.

**The evidence says this is cheap.** Three verified facts:

1. **cordage is identity-agnostic.** `NodeId(u32)`
   (`crates/cordage/src/lib.rs:28`) is an opaque dense index handed out by
   `GraphBuilder::node()` (`:639`), which allocates a counter and nothing else.
   Cordage has no notion of entity, kind, or corpus. **Grafting non-entity nodes
   needs zero cordage changes.**

2. **The interning seam is already generic.** `Projection<K: Copy + Ord>`
   (`src/projection.rs:20`) owns the key↔`NodeId` binding through `intern` /
   `resolve` / `key_of` / `remap_set`. It is generic over the key type today; it
   is instantiated at `Projection<EntityKey>` only by the current callers.

3. **So the whole graft reduces to a key type and a loader arm.** The one real
   change is widening the projection key from `EntityKey { prefix, id }`
   (`src/catalog/scan.rs:85`) to a sum admitting slice-local nodes.

**The binding constraint is `Copy + Ord`.** A slice-local key must be packed and
numeric — `{ slice: u32, phase: u8, class: u8, ordinal: u16 }` or similar. A
`String` phase/criterion id will not satisfy `Copy`, so the loader must map
`PHASE-03` / `EX-1` to numeric coordinates at load time and render back through
an id-form authority the way `EntityKey::canonical()` does.

**Scoping falls out of the loader for free.** The projection is rebuilt per load,
so a corpus-wide load simply never interns obligation nodes, while a
slice-focused load does. No schema change, no per-slice cost at corpus scale, and
no stored artefact that a corpus reader must skip past.

`relation_graph::dep_seq_for` (`src/relation_graph.rs:60`) is the kind-dispatched
read gate whose arms already short-circuit non-authoring kinds to an empty
`DepSeq` before touching disk. It is the natural site for a slice-scoped
obligation arm.

### What this leaves open

- **What is an obligation, concretely?** EX criteria remain the prime candidate —
  immutable ids (`REQ-441`), authored order, and evolution lineage covering
  split/merge/relocation (`REQ-442`) already exist. Treating EX rows as
  obligations is zero new storage. The alternative is an explicit obligation
  array, which must first earn its distinct fact.
- **The EX→VT link is unauthored.** No field ties `VT-1` to the `EX-1` it proves;
  the mapping lives only in prose. Any proof-bearing consumer needs it.
- **Where do requirement→obligation edges come from?** The per-phase
  `requirements` arrays are already authored in ten plans and discarded on read
  (ISS-321). That is the cheapest available source and needs no new authoring.

## Follow-up captured

The stale `src/plan.rs` comment and the discarded per-phase link tables are a
latent work item, not a study finding — see the backlog.
