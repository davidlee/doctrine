# Notes SL-241: Capsule spike rig

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-01 · **PHASE-01 complete** (1/6), slice `started` · bd4dee1b
(phase code tip 29c7acf3; bd4dee1b adds the harvest + observation commits)

### Produced

- design.md written, internally reviewed, externally inquisited — no code yet
  (commits 9d0c852f, 3966256a, eedc26b8, 7fd55d0f)
- RV-323 — 14 findings, all disposed `fix-now`; **all 14 terminal, review done**
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
| EX-5 | no ring → wait ends at its deadline (never hangs) · a ring naming another capsule is accepted as a bare signal · duplicate ring is a no-op · identity echoed is the **argument**, not the file |
| EX-7 / VA-1 | two assertions, recorded separately in `probes/smoke/results.tsv`: `npm ping` pass, `claude -p` pass |
| EX-8 | full cycle on the light fixture: provision (ro-bind toolchain, no `nix`/`direnv`) → worker commit + ring → `npm test` green in the verify capsule |

Raw results: `$SPIKE_CAPSULE_ROOT/probes/smoke/results.tsv`. Re-runnable:
`rig smoke`, `control/probe-capsule.sh [posture|bounds|doorbell]`,
`control/audit-i4a.sh <capsule> [--positive-control]`.

### PHASE-02 boundary — clean range, two NEW undeclared families

`code_start_oid = b3ad3eed3`, `code_end_oid = 8f403cd68`, **3 commits, none
foreign** — unlike PHASE-01. The interior-foreign-commit problem did not recur;
`b3ad3eed3` (SL-233) landed *before* the range start and is correctly excluded.

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

- QUE-200 — ingestion mechanism M-A vs M-B; the rig's whole point
- QUE-201 — declaration home; now ergonomics-only, gains a probe-evidence input
- QUE-202 — how the capsule model *admits* the second result; refusal proven,
  admission not designed
- ASM-007 — exhaustiveness carried, confidence low; falsified only by § 5.4
  step 0's independent enumeration, never by fixture rows
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
