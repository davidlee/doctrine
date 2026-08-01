# Notes SL-241: Capsule spike rig

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-01 · **PHASE-03 complete** (3/6), slice `started`
(PHASE-01 code tip 29c7acf3; PHASE-02 code range b3ad3eed3..d041f6b39, 4 commits,
**none foreign** — recorded via `record-delta` after a post-flip fix, see
§ PHASE-02 boundary; PHASE-03 range acc4b2b34..HEAD, **one foreign interior
commit**, see § PHASE-03 boundary)

### Produced

- design.md written, internally reviewed, externally inquisited — no code yet
  (commits 9d0c852f, 3966256a, eedc26b8, 7fd55d0f)
- RV-340 — 14 findings, all disposed `fix-now`; **all 14 terminal, review done**
- minted: QUE-202 — capsule-model conflict admission path
- amended both tiers: DEC-099 (Amendment 2 + facet), QUE-201, ASM-007
- IDE-009 — third lint canary appended (wholly-empty `[facet]`)
- plan.toml + plan.md — six phases, build/evidence split (4b495982, 2216769f,
  cbce7876, 2dc35d4d); six runtime sheets materialised (2f521a25)
- lifecycle: proposed → design → plan → ready (2626072e); `ready` is ADR-009's
  approved-plan gate, taken on operator approval
- `.gitignore` added as a design-target selector (design § 9.1)
- PHASE-01 runtime sheet expanded — 9 tasks, gitignored, not a durable artifact
- gate: no `src/` change this unit; `doctrine validate` clean at each commit

**PHASE-01 (complete, 29d842dc → 29c7acf3, 8 commits):**

- `scripts/spike-capsule/{rig,lib/common.sh}` — entry point + shared library;
  `guard_not_real_repo` (I6), `rig_enter`, `rig_capsule_root`, assertion helpers
- `control/fixture-{heavy,light}.sh` — both BASE fixtures, provisioned and
  asserting (EX-11: variants F2/F3 deliberately not built)
- `fixtures/{heavy,light}/**` — authored fixture sources and both declarations
- `control/probe-r2.sh` — the R2 probe, re-runnable, 9 asserted edges
- `.doctrine/slice/241/fixtures.md` — the build sheet, F1–F5 (EX-10)
- `.gitignore` — raw-log entry (EX-7, OQ-1 v0)
- `flake.nix` — shellcheck added to the jail (D-P01-4)
- VT-1, VT-2 PASS; `doctrine validate` clean

### PHASE-01 decisions (durable)

- **D-P01-2** — `guard_not_real_repo` refuses **containment**, not just EX-2's
  equality. Equality alone leaves `<repo>/scripts/spike-capsule/fixtures` open,
  which is the exact failure I6 names.
- **D-P01-3** — `rig selftest` degrades to the I6 guard probe until PHASE-03's
  `control/selftest.sh` exists; the arm dispatches to it the moment it does.
- **D-P01-4** — shellcheck 0.11.0 added to the jail before any rig shell was
  written. The slice is six phases of shell and `bash -n` catches syntax only.
  Gate: `shellcheck -x -S style`. shfmt declined (cosmetic; no convention).
- **D-P01-5** — the R2 probe is a committed re-runnable script, not a session
  transcript. The sheet asked for edges 3–6 "in a form PHASE-05 can reuse", and
  a script is that form.

### PHASE-01 findings (durable)

- **F-P01-1 — a guard whose refusal cannot reach the entry point is not a
  guard.** `guard_not_real_repo` refuses by `exit`. Called as
  `rig_dispatch … "$(rig_enter)"` the exit ended only the substitution's
  subshell: refusal printed, substitution empty, **rig dispatched anyway**.
  `set -euo pipefail` does not catch it — a failed command substitution
  propagates in *assignment* position, never in *argument* position. The unit
  probe was green throughout, because it subshells the guard deliberately; only
  the entry-point observation caught it. Fixed by publishing `RIG_ROOT` and a
  `BASHPID == $$` tripwire. → memory `mem.pattern.shell.guard-exit-swallowed-by-command-substitution`
- **F-P01-2 — five interpretation hazards noticed while authoring the light
  declaration**, recorded in the phase sheet and deliberately kept OUT of
  `fixtures/light/interpretation-surface.txt`. **PHASE-04 step 0 must not read
  that list before it enumerates** — step 0's independence is the whole of
  ASM-007's falsification value.
- **F-P01-3 — "declares the script" is not "the script works".** Five npm-script
  assertions read out of `package.json` passed while `build` and `lint` were
  both broken. Replaced with five that RUN each script.
- **F-P01-4 — `doctrine install` runs clean on a non-Rust project.** A-install
  holds; no POL-002 finding. First non-Rust exercise of the independence claim
  in the corpus. One non-fatal advisory: reservation reach degraded to local.

### PHASE-02 decisions (durable)

