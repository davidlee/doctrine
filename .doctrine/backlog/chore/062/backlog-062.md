# CHR-062: Prune the SL-116 extraction expect(unused) in worktree gc/import

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`src/worktree/gc.rs:1` carries a module-wide suppression:

```rust
#![expect(unused, reason = "extraction; PHASE-03 prunes")]
```

left from the `SL-116` PHASE-02 extraction of the worktree machinery out of
`mod.rs`. The promised PHASE-03 prune did not happen, and the blanket now masks
real dead imports.

## Measured (SL-254 design, 2026-08-13)

`gc.rs` and `import.rs` each import three marker symbols and use **none** of
them:

```rust
use super::marker::{DISPATCH_WORKER_AGENT_TYPE, marker_present, write_marker};
```

`land.rs` imports the same three and uses only `marker_present` (`land.rs:173`).
`import.rs`, `land.rs` and `gc.rs` also share byte-identical `use super::allowlist::{…}`
and `use super::shared::{…}` blocks, so the dead surface is probably wider than
the marker symbols alone.

## Why it is worth a chore rather than a shrug

A blanket `expect(unused)` is invisible to `clippy` at zero warnings, so nothing
ever goes red. Its cost is paid by readers: a census of "which modules touch the
marker" over-reported by two whole files during `SL-254`'s design trace, because
an import reads as a use. Any future impact analysis over these modules inherits
the same inflation.

## Scope

- Drop the unused imports in `gc.rs`, `import.rs`, `land.rs`.
- Remove the module-wide `#![expect(unused, …)]` from every module that carries
  it, and let anything genuinely still-unused surface as a scoped, reasoned
  `expect` on the item itself.
- If a module cannot yet lose the blanket, say why in the `reason` string —
  "PHASE-03 prunes" names a phase that is long gone.

## Relations

- Sibling of `IMP-146` (SL-116: distribute remaining lifecycle tests per D2) —
  the same extraction's other unfinished tail.
- Surfaced by `SL-254`'s marker/confinement trace; `DEC-207` records the census
  correction in its consequences. `SL-254` may delete the marker imports outright
  as a side effect, which would narrow but not close this item — the blanket
  suppression and the allowlist/shared import blocks are the real subject.
