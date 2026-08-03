# PHASE-01: Rig foundation, fixtures, and the R2 probe

Disposable phase sheet — runtime scratch under `.doctrine/state/`, gitignored
and `rm -rf`-able. Expands the plan's phase entry into working detail. Durable
risks / decisions / findings are harvested into the slice audit at close-out;
until then, lift anything that must survive into the slice's `notes.md`.

## Objective

Stand up the rig entry point and shared shell library, prove the I6 real-repo
guard fires, build both fixtures, and settle R2 — whether `slice conformance
--against B..S --strict` matches the import belt's undeclared-path semantics —
BEFORE any pipeline code load-bears on it. R2 is probed first by design (§ 8);
a genuine gap is a `/consult`, never an improvised `src/` change.

Authored criteria: `.doctrine/slice/241/plan.toml` PHASE-01 (EN-1..2,
EX-1..12, VT-1..2, VA-1..3). `slice phase` is a writer — there is no read-back
verb, so read criteria from the TOML.

## Reading list

- `.doctrine/slice/241/design.md` § 5.3 (fixtures), § 5.5 I6, § 9 (validation),
  § 9.1 (rig tree). § 5.2 for what the conform stage will later need from R2.
- `.doctrine/slice/241/slice-241.md` — R2, non-goals (no `src/`).
- `src/slice.rs:2894` `actual_from_range` — what `--against … --strict` folds.
- `src/mcp_server/dispatch.rs:467` `import_plan`, belt gather at `:487`.
- `src/git.rs:1257` — the `-z` NUL-delimited diff helper.
- `scripts/pi-spawn-confined.sh` — the bwrap seed (PHASE-02's business; read
  only for the jail idiom).

Memories that bear directly:
- `mem.pattern.doctrine.path-policy-shell-hardening` (high) — the reference
  invocation, and "prefer the existing verb". Written off RV-340 F-4, i.e. off
  this slice's own design review.
- `mem.pattern.dispatch.import-scope-belt-omit-slice-for-coupled-paths` (high)
  — the two-posture fact that reshapes the R2 probe (below).
- `mem.pattern.harness.grep-negative-needs-positive-control` — governs VA-1.

## Assumptions & STOP conditions

Carried:
- **A-jail** — `$HOME` is writable in the jail and `~/capsules/` is creatable
  (verified at plan time). No new mount, no flake change.
- **A-env** — `nix` and `direnv` are ABSENT; `/nix/store`, `bwrap`, `node`,
  `npm`, `claude` present. In-jail is the primary environment (EX-9).
- **A-install** — `doctrine install` works in non-Rust projects (operator
  ruling). A failure here is therefore a real finding, not an expected snag.

**STOP → `/consult`, do not improvise:**
1. R2 finds a genuine `--strict`-vs-belt divergence. The fix is NOT a `src/`
   change (slice non-goal). Record the edge, halt, consult.
2. `doctrine install` fails on the light fixture — contradicts A-install, so it
   is a POL-002 finding.
3. `~/capsules` proves unusable at run time. Do NOT fall back to writing
   fixtures inside the repo; that is precisely what I6 exists to prevent.
4. Any urge to add a selector / verb / flag to make a probe pass. The rig
   reports the coupling; it does not build around it.

## Tasks

- [ ] **T1 — guard first, red then green.** Write the failing case before the
      guard exists: an entry invocation whose resolved canonical root is this
      repo must be *observed* proceeding (red), then refused (green).
      `guard_not_real_repo` lands in `lib/common.sh` (VT-1). It runs at EVERY
      entry point, before any provisioning — a guard that runs late is not a
      guard (§ 5.5 I6). Satisfies EX-2, VA-1.
      → `scripts/spike-capsule/lib/common.sh`

- [ ] **T2 — rig entry point.** `rig <c1a|smoke|c2|c3|c1b|selftest> [rows…]
      [--stub|--agent]`, defaulting `--stub`, `--agent` explicit opt-in
      (DEC-103). First statement of every sub-command is the T1 guard.
      Satisfies EX-1, VT-2.
      → `scripts/spike-capsule/rig`