- **D-P02-1 — the confinement floor is an ALLOWLIST, not `--ro-bind / /`.** The
  seed (`pi-spawn-confined.sh`, ADR-008 D-B3) opens with `--ro-bind / /`:
  everything readable, only writes denied. That is a *writability* floor and
  EX-1 needs a *visibility* one — the canonical repo, other capsules, and git
  credentials must be **ABSENT, not ro**. Absent is achieved by not binding,
  which only an allowlist can express. Copying the seed's floor would read as
  "confined" while silently failing EX-1 and leaving VA-3 unassertable. What
  transfers from the seed is the nesting mechanics and the fail-closed
  empty-argv guard.
- **D-P02-2 — the runners mount at `/rig`, outside the writable root.** I4a
  becomes structural rather than asserted: no writable path lies on the
  runner's mount path at all. Inner paths are fixed (`/capsule`, `/rig`,
  `/agent`, `/source`) rather than the seed's `--bind "$D" "$D"`, which would
  reproduce the host path and read every sibling capsule's parent into
  existence as a mountpoint chain.
- **D-P02-3 — the doorbell waiter is `rig_wait_doorbell` in `lib/common.sh`**,
  not a control script. PHASE-03's pipeline is its only other consumer and the
  design names no file for it. Same class of placement choice as
  `control/selftest.sh` (plan.md § *One placement decision beyond the design's
  tree*).
- **D-P02-4 — the bounds are enforced TRUSTED-SIDE**: `timeout` outside the
  bwrap exec, `ulimit -f` set before it and inherited through the namespace,
  `du` after it. Nothing the capsule can unset. They emit distinguishable
  STATUSES (`RIG_EXIT_DISK`, `RIG_EXIT_TIMEOUT`, `RIG_EXIT_SANDBOX`) and **no
  tokens** — `verify-timeout` / `harvest/resource-cap` are trusted-side-computed
  in PHASE-03's pipeline (I5, plan.md § Notes item 2).
- **D-P02-5 — one profile, parameterised.** Capsule kind selects command,
  bounds, and network; never a profile. `sandbox.sh --print-mounts` emits the
  mount posture ALONE, which is what makes EX-2's uniform-confinement claim
  checkable rather than asserted — the two kinds' output compares equal.
- **D-P02-6 — `control/probe-capsule.sh` is PHASE-02's observation harness**
  (posture / bounds / doorbell). PHASE-04's dispatched `probe-c2.sh` builds on
  it; this is the capsule side's own red/green. `probe-r2.sh` is the precedent
  for a committed re-runnable probe over a session transcript (D-P01-5).
- **D-P02-7 — the disk cap is cumulative over the capsule tree.** `ulimit -f`
  is per-file and a capsule writing many small files walks past it, so the
  `du` leg is the one that catches that. Kept deliberately: the first bounds
  run reddened its own positive control on residue from the previous case,
  which is the bound working, not a defect.

### PHASE-02 findings (durable)

- **F-P02-1 — a must-fail assertion passes vacuously when its subject is
  absent.** `/rig` read-onlyness was asserted by appending to `/rig/verify.sh`
  and scored **green while that file did not yet exist** — the append failed
  for the wrong reason. Fixed by requiring the subject to resolve first. Same
  shape as F-P01-3, and precisely the DQ-3 trap VA-3 names: an absence-shaped
  assertion needs its subject proven present before its refusal means anything.
- **F-P02-2 — RESOLVES is not RUNS.** All three runners were readable inside
  the sandbox and **none could execute**: `#!/usr/bin/env bash` needs
  `/usr/bin/env` in the allowlist, and the kernel resolves the shebang before
  PATH exists. `execvp` reports the missing *interpreter* as `No such file or
  directory` naming the *script*, so the error points at the file that is
  present. The readability assertions were all green throughout. The profile
  now binds `/usr/bin/env`, the probe asserts a runner actually executes, and
  `sandbox.sh` maps exit 127 to `RIG_EXIT_SANDBOX` so "the runner refused" and
  "the runner never ran" cannot read identically in a matrix cell.
  → memory `mem.pattern.shell.shebang-interpreter-is-a-mount-dependency`
- **F-P02-3 — `rig_assert '…' ! cmd` cannot express a refusal.** `!` is a shell
  keyword, so under `"$@"` it arrives as the *command name*; the assertion reds
  on the invocation and never scores its subject. The I4a positive control was
  correct throughout and looked broken. Added `rig_assert_fails` to the
  library — every positive control in PHASE-04/05 wants it. Third member of the
  F-P01-1 family: in shell, the *invocation form* silently changes the
  semantics of a guard or an assertion.
- **F-P02-4 — A2 holds on both legs; R3's credential-proxy branch is not
  taken.** `npm ping` reaches the registry and `claude -p 'print OK'` answers
  **OK** from inside the nested bwrap with the agent home bound **read-only**.
  So the jail's `~/.claude` credential arrangement survives confinement without
  a writable home — R-1 did not bite. Scoped to Linux/bwrap.
