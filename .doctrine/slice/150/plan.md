# Implementation Plan SL-150: Family-grouped help + boot-map projection

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two phases, split at the natural seam: **PHASE-01 builds the shared taxonomy and
the human-facing surface**; **PHASE-02 reuses that taxonomy for the boot
projection**. The drift-guard test lands in PHASE-01 because it guards the
`FAMILIES` table both phases consume — the foundation is asserted before the
boot map is built on it.

## Sequencing & Rationale

**Why PHASE-01 first.** `FAMILIES` + `SPINE` are the one hand-maintained source
everything derives from; nothing renders correctly until they exist and are
drift-guarded. The human help is the harder rendering (bands, shared widths,
the DRY lift from `search.rs`), so it absorbs the comfy-table risk early. The
DRY lift (full-width-band primitive + `is_table_row_start` → shared `listing`
helpers) is sequenced *before* `render_grouped` because the latter consumes it;
search's existing snippet tests are the behaviour-preservation proof and must
stay green across the move (VT-3) — red/green/refactor with the suite as the
gate.

**Why PHASE-02 second.** `render_boot_map` is a pure projection over the same
clap tree + `FAMILIES`, so it can only be written once PHASE-01 lands. It is the
lower-risk phase (plain text, no comfy-table, no color), but it touches the
**boot snapshot contract**, so its verification weight is on determinism: the
byte-stability pair (VT-2), the ordering invariant (VT-3, which also pins that
the existing ADR→Policy→Standard adjacency survives the insertion), and the
`produce(CommandMap)` arm (VT-4). The `--boot-map` flag gives the golden (VT-1) a
black-box target independent of the snapshot file.

**File seams.** PHASE-01: `cli.rs` (FAMILIES/SPINE/render_top_level_help),
`listing.rs` (render_grouped + lifted band primitive), `search.rs` (refactor
onto the primitive). PHASE-02: `cli.rs` (render_boot_map), `main.rs` (--boot-map
arm), `boot.rs` (CommandMap section). The only shared file is `cli.rs`, in
disjoint functions — serial execution, no conflict.

## Notes

- Verification traces design §9: VT goldens are black-box via
  `CARGO_BIN_EXE_doctrine` + `force_no_tty` (color off ⇒ deterministic); the
  color path is a separate VA smoke (escape codes are not byte-golden-able).
- The "~20 line" boot-map budget (slice scope) is an estimate confirmed by the
  PHASE-02 golden, not a hard gate.
- IMP-135 (CLI help consistency pass) remains a follow-up; this plan does not
  touch command descriptions beyond what grouping requires.