- [ ] **T3 — capsule root as a parameter.** `SPIKE_CAPSULE_ROOT`, default
      `~/capsules`, never a hardcoded path (EX-8). Resolve it, then guard it —
      resolution before comparison, or the guard compares an unresolved string
      (`mem.pattern.safety.resolve-every-ref-before-pure-compare`, same failure
      shape: verbatim trust on one operand defeats the guard).
      → `scripts/spike-capsule/lib/common.sh`

- [ ] **T4 — heavy fixture.** `git clone --no-hardlinks` this repo into
      `$ROOT/fixtures/heavy`; strip remotes; assert no `credential.helper`;
      assert full history (merge-base works) and HEAD at exactly `B`.
      Declaration lives OUTSIDE the clone — that placement is what makes the
      F-5 substitution attack non-live in the base fixture, and PHASE-05's
      variant is what manufactures it. Satisfies EX-3.
      → `scripts/spike-capsule/control/fixture-heavy.sh`

- [ ] **T5 — light fixture.** TypeScript project, deliberately unlike this repo:
      trunk `mainline`, `[add] …` commits, `package.json` scripts
      build/clean/test/lint/format, one red→green test. Then `doctrine install`,
      one scratch slice, `design-target` selectors over `src/**`.
      Satisfies EX-4, part of EX-5.
      → `scripts/spike-capsule/control/fixture-light.sh`,
        `scripts/spike-capsule/fixtures/light/**`

- [ ] **T6 — the light declaration, under the contamination guard.** Author
      `exec:`/`interpret:`/`verify:` FROM THE FIXTURE'S BUILD NEEDS ONLY — what
      does this project actually run to build and test? Do NOT reason about what
      TypeScript auto-loads or evaluates; that is PHASE-04 step 0's job and doing
      it here makes step 0 confirmatory (EX-12, and PHASE-04 EX-9 is the other
      half). If a hazard occurs to you while writing this, write it in Findings
      below and leave it OUT of the declaration.
      → `scripts/spike-capsule/fixtures/light/interpretation-surface.txt`

- [ ] **T7 — the R2 probe.** See the decomposition below. Answer in writing,
      name every edge probed. Satisfies EX-6, VA-2, VA-3.
      → `.doctrine/slice/241/notes.md`

- [ ] **T8 — `fixtures.md` build sheet.** All five fixtures, each with its delta
      over its base and its consuming criterion. Concrete literals, not design.
      Satisfies EX-10; EX-11 is satisfied by NOT building variants 2 and 3 here.
      → `.doctrine/slice/241/fixtures.md`

- [ ] **T9 — `.gitignore` raw-log entry.** One line (EX-7).

### T7 detail — R2 decomposes into two questions, not one

The plan states R2 as "does `--strict` match the import belt". Retrieval shows
that phrasing conflates two distinct checks. `worktree import` has **two scope
postures**: with `--slice N` it refuses on `undeclared-scope` against
`design-target` selectors; **without** `--slice` it runs only the R-5 belt
(`.doctrine/`/`.claude/` reject). They are different predicates.

`slice conformance --against … --strict` covers the **selector** predicate only.
So:

- **R2a (selector agreement)** — does `--strict` reach the same undeclared/clean
  verdict as the belt's scope leg over the same range? Both ride the same
  hardened extraction (`actual_from_range`, `src/slice.rs:2894` —
  `core.quotePath=false` / `--no-renames` / `-z`), so a divergence, if any, is
  in the *decision*, not the *path extraction*. Predicts agreement; probe it
  anyway, because that prediction is exactly the kind that goes unchecked.
- **R2b (separation)** — does `--strict` refuse `.doctrine/`/`.claude/` touches
  on its own? It should **not**: those are the R-5 belt's job, and design § 5.2
  makes them a *separate* conform leg (leg 3). If `--strict` happened to cover
  them, leg 3 is redundant; if it does not, **leg 3 is load-bearing and PHASE-03
  must not skip it**. Either answer is a result worth recording.