- **F-P02-6 — the doorbell's name was three bare literals, and nothing joined
  them.** `common.sh` held the constant, `worker-stub.sh` its own copy (it runs
  inside the sandbox and cannot source the library — structural, not careless),
  and `probe-capsule.sh` a third. The probe planted its own bell and observed
  the waiter finding it, so **it would have stayed green while the worker rang a
  differently-named file**: both halves observed, the join between them merely
  inferred. Fixed by passing the name in as `--setenv RIG_DOORBELL`, read
  FAIL-CLOSED by the ringer (`:?`, never a default — a default silently restores
  the drift), plus a live assertion that runs the real worker and waits on the
  real bell. **Verified falsifiable**: drifting the ringer's name reds it.
  Fourth member of the F-P02-1/2 family and a STD-001 violation besides.
- **F-P02-5 — the shellcheck gate needs a UTF-8 locale.** `shellcheck -x -S
  style` aborts with `commitBuffer: invalid argument (cannot encode character
  '\8212')` on any file containing an em-dash, which is every rig file. It is
  an output-encoding abort, not a finding — no line, no rule id — so it reads
  as a lint failure and invites hunting a defect that is not there. The gate is
  `LC_ALL=C.UTF-8 shellcheck -x -S style`. Observation recorded (RFC-011).

### PHASE-02 evidence (what was OBSERVED, not merely coded)

| criterion | observation |
|---|---|
| EX-6 / A1 | nested bwrap exits 0 in-jail under an **allowlist** floor; `id` reports a real uid mapping |
| EX-1 / VA-3 | canonical repo, capsule root, `~/.ssh`, `~/.gitconfig`, credential helper all **do not resolve** inside; `/nix/store` **does** (positive control) |
| EX-2 | `--print-mounts` compares **equal** across `--kind worker` and `--kind verify` |
| EX-4 / VA-2 | `audit-i4a.sh --positive-control`: planted `cp` → audit **REFUSES**; removed → audit **PASSES** |
| EX-3 / VA-4 | hung run killed at 3s (status 124) · 64 MiB write refused against an 8 MiB cap (status 4) · **both observed on both kinds** · each with a positive control |
| EX-5 | no ring → wait ends at its deadline (never hangs) · a ring naming another capsule is accepted as a bare signal · duplicate ring is a no-op · identity echoed is the **argument**, not the file · **live join**: the waiter hears the bell the real worker rang, with a negative control before it and a drift check proving it falsifiable |
| EX-7 / VA-1 | two assertions, recorded separately in `probes/smoke/results.tsv`: `npm ping` pass, `claude -p` pass |
| EX-8 | full cycle on the light fixture: provision (ro-bind toolchain, no `nix`/`direnv`) → worker commit + ring → `npm test` green in the verify capsule |

Raw results: `$SPIKE_CAPSULE_ROOT/probes/smoke/results.tsv`. Re-runnable:
`rig smoke`, `control/probe-capsule.sh [posture|bounds|doorbell]`,
`control/audit-i4a.sh <capsule> [--positive-control]`.

**PHASE-03 (complete, acc4b2b34 → code tip, 6 own commits):**

- `control/pipeline.sh` — the four stages, the closed vocabulary, `stage_emit`,
  `assert_outcome`, the per-run repo trio. Sourceable and executable.
- `control/harvest-fetch.sh` (M-A) / `control/harvest-bundle.sh` (M-B) — one
  signature, so the matrix loops without branching
- `control/selftest.sh` — VT-4: happy · attribution · bundle · falsifiable
- `capsule/sandbox.sh` — SIGXFSZ classified (F-P03-1); two more names crossed
- `capsule/worker-stub.sh` — bundle emission pre-doorbell; stub path passed in
- `control/probe-capsule.sh` — the vacuous ABSENT legs repaired (F-P03-2)
- VT-1..VT-4 PASS; `doctrine validate` clean; shellcheck clean on every rig file

### PHASE-03 decisions (durable)

- **D-P03-1 — the pipeline's canonical is a PER-RUN DISPOSABLE CLONE of the
  fixture.** Design § 5.1 names canonical / quarantine / capsule as three
  zones; `fixtures.md` § *Not fixtures* makes only the quarantine per-run. But
  stage 4 advances an accepted ref and `assert_outcome` asserts canonical is
  byte-identical across a refused run — neither is expressible against the
  pristine base fixture that PHASE-05 also reads. So canonical is instantiated
  per run from F1 and the fixture stays a TEMPLATE. Same class of placement
  choice as D-P02-3.
- **D-P03-2 — the quarantine is CLONED FROM CANONICAL, not `init`'d empty.** It
  must satisfy two things at once: hold S's objects for the conform legs, and
  give `slice conformance -p` a `.doctrine/` registry to read. A clone at B
  does both while remaining a real, separate, per-run, disposable repository —
  all EX-2 requires. Its worktree sits at B, which is NOT the candidate, so
  I4's "no candidate tree materialised trusted-side" holds; every leg is
  plumbing over `B..S`. `--no-hardlinks` so a corrupt object cannot reach
  canonical through shared object files.
- **D-P03-3 — SIGXFSZ is classified as the disk bound.** `sandbox.sh` sets
  `ulimit -f` itself, so status 153 inside the namespace has exactly one cause.
  Folding it into `RIG_EXIT_DISK` is what lets both disk legs map to the single
  token `harvest/resource-cap` rather than mint a second.
