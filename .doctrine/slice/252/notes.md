# Notes SL-252: Conformance fixture readable-input posture

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design surface triage (exploring, 2026-08-10)

Evidence: `research/research.md` (✓ rows verified this session). Nothing below
restates it — this is the shaping layer over it.

### Constraining governance

- `SL-248` `PHASE-05` `EX-8` — no package store ever bound whole; no
  host-shaped default, no fallback. The standard being applied, not a question.
- `SL-248` `PHASE-05` `EX-10` — closure members bind individually, never their
  common parent. The target shape.
- `SL-248` `PHASE-09` `EX-3` — row 2 executes rather than stats, from every
  bound path. Not to be weakened.
- `REQ-459` / `REV-051` — the shortfall this slice discharges is authored spec
  text, so closing it is visible at reconcile.
- `POL-002` facet (3) — a host tool may be acquired, never silently: opt-in,
  and absent ⇒ fail naming what is missing and what would satisfy it.
- **The fixture itself is ungoverned at this granularity.** No authored rule
  keeps the readable set narrow once it is narrowed.

### Shaping decisions (open — for `/design`)

- **D-a. The unit of the readable set.** Resolved `PATH` entry directories, the
  closure of an enumerated payload toolset, or a hybrid. `ISS-341`'s stated
  direction (closure of the shell) is insufficient: sixteen external binaries
  are needed, `git` and `socat` among them.
- **D-b. Who computes the closure.** An operator-style external resolver
  (host-tool dependency, POL-002-declarable), a doctrine-owned enumerator (no
  precedent in the repo — no ELF/`ldd`/maps machinery exists), or no closure at
  all under a different unit for D-a.
- **D-c. What a host with no resolver gets.** `Unavailable { missing, remedy }`
  before any row runs (POL-002-clean, costs the whole run), or a narrowed
  non-closure fallback (keeps the run, risks reinstating the defect).
- **D-d. Where the posture is reported.** `Unrowed` + `Reading` is verdict-free
  and already rendered; the only friction is that it is currently
  credential-shaped.
- **D-e. What guards the narrowing.** `no_configuration_makes_a_host_wide_store_readable_whole`
  (`bubblewrap.rs:1870`) is the production precedent to mirror fixture-side.

### Risks

- **R-1. The crown jewel.** `the_shipped_backend_is_admitted_on_this_host` is an
  unconditional nineteen-row admission. Any narrowing that starves one payload
  turns a row indeterminate and fails it — in-jail, where all development
  happens.
- **R-2. The inner `PATH` collapse.** `derived_inner_path` keeps only host
  `PATH` entries lying *beneath* a bound path. A store-path-granular readable
  set empties the inner `PATH` entirely.
- **R-3. Symlink farms.** `bwrap` dereferences a bind's source; a NixOS `PATH`
  entry is a farm of links into other store paths. Binding the directory alone
  yields dangling links inside the capsule — `ISS-339` run 1 generalised.
- **R-4. `ISS-340`'s dissolution is shape-dependent.** It follows from a
  root-granular narrowing, not from a store-path-granular one. Must be
  re-derived against whichever shape lands, not assumed.
- **R-5. Development happens in the one environment that masks all of this.**
  Every accident this issue family records was invisible in the jail.

### Assumptions

- **A-1.** The jail's own `/nix` bind stays available to the fixture; the jail
  has no `nix` binary (`mem.fact.jail.nix-absent-no-flake-eval`), so a
  `nix-store` resolver cannot be the in-jail path.
- **A-2.** Production's `readable_set` is correct and is the reference; this
  slice moves the fixture toward it and does not touch it.

### Open questions

- **Q-1.** Scope of the red release gate: `nix build` runs the workspace tests
  and nine conformance tests fail unjailed. At most three sit in this slice's
  neighbourhood; four are process/namespace rows and two are unclassified. Does
  SL-252 own the rest, or does a sibling item?
- **Q-2.** If D-c lands on `Unavailable`, `ISS-339`'s off-jail admission stops
  being reachable on a host without a resolver. Is that acceptable?

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open
