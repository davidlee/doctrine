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
fresh-as-of: 2026-08-11 · design/exploring · rev 26

**The design turned over on 2026-08-11.** The narrowing this slice was scoped to
build is blocked by a production defect (`ISS-344`), the owner invoked
`DEC-185`'s S3 fallback, and the programme itself is now under an architectural
question (`RSK-231`). Read `DEC-188` before anything else here.

### Produced

- `DEC-184` — scope: SL-252 stops at its own narrowing.
- `DEC-185` — the bar is representativeness, not production parity. Amended
  with the owner's standing permission to relent to S3 on cost. **Invoked.**
- `DEC-186` — the closure resolver is a self-written loader-trace script.
  **Withdrawn from SL-252**, on two independent grounds: S3 declares no closure
  roots and no resolver, *and* external review refuted the mechanism (the
  pathless-line rule silently converts a missing dependency into a successful
  closure; the trace is a diagnostic format needing a parser; the `ldd`
  equivalence claim is unverified; the loader behaviour is a `POL-002` host
  capability after all; the safety check runs after the target has executed).
  What survives is carried by `IMP-425`.
- `DEC-187` — no inner `PATH`, reach discovered from `/proc/self/mountinfo`.
  **Readable-set half withdrawn**; the mountinfo half stands but is demoted to
  an improvement, not a dependency.
- `DEC-188` — S3: the readable set is declared rather than host-derived, stays
  wide, and the posture is stated in the transcript. Does **not** close
  `REV-051`'s partial against `REQ-459` criterion 2, and says so.
- `ISS-342` — the six live-`bwrap` tests that keep `just gate` red off-jail.
- `ISS-343` — `doctrine-control` cannot build on macOS; release-blocking.
- `ISS-344` — **production**: canonicalized readable roots lose the declared
  name. Carries the authored `SL-248` `PHASE-05` `EX-3`/`EX-9` contradiction and
  the bare-`git` provisioning bootstrap as probably the same repair.
- `RSK-231` — the capsule programme has exceeded its complexity budget; needs
  architectural revision, not another remediation slice.
- `IMP-425` — production closure expansion still has no live exercise; what
  `DEC-186` was for, carried forward without its mechanism.
- `research/research.md` + `raw/` — the pre-design round (runtime tier).
- `mem.fact.linker.ld-trace-loaded-objects-is-the-closure`
- `mem.pattern.capsule.jail-and-host-path-shapes-differ`
- `mem.fact.capsule.closure-resolver-contract`
- `mem.fact.capsule.resolved-path-bind-breaks-multicall-dispatch`
- Correction appended to `mem.pattern.sandbox.readable-roots-are-top-level-ancestors`
  (trust high → low), which taught the defect `ISS-341` names.
- Two friction observations: a text-only node amendment invalidates
  `graph-reviewed`; and `doctrine design apply` reported a payload as
  unparseable JSON and applied it anyway, then locked the correction out on the
  idempotency receipt (record `2f/019fec3e`).

### Learned

Three facts that would otherwise be re-derived, all measured rather than read:

1. **The loader gives a binary's closure for free** — 16 store paths where the
   ancestor rule bound 691.
2. **The jail's `$PATH` shape cannot exercise the host's `$PATH` bugs**, and the
   jail's `/bin/sh` is a *bind mount of bash, not a symlink*, which is why
   `a_single_file_readable_root_is_bound_as_declared` models `/bin/sh → /bin/sh`
   and looks correct to anyone working here.
3. **Binding at the resolved path destroys `argv[0]`**, so a file-granular
   readable set cannot invoke a multicall binary at all. This is what killed the
   narrowing.

The meta-lesson, and the reason for `RSK-231`: four defects in this family
(`ISS-339`, `ISS-340`, `ISS-341`, `ISS-344`) are one pattern — a derivation
accidentally correct in the only environment it has ever run in — and the design
surface is large enough that finding the fifth costs a context window.

### Open

- `inq-5` (host with no usable resolver) **dissolves under `DEC-188`** — S3
  declares no resolver. Dispose it rather than answer it.
- `inq-11` (row 2's coverage derivation with no inner `PATH`) is **no longer
  forced** — under S3 the inner `PATH` populates on and off jail. Keep the
  mountinfo idea as an improvement item; it is better than the `$PATH` proxy
  under any shape.
- `inq-8` (does `ISS-340` dissolve) is now answerable *yes*: S3 is root-granular,
  which is the condition the scope's prediction depended on.
- `inq-3`, `inq-6`, `inq-7` — still live and now cheap. `inq-6` (what the
  transcript says) is the one `DEC-188` load-bears on.
- `permitted_root_entries` off-jail `UNEXPECTED-bin` failure is a live bug S3
  must still fix; it is coupled to `top_level_ancestor(SHELL)` under every shape.
- Graph re-review will need re-confirming after the next node change; it has
  been invalidated three times this session by ordinary edits.