- **D-P03-4 — the accepted ref is READ from the fixture (`symbolic-ref HEAD`),
  never hardcoded.** F1's trunk is `mainline` precisely so anything assuming
  `main` breaks loudly (D5); a pipeline that hardcoded it would pass for a
  reason that says nothing about portability.
- **D-P03-5 — the stub's write path is a CONTROL-PLANE parameter, carried in
  the contract.** Where the worker writes is a join with the slice's
  `design-target` selectors: the happy path needs it declared, the hostile rows
  need it undeclared and forbidden. Burying it in the worker would make every
  happy run refuse at conform leg 2 for a reason about the rig's scaffolding
  rather than about the model.

### PHASE-03 findings (durable)

- **F-P03-1 — the disk cap had two enforcement legs and only one REPORTED.**
  `du` is cumulative and trusted-side; `ulimit -f` is per-file and fires as
  SIGXFSZ, which reached the parent as a raw status `sandbox.sh` never
  classified. PHASE-02's own case did not separate them: it passes because `du`
  reports 8392704 B against a cap of 8388608 B — **one 4096-byte block of
  accounting slop** — so the 64 MiB overshoot its comment relies on is doing no
  work at all (a 64 KiB overshoot is byte-identical). A **sparse** oversize
  separates them: `ulimit -f` fires, the tree stays at 4096 B, `du` has nothing
  to say, and **`sandbox.sh` exited 0** — the bound bit and the sandbox called
  it a pass. Fixed by D-P03-3; the separating leg is now in `probe-capsule.sh`.
  The units themselves were correct, and would have red loudly if wrong.
- **F-P03-2 — two of five ABSENT legs were vacuous, and one carried a second
  vacuity route.** `~/.ssh` and the credential-helper leg both passed
  **unconfined**: `~/.ssh` is hidden by the OUTER jail (it exists on the host —
  operator correction), and no `credential.helper` is configured here at all.
  The helper leg also passes if `git` is not on the sandbox allowlist, since
  the substitution simply comes back empty. Repaired with `absent_inside`,
  which proves the subject reachable OUTSIDE before asserting it does not
  resolve INSIDE, and records `n/a` **with its reason** where it is not. The
  gate is *reachability from where the probe stands*, which makes the leg
  environment-conditional by construction: in-jail `~/.ssh` is `n/a`; on a HOST
  run the same leg becomes load-bearing with no edit (PHASE-01 EX-9's
  discipline applied to an assertion rather than a measurement). `~/.ssh` is
  replaced as the load-bearing subject by the host home ROOT, which carries the
  general claim and certainly exists.
- **F-P03-3 — a `verify:` command was WORD-SPLIT, and a stage that should have
  refused SILENTLY ATTESTED.** `node -e "process.exit(1)"` split naively
  reaches node as three words — `node`, `-e`, `"process.exit(1)"` — with the
  quotes as LITERAL CHARACTERS. Node evaluates that as a harmless string
  expression and exits 0, so the verify capsule attested a run that should have
  failed. Found only because an attribution scenario built to red came back
  green. Fixed with `sh -c`, which still execs a single command so I4 is
  untouched. **The design's own examples (`npm test`, `cargo test`) have no
  quoting, which is exactly why this would have survived to PHASE-06** and
  first bitten on a real client declaration.
- **F-P03-4 — an F-P01-1 in this phase's own code, caught before it ran.**
  `pipeline_setup` calls `guard_not_real_repo` (which refuses by `exit`) while
  returning the run dir on stdout — forcing every caller into `$( … )`, where
  that exit ends only the subshell. Fixed by PUBLISHING `PIPELINE_RUN` rather
  than printing it, the same remedy PHASE-01 applied to `rig_enter`/`RIG_ROOT`.
  Fifth member of the family, and the first found by *rereading for the shape*
  rather than by observing a failure.

### PHASE-03 evidence (what was OBSERVED, not merely coded)

| criterion | observation |
|---|---|
| EX-11 / VA-1 | happy path GREEN on F1 under **both** mechanisms — four stages pass, `npm test` really runs in the verify capsule, one canonical ref advances by CAS. Recorded in `probes/selftest/results.tsv` |
| VA-2 | **four** scenarios across **three** stages: `conform/undeclared-path` · `verify/suite-failed` · `verify/verify-timeout` · `advance/stale-base`. The refusing stage is read from the emitted `stage=` line, never from an exit code — every refusal exits 1 |
| EX-8 / F-14 | `advance/stale-base` refuses at the precondition with the **strict** `assert_outcome` clause holding — object count untouched, so nothing was transferred |
| VA-3 | the DELETED SECOND HOP restored on purpose: canonical **refs unchanged** (the refs clause is blind), object count **grew**, and `assert_outcome` **REDS**. Negative control green first. A wrong admission, not a payload red |
| EX-3 | all four M-B hygiene legs observed refusing — `bundle-unsafe-path` (symlink to a *nonexistent* target, which is what proves the leg ORDER), `bundle-absent`, `bundle-invalid`, `resource-cap` — behind a positive control that an untouched bundle IS ingested |
| EX-9 / I5 | `verify/verify-timeout` observed end to end: PHASE-02's status 124 acquiring its PHASE-03 token. The capsule never authors it |
| EX-4 / D-P03-2 | conform leg 2 runs `slice conformance -p <quarantine> --against B..S --strict` and refuses a real undeclared path |

