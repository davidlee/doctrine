# ISS-241: MCP dispatch_conclude_phase skips the arm-neutral source-delta registry write

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Defect

The CLI verb `dispatch record-boundary` (`run_record_boundary`,
`src/dispatch.rs:1731`) deliberately does **two** writes:

1. the committed boundaries ledger on `dispatch/<NNN>`, and
2. "ALONGSIDE it (SL-147 PHASE-04, T3)" the **arm-neutral recorded source-delta
   registry**, resolved against the PRIMARY tree so a coordination worktree
   still writes the row the integrator reads.

The MCP tool `dispatch_conclude_phase` reuses only `run_record_boundary`'s
**pure `BoundaryRow` compute**, not its live-file write — so on the claude arm it
lands (1) and silently skips (2).

## Impact — a close-time gate that proves nothing while exiting 0

`slice verify-vt <id>` builds its `modified_files` set from that registry
(`src/slice.rs:838` → `state::read_source_deltas`). With the registry empty,
`vtgate::check_vt` (`src/vtgate.rs:124`) short-circuits **every** criterion to
`Unattributable` — which is *visible but non-halting* by design (INV-4). So the
gate **exits 0**.

The close ritual is "`slice verify-vt <id>` — HALT on Fail". A drive concluded
entirely through the MCP arm therefore reaches close with a gate that is green,
silent, and carrying zero signal — the exact failure mode the gate exists to
prevent.

Observed on SL-228 after three phases landed via `dispatch_conclude_phase`: all
8 landed criteria reported `Unattributable`, exit 0. After manually recording
the three deltas from the committed ledger with
`slice record-delta 228 PHASE-NN --start <code_start> --end <code_end>`, the
same gate reported **8/8 PASS**.

## Note on the recording mode

Only PHASE-03's boundary was a single commit (`code_end^ == code_start`), so the
safe `--commit <S>` mode does not generalise: a phase whose base carries
orchestrator-authored `.doctrine/` content spans multiple commits and needs the
`--start`/`--end` range. Whatever fix lands should record the boundary row it
already computed, not re-derive a single-commit patch.

## Fix direction

Have `dispatch_conclude_phase` perform the same `state::record_source_delta`
call alongside its committed boundary write, with the same F-6 guard + upsert
and the same primary-tree resolution. The row is already computed — this is a
missing second write, not new machinery.

Consider a guard so the two ledgers cannot drift silently: at
`dispatch sync --prepare-review` (or in `verify-vt` itself), refuse or warn when
a phase has a committed boundary row but no source-delta row.

## Relations

- **ISS-052** (closed) — the *pi/codex* arm's version of exactly this defect:
  "conformance-registry write never fires; SHAs stranded in the dispatch
  ledger". Same shape, other arm.
- **IMP-272** / **ISS-233** (resolved/closed) — the sibling coord→primary mirror
  gap for the per-phase `completed` flip, fixed by routing it through the single
  writer `set_phase_status`. This defect is the same class, unfixed, for the
  source-delta registry.
- **IMP-228** (closed) — introduced the source-delta intersection that makes the
  registry load-bearing for `verify-vt`.
- **ISS-226** (open) — a separate correctness bug in the `Unattributable`
  message itself.

## Provenance

Surfaced at SL-228 PHASE-03 handover (2026-07-26) while gathering the VT summary
the handover packet is required to embed.
