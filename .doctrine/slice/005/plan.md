# Implementation Plan SL-005: Memory entity v1

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

Six phases turn the approved design ([design.md](design.md)) into code. The shape
is a clean split: **PHASE-01 changes the shared engine and nothing else**
(behaviour-preserving for the four numeric callers), then **PHASE-02 → 05 build the
memory entity on top** as a new `memory.rs` module + CLI, and **PHASE-06** flips the
install manifest so authored items commit while derived subtrees stay ignored.

The engine work is the risk; everything after it is additive. So the engine lands
first, behind its existing suite as the gate, before any memory code can depend on
the new `Named` identity shape.

## Sequencing & Rationale

**Why the engine is one phase, two commits (PHASE-01).** The seam rename (D7) and the
identity widening (D1/D8/D9) are independent in risk but coupled in churn — both touch
every numeric caller. Committing the rename first (isolated, trivially green) keeps
the widening diff focused on identity, and keeps each commit small per project
convention. They are one *phase* because they share a single gate: the pre-existing
entity.rs + slice.rs suite passing unchanged. That suite is the behaviour-preservation
proof — if it stays green through `Kind.mode` removal, `Inputs` reshape, and
`Materialised` → owned enum, the numeric callers are provably untouched in behaviour.

**Why schema before scaffolds before verbs (PHASE-02 → 04).** Pure core first: the
parse/validate layer and the `MemoryRef`/key/tag/uid validators (PHASE-02) have no
disk or clock and are fully unit-testable in isolation. Scaffolds + templates
(PHASE-03) are pure render over those types. Only PHASE-04 (`record`) introduces the
imperative shell — uid mint, the `Named` materialise, the transactional alias — so
impurity enters last and thinnest, exactly the slices-spec architecture split.

**Why show+list share a phase with the integration test (PHASE-05).** Both are read
paths over the same `items/` tree; the integration test (record → show → list) is the
first and only test that touches a real symlink end to end, and it validates both
verbs at once. Grouping them keeps the one real-fs test next to the code it covers.

**Why the manifest is last (PHASE-06).** It is independent of the code and the
cheapest to verify, so it gates nothing and waits until the entity is complete.

**Dependency chain.** 01 (engine `Named`) → 03 (Named Kind) → 04 (record) → 05
(show/list). 02 (schema) feeds 03/04/05. 06 is free-standing.

## Notes

**Plan-time findings beyond the design** (discovered reading the surface; folded into
the phase criteria):

- **`scan_named` is a new engine helper (PHASE-01 EX-6).** `entity::scan_ids`
  (`entity.rs:156`) parses `u32` and so *skips* `mem_…` uid dirs — it cannot drive
  `list`. A sibling `scan_named(tree_root) -> Vec<String>` (non-symlink dirs of any
  name) is required. The design only implied this.
- **The manifest split is an ADD, not a replace (PHASE-06).** There is no blanket
  `.doctrine/memory/*` gitignore in `install/manifest.toml` — it only *creates*
  `.doctrine/memory`. design.md §5.4 and the ed4 review note both assumed a blanket
  to replace; corrected here.
- **`uuid` is commented out in `Cargo.toml`; `time` is already live.** PHASE-02 enables
  `uuid` with the **v7** feature (verify the workspace `[workspace.dependencies]`
  feature set carries v7; extend if not). `time` needs no change (`slice.rs::today()`
  already uses it).
- **Templates auto-embed via `rust-embed`** (`install.rs` `#[derive(RustEmbed)]` over
  `install/`). Dropping `memory.{toml,md}` into `install/templates/` is sufficient —
  no manual asset registration — and the installer copies them to
  `.doctrine/templates/` for free.

**Migration touch-points for PHASE-01** (so the widening commit is mechanical, not
exploratory): `slice.rs` 4 const Kinds drop `mode` (`:29-61`); 5 `materialise` call
sites + 2 test helpers drop `Inputs.existing_id` and pass a `MaterialiseRequest`
(`:340,369,396,449` + tests); 4 scaffolds destructure `EntityId::Numbered`
(`:153-221`); `out.id` reads become `.eid.numeric_id()` (`:354,689` + tests);
import at `:23`. Inside `entity.rs`: the trait/impl/dispatch + its own test fixtures.

**Process reminders.** Each phase ends green (`cargo test` + `cargo clippy` zero
warnings); frequent conventional commits; phase status is tracked in `.doctrine/state/`
via `doctrine slice phase`, never in this file. The `doc/memories/` engine note
([../../../doc/memories/engine-identity-and-claim-seam.md](../../../doc/memories/engine-identity-and-claim-seam.md))
records D7/D8/D9 as *decided-not-built* — update it to *built* when PHASE-01 lands.
