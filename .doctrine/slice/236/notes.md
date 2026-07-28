# Notes SL-236: Worker-guard honours explicit project root

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-29 · ABANDONED · 589147b6

**Slice retired.** The premise both candidate fixes shared was retracted; the
work split in two and left. Nothing here is actionable — follow the ids.

### Produced

- SL-236 — scope, `design.md`, `research/`, 7 `scope-relevant` + 32
  `design-target` selectors. `design.md` §7 (D3/A1/A2/A4 + cost table) and §10
  (F-8, F-10, F-13) hold measured evidence worth NOT re-deriving; §10 carries a
  STOP banner marking the guard premise retracted.
- DEC-093 — **rejected** (was `proposed`); retraction rationale appended in place
- RV-319 — design-facet ledger, raiser `codex-gpt-5.5`. F-1 verified terminal,
  F-2 answered (`design-wrong`)
- IMP-348 — the CLI-surface cleanup, carried forward as a prototype-for-a-slice
- ISS-028 — rewritten around the topological fix; both superseded diagnoses
  archived in place
- `mem.fact.clap.global-and-local-arg-share-id`
  (`mem_019fa8f1ade17f31822539fa80d778f4`)
- `mem.fact.dispatch.worker-confinement-is-actor-based`
  (`mem_019fa94118a37c33ab54b06dfe4b1131`)
- Tests landed green and kept: `tests/e2e_worker_guard_explicit_root.rs`,
  `tests/arg_path_convention.rs`, fixtures in `tests/common/mod.rs`

### Learned

- **RV-319 F-2 (the one that killed it)** — the worker marker identifies the
  ACTOR; every tree a worker must not write to is markerless. Any guard keyed to
  the `-p` target inverts confinement. Reproduced live.
- **RV-319 F-1** — a per-verb `-p` declaration is the machine-checkable record
  that the verb consumes a root; globalising destroys it and leaves four pathless
  guarded verbs steerable.
- Confinement is an **accident-fence, cooperative** (`marker --clear` needs
  `--operator` "to confirm you are the trusted orchestrator"). The test of a
  proposed change is not "does it grant new capability?" but "does it make a
  sanctioned bypass silent?"
- Most of ISS-028's symptom was **already mitigated** — the commit gate clears
  the marker before its run (SL-199 F2), so golden coverage survives; only the
  worker's manual run skips.
- `-p` repetition is **syntactic, not semantic** — 204 declarations each assert a
  different fact. Not a DRY target; the idiomatic remedy is a flattened
  `#[derive(Args)]` bundle (`CommonShowArgs` already exists, used at 5 sites).
- Help output cannot distinguish a swept from an unswept subcommand when the doc
  text matches — and D4 made 116 of 202 match by design (§10 F-10).
- `resolve_mode` requires `is_linked && marker_present` — a marker in a bare
  tempdir is never refused (drives the fixture rule).
- Research-agent reliability is uneven; see `research/raw/VERIFICATION.md`.

### Open

- **RV-319 F-2** — `answered`, not yet verified by the raiser. Blocker severity;
  gates nothing now the slice is abandoned, but the ledger loop is open.
- **ISS-028** — topological fix specified, not implemented.
- **IMP-348** — scoped, not designed.
- **ISS-267** — ~29-file residual; re-measure rather than assume the 19/10 split
  survives, since no guard change landed.