Re-runnable: `rig selftest` (auto-dispatches per D-P01-3), or
`control/selftest.sh [happy|attribution|bundle|falsifiable]`.

### PHASE-03 open questions — vocabulary gaps, RECORDED not filled

Surfaced by pinning the status→token mapping. None blocks the happy path;
each wants an operator ruling or a PHASE-05 `/consult` before a row needs it.
**No token was minted** — the set is closed, and inventing one would forge the
distinction `assert_outcome` keys off.

- **OQ-a — a VERIFY capsule that overruns the DISK cap has no token.** EX-3
  reads bound→token (`timeout`→`verify-timeout`, `disk`→`resource-cap`), but
  § 5.1's prefixes read stage→token, and the refusing stage there is `verify`.
  `verify/resource-cap` is not in the set.
- **OQ-b — a WORKER capsule that overruns the WALL CLOCK has no token.** The
  doorbell wait ends at its deadline and harvest finds no result ref; `harvest`
  has no timeout token. Likely PHASE-05 H15's business ("killed at each stage
  in turn").
- **OQ-c — M-A has no token for an ABSENT RESULT.** `bundle-absent` is M-B's,
  and reusing it would be a mechanism lie in the one column the rig exists to
  compare. `harvest-fetch.sh` raises a **RIG DEFECT** instead of scoring it.

### PHASE-04 step 0 — decisions (durable)

- **D-P04-1 — step 0 ran BEFORE the phase sheet was expanded and before the
  phase flipped `in_progress`.** Deliberate inversion of the handover's stated
  order, which carried two instructions that cannot both be honoured:
  "`/phase-plan` first" and "do step 0 in a context that has not read the
  reading list". `/phase-plan` mandates reading design § 5.4 and `notes.md`, so
  honouring the first destroys the second — silently, and in the direction that
  manufactures a false green, since an empty residue reads as ASM-007 surviving.
  Paid the visible cost (a lifecycle-order blemish, recorded here) over the
  invisible one. Captured as RFC-011 friction: no skill currently says that a
  phase whose criteria constrain what its *planner* may read cannot be
  phase-planned before the criterion is discharged.
- **D-P04-2 — ASM-007 is `invalidated`, not `obsolete`** (operator ruling,
  2026-08-02). `obsolete` reads as "no longer applies" and would lose the fact
  that the claim was posed, tested, and failed. The *reframe* lives in the
  record's prose plus the replacement record, not in the status.
- **D-P04-3 — the replacement is ONE assumption, not two.** ASM-008 carries the
  universal / language-bound responsibility split. The candidate second claim
  ("the declaration format is adequate for the classes it can name") is **not**
  recorded as an assumption: the rig already tests it, so it belongs as a
  criterion, not as a carried belief.
- **D-P04-4 — CPT-001 is NOT amended.** The taxonomy stays active and correct;
  only the exhaustiveness claim about it died. Adding a sixth class and
  re-asserting exhaustiveness would re-run the same error one class later, so
  the residue is tracked as QUE-203 rather than plugged. Operator is on record
  as doubtful QUE-203 resolves usefully — tracked, not scheduled.

### PHASE-04 step 0 — findings (durable)

- **F-P04-1 — ASM-007's residue is NON-EMPTY; the assumption is falsified.**
  96 npm/TypeScript triggers over 11 surfaces (`step0-enumeration.md`), 84 clean
  fits, 6 fits with named strain, 2 out of scope, **4 residue**. R2 (terminal
  escape sequences — the interpreter is the display) and R4 (non-inert parsing /
  prototype pollution) are individually sufficient and were unanticipated
  anywhere in the corpus.
- **F-P04-2 — R1 was over-reported, and the correction matters.** Prompt
  injection into the control-plane agent was first reported as a surprising
  omission. It is not: **CON-005 and design § 1.1 already name it**, as a known
  unbounded threat, with an operator ruling and a structural mitigation
  (refusals carry trusted-side-computed tokens; artifact content is never
  relayed verbatim). It contradicts the exhaustiveness claim — the taxonomy
  cannot *classify* it — while corroborating what the design already knew. Same
  correction applies to R3's LLM-carrier half; R3's shell/template-injection
  half is genuinely unanticipated.
- **F-P04-3 — the failure is the claim's SHAPE, not its truth value** (operator,
  2026-08-02). "These classes are exhaustive" is a universal negative over an
  open-ended adversarial space: unsettleable by search, permanently low
  confidence, and its practical effect was to make an *empty* residue look like
  progress when an empty residue would mostly have measured the searcher. The
  durable diagnostic, now recorded in ASM-007's body: *if this assumption were
  false, what would I do differently?* Exhaustiveness answered "record a new
  class" — bookkeeping, not a decision.
