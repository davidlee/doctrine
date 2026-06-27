# Implementation Plan SL-167: Accept prefixed canonical ids on all CLI id:u32 args

Prose companion to `plan.toml`.

## Overview

Single-phase: add `parse_ref` + `parse_cli_id` to the four governance modules (ADR, policy, standard, RFC), mirroring the already-committed `slice::parse_ref`/`parse_cli_id` pattern, and wire `#[arg(value_parser = parse_cli_id)]` on each `id: u32` field.

## Sequencing & Rationale

**PHASE-01:** All four modules are independent — no ordering constraints. Each gets:
1. A `parse_ref` function (strip canonical prefix, parse as u32)
2. A `parse_cli_id` clap wrapper
3. `#[arg(value_parser = parse_cli_id)]` on the `id: u32` field
4. A `parse_ref_accepts_prefixed_padded_and_bare_ids` test

## Notes

The slice.rs fix is already committed on edge. This phase completes the task.