Probe by constructing known deltas in the light fixture and asserting the verdict
of each. Do **not** attempt to run `worktree import --slice` as the comparator —
F1/F2 in design § 2.1: it needs `HEAD == B`, a clean coordination worktree, and a
durable fork binding. Compare against the belt's *documented semantics* and its
reference invocation instead.

Edges to probe and name (each gets a recorded verdict):
1. path matching no selector → expect undeclared
2. path matching a `design-target` selector → expect clean
3. **non-ASCII path** under a declared selector → expect clean (i.e. the
   quotePath hardening holds and does not mis-read it as undeclared)
4. **rename out of** a declared selector → expect the source leg visible
5. **rename out of `.doctrine/`** → R2b's sharpest case
6. `.doctrine/` touch → R2b: does `--strict` refuse, or is it silent?
7. empty range `A..A` → expect clean, not an error
8. registry incomplete / no selectors → expect **fail-closed** (the help text
   claims "Refuses a clean diff when the registry is unavailable or incomplete")

Edges 3–6 double as the positive controls the design's guard probes need later
(§ 9), so record them in a form PHASE-05 can reuse.

## Risks

- **The guard is never exercised.** Highest-value, cheapest-to-skip. VA-1 exists
  because a safety check never observed refusing is not known to work. T1 is
  written red-first for this reason and no other.
- **Contamination via T6.** Silent and total if it happens: a back-fitted
  enumeration returns an empty residue by construction, and an empty residue
  reads as ASM-007 surviving. Mitigated by T6's discipline and PHASE-04 EX-9's
  fresh context; the residual is that the same session writes both, so the
  Findings section below is the release valve — hazards noticed now go there,
  not into the declaration.
- **R2 predicted-agreement bias.** R2a predicts agreement. A probe run by someone
  expecting agreement finds it. Assert on recorded verdicts per edge, not on an
  overall impression.
- **Shell arg-building footguns.** Not hypothetical in this corpus
  (`mem.pattern.doctrine.review-response-shell-backtick-mangling`): backticks in
  a double-quoted argument are command substitution, and the failure is silent.
  Write prose to a file and pass a path, or use a single-quoted heredoc.

## Decisions

<!-- record here as they are taken; lift durable ones to notes.md -->

- **D-P01-1** — R2 is probed against the belt's documented semantics and
  reference invocation, not against a live `worktree import --slice` run. Reason:
  F1/F2 — import needs `HEAD == B`, a clean coordination worktree and a fork
  binding, none of which a fixture has. Recording this because "we compared
  against the real verb" would otherwise be assumed later.

- **D-P01-2** — `guard_not_real_repo` refuses **containment**, not just the
  equality EX-2 states. Equality alone leaves `<repo>/scripts/spike-capsule/
  fixtures` open, and a mutator running there is precisely the failure I6 names.
  This is the criterion's own justification applied, not a widening of it.

- **D-P01-3** — `rig selftest` degrades in PHASE-01 to the I6 guard probe rather
  than stubbing out. EX-1 wants a self-test entry; `control/selftest.sh` is
  PHASE-03's (VT-4). The guard probe is VA-1's re-runnable home in the meantime,
  and the arm dispatches to `control/selftest.sh` the moment it exists.

- **D-P01-4** — shellcheck 0.11.0 was added to the jail before any rig shell was
  written (operator, 2026-08-01). The whole slice is shell and `bash -n` catches
  syntax only. `shellcheck -x -S style` is the gate; `scripts/smoke.sh` was
  already clean at that level, so there is no inherited debt. shfmt declined:
  cosmetic, and the repo has no shell format convention to enforce.

## Findings

<!-- Hazards noticed during T6 go HERE, never into the light declaration.
     A failed probe row is a finding, never a quiet rig edit. -->

