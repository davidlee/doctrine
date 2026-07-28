# Staging a whole module ahead of its consumer: three scopes, not one blanket

Reconciles [[mem.pattern.lint.dead-code-expect-vs-cfg-test]] (item-level
`cfg_attr(not(test), …)`) with [[mem.pattern.lint.dead-code-blanket-masks-siblings]]
(never blanket a module). Both are right; they answer different questions, and
the way to pick is to **measure**, not to reason from either one.

`expect` is fulfilled per **compilation**, and the two compilations differ:

- `cargo clippy` / `cargo build` (`cfg(test)` off) — a module staged a phase
  early is **entirely** dead.
- `cargo test` (`cfg(test)` on) — the suite makes most of it live. What stays
  dead is the interesting set, and it is usually **much smaller than the module**.

So: apply a throwaway `cfg_attr(not(test), expect(dead_code, reason="probe"))` at
module level, run `cargo test`, and read the error list. That list is the real
suppression surface.

## The three scopes

| scope | when |
|---|---|
| module-level `#![cfg_attr(not(test), expect(dead_code, …))]` | the non-test build, where *everything* is dead. Nothing it could mask is unexpected, and being stripped under `cfg(test)` is what preserves the "you added an item no test exercises" hard error. |
| file-level unconditional `#![expect(dead_code, …)]` | a submodule with **no test consumer at all** — e.g. an exit criterion fixes the suite at N named tests and this file is not among them. |
| per-item unconditional `#[expect(dead_code, …)]` | the individual items the suite does not reach. |

A single unconditional **module-level** blanket also compiles (it is fulfilled in
both builds while anything is dead) and is what you reach for first. Don't: it
swallows every later dead sibling in both builds, which is exactly the SL-059
failure.

## How to apply

- Probe first. On SL-233 PHASE-02 the module was ~1700 lines and 40-odd items;
  only **nine** were dead under `cfg(test)`. The blanket was ~4x coarser than the
  problem for zero extra safety.
- Prefer deleting to suppressing. Several probe hits are speculative accessors —
  a `matches!` wrapper with no consumer is surface, not design.
- Every gate stays self-clearing: the expectation fires only while something is
  dead, so a real consumer forces the attribute's removal.
- Residual trap on any *unconditional* expect: if a later change makes that item
  live in **both** builds it goes unfulfilled and must be removed — which is the
  intended signal, not a breakage.
- Precedent: `src/design_run/` (SL-233 PHASE-02, `5edf47338`).
