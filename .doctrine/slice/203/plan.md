# Implementation Plan SL-203: Dissolve commands↔mcp_server dependency cycle

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

One phase. SL-203 dissolves the `{commands, mcp_server}` 2-node dependency SCC by
inverting its single incidental back edge (`tools.rs:1365`) — supplying
`model_keys` as a `ModelKeysFn` fn-pointer through `McpConfig` rather than having
`mcp_server` reach up into `commands` (D-B, design §7). The forward edge
(`serve → mcp_server::serve`) is legitimate and untouched. Post-change
`mcp_server` is corpus-agnostic; both SCC edges leave the tangle count, so the
`command` baseline ratchets 123→121.

## Sequencing & Rationale

**Why a single phase.** The change is one atomic compile-unit: the fn-pointer
must be threaded through the whole onboard call chain *and* every `#[cfg(test)]`
`dispatch` caller in the same edit, or the crate does not compile. There is no
intermediate green state to split on, and the slice is a deliberate warm-up (R1 —
avoid sprawl). Splitting would manufacture ceremony without a reviewable
boundary.

**Execution order within PHASE-01 (TDD red→green→refactor).**
1. **Introduce the seam** — `ModelKeysFn` type + `model_keys` field on
   `McpConfig` (`mod.rs`); supply the real producer at `serve.rs:28` (fn-item →
   fn-ptr coercion). This unblocks the wiring guard.
2. **Write VT-2 red** — the wiring guard drives `render_onboard` through an
   injected `ModelKeysFn` and asserts a *known corpus key* renders as a bullet
   line. It is red until the render path consumes the injected pointer. Assert on
   **content**, never section-non-emptiness (the F-1 penance: the empty-corpus
   placeholder always makes the section non-empty — `tools.rs:1366-1375`).
3. **Thread the pointer green** — add the param to `dispatch`,
   `handle_tools_call`, `call_tool`, `render_onboard`,
   `render_model_band_guidance`; swap the static
   `crate::commands::prompt::model_keys(root, None)` call for `(model_keys)(root,
   None)`; add the arg to all ~13 `#[cfg(test)]` `dispatch` callers and thread it
   through the `memory_dispatch` helper (`tools.rs:1900`). VT-2 goes green.
4. **Ratchet + confirm** — set `[tangle_baseline] command = 121`
   (`layering.toml`); `architecture_layering` green at 121 (VT-1). Empirically
   confirm the −2 (optionally via the `#[ignore] dump_real_graph` diagnostic for
   SCC-membership evidence — the gate proves the count, membership is inferred;
   design §9 VT-1 / F-4).
5. **Refactor / verify** — `let model_keys = config.model_keys;` bound once before
   the serve loop (Copy; reads cleaner — design §10). Full suite green, onboard
   byte-identical (VA-1); `doctrine slice conformance SL-203` clean and
   `mcp_server` production grep shows zero `crate::commands`/`crate::install`
   (VA-2, INV-1).

**Do NOT touch** the `120` figures in `architecture_layering.rs` (lines 8, 22) —
they are historical SL-112 `PHASE-01 go/no-go` report prose, not the live
baseline (F-2 penance, design §8 R3). The only ratchet edit is `layering.toml`.

## Notes

Inquisition trail: RV-252 (design facet) reconciled four findings into `design.md`
before this plan. The plan carries the F-1/F-3/F-4 penances into verification
(VT-2 content assertion; ~14-signature edit surface; VT-1 count-vs-membership
distinction) and the F-2 penance into an explicit non-goal above.