- **F-P01-1 (T1, fixed in-phase) — a guard that refuses in a subshell does not
  guard.** The first `rig` draft called the entry guard as
  `rig_dispatch … "$(rig_enter)"`. `guard_not_real_repo` refuses by `exit`, so
  inside `$( … )` it ended only the substitution's subshell: the refusal printed,
  the substitution evaluated to the empty string, and **the rig dispatched
  anyway**. `set -euo pipefail` does not catch it — a failed command
  substitution propagates in *assignment* position only, never in *argument*
  position.

  Observed live at exit 0 on a root resolving to `/workspace/doctrine/fixtures`.
  The guard-probe leg passed throughout, because the probe subshells the guard
  deliberately and reads its status — so the unit case was green while the real
  entry point was open. That gap is the finding: "a guard that runs late is not
  a guard" has a sibling, **a guard whose refusal cannot reach the entry point is
  not a guard**, and only the entry-point observation catches it.

  Fixed: `rig_enter` publishes `RIG_ROOT` and is invoked as a statement in every
  arm; it asserts `BASHPID == $$` and refuses if it was subshelled, so a
  recurrence is loud rather than silent. Durable — lift to notes.md and record as
  a memory at harvest.

- **F-P01-2 (T6) — hazards noticed while authoring the light declaration.**
  EX-12's release valve. These occurred to us while working out what `ledger`
  runs to build and test; they are recorded HERE and deliberately kept OUT of
  `fixtures/light/interpretation-surface.txt`, whose `interpret:` list names only
  `package.json`, `tsconfig.json`, `tools/format.mjs` — the files the project's
  own build actually reads.

  **PHASE-04 step 0 must not read this list before it enumerates.** It is written
  down so the observations are not lost, not so they can be reused; step 0's
  independence is the whole of ASM-007's falsification value, and a step 0 primed
  by this list produces an empty residue by construction.

  1. npm lifecycle scripts — `preinstall` / `postinstall` / `prepare` run on
     `npm install`. `ledger` declares none; EX-10's H11 row plants one.
  2. `npm run` prepends `node_modules/.bin` to PATH, so a planted executable
     there is run by any script, including ones that look inert.
  3. `.npmrc` can set `script-shell` and the registry — it redirects both what
     runs scripts and where code comes from.
  4. `tsconfig.json` `extends` resolves through node resolution, so a config can
     be pulled from a package rather than the repo.
  5. Node 26 strips TypeScript types natively, so a `.ts` file is directly
     executable input. There is no compile step gating it — which is why this
     fixture's tests run without a build, and why "it has not been compiled yet"
     is not a safety property here.

- **F-P01-3 (T5, fixed in-phase) — "declares the script" is not "the script
  works".** The first light-fixture assertions read the five npm scripts out of
  `package.json` and passed. Running them showed `build` and `lint` both broken:
  `tsc` cannot resolve `node:test` / `node:assert/strict` without `@types/node`,
  which the dependency-free fixture does not carry.

  Fixed by scoping `tsconfig.json` to the library (`exclude: src/**/*.test.ts`) —
  the tests are type-stripped and run by Node directly, so nothing type-checks
  them and nothing needs to — and by replacing the five key-existence assertions
  with five that RUN each script. Same shape as VA-1 and as the per-cell positive
  controls: an assertion that cannot distinguish authored from working is not
  evidence. Cheap here; in PHASE-04 a broken `build` would have surfaced as a
  P-C1a cost measurement rather than as a fixture defect.

- **F-P01-4 (T5) — `doctrine install` on a non-Rust project: A-install holds.**
  Ran clean on `ledger` (TypeScript, no Cargo.toml), scaffolding `.doctrine/`,
  hymns, project-orientation, and accepting `slice new` + `slice selector add`.
  One advisory, non-fatal: `reservation reach degraded to local (no remote
  configured)`. No POL-002 finding. This is the first non-Rust exercise of the
  independence claim in the corpus, so it is worth recording as evidence even
  though EX-8 withdrew it as a criterion.
