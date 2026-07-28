# Review RV-319 — design of SL-236

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject under interrogation: the fix direction, and only the fix direction.**
SL-236's diagnosis (the worker guard resolves the project root from CWD, not from
the tree the command operates on) is settled and evidenced. This review does not
re-open it. What is open is *how* to fix it, recorded as DEC-093 (status
`proposed`) and contested at design §7 A4 / §10 F-13.

The two candidates:

- **D3** — make `-p/--path` a `global = true` arg on `Cli` and delete the 204
  per-subcommand declarations across 27 files. Buys a **structural** invariant
  (one declaration, compiler-visible) and unifies two real surface
  inconsistencies. Costs a large mechanical sweep, ~40+ golden updates, and
  leaves R3 (a half-swept tree compiles and passes every behavioural test) as the
  primary risk, guarded only by VT-s.
- **A4** — leave all 204 declarations alone; have `worker_guard` read the
  explicit path out of `ArgMatches` by arg id, walking to the deepest subcommand.
  ~10 lines, one call site. Dissolves R3 and OQ-1 outright. Buys only a
  **conventional** invariant: every `-p` must stay named `path` and typed
  `Option<PathBuf>`, enforced by nothing.

### Lines of attack

1. **Is A4 actually sound, or does the spike flatter it?** It was measured on an
   isolated clap 4.6.1 project mirroring doctrine's shapes, not on doctrine
   itself. Look for a real shape the spike did not model — subcommand-level
   `global` args, `ArgGroup`, `default_value`, `value_parser` overrides, an arg
   named `path` that is *not* a project root, or a `-p` reachable only through a
   flatten-within-flatten. A single counter-example is decisive.
2. **Is the structural-vs-conventional trade weighed correctly?** D3's real
   product is a permanent invariant; A4's real cost is a silent-miss failure mode
   (a future field named `project_root` goes unguarded) of the *same class* as
   R3, merely relocated and cheaper to test. Which risk is actually worse over
   the life of the codebase? Argue it, do not assert it.
3. **Does the F-8 retraction undercut D3 more than the record admits?** D3 was
   taken partly on "the compiler enumerates the work list". That was measured
   false and retracted; VT-s replaced it. DEC-093 states the decision is
   unchanged. Is that defensible, or did the conclusion survive the death of its
   premise by inertia?
4. **Scope coupling.** The slice is *"Worker-guard honours explicit project
   root."* D3 bundles a CLI-surface refactor into a guard bug fix. Is splitting
   them (fix direct from ISS-028; park the refactor) sound, or does the split
   create a worse intermediate state than either endpoint?
5. **The VT contract under each option.** VT-a…VT-d are claimed to hold unchanged
   under both, with only VT-s D3-specific. Verify that claim. If A4 wins, what
   replaces VT-s, and is a convention scan genuinely sufficient?
6. **Anything neither option covers.** F-4 records an accepted, untested semantic
   shift in `worktree fork`'s guard behaviour. Does it behave differently under
   A4 than under D3?

### Invariants the subject must be held to

- ADR-006 D2a worker-mode formula `(is_linked_worktree && marker_present) OR env`
  — this slice fixes *which tree*, never the test.
- SPEC-012 REQ-192 — `write_class` exhaustive; "a new verb is a compile error"
  must not degrade into a discipline concern.
- ADR-001 / POL-002 — confinement policy stays out of the leaf layer.
- Guard laziness — a Read verb in a rootless CWD gains no new failure path.
- Env leg stays root-independent.
- Behaviour preservation — existing suites green unchanged; help goldens are the
  sole sanctioned churn.

### Pre-reading

`.doctrine/slice/236/design.md` — **§10 review notes first** (F-1 is superseded;
F-8, F-10, F-13 carry the corrections), then §5.2, §7, §9.
`.doctrine/slice/236/slice-236.md` — scope, R1 retracted, R5/R6 settled.
`DEC-093` — the decision record and its corrected rationale.
Code: `src/commands/guard.rs:473`, `src/main.rs:107-114,217,235-242`,
`src/root.rs:22-25`, `src/worktree/marker.rs:161-166`,
`src/commands/cli.rs:462-465`.

**Standing instruction:** a finding that merely restates a §10 disposition is
noise. Raise what §10 got wrong, or what it never considered.