- **F-P04-4 — the falsification protocol itself worked and is reusable.** The
  two-commit discipline (enumerate with the taxonomy unread, commit; classify,
  commit) makes independence *checkable* rather than asserted — the diff between
  `beb4b665` and `61ea9f08` is append-only, 190 insertions, 0 deletions. RV-340
  F-8 sharpened this protocol and earned its keep.
- **F-P04-5 — the clean fits are real portability evidence, not filler.** Three
  land unforced from opposite ecosystems: `binding.gyp` ≡ `build.rs`,
  `packageManager` ≡ `rust-toolchain.toml`, `.mise.toml` ≡ `.envrc`. That is
  DEC-101/DEC-102's claim (a known class has a TypeScript instance) corroborated
  independently of the residue.
- **F-P04-6 — an agent `git reset --hard` in the shared tree discarded
  uncommitted work mid-phase.** ASM-007's both-tier edits were lost and had to
  be re-applied; ASM-008/QUE-203 survived only because untracked files are not
  touched by a hard reset. Nothing was ultimately lost, but the working tree is
  not a safe place to hold authored state between steps. **Commit each coherent
  unit immediately** — AGENTS.md's "never discard uncommitted work" is a rule
  other agents can break on your behalf.

### PHASE-04 step 0 — evidence (what was OBSERVED, not merely asserted)

| criterion | observation |
|---|---|
| EX-1 / EX-3 / EX-9 | enumeration authored with CPT-001 unread, committed `beb4b665`; classification appended `61ea9f08`. **Append-only diff, 0 deletions** — independence is checkable, not claimed |
| EX-2 / VA-1 | `doctrine knowledge show ASM-007` renders `invalidated` in **both tiers** — structured `invalidated_by`/`invalidated_on`/`contradicts` and the prose body. Never the `.md` alone |
| EX-2 | recorded **invalidated**, never *discharged*; replacement claim carried forward as ASM-008 rather than left vacant |

### PHASE-04 — FOR RECONCILIATION (not edited in-phase, per operator ruling)

- **design.md § 9 closure bullet is now false.** It reads *"ASM-007 is recorded
  **strengthened, not discharged**, whatever step 0 returns"* — anticipating only
  the strengthening branch. Step 0 returned the third outcome: falsified. The
  design is locked and this is `/reconcile`'s edit to make, not this phase's
  (operator ruling, 2026-08-02). Replacement should reference ASM-008 as the
  claim the post-spike REV may actually lean on, and must not restate an
  exhaustiveness claim.
- Knock-on to check while there: § 9's *Closure* paragraph gates on "every
  measurement row filled or recorded after-side-only" plus the ASM-007 line;
  only the ASM-007 line is affected.

### PHASE-03 boundary — ONE foreign interior commit

Range `acc4b2b34..e14c22cc`. The start is an RFC-026 observation commit,
correctly EXCLUDED — the range is `start..end`. Eight own commits, and **one
foreign interior**: `28f0d4c08 chore(SL-233): ISS-290 …`, touching
`.doctrine/backlog/issue/290/**` only. Interior, so `record-delta` cannot
excise it — neither `--commit` nor `--start/--end` excises a middle commit.
Left as recorded and flagged here, exactly as PHASE-01's `ad65512dc` was.

**The range was deliberately NOT narrowed to the code tip.** PHASE-02 used
`record-delta` to *correct* a range that closed before a post-flip fix; this
flip came after everything, so no correction is due. Narrowing `--end` to
`2204ce267` would have dropped the notes and memory commits out of range and
cut `undeclared` from 16 paths to 3 — which is precisely the harm this slice
has twice refused from the other direction. Silencing a legible structural
finding by moving the boundary is the same act as silencing it by widening a
selector; only the lever differs.

`slice conformance 241` reports `undeclared (16)`:

| family | disposition | why |
|---|---|---|
| `.doctrine/backlog/issue/290/**` (3) | **boundary pollution** | the foreign interior SL-233 commit — not this slice's work, and not a selector question |
| `.doctrine/memory/items/**` (9) | **aligned** | three memories recorded across PHASE-02/03; `/record-memory` is a mandated step of `/execute`, so the corpus write is a deliverable |
| PHASE-01/02's structural set (4) | **aligned** | the slice's own authored deliverables |

Selectors were again deliberately NOT widened. The memory family stays a
**corpus-wide** question for audit, as PHASE-02 raised it.

### PHASE-02 boundary — clean range, two NEW undeclared families

`code_start_oid = b3ad3eed3`, `code_end_oid = d041f6b39`, **4 commits, none
foreign** — unlike PHASE-01. The interior-foreign-commit problem did not recur;
`b3ad3eed3` (SL-233) landed *before* the range start and is correctly excluded.

The flip closed the boundary at `8f403cd68`; the F-P02-6 fix and its harvest
landed after it, so the range was corrected with
`slice record-delta 241 PHASE-02 --start b3ad3eed3 --end d041f6b39` (the
multi-commit escape hatch, not `--commit`). `--end` is the cumulative CODE
tip: the two trailing harvest commits are knowledge records and sit outside
the range by design. VT-1 re-verified PASS afterwards.
A foreign edit to `.doctrine/rfc/026/rfc-026.md` was live in the working tree
throughout and was **never staged** — every commit was path-limited.

At audit, `slice conformance 241` reports `undeclared (7)`. Four are PHASE-01's
already-dispositioned structural set (`fixtures.md`, `notes.md`, `flake.nix`,
and the slice's own authored deliverables). **Two families are new to this
phase** and want a disposition of their own:

| path | disposition | why |
|---|---|---|
| `.doctrine/memory/items/mem_019fbd3c…` (+ its slug symlink) | **aligned** | F-P02-2's memory. `/record-memory` is a mandated step of `/execute`, so the corpus write is a deliverable of the phase, not drift |
| `.doctrine/observations/records/2b/019fbd3b….toml` | **aligned** | RFC-011 instrumentation is mandated by project governance for every skill invocation; records are authored and committed by design |

**Selectors were again deliberately NOT widened.** Design § 9.1's code-impact
table is the selector source and it does not name `.doctrine/memory/**` or
`.doctrine/observations/**` — which is correct, because those are process
outputs of *any* phase rather than this slice's subject matter. Adding a
selector to silence them would convert a legible structural finding into a
silent pass, and would do it for every future slice that records a memory.
Worth raising at audit as a **corpus-wide** question, not a per-slice fix.

### PHASE-01 boundary — read before believing any conformance finding

`code_start_oid = 25540cfe1`, `code_end_oid = 29c7acf35`, 8 commits, of which
**one is foreign**: `ad65512dc build: pin CLAUDE_CODE_SHELL to bash for jailed
agents` (touches `flake.nix` only). It is **interior** to the range, so
`record-delta` cannot exclude it — neither `--commit` nor `--start/--end`
excises a middle commit. Left as recorded, flagged here instead.

At audit, `slice conformance 241` reports `undeclared (3)`:

| path | disposition | why |
|---|---|---|
| `.doctrine/slice/241/fixtures.md` | **aligned** | the slice's own authored deliverable — structural (`mem.fact.conformance.rev-only-slice-undeclared`) |
| `.doctrine/slice/241/notes.md` | **aligned** | same |
| `flake.nix` | **split** | the shellcheck line is a coupled deliverable of this phase (D-P01-4) → `aligned`; the `CLAUDE_CODE_SHELL` hunk is foreign → boundary pollution |

Selectors were deliberately NOT widened to silence any of these — adding one
mid-phase would convert a legible structural finding into a silent pass.

### Learned

- mem.pattern.doctrine.amend-knowledge-both-tiers
- mem.pattern.doctrine.repair-overshoots-the-named-axis
- mem.pattern.doctrine.path-policy-shell-hardening (raiser-side, 3afa085e)
- CPT-001 — five numbered classes, one with a git-level sub-class
- **environment**: `nix` and `direnv` are ABSENT in the jail (`/nix/store`,
  `bwrap`, `node`, `npm`, `claude` present); `$HOME` writable, `~/capsules/`
  creatable with no new mount. Pinned as PHASE-02 EX-8 / PHASE-04 EX-4.
- **R2 decomposes into two questions**, not one — `worktree import` has two
  scope postures and `conformance --strict` covers only the selector predicate
  (PHASE-01 sheet § T7; rides
  mem.pattern.dispatch.import-scope-belt-omit-slice-for-coupled-paths)
- **the step-0 contamination guard is two-sided** — design § 5.4 protects step 0
  from CPT-001; authoring the light fixture's `interpret:` list is itself a
  trigger enumeration, so PHASE-01 EX-12 + PHASE-04 EX-9 protect the other side.
  Not in the design because the design does not schedule the work.

### Open

- **OQ-a / OQ-b / OQ-c** — three closed-vocabulary gaps found in PHASE-03,
  recorded and NOT filled (§ PHASE-03 open questions)
- QUE-200 — ingestion mechanism M-A vs M-B; the rig's whole point. **First
  evidence input banked**: M-B's trusted-side file-ingestion boundary is four
  observed refusal legs that M-A does not carry at all — the asymmetry is a
  cost, and it is now measured rather than argued
- QUE-201 — declaration home; now ergonomics-only, gains a probe-evidence input
- QUE-202 — how the capsule model *admits* the second result; refusal proven,
  admission not designed
- ~~ASM-007~~ — **FALSIFIED and `invalidated` in PHASE-04 step 0**, exactly as
  its own validation plan specified. Not discharged: replaced by **ASM-008**
  (universal / language-bound responsibility split), which is the claim the
  design actually load-bears on. Do **not** repair ASM-007 by adding a sixth
  class — see F-P04-3
- **ASM-008** — the responsibility split, `held`, confidence medium. Falsified by
  a trigger that is language-bound but not client-declarable, or universal but
  not doctrine-shippable — **not** by discovering a new class
- **QUE-203** — can the taxonomy name interpreters *downstream* of the toolchain
  (LLM, terminal, human reviewer), whose trigger paths are not enumerable?
  Operator doubtful it resolves usefully; tracked, not scheduled
- CON-004 — landed state append-only · CON-005 — threat-model fence
- DEC-099/101/102/103/104 — settled, carried
- OQ-1 — evidence-log storage tier; v0 ruling in slice § Risks
- ~~R2~~ — **SETTLED in PHASE-01 T7**, see § R2 below. R2a agrees; R2b separates,
  so conform leg 3 is load-bearing. No divergence, no `/consult`, no `src/` change
- no `research.md` exists — the pre-design research round was never run; the
  drift advisory reports an absent artifact, deliberately not restamped
  (rationale in plan.md § Notes)

## R2 — SETTLED (PHASE-01 T7, EX-6, VA-2, VA-3)

Probed 2026-08-01 by `scripts/spike-capsule/control/probe-r2.sh`, re-runnable,
against a disposable clone of the light fixture. Nine rows, every expectation
declared before the run and asserted; results at
`$SPIKE_CAPSULE_ROOT/probes/r2/results.tsv`.

**R2 was two questions, not one.** `worktree import` has two scope postures —
with `--slice N` it runs the selector predicate, without it only the R-5 belt —
and `slice conformance --against … --strict` covers the selector predicate only.

### R2a — selector agreement: YES

`--strict` reaches the belt's scope-leg verdict on every edge probed. This is
not merely predicted from shared code: the belt's scope leg
(`src/worktree/import.rs:159`) and conformance both call
`crate::conformance::undeclared_paths`, so what could genuinely diverge is the
**path extraction** in front of it, and the two gathers differ — the belt uses
`diff --name-only` (`src/mcp_server/dispatch.rs:487`), conformance uses
`diff --name-status` folded by `actual_from_range` (`src/slice.rs:2894`). The
edges that would expose a difference do not:

| edge | delta | `--strict` | reading |
|---|---|---|---|
| 1 | `docs/notes.md` | refuse | ordinary undeclared path |
| 2 | `src/tax.ts` | clean | matches a `design-target` selector |
| 3 | `src/naïve.ts` | clean | **non-ASCII extracted verbatim** — the `core.quotePath=false` hardening holds through the `--name-status` fold |
| 4 | `src/money.ts` → `docs/money.ts` | refuse | **both legs visible** — `--no-renames` holds; the destination is the undeclared path and the source did not vanish |
| 7 | `A..A` | clean | empty range is clean, not an error |
| 8b | `src/audit.ts` | clean | edge 8's positive control |

### R2b — separation: `--strict` does NOT cover the prefix legs

**This is the load-bearing result.** `--strict` has no `.doctrine/`/`.claude/`
predicate at all; the belt runs those legs *before* the scope leg and
independently of it (`classify_import`, `import.rs:146-152`, and its own test
`classify_import_doctrine_path_is_doctrine_touch_even_when_undeclared`).

| edge | delta | `--strict` | belt prefix legs |
|---|---|---|---|
| 5 | `.doctrine/probe/payload.md` → `src/payload.md`, **both declared** | **clean** | `doctrine-touch` |
| 6 | `.doctrine/probe/kept.md` modified, **declared** | **clean** (reported *conformant*) | `doctrine-touch` |

**⇒ design § 5.2's conform LEG 3 (forbidden paths) is LOAD-BEARING. PHASE-03
must not skip it, and must not fold it into leg 2.** A pipeline that ran only
`--strict` would pass a `.doctrine/` touch whenever a selector happened to
declare that path — and this slice's own selectors (`.doctrine/rfc/025/**`,
`.doctrine/knowledge/**`) are exactly that shape.

**Edges 5 and 6 had to be run against a slice that DECLARES the `.doctrine/`
path.** Against the fixture's base slice (`src/**` only) a `.doctrine/` path is
undeclared, so `--strict` refuses it — for the wrong reason — and R2b scores
backwards as "`--strict` covers the prefix legs". The probe builds SL-002 with
`.doctrine/probe/**` declared for this reason alone. This correction was found
in the pre-execution re-check, not during the probe; the sheet's original edge
list would have walked into it.

### Edge 8 — the empty-selector asymmetry

`--strict` against a slice with **no** selectors refuses (everything is
undeclared); `classify_import` with empty selectors is a documented no-op
(`import.rs:668`). Divergent but **benign**: empty selectors is precisely the
belt's no-`--slice` posture, where the scope check is meant to be absent.

Note the help text's "refuses a clean diff when the registry is unavailable or
incomplete (fail-closed)" describes the *other* arm — `run_conformance`'s
`--against` path documents that it bypasses both the registry read and the
completeness ladder.

### No `/consult`, no `src/` change

STOP-1 covers a genuine `--strict`-vs-belt divergence. R2a **agrees**, and
R2b's separation is the *predicted* answer that makes leg 3 load-bearing — a
result, not a defect. Nothing in `src/` was touched.

## Forward compatibility

- **RFC-023 (executable plan gates / adversarial TDD)** — substantial revisions
  to plan gates are expected. Operator ruling 2026-08-01: adopt current plan
  machinery as-is for this slice; expect heavy revision to follow. Nothing in
  the four-stage capsule pipeline (CON-004, DEC-104) depends on plan-gate
  mechanics, so the revisions should land orthogonally to this rig.
