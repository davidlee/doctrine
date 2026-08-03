# PHASE-05: The hostile matrix (P-C3), the guard probes, and EVD

Disposable phase sheet — runtime scratch under `.doctrine/state/`, gitignored
and `rm -rf`-able. Expands the plan's phase entry into working detail. Durable
risks / decisions / findings are harvested into the slice audit at close-out;
until then, lift anything that must survive into the slice's `notes.md`.

## Objective

Run the sixteen rows against both mechanisms, scored against § 5.6's
re-derivation — NOT probe-specs' inherited expected-kill column, which was
authored against a pipeline tail this design removes. Five rows there name
machinery this design deletes or never specifies; scoring them against the
inherited column would produce results that are properties of the matrix
bookkeeping rather than of the capsule model, and § 9's closure gates on that
table. This phase wants its own context.

## Reading list

Cite ids; do not re-survey. `notes.md` §§ PHASE-01..04 are fresh at `8e962656`.

| what | where |
|---|---|
| the sixteen rows, **re-derived** — the source of truth for `expected-*` | `design.md` § 5.6 (`:672-716`) |
| the closed token vocabulary | `design.md` § 5.1 (`:228-252`); `control/pipeline.sh:51-84` |
| the matrix harness loop and `Hnn.planted?` | `design.md` § 5.4 (`:539-571`) |
| altitude / `n/a` discipline | `design.md` § 5.4 (`:564-567`), § 9.1 (`:972-973`) |
| the five guard probes | `design.md` § 9 (`:882-888`) |
| `assert_outcome`'s three clauses (I1) | `design.md` § 5.5 (`:588-604`); `control/pipeline.sh:520-561` |
| the H10/H16 split, and what it does not cover | `design.md` § 5.1 (`:254-295`) |
| provenance invariant — declaration read from `B`, never `S` | `design.md` § 5.2 (`:418-441`) |
| M-B bundle hygiene, four legs | `design.md` § 5.2 (`:330-350`) |
| fixture variants F2 / F3 / F5 | `fixtures.md` §§ F2, F3, F5 (`:87-207`) |
| the pipeline seam the harness plants into | `control/pipeline.sh:196-222` (`pipeline_capsule`), `:230-501` (`pipeline_run`) |
| the row-recording shape to copy | `control/probe-c2.sh:84-153` (`row_begin`/`row_outcome`/`record_row{,_na}`) |
| the scenario shape to copy | `control/selftest.sh:67-97` (`scenario`, `declaration_with_verify`) |
| M-B legs already exercised — reuse, don't reimplement | `control/selftest.sh:217-283` (`selftest_bundle`) |
| the DQ-4 exemption PHASE-05 inherits | `notes.md` F-P04-9 (`:483-495`) |
| open vocabulary gaps (S1 lives here) | `notes.md` § PHASE-03 open questions (`:326-343`), F-P04-10 |

Knowledge, **both tiers** (`doctrine knowledge show <ID>`): QUE-200 (settles on
this evidence) · QUE-201 (guard (e) is its only evidence input) · QUE-202 ·
QUE-203 · ASM-008 · CON-004/005 · DEC-099/101/102/103/104 · CPT-001 (the five
classes + 3g — the `vector-class` column's vocabulary).

**Research advisory:** `doctrine slice research 241` reports drift. No
`research.md` was ever authored — the pre-design round was not run, so the
advisory is reporting an absent artefact. Standing decision: not restamped
(`notes.md:722`, rationale in `plan.md` § Notes). Do not chase it.

## Assumptions & STOP conditions

Assumptions carried in:

- **A-1** — the happy path is green *before* any hostile row claims a kill
  (§ 9). Re-run `rig selftest` at T0; a red there means every "refused" below is
  indistinguishable from "rig broken" (R4).
- **A-2** — `pipeline_capsule` and `pipeline_run` are separate calls precisely
  so the harness can plant between them (`pipeline.sh:196-200`). `Hnn.mutate`
  runs on the capsule repo trusted-side — the rig plays the adversary. Rows
  whose hostility is *capsule-time* (H13 bundle path, H14 doorbell) need a
  capsule-side vehicle instead; see T4c.
- **A-3** — altitude's "both" is **both fixtures** (light + heavy), not both
  mechanisms: `n/a` on light ⇒ `unproven-beyond-rust` only parses that way
  (§ 5.4). So declining the heavy column caps every row at `client-local`.

STOP conditions — `/consult`, never improvise past:

- **S1 (live, inherited).** A row that lands on a **verify capsule overrunning
  the disk cap** has no legal token: `verify/resource-cap` is not in the closed
  set, and `verify_stage`'s `case` currently mis-attributes it to
  `verify/suite-failed` (F-P04-10, OQ-a). **Do not mint a token** — the closed
  set is what `assert_outcome` keys off. Record the STATUS and consult.
  Also live: OQ-b (worker overruns wall clock — H15's business) and OQ-c (M-A
  has no absent-result token; `harvest-fetch.sh` raises a RIG DEFECT).
- **S2.** A failed row is a **FINDING and a `/consult`**, never a quiet rig edit
  (EX-12, probe-specs § Order and gating). A kill at a *later* boundary than
  expected is a **partial** fail (defence held, layer missing); no kill is a
  fail. `n/a` is a legal recorded outcome; a silent pass is not.
- **S3 (the inherited DQ-4 condition).** `audit-dq4` exempts exactly one site
  family: `fixture-light.sh` runs `npm` **trusted-side** at build time
  (`:88`, `:101`, `:139-140`). That is outside DQ-4's subject only because the
  tree is assembled from this repo's authored sources. **A payload-bearing
  fixture variant that reuses that build loop executes the planted payload
  trusted-side and breaks DQ-4 for real.** Therefore: payloads are planted into
  the *capsule's result tree* per cell, never into a fixture build. Re-run
  `control/audit-dq4.sh` after T1 and after T4d; a red there is a STOP.
- **S4.** No `src/` changes (slice non-goal). A blocked read verb or a
  candidate-verb precondition is a `/consult`, not a patch.
- **S5.** No RFC-025 prose edits (slice non-goal). § 5.6 is the source of truth
  for `expected-stage`/`expected-token`; probe-specs' inherited column is not.

## Tasks

Commit at every `⌁` — the tree is shared and actively hostile (F-P04-6). Path-
limit both the `add` and the `commit`; `git status --porcelain` first.

### T0 — baseline re-green (A-1, EX-2 precondition) — [x] DONE 2026-08-02

- [x] `./rig selftest` green; `control/audit-dq4.sh` and `control/audit-nohooks.sh`
      clean (they are re-run at T1/T4d too, but a dirty baseline must be known now).
- [x] Record the observation in this sheet's § Findings — VA-1's "the run is
      recorded" is a file, not a memory of a terminal. See F-P05-1.

### T1 — fixture variants F2 and F3 (EX-11, EX-15) — [x] DONE 2026-08-02 · `7c134057`

- [x] **F2 — light + IN-REPO declaration copy.** Extend `control/fixture-light.sh`
      with `--variant <base|inrepo|plan>` (DRY — do not fork a second provisioner);
      `inrepo` lands at `fixtures/light-inrepo/repo` with the declaration copied
      **inside** the repo and committed. This manufactures the F-5 exposure the
      rig does not otherwise have.
- [x] **F3 — light + plan + phases.** SL-001 gains `plan.toml` with ≥1 phase
      driven to `completed`, so the sub-probe meets `prepare-review`'s
      phase-completion gate. Provisioned **up front** (EX-15) — discovering it
      mid-phase costs a rebuild.
- [x] Amend `fixtures.md` F2/F3 rows NOT BUILT → BUILT with their build command.
      **Amended, never rewritten** (EX-11 of PHASE-01).
- [x] Re-run `audit-dq4` (S3): the variants carry no payload, so it must stay clean.

  See F-P05-4 (the run), F-P05-5 (the runtime-state constraint T5 inherits).

### T2 — the authored matrix spec (EX-1, EX-4, EX-8, VT-1) — [x] DONE 2026-08-02 · `49fbce82`

See D-P05-2 (the encoding), D-P05-3 (where the validator lives), F-P05-6 (the
run + positive controls), F-P05-7 (the `IFS=$'\t'` collapse). VT-1 now reads
**UNATTRIBUTABLE** rather than FAIL — the file exists and its keywords are
present; attribution resolves on the flip to `completed` (F-P04-11, T9).

- [x] Author `scripts/spike-capsule/probes/c3/matrix.tsv` — **committed input**,
      not a results file. Columns exactly per § 5.4:
      `row | fixture | mechanism | harness | vector-class | instantiation |
      expected-stage | expected-token | planted | outcome | altitude`.
- [x] `expected-stage`/`expected-token` from the **closed vocabulary**
      (§ 5.1) and the **re-derived** boundary (§ 5.6). Assert mechanically that
      every non-empty `expected-token` is in `token_legal`'s set — an authored
      typo would otherwise score as a rig defect at run time.
- [x] `planted`/`outcome`/`altitude` cells are **empty in the authored file** and
      filled only by the harness (EX-5, VA-1: altitude is verified generated, not
      hand-authored — assert the authored column is empty).
- [x] Cells known `n/a` up front carry `n/a` **and a reason** (EX-4, EX-8):
      H13/M-A (no trusted-side-read artifact exists at all — an asymmetry that is
      itself a QUE-200 verdict input), H12/light (no `.envrc`/`flake.nix`).
- [x] Cell plan — `(row × fixture × mechanism)`, 16 × 2 × 2, minus recorded `n/a`:

  | row | boundary (§ 5.6) | fixture | note |
  |---|---|---|---|
  | H1 | `conform/ancestry-not-descendant` | both | |
  | H2 | `harvest/oid-mismatch` | both | pin read once (RT-5) |
  | H3 | `conform/ancestry-merge-commit` | both | I3 ordering |
  | H4 | `conform/undeclared-path` | both | |
  | H5 | `conform/forbidden-path` | both | **three forms** — plain, non-ASCII, rename-out; guard (b)+(c) |
  | H6 | **dissolution** | both | structural observable: config/hooks are repo-local, never objects |
  | H7 | `harvest/resource-cap` | both | I1's object-count clause is the observable |
  | H8 | `conform/gitlink`, `conform/gitmodules` | both | guard (a) |
  | H9 | **dissolution** at ingest + containment at `verify` | both | two-part, scored as two |
  | H10 | `advance/stale-base` (pipeline) **+** sub-probe | light / F3 | `harness=both`, legs scored separately |
  | H11 | `verify` — the sandbox profile | both | `build.rs` on heavy, `postinstall` on light |
  | H12 | **dissolution** (audit row) | heavy; `n/a` on light | ⇒ at most `unproven-beyond-rust` |
  | H13 | M-B: `bundle-unsafe-path`/`bundle-absent`/`bundle-invalid`/`resource-cap`; **M-A `n/a`** | both | re-homed (F-6) |
  | H14 | `harvest` — idempotency (I2), poll fallback, no authority | both | three legs |
  | H15 | **dissolution** — killed at **each stage in turn** (EX-9) | both | |
  | H16 | `advance/stale-base` (pipeline) **+** sub-probe | light / F3 | as H10 |

### T3 — the harness (EX-2, EX-3, EX-5, EX-12, VA-1) ⌁

- [x] **Refactor first (DRY, no parallel implementation).** Lift
      `row_begin`/`row_outcome`/`record_row`/`record_row_na` out of
      `probe-c2.sh:84-114` into `lib/` and source it from both. Outcome is
      **derived from the assertions the row actually made**, never passed in —
      that shape is VA-3 made mechanical and PHASE-04 earned it. Behaviour-
      preservation gate: `rig c2` must stay green **unchanged** after the lift.
      — **DONE 2026-08-02 · `3ad74d83`.** `lib/rows.sh`; `rig c2` output
      **byte-identical** before and after (diff empty), which is why
      `rows_assert_complete` takes the label `VA-3` rather than hardcoding the
      message. Parameterised by `ROWS_OUTCOME_FIELD` / `ROWS_OBSERVABLE_FIELD`
      (P-C2 records 4 columns, P-C3 11) — for P-C3 the observable that must not
      be empty is **`planted`**, the per-cell positive control. The non-default
      splice positions have no caller yet, so they were checked directly: first,
      middle, last-of-four, and 10-of-11 all land right, and the last-field case
      is the off-by-one that would have silently DROPPED the verdict.

  **The harness landed 2026-08-02 · `1408f1ae`.** See F-P05-8 (the run,
  the falsifiability pass, and what is NOT yet proven) and D-P05-4 (an
  unwritten row is a refusal, not an `n/a`). Next: T4a.
- [x] `control/probe-c3.sh`, dispatched by the existing `rig c3` arm
      (`rig:146`). Loop per § 5.4, in this order, no shortcuts:
      `guard_not_real_repo` (I6, before anything) → `Hnn.mutate` →
      `Hnn.planted?` → harness (`pipeline` | `conflict` | `both`) → record the
      **first refusing stage + token** → `Hnn.assert` → `assert_outcome` →
      emit result row. — hooks are `H<n>_mutate` / `H<n>_planted` /
      `H<n>_assert`, derived mechanically from the `row` column (no translation
      table), each taking `<run> <fixture> <mechanism> <alt>` and `_assert` the
      observed boundary as a fifth. `matrix_validate` runs before provisioning
      (D-P05-3); `lib/matrix.sh` gained `matrix_read` so the harness rides the
      one split rather than re-acquiring F-P05-7.
- [x] `pipeline_run` is **redirected, never piped** (a pipe subshells it and a
      RIG DEFECT return cannot reach the caller — F-P01-1 family, and
      `selftest.sh:75-87` is the reference form). — `cell_pipeline_leg`
      publishes `CELL_OBSERVED` rather than printing it, for the same reason:
      a caller forced into `$( … )` to read the refusal could not receive the
      DEFECT either.
- [x] Results to `$SPIKE_CAPSULE_ROOT/probes/c3/results.tsv` (runtime tier),
      same columns as the spec with `planted`/`outcome`/`altitude` filled.
      **`results.tsv` is the only thing the driving session reads** (R5, EX-14).
- [x] Refuse to finish if any cell left `outcome` or its observable empty —
      the probe-c2 precedent. Silent absence is the failure a results table is
      least able to show. — `rows_assert_complete 'VA-1'` against a count
      derived **from the spec** (`c3_expected_entries`), which is the only form
      that can see a cell that never ran.
- [x] **Altitude computed** (EX-5): holds under both fixtures ⇒ `model-level`;
      one only ⇒ `client-local` **and the divergence is a finding**; `n/a` on
      light ⇒ `unproven-beyond-rust`. `n/a` is **excluded** from the
      computation, never counted as a hold. — `cell_altitude`, stamped onto the
      row's entries once all its legs have run (the column is a property of the
      ROW; the outcome beside it is derived per leg, so the two are filled at
      different moments). A row that holds nowhere claims no altitude.
- [x] `--positive-control` self-check on the harness itself: a cell whose
      `planted?` is forced false must **red**, not pass. Otherwise EX-3 is a
      claim about code rather than an observation. — and it runs at the head of
      **every** invocation, not only under the flag: a scorer proven after the
      fact is proven for the next run, not for this one.

### T4 — the row instantiations, `Hnn.{mutate,planted,assert}` ⌁ per group

Grouped so each group is a commit and a re-runnable `rig c3 <rows…>`.

- [x] **T4a — DONE 2026-08-02 · `ac9b534c` (H1 H3 H4) + `4b6ee92b` (H2 H5).**
      All five result-tree rows live, every cell `pass`, every row
      **model-level** — F-P05-24. The two blockers were ruled in `/consult`:
      H2 dissolved (D-P05-8, was F-P05-13), H5's form set made per fixture
      (D-P05-10, was F-P05-14/21/22). R-B is answered and the heavy join is
      proven — F-P05-10. See also D-P05-5 (where the rows live).
      **result-tree rows: H1 H2 H3 H4 H5.** Ordinary git mutations on the
      capsule repo between `pipeline_capsule` and `pipeline_run`. `probe-r2.sh`
      edges 3–6 are already the positive controls (`fixtures.md:206`) — cite
      them, do not re-derive.

      **H5's form set is per fixture (D-P05-10), and both paths are constrained
      by leg 2** (F-P05-18, F-P05-21): a plain `.doctrine/` edit and a
      **non-ASCII** path, BOTH under `.doctrine/rfc/025/evidence/` — the only
      `.doctrine/` prefix that is design-target on heavy, and covered by
      `.doctrine/**` on light — plus a **rename out of** `.doctrine/` on **light
      only**, whose destination must also be design-target (`src/**`). The
      spec's literal `.doctrine/naïve.md` would refuse `undeclared-path` on
      heavy. The belt-hardening GUARDS the three forms used to carry moved to T6
      as isolated probes; the row's job is the boundary.
- [x] **T4b — dissolutions: H6 H9 H12 H15. DONE 2026-08-02** — written, scored
      green (330/0), and falsified (6 mutants, 6 reds, no survivors).
      Scored as **recorded dissolutions
      with their structural observable** (EX-7), not silently skipped and not
      forced into a stage kill. A dissolution is a hazard the model *removes* —
      the design's best result. H15 is killed at **each stage in turn** (EX-9),
      not at one interruption point. H9 splits: inert-at-ingest (dissolved, no
      tree materialised trusted-side, I4) + containment at `verify`.

      **All four rows are WRITTEN** (`efbb097a` H6/H9/H15; H12 follows on
      D-P05-11). Per-cell shakeouts green — H6/H9/H15 on light across both
      mechanisms and both of H9's alternatives, H6 and H9 on heavy. Two operator
      rulings landed mid-task: **D-P05-11** (H12 plants at the design-target
      directory, F-P05-26) and **D-P05-12** (H15 interrupts three stages;
      stage 4's indivisibility is observed, not raced).

      **THE AUTHORITATIVE RUN IS DONE — 2026-08-02, and it is GREEN.**
      `rig c3 H6 H9 H12 H15` exit **0**, **330 assertions, zero FAIL**. Log:
      `drivers/T4b-scored-authoritative.log`. Preceded by `rig selftest` exit 0,
      57 assertions (`drivers/T4b-selftest-prerun.log`) — A-1 established before
      anything the harness said was believed, which is the `apparatus` section
      D-P05-13 added doing its job on its first real use.

      | row | legs | verdict | altitude |
      |---|---|---|---|
      | H6 | 4 | pass | `model-level` |
      | H9 | 8 (both alternatives) | pass | `model-level` |
      | H12 | 2 heavy (light `n/a`) | pass | `unproven-beyond-rust` |
      | H15 | 4 | pass | `model-level` |

      `results.tsv` now carries nine rows under two stamps — T4a's `H1..H5` and
      T4b's `H6 H9 H12 H15`. H12's `unproven-beyond-rust` is **correct by
      construction, not a divergence**: light is a structural `n/a` (no `.envrc`,
      no `flake.nix`) and A-3 caps it there.

      **H15 scored `model-level` on all four cells**, so the older prediction of
      `client-local` + a divergence FINDING — already void once F-P05-28 was
      settled — is now falsified by measurement as well.

      **This run is F-P05-29's control.** Same rows, same code, a doctrine binary
      that runs: 32 FAIL → 0. The diagnosis is confirmed rather than merely
      argued. `DOCTRINE_BIN` was pinned to the bwrap-bound read-only store build
      (0.35.0, matching the tree) so a co-agent's rebuild could not move the
      apparatus mid-run — the mechanism that voided the previous run.

      **THE FALSIFIABILITY ROUND IS DONE — 6 mutants, 6 reds, NO SURVIVORS.**
      Driver: `drivers/falsify-t4b.sh` (throwaway, never calls `rows_write`).
      Sweep: `drivers/falsify-t4b-sweep.log`, 18 assertions, reproducible.

      | # | perturbation | red | isolated by |
      |---|---|---|---|
      | M1 | H6's hooks written non-executable | `planted?` x-bit clause | hooksPath still absolute, hook file present |
      | M2 | H6's `core.hooksPath` made RELATIVE | `planted?` absoluteness clause | hooks still executable AND one still FIRED |
      | M3 | H9's escape symlink dereferenced | `planted?` mode-`120000` clause | all four paths still in the range |
      | M4 | H15's kill made a no-op | `planted?` `=COMPLETED` clause | three attempts still made |
      | M5 | H12 planted at the ROOT | `conform/undeclared-path` | payload proven landed first |
      | M5b | root `.envrc` — F-P05-26's SECOND reason | `git add` refuses (gitignored) | file proven present in the worktree |

      **Each mutant carries an ISOLATION CONTROL** asserting the clauses it did
      NOT target still hold, so an empty `planted?` is attributable to the one
      clause under test. A mutant that reds by breaking everything proves nothing
      about the clause it was written for. Three unmutated controls green.

      **Every mutation WRAPS the real function rather than restating it**
      (`rebind` + a wrapper), so the driver cannot drift from the row it
      measures — a hand-copied function body is precisely the
      vehicle-differs-from-the-real-one failure this phase keeps meeting
      (F-P05-25, F-P05-29).

      **M2 is the one that mattered.** The vacuity trap reds on the absoluteness
      clause *specifically*, with the hooks still executable and one still
      observed firing — so H6's observable is shown non-vacuous by the check
      written for it, not by the payload having gone dead.

      **The round validated the landing guard against the driver itself.** M5's
      first run emitted the CORRECT `conform/undeclared-path` while `planted?`
      was empty: the override used `./flake.nix`, but `c3_planted_paths` matches
      `grep -qxF` against git's changed-path list, which reports `flake.nix`.
      Reading the token alone would have banked a refusal of nothing. Fixed in
      the driver; the row was never implicated.

      **Cost — the round ran in seconds, not the ~75 min a row sweep costs.**
      `planted?` is evaluated BEFORE the pipeline leg (`probe-c3.sh:571`) and the
      harness's own positive control already proves a forced-false `planted?`
      reds the cell (`probe-c3.sh:301`). So M1–M4 need no verify leg at all: the
      two halves of the composition are established at opposite ends. Only M5
      needs a pipeline, and it refuses at conform, upstream of verify.

      **Controls that the round left nothing behind:** fixture HEADs unchanged
      (light `7f43b875`, heavy `4fc9d32f`), `results.tsv` still exactly the nine
      real rows with no mutant append, run dirs torn down, `~/capsules` flat at
      188M.
- [x] **T4c — capsule-time rows: H7 H13 H14. DONE 2026-08-03 (`6312f519`).**
      H13 and H14 done 2026-08-02; **H7 scored 2026-08-03 — the last row in the
      matrix. ALL SIXTEEN ROWS ARE NOW SCORED.**

      **H7 — scored `rig c3 H7` exit 0, 100 ok / 0 FAIL, 4s.** All four cells
      `pass`, the row **model-level**. Log: `drivers/T4c-scored-h7.log` (scratch;
      the durable evidence is `results.tsv` + the committed drivers).

      | cell | verdict | altitude |
      |---|---|---|
      | light fetch / light bundle | pass | `model-level` |
      | heavy fetch / heavy bundle | pass | `model-level` |

      **THE ROW DOES THE HONEST WORK FIRST, AND THAT IS THE ROW.** The capsule
      commits, publishes `RIG_RESULT_REF`, writes a VALID bundle and rings —
      and only then exhausts its bound. A harvestable result therefore stands
      at the moment of refusal, so the row can say the cap arrived **before**
      ingestion rather than *instead of* it. A capsule that merely died would
      refuse at stage 1 for the uninteresting reason that there was nothing to
      ingest, and `harvest/resource-cap` would be indistinguishable from an
      empty capsule. M28 is the mutant that proves this clause carries alone.

      **Cheap by construction — 4s for the row, against a 20G blast-radius
      bound that is never approached.** The blob is SPARSE, so the **per-file**
      leg fires (`ulimit -f` → SIGXFSZ → 153 → `RIG_EXIT_DISK`) and no real
      bytes are written. The **cumulative** leg is deliberately left to P-C2,
      which already drives both legs against both capsule kinds with a positive
      control at an 8 MiB cap (`probe-capsule.sh:139-177`). Re-proving it here
      would have been a second copy of someone else's claim, and on H7's cap it
      would have cost cap-many REAL bytes per execution — the cost D-P05-18
      declined.

      **Measured attribution.** Heavy capsule = **201M** against the 512 MiB
      row-local cap (39%); light = 1.5M. F-P05-10's 195M plus the deep tree.
      The clause that measures it is falsifiable rather than decorative — M30
      reds it on heavy and it HOLDS on light.

      **The seam this row needed is F-P05-37 / D-P05-19**, landed separately at
      `2b710243` and proven inert by selftest 57/0 unchanged.

      **Falsified — 6 mutants, 6 reds, no survivors, 18 assertions.** Driver
      `drivers/falsify-h7.sh`, sweep `drivers/falsify-h7-sweep.log`. **All
      committed this time** — see F-P05-39.

      | # | perturbation | red | isolated by |
      |---|---|---|---|
      | M26 | the sandbox never REPORTS the bound | status + ingestion + boundary | `planted?` still holds — the payload was real, only the report changed |
      | M27 | the oversize is not empty | `planted?` zero-length half | the capsule still died on the disk bit; boundary untouched |
      | M28 | the capsule died holding NOTHING | `_assert` clause 2 **alone** | token, boundary, ingestion and `planted?` ALL unchanged |
      | M29 | the pathological tree never landed | `planted?` depth clause | blob and refusal untouched |
      | M30 | the cap set BELOW the honest footprint | `planted?` on HEAVY only | LIGHT holds — so the clause measures the capsule, not a constant |
      | M31 | canonical gains the capsule's objects | `assert_outcome` object count | refs clause BLIND to it; every row clause held |

      **M28 is the one the row exists to survive.** Every observable except
      `_assert`'s second clause is identical between a capsule that died holding
      a valid result and one that died holding nothing. Without that clause the
      row would have claimed the cap beat ingestion while only ever having shown
      there was nothing to ingest.

      **M30 is the one that vindicates D-P05-18.** It reds on heavy and holds on
      light over one capture — a clause reporting the same thing under both
      fixtures would not be measuring the capsule at all. That is
      `c3_assert_range`'s two-directions-over-one-computation discipline applied
      to a threshold instead of a range.

      **THE FRAMING BELOW IS PARTLY WRONG AND IS CORRECTED HERE.** "These need
      hostility *inside* the capsule, so a hostile worker vehicle rather than a
      trusted-side mutate" holds for **H7 only**. Measured while sizing the
      task:

      - **H13** attacks a capsule-authored FILE at a control-plane-chosen path.
        `selftest_bundle` already performs all four attacks trusted-side, so no
        hostile worker is needed and none was written.
      - **H14** manipulates the doorbell — control plane (`rig_wait_doorbell`,
        `PIPELINE_DOORBELL_DEADLINE`), also reachable trusted-side.
      - **H7** is the row that genuinely needs capsule-time hostility, and it
        carries the resource risk: it must set its own disk cap, the heavy
        worker capsule already sits at 195M against 256 MiB (F-P05-10), and
        `capsule/worker-hostile.sh` (§ 9.1) **does not exist yet**. Cap
        disposition is D-P05-18; the 195M/256 MiB ambiguity it names is
        answered by a control, not by a wider cap.

      **H13 — DONE (`5c5ed437`), scored `rig c3 H13` exit 0, 110 ok / 0 FAIL in
      7.2s.** Nothing reaches verify: it is the first instantiated row to refuse
      at stage 1, so `_assert` observes the ingestion boundary itself — no
      result ref in the quarantine, and harvest emitted alone.

      | cell | verdict | altitude |
      |---|---|---|
      | light/heavy fetch | `n/a` (matrix, up front) | — |
      | light bundle × 4 alternatives | pass | `model-level` |
      | heavy bundle × 4 alternatives | pass | `model-level` |

      The fetch `n/a` is **structural, not unwritten** (R-C): M-A reads no
      capsule-authored artifact, so the row has no subject there. That
      asymmetry is a QUE-200 verdict input in its own right (F-6).

      **Reuse, per the plan** (`35431bc6`): the four mutators moved from
      `selftest.sh` to `lib/common.sh` beside `RIG_BUNDLE`, so H13 and
      `selftest_bundle` attack the same path through one copy. The size-cap leg
      stayed out — it is a THRESHOLD, not a mutation, and capping below an
      honest bundle is the point. Behaviour-preserving: selftest unchanged at 57.

      **Falsified — 4 mutants, 4 reds, no survivors, 4 controls** (one per
      alternative). Driver `drivers/falsify-t4c.sh`, sweep
      `drivers/falsify-t4c-sweep.log`, 16 assertions.

      | # | perturbation | red | isolated by |
      |---|---|---|---|
      | M6 | the capsule writes a ZERO-BYTE bundle | pre-mutation size control | **the leg's own observable STILL HELD** |
      | M7 | the symlink is never planted | `-L` clause | path is a regular file, byte-identical |
      | M8 | the truncation never happens | size + verify clauses | bundle unchanged AND still valid |
      | M9 | the cap raised ABOVE the honest bundle | threshold-bites clause | artifact present and untouched |

      **M6 is the one this row exists to survive.** Three of H13's four legs are
      absence-shaped, which is the class that passes when its subject was never
      reachable. Under M6 the bundle really was absent — the leg's own check
      passed — and ONLY the pre-mutation size control red. Every leg therefore
      pairs its observable with the honest bundle that stood there first.

      Scaffolding for both rounds now lives in `drivers/falsify-lib.sh`; the
      T4b sweep re-ran at 18/0 after the extraction (which is how a silently
      dropped isolation control was caught).

      **H14 — DONE (`6a9b7336`), scored `rig c3 H14` exit 0, 88 ok / 0 FAIL.**
      Log: `drivers/T4c-scored-h14.log`. All four cells `pass`, the row
      **model-level**.

      | cell | verdict | altitude |
      |---|---|---|
      | light fetch / light bundle | pass | `model-level` |
      | heavy fetch / heavy bundle | pass | `model-level` |

      **THREE LEGS IN ONE CELL, and that is the matrix as authored at T2** — one
      `expected-stage=harvest`, no token, so `cell_alternatives` yields a single
      alternative and the legs live inside it. H15's shape for D-P05-2's reason:
      the alternation encodes BOUNDARIES, and these three share one. No matrix
      edit was needed or made.

      **The doorbell is UPSTREAM OF STAGE 1**, which is what shapes the row.
      `pipeline_capsule` waits on the ring and `pipeline_run` harvests
      afterwards, so the ring is not something the pipeline leg can be made to
      observe. Each leg therefore takes its own observation at that seam
      (`rig_wait_doorbell`, never a second implementation), and the single run
      that follows is the JOIN: the bell is left **forged** for it, and the
      trusted side still pins the capsule's own published OID. Duplication is
      observed where it costs something — a second harvest into a **fresh**
      quarantine, never the run's own (F-P05-28's rule: an assertion must not
      write to its own subject).

      `probe_doorbell` (`control/probe-capsule.sh:182`) already exercises the
      four properties against a bell THE PROBE rang. H14 adds the two things
      that probe cannot reach: the bell the WORKER rang, and a real pipeline
      downstream of the forgery. Its `rung = published` clause is that join made
      a precondition.

      **Reuse, per the plan** (`e3e4f87d`): `pipeline_quarantine` and
      `pipeline_harvester` extracted from `pipeline.sh` rather than hand-rolled
      in the row — a second quarantine that differed from the one every stage
      ran against would be invisible until it mattered. `pipeline_harvester`
      PUBLISHES rather than prints (F-P01-1: a `$( … )` caller would swallow its
      `rig_die`). Behaviour-preserving: selftest 57, assertion list
      byte-identical before and after.

      **The three hostile acts have NAMES** — `c3_h14_rering` / `c3_h14_silence`
      / `c3_h14_forge`, in `bundle_symlink`'s shape — so the falsifiability
      round can no-op each one. That is what caught F-P05-30.

      **Falsified — 5 mutants, 5 reds, no survivors, 1 control, 19 assertions.**
      Sweep: `drivers/falsify-t4c-h14-sweep.log`. H13's half of the same driver
      re-checked at 16/0 afterwards.

      | # | perturbation | red | isolated by |
      |---|---|---|---|
      | M10 | the worker rings a CONTENT-FREE bell | `rung` join clause | leg 1 still reads clean — two rings, one distinct |
      | M11 | the waiter gives up instantly instead of polling | `lost-elapsed` clause | bell really gone AND the timeout still reported |
      | M12 | the forgery is never written | leg 3's forged-capsule clause | legs 1 and 2 both still read clean |
      | M13 | the ring is never lost | `lost-bell` / `lost-status` clauses | the wait ran and SUCCEEDED — the loss never happened |
      | M14 | the duplicate never happens | `rings` count clause | echoes and the unmoved ref still hold |

      **M14 is the one that changed the row** — see F-P05-30. **M11 is the one
      that matters most for the claim**: fail-fast and poll-to-deadline are
      indistinguishable on the returned status alone, and only the elapsed
      clause separates a lost ring costing latency from one costing correctness.

      **M12's isolation control red on its first run, and the control was
      wrong, not the row.** It asserted the honest ring was still intact at the
      bell; in fact leg 2 had already destroyed the bell and the no-op'd forgery
      never rewrote it, so leg 3 faced an ABSENT bell. **An isolation control
      must assert the state the mutation actually leaves, not the tidy state it
      is imagined to leave** — the composition of a no-op with the legs that ran
      before it is not always the state the un-mutated run would have reached.

      **Original framing, retained for the parts that still hold:** These need
      hostility *inside* the
      capsule (bundle path, doorbell ring, oversized blob), so they want a
      hostile worker vehicle rather than a trusted-side mutate — design § 9.1
      already names `capsule/worker-hostile.sh`. **H13's four M-B legs are
      already exercised by `selftest_bundle` (`selftest.sh:217-283`) — reuse the
      function, do not write a second copy.** H14's three legs: duplicate ring
      (no-op by content-addressing, I2), lost ring (poll to deadline), spoofed
      ring (carries no authority — content is never read).
- [ ] **T4d — payload rows: H8 H11. H8 DONE 2026-08-02 · `d1e67fb7`** — written,
      scored green, falsified. H11 remains.

      **H8 — scored `rig c3 H8` exit 0, 100 ok / 0 FAIL**, all eight legs
      (4 cells × 2 alternatives) `pass`, both rows `model-level`. Log:
      `drivers/T4d-scored-h8.log`. Cheap on heavy as well — the row refuses at
      **stage 2**, so like T4a it never pays the verify capsule.

      **The two alternatives are planted SEPARATELY, not together as the
      matrix's `instantiation` prose reads.** Leg 4 returns on the first entry
      matching either arm (`pipeline.sh:487-501`), so a tree carrying both
      refuses once and the other alternative's leg would score its sibling's
      refusal as its own. That is **F-P05-22's short-circuit arriving from a
      second direction**, and the same resolution applies: isolation is what
      makes a form a control. No matrix edit — the alternation already splits
      the row into two scored legs; the prose describes the attack.

      **`_assert` MEASURES F-2 rather than asserting it.** The row's whole
      reason is that every other leg passes it — `reject_submodules`
      (`src/git.rs:2432`) scans `git ls-files --stage` and is index-scoped, so
      it is unreachable from an object-only pipeline. So the row runs **leg 2's
      own verb** over the same range and observes it ACCEPT. The kill is
      attributable to leg 4 alone, as an observation of the other leg rather
      than a claim about code that is not being run.

      **Both forms sit under the fixture's design-target directory (D-P05-11's
      reason, the trap's fourth appearance).** Leg 2 runs BEFORE leg 4, so a
      form at an undeclared path refuses `undeclared-path` and the cell scores a
      defect of the MODEL. For `.gitmodules` that means a **nested** one, which
      leg 4's `*/.gitmodules` arm matches deliberately. **What the row does not
      show**: that a ROOT `.gitmodules` — the only one git itself reads — is
      reachable under these fixtures' selectors. It is not, and that is a fact
      about a selector list, stated in the row rather than left for the results
      table to imply.

      **Falsified — 3 mutants, 3 reds, no survivors, 2 controls, 10
      assertions.** Driver `drivers/falsify-t4d.sh`, sweep
      `drivers/falsify-t4d-sweep.log`.

      | # | perturbation | red | isolated by |
      |---|---|---|---|
      | M15 | the gitlink arrives as an ORDINARY FILE | `160000` mode clause | the path still lands in the range, as a real file |
      | M16 | the `.gitmodules` is planted at an UNDECLARED path | `conform/undeclared-path` at the BOUNDARY | `planted?` stays LIVE — the payload really landed |
      | M17 | the `.gitmodules` arrives as a GITLINK | `100644` mode clause | the name is still `.gitmodules`, and the sibling arm would fire |

      **M16 is the trap made deliberate.** It is scored at the boundary rather
      than at the positive control because `planted?` **cannot** see it: the
      payload lands, the range carries it, and only the observed token says the
      row measured the wrong leg. That is F-P05-14/18/26 stated as a
      reproducible measurement instead of a lesson. **M17 is the adjacent
      observable one arm over** — without the `100644` clause the `.gitmodules`
      leg would report its guard firing on evidence belonging to the mode arm.

- [x] **T4d, remaining: H11 — DONE 2026-08-02. SCORED GREEN ON BOTH FIXTURES,
      at two different boundaries.** `rig c3 H11` exit **0**, **80 assertions,
      zero FAIL**, all four cells `pass`, the row **model-level**. Log:
      `drivers/T4d-scored-h11.log`, preceded by `rig selftest` exit 0 / 57
      (`drivers/T4d-h11-selftest-prerun.log`) — A-1 before anything the harness
      said was believed. `results.tsv` now carries **thirteen** rows.

      | cell | boundary | verdict |
      |---|---|---|
      | light fetch / bundle | `verify` — containment, token-less | pass |
      | heavy fetch / bundle | `conform/undeclared-path` | pass |

      Both operator calls landed in `/consult` (2026-08-02): **D-P05-14** on the
      network half and **D-P05-15** on the heavy cell. The two findings below
      are DISPOSED, not outstanding — read them with their decisions.

      **Shakeout — 12 assertions, 0 FAIL, both mechanisms**, logs
      `drivers/shake-h11-light-{fetch,bundle}.log`. All four stages pass, so the
      observed refusal is empty and a token-less `verify` row scores `pass` on
      that; the claim is in `_assert`, as § 5.6 intends for this row.

      | observable | measured |
      |---|---|
      | the payload EXECUTED, in `/capsule/repo` | yes — `escape=denied:ENOENT`, `canary=connected`, `cwd=/capsule/repo` |
      | host sentinel after the run | **ABSENT** — contained, and the attempt is recorded beside it |
      | arrivals at the trusted-side canary | **1** — reported, deliberately unscored (F-P05-32) |

      **Falsified — 5 mutants, 5 reds, no survivors, 2 controls, 18
      assertions.** Driver `drivers/falsify-t4d.sh`, sweep
      `drivers/falsify-t4d-h11-sweep.log`.

      | # | perturbation | red | isolated by |
      |---|---|---|---|
      | M18 | the canary is never armed | `planted?` (armed) | the payload is planted, reachable, in the range |
      | M19 | the canary is armed but its log is already dirty | `planted?` (quiet) | it IS listening — only the log is dirty |
      | M20 | the escape is never attempted | the `TRIED` clause | exactly one red; containment still reads clean |
      | M21 | containment fails open (sentinel placed by the rig) | the `ABSENT` clause | exactly one red; the payload still reports denied |
      | M22 | the payload is INERT — an empty file at the same path | EXECUTED + cwd + 2 | **the containment clause PASSES ANYWAY** |

      **M22 is the row's reason for existing.** With nothing running, the
      sentinel is absent because nothing tried — H13/M6, H14/M14 and H8/M16 in
      a fourth costume — which is why the execution evidence is asserted before
      any clause that is shaped like an absence.

      **M18 found a real defect rather than confirming one** (F-P05-34): the
      arming check was `kill -0 "${C3_H11_CANARY_PID:-0}"`, and `kill -0 0`
      addresses the caller's own process group and SUCCEEDS. A never-armed
      canary read as armed. A `:-` default that lands on a meaningful value
      answers a different question than the one being asked.

      **`audit-dq4` refused the first canary on sight** — six lines of `node`,
      trusted-side, and `node` is a light `exec:` token. F-P05-3 predicted that
      audit would be the thing that caught a payload-bearing row reaching for
      the project's toolchain; it was, on the first run, against the row it
      named. The listener is `socat` now, in neither fixture's list, and it logs
      the BYTES it receives so the rig's own readiness knock cannot be counted
      as an arrival. Guard kept, dependency dropped — not an exemption.

      **A third entry point in the round** (`expect_assert` in
      `falsify-lib.sh`): H11's reds land inside `_assert`, which neither
      `expect_planted` nor `expect_refusal` reaches. It runs the full leg, then
      calls `_assert` with its verdicts captured and its failures rolled back
      out of the round's own count. **`ASSERT_REDS` doubles as the isolation
      control** — "exactly one clause red, and it is this one" says in one line
      what a separate isolation function says in several, because every other
      clause of the row is standing in the same captured log.

      **DISPOSED — the two findings and what became of them** (2026-08-02).
      Preconditions probed
      trusted-side against the profile itself, log `drivers/T4d-h11-probe.log`:

      - **F-P05-32** — the verify capsule **shares the network namespace**
        (`--share-net`, and `--kind` never reaches the profile), so the canary
        clause is unmeetable *by design*: the filesystem half of H11 is contained
        and the network half was never claimed by any mechanism. EX-2's uniform
        posture is why. A one-line `--no-net` would green the row and retire EX-2
        silently — S2 forbids it.
      - **F-P05-33** — **heavy has no reachable trigger under SL-241's
        selectors.** Light's `npm test` runs `node --test src/*.test.ts` and
        `src/**` IS light's design target, so light is fully live. Heavy's
        `just web-build && cargo test` executes only undeclared paths, and every
        declared path is shell that `cargo test` never reads.

      **Both calls landed.** F-P05-32 → **D-P05-14**: the canary stays measured
      and unscored, and the disposition is not "unclaimed forever" but *gated on
      QUE-204* — egress becomes allowlistable (not binary) and the row is
      re-runnable against it, the row and its canary harness already existing.
      F-P05-33 → **D-P05-15**: heavy is not `n/a`. It plants where cargo really
      runs — the root `build.rs` — and scores the refusal that actually happens,
      `conform/undeclared-path`, one boundary EARLIER than the matrix expected.
      See F-P05-35 for what that measured and F-P05-36 for what it cost.

      (the original task text, H8's half now discharged
      above; guard (a) falls out of it for T6). H11 plants a hostile
      `build.rs` (heavy) / `postinstall` (light) writing outside the workspace,
      plus a network canary; observable is sentinel-absent on the **host** path
      and canary unreached — *not* absence of error output, and *not* a `/tmp`
      path (tmpfs inside; F-P04-12). **S3 applies here**: plant into the capsule
      result, never into the fixture build loop. Re-run `audit-dq4` after.
- [x] **T4e — H10 / H16 pipeline leg. DONE 2026-08-03. SCORED GREEN, 8/8 CELLS
      `model-level`.** Move the accepted ref **before** harvest so the row lands
      on stage 4's **precondition** — `advance/stale-base`, nothing transferred,
      and therefore the **strict** `assert_outcome` clause (F-14). Getting stage
      4's ordering backwards reds exactly here. `selftest.sh:184+` was the
      reference scenario.

      **THE AUTHORITATIVE RUN IS DONE AND IT IS GREEN.**
      `SPIKE_C3_LEGS=pipeline rig c3 H10 H16` exit **0**, **128 assertions, zero
      FAIL**, 1478s. Log `drivers/T4e-scored-h10-h16.log`; preceded by `rig
      selftest` exit 0 / 57 assertions (`drivers/T4e-selftest-prerun-scored.log`)
      — A-1 before anything the harness said was believed.

      | row | cells | verdict | altitude |
      |---|---|---|---|
      | H10 | 4 (both fixtures × both mechanisms) | pass | `model-level` |
      | H16 | 4 (both fixtures × both mechanisms) | pass | `model-level` |

      **`SPIKE_C3_LEGS=pipeline` was REQUIRED and is recorded in the results
      stamp.** Both rows carry `harness=both`, so the harness refuses to run
      them until T5's `conflict_subprobe` exists — unless the pipeline-only
      selector is passed, which `probe-c3.sh:693` then writes into the
      `results.tsv` stamp (`legs=pipeline rows=H10 H16`). The conflict leg stays
      visibly OWED rather than the run looking complete; each row's own line
      still reads `harness=both`. That is the design working, not a workaround.

      **Fifteen of sixteen rows are now scored. Only H7 remains.**

      Shakeout was 8/8 green beforehand (light ~30s a cell; heavy 413 / 361 /
      376 / 411s). Heavy costs a full four-stage run per cell **by
      construction** — the row cannot be measured without reaching stage 4 — so
      the ~25-minute scored run is the floor for this pair, not an overrun.

      **The rows are the FIRST AND ONLY ones that write `canonical`,** and the
      instantiations preamble's "the capsule clone, and nothing else" is now
      SCOPED by them rather than quietly broken: their hazard *is* a
      canonical-side event, so no instantiation of them avoids it. What the rule
      protects survives — both re-snapshot through `pipeline_snapshot`, which
      `pipeline.sh:248-256` is re-callable for exactly this case, so
      `assert_outcome` keeps answering *did the PIPELINE change canonical*
      rather than *did anything at all*. `quarantine` is still untouched by
      every row.

      **Neither row plants a payload.** H10's conflicting pair meets on the
      fixture's own **stub path**, so the capsule's half of the pair is the
      worker's own commit and the row supplies only the peer; H16's mover takes
      a path the result never names. The peer landing is **simulated** — a
      commit on canonical's accepted ref reaches the same end state a real
      peer's stage-4 CAS would, and running a second full pipeline inside the
      mutate seam would put an unscored run inside a scored cell and double
      heavy's cost to prove the happy path `selftest_happy` already owns.

      **THE TWO ROWS SCORE IDENTICALLY, AND THAT IS THE RESULT.** Both refuse
      `advance/stale-base`. Stage 4 compares two OIDs and never reads a tree, so
      the genuinely conflicting pair and the trivially mergeable one are
      indistinguishable to it. That IS § 5.1's split made observable — the model
      HAS safety, does NOT have resolution — and since the token cannot carry
      it, each row carries it in `_planted` / `_assert` instead.

      **Light shakeout: 4/4 cells green**, all four stages run, refusal at
      `advance/stale-base`, strict clause (refs **and** object count) holding on
      every cell. Preceded by `rig selftest` exit 0 / 57 assertions (A-1).

      **FALSIFIED — 6 mutants, 6 reds, no survivors.** Driver
      `drivers/falsify-t4e.sh` (throwaway, never calls `rows_write`), sweep
      `drivers/falsify-t4e-sweep.log`, 16 assertions.

      | # | perturbation | red | isolated by |
      |---|---|---|---|
      | M20 | H10's peer misses the contested path | `planted?` conflict clause | the ref still moved, still to a child of B |
      | M21 | the mover lands a GRANDCHILD of B | `c3_stale_planted` parentage clause | ref moved; mover still touches the path |
      | M22 | the accepted ref never moves | `c3_stale_planted` moved clause | the capsule's own result still landed |
      | M23 | H16's mover takes the stub (H16 becomes H10) | `planted?` disjointness clause | ref moved, parent B, still exactly one path |
      | M24 | the RE-SNAPSHOT is dropped | `assert_outcome` — refs **and** objects | every clause of the ROW still held |
      | M25 | **stage 4's ordering INVERTED** — transfer before precondition | `assert_outcome`'s OBJECT-COUNT clause | **the REFS clause still HELD**; exactly one red |

      **M25 is what this round exists for, and its isolation control is the
      finding.** Under an inverted stage 4 the token is still `stale-base`, the
      refs clause still holds, and the row's own seven clauses all hold —
      **only the object count moves.** So T4e's prediction that "getting stage
      4's ordering backwards reds exactly here" is now MEASURED rather than
      argued, and the corollary is sharper than the prediction: had § 5.5 given
      `stale-base` the refs-only clause that `cas-lost` legitimately carries,
      this defect would have scored GREEN, and H10/H16 are the only place in the
      matrix where that shows. F-14's carve-out being a scope correction rather
      than a weakening is no longer an argument about it.

      **`falsify-lib.sh` gained a fourth shape** rather than a fourth entry
      point: `expect_assert` now takes an optional `also` function called inside
      the capture, because M24/M25 red in `assert_outcome`, which none of the
      three existing shapes reach. Default empty — the T4b/T4c rounds already
      scored are byte-for-byte untouched.

      **DRY, taken while here:** "is this path in the range" had drifted into
      two spellings (`c3_planted_paths` piped, `c3_assert_ingested` herestring).
      Both now ride one `c3_lines_have`, and H16's negative asserts over the
      same primitive through `c3_assert_range`, which takes the negative and its
      positive controls together over ONE computed range — H13's M6 lesson
      applied at the range instead of at the bundle.

### T5 — the conflict sub-probe (EX-6, EX-15, VA-4) ✅ DONE

- [x] Against **F3**: `dispatch setup` → `prepare-review` → `candidate create` →
      `admit` → integrate, **staging and all**, on the real candidate layer.
      Body in `lib/conflict.sh`; `c3_run_conflict` already called it.
- [x] Scored as a **separate entry**, explicitly marked as an incumbent-layer
      regression check that **counts toward nothing** (F-9). It is not
      capsule-model evidence and must never enter a coverage number.
- [x] If it is blocked by a candidate-verb precondition: that is **S4** — a
      `/consult` and a finding, not a patch. **Not blocked.** Two preconditions
      met by parameter, neither a patch: `DOCTRINE_TRUNK_REF` (F1's trunk is
      `mainline`, and no `[dispatch] trunk` is configured) and a coord dir
      **under the project root** (`setup` refuses one outside it under the
      claude arm, and names the convention in its refusal).

**THE TWO ROWS PARTITION THE SEQUENCE.** Neither drives all of it, because each
refuses at its own point — and between them the whole sequence runs:

| row | refuses at | how the refusal is signalled | recovery it names |
|---|---|---|---|
| H10 | `candidate create` | **exit ZERO**, `status = "conflicted"`, `merge_oid = ""` | hand-resolve + `candidate ingest` |
| H16 | `sync --integrate --trunk` | **non-zero**, FF/CAS | supersede the close-target on the new base |

H16's `create` and `admit` **both accept** a moved trunk — admission never reads
trunk — so the fast-forward CAS at integrate is the sole place staleness is
caught on this layer. That is the incumbent's analogue of the ordering
`pipeline.sh:560-566` holds for stage 4, and it is why the H16 leg drives
admission rather than stopping at `create`.

**Recorded** (`results.tsv`, block `legs=conflict`, 2 entries, both `pass`):
`altitude` reads `counts-toward-nothing` — the F-9 claim enforced in the column
a reader would sum, not left to prose. `expected-stage`/`expected-token` carry
values **deliberately outside** `MATRIX_STAGES` / `token_legal`
(`candidate-create`/`conflicted`, `integrate`/`stale-trunk`), so a tally that
tried to fold these into the pipeline's vocabulary fails loudly instead of
silently counting an incumbent refusal as model coverage. H10/H16 now own both
legs; their earlier block stays stamped `legs=pipeline`.

**Falsified** — `drivers/falsify-t5.sh`, four mutants, all red-where-predicted
and held-where-predicted:

| mutant | perturbs | reds | isolation control |
|---|---|---|---|
| M32 | peer not a child of B | the parentage clause | the pair still conflicts, still classified |
| M33 | the two halves agree | the classification (ledger says `created`) | still one base; F3 still cleared the gate |
| M34 | trunk never moves | the staleness control + the refusal | **both admissions still hold** |
| M35 | the move happens *before* the pinning | the refusal only | the advance still happened — planted cannot see order |

M34/M35 are a deliberate pair: M34 deletes the hazard, M35 keeps it and moves it
in time. A leg asserting only "it moved and integrate refused" scores M35 green.

**F3 was copied, never cloned** — `prepare-review`'s phase-completion gate reads
`.doctrine/state/slice/001/phases/phase-01.toml`, which is gitignored runtime
state, so `phases: 1/1` is a property of the fixture DIRECTORY. The leg asserts
the gate CLEARED (EX-15 made observable) rather than merely not dying.

### T6 — the five guard probes (EX-10, EX-11, VA-2, VA-3) ✅

**DONE 2026-08-03.** `control/probe-guards.sh`, `rig guards [a…e]` (D-P05-20).
Order per F-P05-31 honoured: shakeout → falsify (4/4, `drivers/falsify-t6.sh`,
m36–m39) → score. **Nine legs, all `pass`, 71 assertions, 0 red** —
`~/capsules/probes/guards/results.tsv`, run `2026-08-03T00:16:02Z`.

| leg | observed |
|---|---|
| a/cite:H8/gitlink · a/cite:H8/gitmodules | 4 scored entries each, cited (D-P05-21) |
| b/light · b/heavy | `conform/forbidden-path` |
| c/light | `conform/forbidden-path` |
| d/light | `verify/suite-failed` |
| e/light baseline · e/light-inrepo worktree | no refusal, **byte-identical** |
| e/light-inrepo committed | `conform/undeclared-path` (F-P05-43) |

Gates green after: `rig selftest`, `audit-matrix`, `audit-dq4`,
`audit-nohooks`, `doctrine validate`, `shellcheck -x -S style`.

Below is the task as written. Three things it did NOT anticipate are
F-P05-41/42/43; two citations in it were already stale when written and are
corrected in D-P05-20's note.


Each must be **OBSERVED REFUSING at least once** — a guard never seen to fire is
not known to work. (a) falls out of T4d; **(b) and (c) are their own probes as of
D-P05-10** — a combined range cannot observe either, because leg 3 returns on the
first matching path and the other forms refuse on its behalf (F-P05-22). (d) and
(e) were always their own probes.

- [x] (a) gitlink + `.gitmodules` (F-2) — from H8. **CITED, not re-run** (D-P05-21).
- [x] (b) non-ASCII `.doctrine/` path (F-4) — **isolated probe**: the non-ASCII
      path is the SOLE `.doctrine/` path in the range. Claim it proves: leg 3
      fires on a non-ASCII path. Claim it does NOT prove: that
      `core.quotePath=false` is load-bearing — `-z` already defeats that evasion
      (F-P05-23). Both fixtures (the path sits under a design-target prefix on
      each: light `.doctrine/**`, heavy `.doctrine/rfc/025/evidence/**`).
- [x] (c) rename out of `.doctrine/` (F-4) — **isolated probe**, and the
      falsifier is real here: with `--find-renames` the `.doctrine/` source leg
      vanishes from the range and leg 3 passes. Assert both directions.
      **LIGHT ONLY** — heavy's sole design-target `.doctrine/` prefix is empty at
      B, so it has no source to rename out (F-P05-21). Not an `n/a` against the
      model: a guard is observed once, not per fixture (D-P05-10).
- [x] (d) a capsule that **overwrites its own `verify.sh`** — the verdict must be
      **UNAFFECTED** (F-11/I4a). The runner is ro-bound at `/rig/verify.sh`
      (`pipeline.sh:438`); the probe writes `capsule/verify.sh` and asserts the
      verdict is the ro-bound runner's. `audit-i4a.sh` is the **static**
      complement (no runner copied in) — this is the behavioural half, and
      neither substitutes for the other.
- [x] (e) a capsule that **rewrites its own interpretation-surface declaration**,
      against **F2** — trusted-side behaviour must be **BYTE-IDENTICAL** to the
      F1 run (VA-3, EX-11). Evidence is the comparison, not the claim. This is
      **QUE-201's only evidence input**; without it QUE-201 settles post-spike
      on argument alone. `pipeline_setup:144-149` carries the note that this
      read becomes `git show <B>:<path>` and must **never** become `<S>:<path>`.

### T7 — EVD records (EX-13, VA-5) ✅ DONE (2026-08-03, `0b9581b1`)

- [x] `doctrine knowledge new evidence <title>` per verdict input, linked
      `supports`/`disputes` → QUE-200: per-mechanism H6/H7 behaviour,
      trust-bearing lines at the boundary, forensic completeness, operational
      friction, and **M-B's trusted-side file-ingestion boundary that M-A does
      not carry** (already banked as a first input, `notes.md:1229-1231` —
      §Harvest→Open; the `:699-702` citation this line carried was stale).
- [x] A further EVD linked to **QUE-201** from guard probe (e).
- [x] `doctrine knowledge show QUE-200 / QUE-201` renders the links in **BOTH
      TIERS** (VA-5). Amending a record means both tiers; verify via `show`.

| record | edge | input |
|---|---|---|
| EVD-006 | `supports` QUE-200 | per-mechanism H6/H7 behaviour — mechanism-neutral, 8 cells |
| EVD-007 | `supports` QUE-200 | trust-bearing lines — all trusted-side, downstream of the mechanism |
| EVD-008 | `supports` QUE-200 | forensic completeness — both preserve worker commit identity |
| EVD-009 | `supports` QUE-200 | operational friction — cost equal; M-B trips git auto-maintenance |
| EVD-010 | **`disputes`** QUE-200 | M-B's trusted-side file-ingestion boundary (the banked input) |
| EVD-011 | `supports` QUE-201 | guard (e) — byte-identical trusted-side behaviour |

`disputes` is used once and deliberately: EVD-010 is counter-evidence to a claim
QUE-200's own body makes for candidate 2 ("cleanest trust story"), not to the
question. The other five are inputs the question consumes, so `supports`.

### T8 — evidence artefacts (EX-14) ✅ DONE (2026-08-03, `d730ee50`)

- [x] Committed **text summaries** under `.doctrine/rfc/025/evidence/` —
      `README.md` (verdict + six things it does NOT establish), `matrix.md`
      (the 16 rows), `guards.md` (five guards + the sub-probe).
- [x] **Raw logs** under `.doctrine/state/rfc-025/raw/` — 80 files, 488K,
      runtime tier, gitignored by `.doctrine/state/`, **no new `.gitignore`
      entry**. Verified with `git check-ignore -v`.

**Beyond the authored brief, on operator ruling (2026-08-03):**

- [x] **`results-c3.tsv` + `results-guards.tsv` committed verbatim** — the
      "generated measurement table" of § 5.3. These were the **only** copies and
      lived outside the repo under `~/capsules`; this is the durability step.
- [x] **`phase-sheets/` — the runtime sheets copied straight**, with a README
      framing them as frozen exhibits and naming what is authoritative instead.
      Operator's rationale: a citational summary is only useful while its
      citations resolve, and distilling costs context on reasoning that may
      never be read — whereas losing it is irreversible (F-P05-39).
- [x] **`drivers/` committed** — T5's/T6's falsification + diagnostic drivers.
      My call, not the operator's brief; F-P05-39's precedent is exactly this
      loss, and the copy is free.

**⚠ T9 MUST RE-COPY `phase-05.md`.** The archive was taken *before* these boxes
were ticked and before T9's own entries exist, so `phase-sheets/phase-05.md` is
already one edit stale. Re-copy as the **last** authored act of the phase:

```bash
cp .doctrine/state/slice/241/phases/phase-0*.md \
   .doctrine/state/slice/241/phases/phase-0*.toml \
   .doctrine/rfc/025/evidence/phase-sheets/
```

### T9 — close ✅ DONE (2026-08-03)

- [x] **Re-copy the phase sheets** (above) — last authored act, after every other
      T9 edit to this sheet had landed.
- [x] `LC_ALL=C.UTF-8 shellcheck -x -S style` on every touched rig file — **exit
      0, clean** across `rig` + every `*.sh` outside `probes/`. No rig file was
      touched after T6, so this confirms rather than re-earns.
- [x] `doctrine validate` — corpus clean. **No `doctrine check`**: S4 held, no
      Rust appeared in PHASE-05 at all.
- [x] `./rig selftest` — **all assertions hold**, exit 0. Not owed (no rig change
      since T6) but run anyway: this phase's whole discipline is that an
      observation beats an assertion, and "still green" was an assertion.
- [x] `doctrine slice verify-vt 241` — PHASE-05 VT-1 read **UNATTRIBUTABLE**
      before the flip, exactly as F-P04-11 predicted. Not chased.
- [x] Lift durable decisions / findings / evidence into `notes.md` §§ PHASE-05
      (`09e6a69f`). Written as an **INDEX, not a transcription** — the sheet is
      no longer discarded (T8 committed it verbatim), so restating 23 decisions
      and 46 findings would duplicate a file in the same tree. Expanded only
      where the lesson binds work outside SL-241. The sections say so in their
      own headers, so a future reader is not left wondering whether the lift was
      lazy.
- [x] `doctrine slice record-delta` / `phase --status completed`.

**PHASE-05 CLOSED.** Six of six tasks' criteria met; EX-1..EX-15 and VA-1..VA-5
discharged across T0–T8. Next: PHASE-06 (`go-no-go.md` — its VT-1 currently
FAILs on the absent file, which is correct for an unstarted phase).

## Risks

- **R-A — this phase is the largest in the slice by a wide margin**: ~60 cells,
  48 `Hnn.*` functions, two fixture variants, a sub-probe against real dispatch
  machinery, and the EVD set. Mitigation: T4a–T4e are independently re-runnable
  (`rig c3 <rows…>`) and each is its own commit; nothing accumulates in the
  working tree. If the sub-probe (T5) proves deeper than a phase, it is an S4
  `/consult` — its leg counts toward nothing, so it cannot block the model-level
  claim.
- **R-B — heavy-fixture cost.** `pipeline_setup` clones three repos per cell and
  the heavy fixture is ~100M of packs. Measure one heavy cell at the start of
  T4a before committing to 32 of them. **Do not resolve this by dropping the
  heavy column** — A-3 says that caps every row at `client-local` and guts the
  table. If cost is prohibitive, it is a `/consult` about scope, not a silent
  narrowing.
- **R-C — `n/a` inflation.** Every `n/a` is legal but costs altitude. The
  temptation under time pressure is to record `n/a` where a real instantiation
  was merely hard. Each `n/a` needs a reason that names the *structural* absence
  (H13/M-A: no artifact exists), never "not attempted".
- **R-D — a dissolution scored as a kill, or vice versa.** H6/H9/H12/H15 have no
  refusing stage by construction. A harness that requires an `expected-token` on
  every row would force them into a fake kill; one that skips them silently loses
  the design's best result. The `expected-stage` column must accept a
  `dissolution` value distinct from empty.
- **R-E — the shared tree.** Two destructive incidents on record (F-P04-6):
  a `git reset --hard` and a `git stash`/`pop` that wrote conflict markers into a
  knowledge record. Commit each coherent unit the moment it is coherent; never a
  pathless `git commit`; check `git reflog` before concluding a file was
  imagined.

## Decisions

<!-- D-P05-n, appended as they are taken. Lift to notes.md at close. -->

- **D-P05-1 (provisional, confirm at T2).** `matrix.tsv` is the **authored
  input** (committed, VT-1's subject) and `probes/c3/results.tsv` under
  `$SPIKE_CAPSULE_ROOT` is the **generated output**. The authored file carries
  the full header — which is what VT-1's `planted`/`outcome`/`altitude`/`n/a`
  keywords grep — with those cells empty except for the up-front `n/a` reasons.
  Alternative rejected: writing outcomes back into the authored file, which
  would put derived data in the authored tier (the storage rule) and make
  "altitude is generated, not hand-authored" unverifiable.

  **CONFIRMED at T2, unchanged.**

- **D-P05-2 — sixteen row ids; a `|` alternation carries a multi-part boundary.**
  § 5.6 has sixteen rows and VA-1 counts sixteen, but four rows' re-derived
  boundaries name more than one thing: H8 (two tree-mode tokens), H9
  (dissolution at ingest **and** containment at `verify`, "scored as two"), H13
  (four bundle-hygiene legs). Two encodings were available — split those into
  sub-rows, or let `expected-stage`/`expected-token` hold an alternation.

  Alternation chosen. Sub-rows would have put the file at 17–19 ids against
  VA-1's sixteen, and the arithmetic settles it: sixteen ids × 2 fixtures × 2
  mechanisms is **exactly** the sheet's own `16 × 2 × 2`, and the four up-front
  `n/a` cells leave **exactly** the "~60 live cells" it predicts. `harness=both`
  is the precedent inside the design itself — one authored line already produces
  two scored result entries — so a multi-part row does the same at results time.

  What this protects: `dissolution` stays a first-class `expected-stage` value
  distinct from empty (R-D), rather than H9's ingest half being collapsed into
  its verify half or forced into a fake kill. The validator refuses a cell that
  claims a refusal token alongside a multi-stage or `dissolution` stage, because
  such a cell cannot say which stage the token belongs to.

- **D-P05-3 — the spec's validator lives beside its reader, and runs before
  provisioning.** `lib/matrix.sh` holds `matrix_rows` (the single definition of
  what a data line is) and `matrix_validate`; `control/audit-matrix.sh` is the
  entry, and T3's harness sources the lib rather than reimplementing either.
  A validator that re-derived "what a row is" could disagree with the harness
  about where a row starts — the one disagreement neither would report. Same
  argument `lib/common.sh` makes for `declaration_field` having two consumers.

  Running it **before** anything is provisioned is the point: an authored
  `expected-token` outside the closed vocabulary would otherwise surface as
  `stage_emit`'s RIG DEFECT at run time, after the cell had already run, and be
  attributed to the rig instead of to the file that has it.

- **D-P05-4 — an unwritten row is a REFUSAL, never a recorded `n/a`.** The
  harness could have scored a row with no instantiation as `n/a`, which would
  have kept every invocation green and produced a complete-looking results file
  throughout T4. It refuses instead: `rig c3 H7` exits 2 naming the missing
  `H7_mutate`/`H7_planted`/`H7_assert` and the task that provides them.

  R-C is the reason and it is not a style preference. `n/a` is excluded from the
  altitude computation, so an unwritten row recorded as `n/a` **costs the row its
  altitude** — H7 would read `unproven-beyond-rust` or worse, in a table whose
  whole purpose is the portability claim, for a reason that is about the rig's
  build order rather than about the capsule model. That is R4's direction of
  damage: the finding least likely to be re-examined, because it looks like the
  answer the spike was built to find.

  The same argument governs the conflict leg, with one difference: T4e's rows
  land before T5's sub-probe exists, so narrowing has to be *sayable*.
  `SPIKE_C3_LEGS=pipeline` is that, and the selector is written into the results
  preamble — a run that covered only the pipeline legs says so in the file
  rather than looking complete. An env var rather than a flag because `rig`'s
  parser owns the flag space (`rig:127`) and a probe-specific flag in a global
  parser is the wrong seam.

- **D-P05-5 — the rows live beside the harness, not inside it, and they
  republish BOTH the ref and the bundle.** `lib/instantiations.sh` holds the
  `H<n>_{mutate,planted,assert}` trios; `control/probe-c3.sh` sources it last
  (the instantiations ride `contract_field`, `RIG_RESULT_REF` and `rig_assert`
  from the three libs above it). Two responsibilities with different lifetimes:
  the loop, the scorer and the recorder were finished at T3, the rows arrive one
  group at a time through T4e. Folding them into the 663-line harness would have
  put a file that changes every commit on top of one that should not change at
  all.

  The shared vehicle mutates the CAPSULE'S CLONE and nothing else — not
  `canonical` (the subject of `assert_outcome`), not `quarantine` (the trusted
  side's workspace). A row reaching into either would be testing the rig's
  bookkeeping. The rig plays the adversary trusted-side purely for convenience:
  everything planted is inside the capsule's writable root, so a hostile worker
  could author all of it.

  **`c3_publish` rewrites the bundle unconditionally, and that is the decision.**
  The worker wrote `result.bundle` inside `pipeline_capsule`, BEFORE any mutate
  runs. A row that moved only `RIG_RESULT_REF` would leave every M-B cell
  harvesting the UNMUTATED result — half the matrix scoring a payload that never
  shipped. `planted?` cannot catch it, because it reads the capsule's ref and
  the ref *did* move; only the boundary comparison and `_assert`'s trusted-side
  range can. Rejected: rewriting only under `mechanism=bundle`, which is a
  conditional a reader would have to re-verify per row, for ~0.5s a heavy cell.

- **D-P05-6 — every SANDBOX-INJECTED status is named; `*)` means the command's
  own exit.** Operator-ruled in `/consult`, 2026-08-02, `c4a26004`. The ruling's
  framing — *stop failures being silent as to reason* — is what shaped the fix,
  and it is sharper than the "mint a token" proposal it replaced: `*)` →
  `suite-failed` is HONEST for a verify command's own nonzero exit, so the defect
  was never the catch-all itself but the injected statuses colliding with it.
  `verify/resource-cap` joins the closed set (design § 5.1) as the same bound
  `harvest/resource-cap` already carries on the worker kind. Rejected: widening
  `*)` to raise a defect on any reserved status — a suite legitimately exiting
  `2` would then be scored a rig defect, trading a silent mis-attribution for a
  loud one. The channel ambiguity is recorded in F-P05-15 instead.

- **D-P05-7 — the heavy capsule BUILDS ITS ASSETS ON SITE rather than having
  them provisioned in.** Operator-ruled. `verify:` becomes a build-then-test
  command so the capsule regenerates `web/map/dist/` from its own tracked
  `web/map/` source. Rejected: carrying the built artefact into the fixture,
  which works but pins a `dist/` built at *edge* against a base commit that is
  not edge — an approximation with a staleness question attached. Building on
  site has no such question: B is self-consistent by construction. It also
  lands where DQ-4 says it must — inside the verify capsule, via the
  declaration, never trusted-side in `control/**`. Costs ~5 min a heavy stage-3
  cell, which is why only the rows that need a verify PASS pay it.

- **D-P05-8 — H2 is re-derived in § 5.6 as DISSOLVED.** Operator-ruled. Its
  hazard is not undefended, it is unreachable by construction: the harvester
  pins the OID itself, so a rewrite past the doorbell gains the capsule nothing
  (F-P05-13). Treated exactly as H13's M-A leg already is — a dissolution is a
  RESULT and a QUE-200 input, not a gap. Rejected: a background racer, which is
  nondeterministic and therefore not a probe.

- **D-P05-9 — F1 gains a `.doctrine/**` design-target selector so H5 can reach
  conform leg 3 on light.** Operator-ruled. SL-001 declares `src/**` alone, so
  leg 2 refuses `undeclared-path` first and H5/light would merely restate
  H4/light (F-P05-14). Rejected: re-deriving H5's light boundary to
  `undeclared-path`, which makes the row a duplicate and costs it its reason to
  exist; and `n/a`, which R-C reserves for STRUCTURAL absence — a selector list
  is a rig choice, not an absence.

- **D-P05-10 — GUARDS (b) AND (c) BECOME THEIR OWN T6 PROBES; H5 PLANTS WHAT
  EACH FIXTURE CAN EXPRESS.** Operator-ruled, 2026-08-02. Settles F-P05-21 and
  F-P05-22 together, because they have one resolution.

  § 9 requires each guard to be *observed refusing at least once*, and F-P05-22
  measured that a combined range cannot deliver that: leg 3 returns on the first
  matching path, so with all three forms planted, a leg 3 that had dropped
  `--no-renames` still refuses — on another form's path — and the cell scores
  `pass`. **Isolation is what makes a form a control.** Guards (b) and (c)
  therefore move out of "falls out of T4a" and into T6 as standalone probes,
  which is where guards (d) and (e) already live; each gets a run in which its
  form is the SOLE `.doctrine/` path.

  The H5 **row** keeps its § 5.6 boundary untouched (S5) and plants what each
  fixture can express: plain edit and non-ASCII on both, rename-out on light
  only. It reaches `conform/forbidden-path` identically on both, so the row
  keeps **model-level**.

  What this buys against the alternatives:

  - Nothing moves in the harness (D-P05-5) or in the matrix's sixteen ids and
    16 × 2 × 2 arithmetic (D-P05-2). Splitting H5 into per-form matrix lines
    would have broken the second.
  - **R-C is satisfied rather than argued around.** A guard is "observed
    refusing at least once", not a per-fixture claim, so guard (c) being
    light-only costs no altitude. Had form (c) stayed inside the row, heavy
    would have needed an `n/a` — against a row whose whole point is the
    portability claim, for a reason about a fixture's selector list.
  - *Rejected: re-pin heavy's B to a commit carrying
    `.doctrine/rfc/025/evidence/` content.* Circular — that content is T8's
    output and T4 precedes it — and it costs a fixture rebuild plus F-P05-16's
    verify-green re-validation (~6 min), discarding the pin F-P05-19 measured
    the entire heavy column against.

  **One thing the results table must not say** (F-P05-23): guard (b) shows leg 3
  firing on a non-ASCII path, which is what § 9 asks. It does NOT show
  `core.quotePath=false` load-bearing — `-z` already defeats that evasion.

- **D-P05-11 — H12 PLANTS ITS EVALUATION SURFACES UNDER THE SLICE'S
  DESIGN-TARGET DIRECTORY, NOT AT THE REPOSITORY ROOT.** Operator-ruled,
  2026-08-02, on F-P05-26's measurement.

  The matrix's literal `.envrc` / `flake.nix` are unreachable on heavy for two
  independent reasons: root `flake.nix` is undeclared under SL-241's selectors,
  so conform leg 2 refuses `undeclared-path` and **any** refusal falsifies a
  dissolution (R-D); and `/.envrc` is gitignored in this repository, so `git
  add` refuses it before conform is ever reached. The pair moves to
  `scripts/spike-capsule/.envrc` and `scripts/spike-capsule/flake.nix`.

  **The hazard class is untouched, which is why this is not a narrowing.**
  `.envrc` is a PER-DIRECTORY direnv surface and nix evaluates a `flake.nix`
  wherever it is invoked, so the class-2/3 trigger is the same trigger; only the
  directory differs. § 5.6's `expected-stage` is untouched (S5) — the amendment
  is to `matrix.tsv`'s `instantiation` prose, which is authored but is not the
  boundary.

  Rejected: **`n/a` on heavy** — R-C reserves `n/a` for a STRUCTURAL absence and
  a selector list is a rig choice, the same ground F-P05-14 refused it on; with
  light already `n/a` the row would then claim no altitude at all, which is a
  fact about the fixture's selectors dressed as a fact about the model.
  **Re-pinning heavy's B** — F-P05-21's reasons, unchanged: a rebuild plus a
  ~6-minute `verify:`-green re-validation, discarding the pin the whole heavy
  column is measured against. **A `harness=audit` value** — honest to § 5.6's
  own words, but a T2/T3 change with an `audit-matrix` invariant behind it, for
  a row that (a) already scores.

- **D-P05-12 — H15 INTERRUPTS THREE STAGES, AND STAGE 4's INDIVISIBILITY IS
  OBSERVED RATHER THAN INTERRUPTED.** Operator-accepted, 2026-08-02. A narrowing
  of EX-9's literal "killed at each stage in turn", taken deliberately and
  recorded as such.

  There is no interruptible interior to stage 4. A kill during `advance` lands
  either before the `update-ref` — in which case canonical is untouched and the
  attempt is indistinguishable from the during-verify one — or after it, in
  which case the run COMPLETED and the cell's own scored run would then refuse
  `advance/stale-base`, failing the dissolution **whenever the race was lost**.
  Racing for the microseconds between them is the nondeterminism D-P05-8 already
  refused on the grounds that a nondeterministic probe is not a probe.

  So the row observes the CONSEQUENCE: after the resume lands, a direct repeat
  `advance_stage` refuses `stale-base` having transferred nothing — the CAS
  applied exactly once. That is § 5.6's atomicity claim stated as an
  observation. Rejected: a fourth kill against an advance already doomed to
  refuse, which costs ~12 more heavy minutes and tests strictly less.

- **D-P05-13 — A TOOL THE RIG CANNOT INVOKE IS A DEFECT OF THE RIG, AND EVERY
  RUNG OF THE LADDER MUST BE PROVEN TO RUN.** Operator-agreed, 2026-08-02, on
  F-P05-29's diagnosis. Two independent fixes and a control, all in the rig
  (no `src/`, so S4 is untouched).

  1. **`rig_doctrine_bin` probes runnability, not the executable bit**
     (`lib/common.sh`). `rig_doctrine_runs` execs `--version`; `[ -x ]` is true
     for a binary whose ELF interpreter is absent, which is how the ladder took
     a dead rung and never reached the working one **bwrap already provides** —
     `flake.nix:131` ro-binds the crane build at `~/.cargo/bin/doctrine` and
     `:136` puts it on PATH. The exposure the operator asked for was already
     there; only the ladder's belief was wrong. The last rung stays UNPROBED on
     purpose: if PATH cannot be invoked either, leg 2's defect says so with more
     context than a resolver could.
  2. **Conform leg 2 separates a refusal from a failure to invoke**
     (`control/pipeline.sh`). 126/127 → `RIG_EXIT_DEFECT`, and the discarded
     stderr is captured and re-emitted (it reads `bad interpreter: No such file
     or directory` — the sentence `2>&1` was throwing away). RETURNED, never
     `exit`ed: `conform_stage` runs inside `$( … )`, where an exit ends only the
     substitution's subshell and the caller reads a bare nonzero with an empty
     token (F-P01-1). `pipeline_run` re-raises it exactly as it already does the
     harvester's.

  **The control is the point, and it is new**: `selftest.sh` gains an
  **`apparatus`** section — capsule-free, runs FIRST, one rung below A-1. It
  asserts the picked binary runs, that a `-x`-but-unrunnable fake is refused by
  the ladder, and that leg 2 raises a defect rather than `undeclared-path` for
  it, with two controls (the fake really is `-x`; the real ladder still returns
  a real token). It went **5 red → 8 green** across the fix. Had it existed this
  morning it would have cost seconds instead of a scored run.

  Rejected: **rebuilding `target/debug/doctrine` in this jail** — it repairs the
  symptom here while clobbering a binary a co-agent may depend on in theirs
  (R-E), and leaves the rig still unable to tell a broken tool from a hostile
  capsule. Rejected: **pinning `DOCTRINE_BIN` for rig runs** — same, an
  environment workaround for a rig that lies about what it measured.

- **D-P05-14 — EGRESS IS ALLOWLISTED, NOT BINARY, AND THE ALLOWLIST IS PER
  CAPSULE KIND.** Operator-ruled 2026-08-02, in the `/consult` on F-P05-32.
  The third option neither the finding nor its recommendation considered: the
  choice was framed as `--share-net` vs `--no-net`, and the answer is a
  trusted-side allowlist naming exactly the hosts a build needs. Already
  half-recorded — QUE-204 carries the operator's earlier *"egress acceptable if
  allowlisted, but that is per-project toil"*; F-P05-32 and QUE-204 share a
  lever and were being settled apart.

  **Content per kind, mechanism shared.** The worker kind's list must name the
  model/harness endpoints; the verify kind's names build hosts only, and an
  absent or empty list denies everything — so verification runs with the agent
  hosts trivially disabled rather than inheriting them. That split is the whole
  decision: a union list would let the verify capsule reach the harness's API
  for no reason any mechanism claims.

  **The coupling is real, accepted, and NOT a new axis.** Naming the LLM's hosts
  bakes an assumption about the model/harness union into the worker capsule's
  configuration. ADR-011 D3 already owns exactly this — *"the contract is
  uniform; the reachable altitude is not"*, admitting per-harness capability
  layering **"without a uniform altitude lie"**. Per-kind allowlists are that
  rule at the network layer; a shared list would be the lie. Record the coupling
  when this is written up; do not treat it as novel.

  **EX-2 survives, restated.** The uniform-posture claim becomes *same mount
  posture and same egress MECHANISM, allowlist content per kind* — still one
  profile, still checkable against an array rather than asserted. Rejected:
  `--no-net` on the verify kind (retires EX-2's uniformity as a side effect of a
  one-line change, and breaks heavy's verify outright while QUE-204 keeps build
  inputs fetched at verify time). Rejected: vendoring the build inputs so
  `--no-net` becomes reachable — no proxy at all, but for heavy it means
  committing `cargo vendor` + `web/map/node_modules` into the clone so
  git-object transfer carries them, against a 256 MiB worker cap and a measured
  4.4 G verify disk. The proxy does not inflate the fixture.

  **Topology forces an EXPLICIT proxy, and that is a property worth having.**
  The capsule gets no route at all; its only way out is a unix socket bound in
  from the trusted side (a unix socket crosses a netns via the filesystem, not
  the network stack), with `socat` bridging it to a loopback endpoint inside and
  `tinyproxy` as the sole chokepoint outside. Transparent interception is not
  merely unavailable here (no `nft`/`iptables`, no route to redirect) but
  unwanted: it would need a private CA in the capsule's trust store, and it
  would let a tool that ignores its proxy configuration reach the network
  anyway. Under this topology such a tool gets nothing and fails loudly —
  **fail-closed by construction**, which is the same property `guard_not_real_repo`
  and the ro-bound runner are built on.

  **Consequence for H11's control set, and it is the row's fifth vacuity trap.**
  A payload that does not speak proxy protocol reaches nothing whether or not an
  allowlist exists, so a lone refusal would demonstrate `--no-net` and be scored
  as an allowlist. Three observations, not one:

  | # | observation | what its absence would hide |
  |---|---|---|
  | 1 | an ALLOWLISTED host, via the proxy → **reachable** | that the mechanism is an allowlist rather than a blanket block |
  | 2 | the canary's host, via the proxy → **refused by the proxy** | that the list is what bites |
  | 3 | a direct connect, bypassing the proxy → **no route** | that the chokepoint is the only path out |

  Note 3 is also why the payload's own view is never the observable (the probe
  log's `write-status=0` lesson): a hostile payload has no obligation to honour
  `http_proxy`, and its failure to connect is not evidence about the allowlist.

  **`tinyproxy` over a hand-rolled proxy, and the reason is evidentiary.**
  `FilterDefaultDeny Yes` plus a filter file is hostname allowlisting for
  CONNECT with no TLS interception; `ConnectPort 443` bounds the ports. A
  `python3` proxy would work — and would be control-plane code H11's evidence
  rests on, owing its own falsifiability round under the same rule that made
  every mutant WRAP the real function rather than restate it (F-P05-25,
  F-P05-29). DQ-4 is clean either way: neither `tinyproxy` nor `socat` nor
  `python3` appears in either declaration's `exec:` list (light `node npm tsc`,
  heavy `cargo nix direnv just rustc`) — unlike the first canary, which reached
  for `node` and was refused on sight.

  **Still open, and not settled by this decision**: where the allowlist work
  lands (a PHASE-05 task / a new phase / a follow-on slice with QUE-204), and
  both H11 dispositions — F-P05-32's canary clause and F-P05-33's heavy cell.

- **D-P05-15 — H11 SCORES AT A DIFFERENT BOUNDARY PER FIXTURE, AND HEAVY'S IS
  `conform`.** Operator-ruled 2026-08-02 (delegated in `/consult`), disposing
  F-P05-33. The finding's own three options were `n/a` with the measured reason,
  an inert heavy plant scored red, or amending SL-241's selectors. All three
  read the situation as *heavy cannot do this row*. The re-derivation says
  otherwise: heavy's `verify:` command is the one that genuinely evaluates build
  systems — `cargo` runs a root `build.rs` — so the class-2 trigger is LIVE
  there and undeclared, and conform stops it at leg 2. That is a result, not an
  absence, and § 5.6 is exactly the place inherited expectations get re-derived
  against what the model does.

  So the heavy cells plant the payload at the path cargo really runs, and score
  `conform/undeclared-path`. **Cheap by construction** — a conform refusal never
  builds the verify capsule, so the two heavy cells cost what T4a's do and not
  what a stage-3 cell does. The earlier costing of "~2 heavy verify runs"
  belonged to the inert-plant option, not to this one.

  `matrix.tsv` needed **no new machinery**: its lines are already per fixture,
  so the two heavy lines carry their own `expected-stage`/`expected-token` while
  light's keep `verify`. No alternation, no sub-row, no schema change — and
  D-P05-2's sixteen-id arithmetic is untouched.

  Rejected, and for a reason worth keeping: **a fixture-local slice for heavy**
  declaring `build.rs`, which is the only option that would get a class-2
  payload all the way into the sandbox on heavy, and is not the objection it
  first appears — light's SL-001 is FABRICATED by `fixture-light.sh`, so a
  rig-authored fixture slice is what light already has and the "authored
  governance bent to make a cell live" charge does not reach it. It is rejected
  on cost and timing, not principle: a fixture rebuild, two heavy stage-3 runs
  at the 4.4G/352s measurement, and a re-check of every heavy row that depends
  on `build.rs` being undeclared. It remains the move if the containment claim
  is later wanted on heavy — F-P05-35 is what says whether it is.

- **D-P05-16 — H7's DISK BOUND IS 20G.** Operator-ruled 2026-08-03, disposing
  the standing H7 call that seven packets have carried. H7 is the phase's only
  resource-exhaustion risk: it needs capsule-time hostility (`worker-hostile.sh`,
  § 9.1, still unwritten) authoring an oversized blob, and the heavy worker
  capsule already sits at 195M against a 256 MiB default (F-P05-10), so the row
  cannot run at the default without the overrun being ambiguous about what
  caused it. **20G is the number.** Headroom is not the constraint — 300G free
  on `/home/david` at the ruling, capsule root 188M.

  **WHICH cap the 20G binds is NOT settled here and must not be inferred.** Two
  readings are live and they are different work: (i) H7's own **worker-capsule**
  `--disk-cap`, raised from 256 MiB so the hostile worker's blob is the only
  thing that can trip it; or (ii) a **host-level ceiling** on what the row may
  consume, with the capsule cap set separately below it. Named as open rather
  than guessed into the sheet as settled detail. **Resolve it in the H7
  `/consult`, which the packet already requires before the row starts** — the
  ruling unblocks H7, it does not start it.

- **D-P05-17 — THE ALLOWLIST WORK LANDS IN A FOLLOW-ON SLICE, NOT HERE.**
  Operator-ruled 2026-08-03, closing the placement half D-P05-14 left open.
  D-P05-14 settled *what* egress becomes — allowlisted rather than binary,
  content per capsule kind, agent hosts absent when no agent runs — and
  explicitly did not place the work. It is placed now: **a follow-on slice**,
  which is the recommendation that was on the table, settled together with
  **QUE-204** (build inputs git cannot carry) since the two are one lever.

  **PHASE-05 builds none of it.** `tinyproxy` and `iproute2` were installed
  during the feasibility work and `socat` + `python3` were already present; all
  four are DQ-4-clean. That is a *feasibility* result and the extent of it —
  nothing is built, and nothing in this phase's remaining tasks depends on it.
  The finding trail (F-P05-32) and D-P05-14's reasoning are what the follow-on
  slice inherits; do not re-derive them there.

- **D-P05-18 — THE 20G BOUNDS THE BLAST RADIUS, NOT THE CAP H7 CROSSES.**
  Operator-ruled 2026-08-03, closing the half D-P05-16 left open. Neither
  reading D-P05-16 named was taken: the ruling **decouples the two numbers**.

  - **H7's worker capsule cap is raised MODESTLY and ROW-LOCALLY** — an override
    for H7 alone, not a `fixture_worker_disk_cap` seam mirroring
    `fixture_verify_disk_cap`. The named seam was offered and declined: it costs
    a refactor to express a value only one row wants.
  - **The 20G is a blast-radius bound**, i.e. what the row may consume in total.
    Nothing enforces it — it is an observed bound under the existing teardown
    discipline, not a mechanism. **A host-level ceiling is NOT built here**
    (reading (ii) rejected on the D-P05-17 precedent: new machinery, one row
    left).
  - **Standing constraint from the ruling: do not fill the disk and do not
    spend the day pumping `dd`.** This binds the row's design, not just its
    setting — see the leg asymmetry below.

  **Why the cap could not simply be set to 20G** (reading (i), rejected on
  cost). Under (i) the cap is not a threshold the row *observes* but one it must
  *cross*: H7's expected boundary is `harvest/resource-cap`, so the hostile blob
  must exceed the cap on every cell and on every mutant of the falsification
  round. The two enforcement legs price that differently, and the difference is
  the whole argument:

  | leg | mechanism | cost at cap C |
  |---|---|---|
  | per-file | `ulimit -f` → SIGXFSZ → 153 → `RIG_EXIT_DISK` (`sandbox.sh:271-277`) | **~free** — a sparse oversize trips it without writing the bytes |
  | whole-tree | `du -s` after the run (`sandbox.sh:283-288`) | **C real bytes**, and it cannot be made sparse — that is its reason for existing |

  So a 20G cap costs ~20G of real writes per execution on the whole-tree leg
  alone. Feasible against 300G free; not free, and not what the bound was for.

  **Attribution is earned by a CONTROL, not by a MARGIN.** The ambiguity
  D-P05-16 worried about — heavy worker at 195M against a 256 MiB default, so an
  overrun cannot be attributed to the blob — is dissolved by a **pre-mutation
  footprint control** proving the honest run sat under the cap. That is this
  phase's own established pattern, twice: H13's **M6** (every leg pairs its
  observable with the honest artifact that stood there first) and H14's **M12**
  (an isolation control must assert the state the mutation actually leaves). A
  100× margin buys the same attribution by brute force and pays for it in the
  one leg that cannot be made cheap.

  **The `pipeline.sh:48` invariant is preserved and is why this stayed hard.**
  `PIPELINE_VERIFY_DISK_CAP` is deliberately NOT `SPIKE_SANDBOX_DISK_CAP`
  because the latter also bounds the WORKER capsule, where it is doing real
  work — *"H7 plants against exactly that headroom. Widening one must not widen
  the other."* A row-local worker override widens neither name globally.

- **D-P05-19 — THE CAPSULE-TIME SEAM IS A DECLARATIVE PER-CELL LOOKUP, NOT A
  HOOK.** Operator-ruled 2026-08-03, disposing F-P05-37 (H7's hazard cannot
  reach the trusted-side plant seam). `pipeline_capsule` takes the worker
  **vehicle** and worker **disk cap** as parameters defaulted to today's values;
  `cell_run` sets them from a lookup in `lib/instantiations.sh`, **beside the
  two lines that already do exactly this for the verify bounds**
  (`probe-c3.sh:562-563`).

  **This does not breach D-P05-5, on D-P05-5's own stated reason.** That
  decision kept rows out of the harness because *"folding them into the 663-line
  harness would have put a file that changes every commit on top of one that
  should not change at all."* The harness here learns a **parameter**, not a
  row: every row-specific value stays in `instantiations.sh`. The harness
  delta is ~2 lines and adds no row knowledge.

  **Leak safety is inherited, not newly argued.** The D-P05-7 scoping comment
  already at that site — *"set for this cell only … so that a cell reading them
  cannot inherit the previous cell's fixture"* — covers the worker bounds for
  free. That is the decisive advantage over the alternative.

  Rejected: **(b) optional `H<n>_arm` / `H<n>_disarm` hooks** — more harness
  surface, and side-effecting, so it needs a teardown that can silently not run
  and leak into the next row. Rejected: **(c) keying the vehicle off the run
  name** (`c3-H7-…`) — D-P05-5 already rejected that shape by name, *"a
  conditional a reader would have to re-verify per row."*

  **Naming tripwire, steered around by construction.** `audit-nohooks.sh:63,174`
  forbids the token `worker_mode` anywhere in the rig (B2 role detection, with a
  planted positive control at `:248`). New identifiers take a
  `PIPELINE_WORKER_*` name. `SPIKE_WORKER_MODE` (uppercase, `rig:115-122`) is a
  different thing and is NOT the seam — `pipeline.sh` never consumed it.

- **D-P05-20 — THE GUARD PROBES GET THEIR OWN EXECUTABLE AND THEIR OWN RESULTS
  FILE.** `control/probe-guards.sh`, dispatched by `rig guards [a…e]` through the
  existing `rig_dispatch` pattern. Taken 2026-08-03; the handover framed it as an
  open call and named the default to avoid.

  **Not `probe-c3.sh`.** The guards have no `Hnn_{mutate,planted,assert}` trio,
  no fixture × mechanism cross-product and no altitude — every structure that
  harness has. Five probes accreting into it was the path of least resistance and
  is what this decision refuses.

  **Not `lib/instantiations.sh` either**, for the same reason inverted: that file
  is *the rows'* instantiations, and a non-row probe living in it would break the
  R-C contract's meaning (`c3_assert_ready` refuses a row whose trio is absent).

  **But it SOURCES `lib/instantiations.sh`, deliberately.** Guard (b) plants
  `C3_H5_NONASCII` and guard (c) renames `C3_H5_RENAME_SRC` — H5's own literals.
  A guard probe spelling its own path would prove something about a path the
  matrix never touched (STD-001), and the fixture-surface joins are exactly what
  F-P05-14/18/21 kept costing.

  **Its own results file**, `probes/guards/results.tsv`. `lib/conflict.sh` solved
  the same "must not be summed" problem with a `counts-toward-nothing` altitude
  because its entries genuinely sit on the matrix's axes; these do not, and a
  discriminator column a reader has to *notice* is weaker than a separate file.

  **`lib/fixtures.sh`** lifts the fixture table out of `probe-c3.sh` so both
  probes share one set of joins, and gains the `light-inrepo` arm. The move
  `lib/rows.sh` made at T3, on the same bar: a second real caller, not a
  generalisation invented for one. Behaviour-preserving —
  `probe-c3.sh --positive-control` and `audit-matrix.sh` green across it.

  **Two sheet citations were stale when T6 was written**, corrected in the
  handover and confirmed at implementation: the verify.sh ro-bind is
  `control/pipeline.sh:568` (not `:438`), and the declaration pin is
  `control/pipeline.sh:205-212` (not `pipeline_setup:144-149`).

- **D-P05-21 — GUARD (a) IS A MECHANICAL CITATION OF H8, NOT A FIFTH SCRIPT.**
  EX-10 asks for the OBSERVATION — "each guard OBSERVED REFUSING at least once" —
  and H8 *is* guard (a): `probes/c3/matrix.tsv` names it so in the instantiation
  column, and its `conform/gitlink` and `conform/gitmodules` alternatives are each
  already observed passing on both fixtures and both mechanisms in a scored run.
  Re-running it would put a second copy of that observation in the corpus and
  tempt a reader into counting it twice.

  **The citation can FAIL, which is the whole of why it is admissible.** The leg
  reads the committed `probes/c3/results.tsv` and counts H8 entries at each token
  with outcome `pass`; if H8 were removed, re-scored without an alternative, or
  scored anything else, it reds. A NEGATIVE CONTROL runs first — `forbidden-path`
  is a real conform token H8 never produces and must count zero — because
  otherwise `n > 0` would prove only that awk ran. A citation that cannot fail is
  prose, which is the shape D-P01-5 exists to refuse.

- **D-P05-22 — GUARD (e) IS THREE LEGS, AND THE BASELINE IS ONE OF THEM.**
  VA-3 wants a byte-identical comparison, so the thing compared against has to be
  a recorded run rather than a remembered one: **e1** F1 unmutated, **e2** F2 with
  the in-repo declaration rewritten IN THE WORKTREE, **e3** F2 with the rewrite
  COMMITTED into the range.

  e2 and e3 are different attacks, not two spellings of one. e2 asks whether the
  control plane reads anything the capsule's working tree holds; e3 asks whether
  it reads `<S>:<path>` — the read `pipeline.sh:205-212` pins as one that must
  never become the S side. **Only e2 can be byte-identical by construction**
  (a committed rewrite perturbs the range), and only e3 can say what happens when
  the substitution is published.

  **The trusted-side observable is `(verify contract field, stages file)` and
  excludes the OIDs.** `base` and `accepted` differ between F1 and F2 because the
  fixtures are different repositories — that is fixture identity, not behaviour,
  and including it would make the comparison always-unequal and never
  informative. What remains is the command the control plane RESOLVED, which is
  guard (e)'s actual subject, and the stage ladder it emitted.

- **D-P05-23 — `fx_case` LIFTED TO `falsify-lib.sh`, AS T5 SAID IT WOULD BE.**
  T5 kept `t5_run` local on the stated rule that *"one caller does not earn a
  library entry — if T6 wants the same, it is lifted then."* T6 wants the same
  "run and assert freely" shape, so it moved, same body. `fx_run` gains
  `FX_PROBE` / `FX_MUTANT_ENV` defaults (`c3` / `SPIKE_C3_MUTANT`) so every T5
  call site stays byte-identical; `falsify-t5.sh` was re-run green as the
  regression check. The overlay variable is per-probe (`SPIKE_GUARDS_MUTANT`)
  because each results file stamps `MUTATED=` in its preamble, and one shared
  name would let two files mean different things by the same marker.

## Findings

- **F-P05-47 — THE COMPLETION FLIP CAPTURES `HEAD` AS `code_end_oid`, AND IN A
  SHARED TREE `HEAD` IS WHOEVER COMMITTED LAST** (2026-08-03, T9, caught by
  reading `boundaries.toml` after the flip rather than trusting it).

  `doctrine slice phase 241 PHASE-05 --status completed` wrote

  ```toml
  code_end_oid = "5c78c892…"   # design(SL-244): DEC-121 — another agent's commit
  ```

  PHASE-05's real code tip is **`fdebae1e`** (T6's falsification round); T7, T8
  and T9 touched no source at all. The flip captured HEAD, and HEAD at that
  instant belonged to a **different slice**.

  **The tool warned, and the warning is not the same shape as the defect.** It
  said *"PHASE-05 boundary spans 12+ commits — any that are not this phase's are
  attributed to it"* and offered `record-delta --commit <code tip>`. That reads
  as "your range is wide", which for a multi-commit phase in this tree is
  unavoidable and expected — §§ PHASE-05 boundary documents 111 foreign commits.
  It does **not** read as "the endpoint I just wrote is another slice's commit",
  which is a different and worse thing. **I nearly filed the warning as known.**

  **Corrected by hand** in `boundaries.toml` (runtime tier, gitignored,
  hand-editable) with a comment naming the cause. The `record-delta` registry was
  already correct — it takes an explicit `--start`/`--end` and I gave it
  `8e962656..fdebae1e`. So the two stores disagreed, and only one of them asks.

  **The generalisation, and it is not about this phase.** *Any* phase closed in a
  shared tree has this exposure, and the four earlier boundaries in this file
  were captured the same way — **PHASE-01..04's `code_end_oid`s are unverified
  against the same failure.** Worth a look at audit before their ranges are
  believed. Filed as **ISS-307**.

  Sibling of F-P05-31's lesson at a different altitude: there, a run started
  before the row was final was void; here, a boundary captured at an arbitrary
  instant records whatever the tree happened to be.

- **F-P05-46 — THE FALSIFICATION ROUND IS RECORDED IN-BAND, SO FOUR `fail` ROWS
  IN THE SCORED TABLE ARE FOUR SUCCESSES** (2026-08-03, T8, found while writing
  the evidence summaries — not by a red).

  `results-c3.tsv` carries four rows with `outcome=fail`. All four are the
  conflict sub-probe's mutants, and each is stamped on the **preceding**
  `p-c3:` preamble line: `MUTATED=m32-peer-not-from-b.sh`,
  `m33-halves-agree.sh`, `m34-trunk-never-moves.sh`,
  `m35-move-before-pinning.sh`. A mutant that reds is the mutant **working**.

  **The hazard is that the discriminator is not on the row.** Anything that
  counts outcomes — a reader, an `awk`, a future audit — must carry the
  preamble's `MUTATED=` state forward across rows to read the table correctly. A
  naive `awk '$10=="fail"'` reports four failures in a matrix that has none.
  This is the same shape as D-P05-20's reason for giving the guards their own
  file: **a discriminator a reader must *notice* is weaker than a separation
  they cannot miss.** The guards got the separation; the sub-probe's mutants did
  not, and this is the cost.

  Not repaired — the table is scored evidence and rewriting it post-hoc is worse
  than documenting it (S2's spirit). Documented instead, in the two places a
  reader arrives from: `evidence/README.md` (limits list, item 7) and
  `evidence/guards.md` (opening section). **A future probe should stamp the
  discriminator per row, not per preamble.**

- **F-P05-45 — THE PROBE'S RESULT IS THAT THE MECHANISM AXIS BARELY MATTERS, AND
  THE ONE MEASURED ASYMMETRY RUNS AGAINST THE BUNDLE** (2026-08-03, T7, from
  deriving QUE-200's four unbanked inputs rather than from a new run).

  On every axis expected to separate M-A from M-B — hostile config/hooks,
  resource bounds, forensic retention, wall-clock cost — the two are **equal**
  (EVD-006, EVD-008, EVD-009). Every scored refusal token is produced by the
  mechanism-*independent* downstream pipeline and computed trusted-side
  (EVD-007), so the mechanism decides the transfer, not the admission boundary.

  The single measured asymmetry is **H13**: M-B needs four trusted-side refusal
  legs for a capsule-authored file, and on M-A the row is structurally `n/a`
  because no trusted-side artifact is read (EVD-010). That is counter-evidence to
  candidate 2's stated advantage — the bundle **moves** the surface rather than
  removing it.

  **Two costs sit on the M-B side and neither is a security failure**: the
  two-site `harvest/fsck-failed` token (one token, two causes — EVD-007) and the
  quiesce obligation the auto-maintenance landmine forced (EVD-009).

  **The gap worth naming before anyone reads this as a verdict.** QUE-200's own
  flagged worry about candidate 1 — *"git's protected-config rules cover
  everything a hostile repo config can do to upload-pack"* — is **still argument,
  not evidence**. H6 shows the config/hooks *hazard class* does not reach the
  parent under either mechanism, which is weaker and different: `upload-pack`
  running in the capsule repo's context was never exercised. A settle that reads
  the matrix as "fetch is proven safe against hostile config" overreads it. Both
  EVD-006 and QUE-200's prose say so explicitly.

- **F-P05-44 — `knowledge show`/`inspect` RENDER NO INBOUND RECIPROCITY, SO VA-5
  IS NOT SATISFIABLE BY THE TIER-1 EDGE ALONE** (2026-08-03, T7; filed as
  **ISS-306** + an observation, `src/` untouched per S4).

  `using-doctrine.md` says *"`inspect` / `show` render both directions"*, and
  `src/relation.rs` documents `Supports` as *"inbound renders `supported_by`"*.
  Neither holds for knowledge records: after `doctrine link EVD-006 supports
  QUE-200`, `show QUE-200` and `inspect QUE-200` render **nothing**, and
  `--json`'s `relationships` map carries outbound only. EVD-006's own view
  renders `supports: [QUE-200]` correctly. **Pre-existing, not ours** — `show
  DEC-048` likewise omits `supported_by: EVD-002`, an edge authored in July.

  **So VA-5's "BOTH TIERS" resolves differently than the sheet implied.** It is
  not (Tier-1 edge = TOML) + (prose = MD). It is the *question's own* record,
  both tiers:

  - **structured** — the question's `[evidence]` block, whose `supports` /
    `contradicts` / `notes` are documented as free-text citations and which
    `show` does render; and
  - **prose** — an `## Evidence` section in the question's `.md`.

  Both populated on QUE-200 and QUE-201, and verified via `show`. The Tier-1
  edges exist too and discharge EX-13 — they are simply invisible from the
  question's side.

  **The cost this leaves standing:** the authoritative edge and the rendered
  citation are now maintained separately and can drift. That is ISS-306's
  problem, not this phase's, but a reader who edits one should edit both.

- **F-P05-43 — ON F2, A *COMMITTED* DECLARATION REWRITE REFUSES
  `conform/undeclared-path`, AND THAT IS FIXTURE-SPECIFIC.** Guard (e)'s third
  leg publishes the substituted declaration into the range. It never reaches the
  trusted side's read — the invariant holds — but WHERE it dies is a property of
  this fixture and not of the model: F2 keeps its declaration copy at the
  repository ROOT, and SL-001 declares selectors for `src/**` and `.doctrine/**`
  only, so conform leg 2 refuses it as undeclared before leg 3 or anything later
  ever looks.

  **A project that declared its own declaration path would get past leg 2**, and
  the clause that would still hold is the one this leg actually asserts: the
  control plane resolved B's command regardless. So the leg records the refusal
  as an OBSERVATION and asserts the provenance separately — it does not score the
  token against an expectation, because the expectation would be about SL-001's
  selector list.

  Same discipline as F-P05-40: **state it, do not smooth it.** A reader who takes
  `undeclared-path` here as "the model refuses declaration substitution at
  conform" has learned something false about every other project.

- **F-P05-42 — GUARD (d) IS VACUOUS WITHOUT A GENUINELY BROKEN SUITE, AND ITS
  TWO MOST CONVINCING CLAUSES CANNOT TELL.** The honest `verify:` command PASSES
  on the light fixture. So a leg that planted a capsule-authored `verify.sh` and
  watched the run would see a pass whether the ro-bind held or had been undone —
  real runner and hostile runner give the same answer, and the observation is
  hollow. The leg therefore ALSO breaks the module the test file imports, so the
  two runners disagree (`verify/suite-failed` vs 0) and observing the refusal is
  observing the guard.

  **M38 is what makes this a finding rather than a design note.** With the suite
  left honest, the verdict clause reds — and BOTH I4a-flavoured controls stay
  green: `audit-i4a REFUSES the mutated capsule` and `the planted runner exits 0`.
  Two green controls bracketing an observation that never happened. That is EX-3's
  per-cell positive-control hazard met one level up, at the LEG rather than at the
  cell, and it is the argument for why a guard probe needs a falsification round
  of its own and not just a shakeout.

- **F-P05-41 — "REFUSES NOWHERE" IS SATISFIED BY A RUN THAT SKIPPED A STAGE, AND
  GUARD (e)'S BASELINE IS ANCHORED TO IT.** The first draft of e1 asserted
  `pipeline_first_refusal` was empty and `assert_outcome[pass]`. Both hold for a
  four-stage run AND for a run in which a stage never emitted at all — and e2's
  byte-identical comparison would then reproduce the hole faithfully and compare
  EQUAL, scoring the strongest claim in T6 against a ladder with a gap in it.

  Fixed by asserting every stage BY NAME (`c3_assert_stage_passed` × 4), which is
  what `selftest_happy` already does and says why: *"asserting only the final
  status would score a run that skipped a stage entirely as green."* Caught by
  reading, not by a red — the shakeout was green before and after.

  **Generalises past this leg:** whenever a comparison's baseline is itself a
  run, the baseline needs its own positive control. Otherwise the comparison
  inherits the baseline's blind spots and reports agreement as evidence. Same
  family as F-P04-8's "unrepresentable witness written as *nothing crosses*" —
  the absence-shaped assertion that is both wrong and green-looking.

- **F-P05-40 — ON THE CANDIDATE LAYER, A CONFLICT REFUSAL IS LEDGERED; A
  STALENESS REFUSAL IS STATUS-BORNE. THE TWO DISAGREE** (2026-08-03, T5;
  probe corrected, `src/` untouched).

  `dispatch candidate create` **exits ZERO** on a genuine content conflict. It
  CAS-creates the branch, writes a `Conflicted` row with an empty `merge_oid`,
  provisions a worktree and materialises the conflict stages — then returns
  `Ok(())` (`src/dispatch.rs:2803`, deliberate; SL-212 made the conflicted
  lifecycle a recorded state with `ingest` as its continuation, not a failure).
  So the refusal to auto-resolve lives in `candidates.toml`, and a caller
  reading exit status alone cannot see it.

  `sync --integrate --trunk` refuses the **staleness** case **non-zero**. Within
  one layer, the conflict path and the staleness path disagree about how a
  refusal is signalled. The four-stage model has no such split — every stage
  refuses through one token vocabulary, emitted and asserted by name (VA-2).

  **How it surfaced, and what I did NOT do.** The clap help says a conflict
  "aborts cleanly, writing no row/ref/worktree" — false on the arm actually
  taken, and for a `review_surface` the described arm is *unreachable*
  (`--worktree` is mandatory there). I wrote the probe's assertion against that
  documented contract and it reddened against correct software. **This is not
  S2**: S2 governs a rig edit that quiets a misbehaving SUBJECT, and here the
  subject behaves to its own design while my expectation was wrong. Resolved by
  reading the source, not by assuming. The probe now asserts the real contract —
  exit zero, refusal in the ledger — with the asymmetry named at the assertion,
  so the file records the finding instead of hiding it. `src/` is untouched (S4);
  the help-text defect is filed as **ISS-305**, and an observation is recorded.

  **T7/T8 owe this as a QUE-202 input.** It is an ergonomics fact about the
  machinery being replaced, and it belongs in the evidence exactly as stated —
  the sub-probe counts toward nothing as *coverage*, which does not make what it
  observed worthless.

- **F-P05-39 — THE FALSIFICATION DRIVERS FOR T4a–T4e WERE NEVER TRACKED, AND
  ARE GONE** (2026-08-03, found while starting H7's round; **fixed forward**).

  Four handovers and this sheet cite `drivers/falsify-lib.sh`,
  `drivers/shake.sh` and four `drivers/falsify-t4*.sh` + sweep logs as if they
  were durable artefacts. **None was ever git-tracked, none is gitignored, and
  none exists anywhere on the filesystem** — checked across every worktree, the
  stash list, the whole of history, and a `find /` sweep. Of the sweep logs only
  T4e's survives, in a prior session's `/tmp` scratchpad.

  **What this cost and what it did not.** `results.tsv` survives intact at
  `~/capsules/probes/c3/` with all fifteen prior rows, so the SCORED evidence is
  whole. What was lost is the ability to RE-RUN any prior falsification: for
  T4a–T4e the falsifiability claim now rests on this sheet's prose alone, and
  the sheet is gitignored runtime state.

  **Fixed forward, not reconstructed.** H7's round is committed inside the rig
  (`drivers/`), and mutants ride a `SPIKE_C3_MUTANT` overlay so they WRAP the
  real harness rather than fork it. Reconstructing T4a–T4e's drivers from prose
  was NOT attempted — a rebuilt driver whose contract I inferred would not be
  the one those rows were falsified with, and a re-run of it would be new
  evidence wearing an old claim's name. **T7/T8 should state the asymmetry
  rather than paper it: H7's round is re-runnable, the earlier five are
  attested.**

  **The root cause is a tier confusion, not carelessness.** A harvest cited
  untracked working-tree scratch by path without recording its tier, and nothing
  in the citation distinguishes a durable artefact from a disposable one. An
  observation is recorded.

- **F-P05-38 — NO FILE OVER THE BOUND EVER EXISTS, AND THAT IS WHAT THE BOUND
  MEANS** (2026-08-03, T4c/H7, found by writing the wrong `planted?` first).

  The obvious observable for a disk-cap row is an oversized artefact. It is
  unobtainable. `ulimit -f` refuses the write **at** the bound, so the oversize
  is never materialised: `dd` opens the file, seeks past the limit, takes
  SIGXFSZ on the write, and leaves a **zero-length** file behind. The first
  `H7_planted` written here counted files at or over the cap and found none —
  it could only ever have red, on every host, forever.

  **The observable is the refused write's SIGNATURE, not its product**: the file
  the capsule attempted is present and empty. Paired — as always in this phase —
  with the clause that makes it mean something: the capsule's cumulative
  footprint UNDER the cap, which says both that the honest work is nowhere near
  the bound (attribution, D-P05-18) and that the `du` leg had nothing to say, so
  the leg that fired was the per-file one (F-P03-1).

  **It also removed a clause from `_assert`.** "No file over the bound reached
  the quarantine / canonical" reads like containment and is vacuous — no such
  file exists anywhere, so it would pass on every run for a reason that has
  nothing to do with blast radius. Replaced with the artefacts' absence by path
  and by depth. **A clause that cannot fail is not a control**, and this phase
  has now paid for that lesson three times (M6, M12, here).

- **F-P05-37 — H7'S OBSERVABLE IS PRODUCED BY AN IN-CAPSULE ENFORCEMENT POINT,
  SO THE PLANT SEAM STRUCTURALLY CANNOT REACH IT** (2026-08-03, found while
  sizing T4c/H7; disposed by D-P05-19).

  Every scored row plants **trusted-side, between `pipeline_capsule` and
  `pipeline_run`** — that seam IS the `H<n>_mutate` mechanism
  (`probe-c3.sh:571-573`), and D-P05-5 licenses it: *"the rig plays the
  adversary trusted-side purely for convenience: everything planted is inside
  the capsule's writable root, so a hostile worker could author all of it."*

  **The convenience holds for fifteen rows and fails for H7.** The disk cap is
  evaluated by `sandbox.sh` at the END of the capsule run — `ulimit -f` inside
  the namespace, then `du -s` at `sandbox.sh:283-288` — i.e. *before control
  returns to the harness*. A blob planted at `H7_mutate` time arrives after
  every enforcement point has already passed: stage 1 never produces
  `harvest/resource-cap`, and the row scores nothing. This is what "genuinely
  needs capsule-time hostility" means mechanically, and it is why
  `capsule/worker-hostile.sh` (§ 9.1) was named and never written.

  **The generalisation worth carrying past this row:** the trusted-side plant
  seam reaches any hazard whose observable is evaluated TRUSTED-SIDE (a path, a
  payload, a ref, a bundle). It cannot reach a hazard whose observable is
  evaluated by the SANDBOX ITSELF. That is a property of where the check lives,
  not of how hostile the row is — and H13/H14 look like counterexamples only
  because their checks turned out to be control-plane (sheet `:343`).

  **Only the front half was missing.** `pipeline_capsule:281-283` already reads:
  *"The worker's status is DATA, not an error here. A worker that hit the disk
  cap must flow into a stage-1 refusal carrying `harvest/resource-cap` — dying
  on it would lose the very outcome the bound exists to produce."* The
  downstream plumbing was built for H7 from the start.


<!-- F-P05-n. Lift to notes.md at close. -->

- **F-P05-36 — `model-level` HERE IS EARNED BY TWO DIFFERENT BOUNDARIES, AND THE
  RESULTS TABLE CANNOT SAY SO** (2026-08-02, D-P05-15's cost, stated rather than
  implied).

  H11 scored `model-level`: both fixtures live, every live leg passed. That is
  the correct reading of the vocabulary — *the model handled this hazard on both
  fixtures* — and it is not the reading a reader will take. Light held by
  CONTAINING an executed payload at `verify`; heavy held by REFUSING it at
  `conform`. The sandbox-profile claim H11 was written to make is therefore
  exercised on **light only**, and the altitude column has no way to show it.

  Two consequences. The row's `_assert` carries an explicit heavy clause — *the
  verify capsule was NEVER BUILT, this cell claims no containment* — so a later
  change that admitted the path would red the cell instead of silently
  rescoring it as containment evidence. And the go/no-go must not read H11's
  `model-level` as portable evidence about the sandbox: it is portable evidence
  about the MODEL, which is a different and, here, larger claim.

  **`cell_altitude` also has an overloaded value, and it is latent rather than
  live** (found while costing the `n/a` option D-P05-15 rejected).
  `cell_altitude 0 0 2 2` — nothing to plant on light — reads
  `unproven-beyond-rust`. Its mirror, `cell_altitude 1 1 0 0` — nothing to plant
  on HEAVY — falls through to **`client-local`**, whose own comment defines it
  as *"one fixture held and the other FAILED — a divergence, a real result about
  portability"*. `probe-c3.sh:342` and `:346` assert both, three lines apart,
  with contradictory labels. No row currently lands there, so the vocabulary is
  NOT amended mid-phase (that is a § 5.4 design change for a case with no
  instance). Recorded so the next row that would have taken the `n/a` route
  meets it before it scores rather than after.

- **F-P05-35 — CLASS-2 TRIGGERS LIVE AT REPO ROOTS AND SLICE DESIGN TARGETS ARE
  SUBDIRECTORIES, SO CONFORM DISSOLVES THE BUILD-EVALUATION HAZARD UNLESS A
  SLICE DECLARES A ROOT BUILD FILE** (2026-08-02, measured on both fixtures;
  D-P05-15's substance).

  F-P05-33 read as a fixture defect — *heavy cannot instantiate this row*. It is
  a property of the model, and it holds on both fixtures for the same reason:

  | | class-2 trigger | where it lives | declared? |
  |---|---|---|---|
  | light | `postinstall` (`package.json`) | repo ROOT | no — SL-001 declares `src/**`, `.doctrine/**` |
  | heavy | `build.rs` | repo ROOT | no — SL-241 declares `scripts/spike-capsule/**` |

  A slice's design target is a subdirectory; the files a build system evaluates
  are at the root. So a hostile build script is refused at `conform` **before
  the sandbox is needed at all** — defence in depth, and one layer earlier than
  § 5.6 predicted for this row.

  **The light instantiation was already stale and nobody had noticed.**
  `matrix.tsv` said *"a hostile postinstall script"* for light. `npm test` never
  runs `postinstall` — that is an `npm install` lifecycle hook — so the authored
  attack was unreachable there too, and the row as built plants a TEST FILE that
  `node --test src/*.test.ts` globs. Reachable, and a different vector: class 1
  (explicit execution), not class 2. The matrix now says so, per fixture — light
  `1`, heavy `2` — which is the honest reconciliation and also the better
  coverage: each fixture instantiates the class its ecosystem actually makes
  live.

  The general shape, and it is the third time this phase has met it from a new
  direction (F-P05-14/18/26, then H8/M16): **a plant is only evidence if the
  thing that would trigger it can reach it.** `c3_h11_reachable` asks that
  question mechanically, per fixture, so the next execution row cannot forget
  to.

- **F-P05-34 — A `${x:-DEFAULT}` THAT LANDS ON A MEANINGFUL VALUE ANSWERS A
  DIFFERENT QUESTION** (2026-08-02, found by H11's M18 — a mutant that failed
  rather than confirming, which is the round earning its keep).

  H11's `planted?` checked its canary was armed with:

      kill -0 "${C3_H11_CANARY_PID:-0}" 2>/dev/null || return 1

  The `:-0` was written to keep `set -u` quiet on a never-armed canary. But
  **`kill -0 0` addresses the caller's own process group and succeeds**, so the
  guard passed for exactly the state it existed to catch, and M18 — which
  no-ops the arming — SURVIVED its first sweep. Fixed by testing emptiness
  separately before signalling.

  The general shape, and it is not a bash trivium: a fallback is only inert if
  the fallback value is inert **in the operation that consumes it**. `0` is a
  fine sentinel for arithmetic and a live process-group id for `kill`. Every
  `:-` wants the question *what does this value MEAN to the command below?*

  Compare F-P05-30 (an observable of a repeat must count the repeat) — both are
  clauses that pass for a reason unrelated to their subject, and both were found
  the same way. The round is worth its cost when a mutant survives.

- **F-P05-33 — H11's HEAVY INSTANTIATION HAS NO REACHABLE TRIGGER UNDER ITS OWN
  FIXTURE'S SELECTORS, AND RELOCATING IT IS WHAT DESTROYS THE REACHABILITY**
  (2026-08-02, measured before writing the row; `drivers/T4d-h11-probe.log` § 3).
  **BLOCKS the heavy half of H11 — `/consult` owed.**

  D-P05-11's trap has been resolved four times by moving the plant under the
  fixture's design-target directory. H11 is the row where that move *is* the
  damage, because H11's payload has to be **executed**, not merely admitted.

  | | light | heavy |
  |---|---|---|
  | selectors | `src/**`, `.doctrine/**` (SL-001) | `scripts/spike-capsule/**` (SL-241) |
  | `verify:` | `npm test` = `node --test src/*.test.ts` | `just web-build && cargo test` |
  | trigger inside the selector? | **yes** — a new `src/*.test.ts` is executed | **no** |

  Everything heavy's verify command executes — root `build.rs`, `justfile`,
  `web/map`, `src/**` — is undeclared under SL-241, so it refuses `undeclared-path`
  at conform leg 2 and never reaches stage 3. Everything SL-241 declares is shell
  the Rust build never reads (`grep -rln spike-capsule src/ tests/` → no match).
  So the heavy cell can be planted or executed, never both.

  **This is not a language finding.** It is a property of the two fixtures'
  `verify:` lines, and the heavy declaration's own comment predicted the shape
  without checking it: *"`build.rs` is listed because H11 plants one, not because
  the repo carries one today"* — the surface was declared, the reachability was
  not measured.

  Options, none takeable without the operator: (a) heavy `n/a` **with the
  measured reason** — the H12/light and H13/M-A precedent, but discovered late
  rather than known up front, and it drops H11 from `model-level` to
  `client-local` via `cell_altitude 1 1 0 0` (asserted in `c3_positive_control`);
  (b) run heavy anyway with an inert plant and score the *execution-evidence*
  clause red — buys the `n/a` its measurement at ~2 heavy verify runs; (c) amend
  SL-241's selectors or re-join the heavy fixture to another slice — declined
  here: authored governance edited to make a rig cell live is the tail wagging
  the dog, and it would move every other heavy cell's conform semantics.

- **F-P05-32 — THE VERIFY CAPSULE SHARES THE NETWORK NAMESPACE, SO H11's CANARY
  OBSERVABLE CANNOT HOLD — AND EX-2 IS THE REASON, NOT AN OVERSIGHT**
  (2026-08-02, measured; `drivers/T4d-h11-probe.log` § 1). **BLOCKS H11's second
  clause — `/consult` owed.**

  H11's matrix instantiation asks for two observables: *sentinel-absent on the
  host path* **and** *canary unreached*. Measured against `--kind verify`:

  | clause | measured |
  |---|---|
  | filesystem escape | **contained** — the host path is not in the namespace at all, and the sentinel is absent |
  | network canary | **REACHED** — two clients, two arrivals in the trusted-side listener's log |

  The profile carries `--unshare-all` and then `--share-net` (`sandbox.sh:163,234`),
  and `--kind` is validated once and never consulted again — *"Mount posture ONLY.
  … EX-2 is the claim that both kinds get the same posture, and this array is what
  makes that claim checkable rather than asserted."* EX-7 wants egress for the
  **worker** capsule (the agent needs the API). The verify capsule inherits it
  because EX-2 says the posture is uniform.

  So the tension is between two things the design asserts, and it is a **§ 5.6
  re-derivation**, not a bug report: an inherited expectation ("the verify capsule
  contains build-time code") is **half dissolved** by the profile the design
  specifies. The filesystem half is real and holds; the network half was never
  claimed by any mechanism.

  **Not fixed, not worked around, not scored** (S2 — a failed clause is a FINDING
  and a `/consult`, never a quiet rig edit). Passing `--no-net` to `verify_stage`
  would make H11 green in one line and would silently retire EX-2's uniformity
  claim on the way — the one thing the profile array exists to prevent.

- **F-P05-31 — A SCORED RUN STARTED BEFORE THE ROW WAS FINAL IS VOID, AND THE
  MECHANISM IS F-P05-29's FROM A NEW DIRECTION** (2026-08-02, self-inflicted,
  cost ~5 min and a 985M run dir).

  `rig c3 H14` was launched after the light shakeouts and *before* the
  falsifiability round, on the reasoning that the round only perturbs a copy.
  It does not: the round's findings CHANGE THE ROW, and `probe-c3.sh` sources
  `lib/instantiations.sh` once at process start. Editing the row under a running
  scorer produces a log and a `results.tsv` stamp for a version of the row that
  no longer exists — F-P05-29's "the apparatus moved mid-run", with the co-agent
  replaced by me.

  Killed, torn down, and the log set aside as `drivers/T4c-h14-void-preedit.log`
  rather than deleted. **`results.tsv` was untouched**, because `rows_write`
  writes the whole file at the end rather than appending per cell — worth
  knowing before assuming a killed run must be cleaned up out of it.

  **The order is: shakeout → falsify → THEN score.** The scored run is the last
  thing that happens to a row, because it is the only thing that claims to be
  evidence about the row as committed.

- **F-P05-30 — AN OBSERVABLE OF A REPEAT MUST COUNT THE REPEAT: every other
  clause of H14's duplicate leg held against a SINGLE ring** (2026-08-02, caught
  by M14 before the row was scored).

  The duplicate-ring leg observed that the waiter returned the same answer twice
  and that the published ref had not moved across the second ring. Both hold
  when there is no second ring at all — a waiter that answers the same way twice
  answers the same way when asked twice about ONE ring, and a ref that never
  moves never moves. With `c3_h14_rering` no-op'd the mutant **SURVIVED**: the
  leg reported duplication being harmless in an experiment where nothing was
  duplicated.

  Fixed by counting the rings themselves — `rings=2` and `rings-distinct=1`, so
  the bell is shown to have been rung twice with one line of content (a verbatim
  duplicate, not a third forgery wearing leg 1's name). The count clauses are
  ordered FIRST in `_planted` because they are the ones the others cannot imply.

  **The general form, and it is not specific to doorbells**: when the hostile
  input is *that something happened again*, the sameness of the outcome is not
  evidence — sameness is exactly what a single occurrence also produces. The
  positive control has to be the occurrence count. This is R-C/F-7's hazard one
  level in, and it is the same family as F-P05-28's absence-shaped legs: the
  observable that passes when its subject was never reachable.

- **F-P05-28 — H15/heavy/BUNDLE FAILS: the resume after a killed harvest refuses
  `harvest/fsck-failed`. OPEN, S2, and NOT diagnosed.** The only red in T4b, and
  the reason the mechanism column exists.

  | leg | result |
  |---|---|
  | H15 light fetch / light bundle / **heavy fetch** | **pass** — three kills, resume re-pins the same OID, CAS once |
  | H15 **heavy bundle** | **FAIL** — resume refused `harvest/fsck-failed` |

  All three canonical-unchanged clauses PASSED on the failing leg, and both
  same-OID clauses passed. What failed is the resume itself.

  **Hypothesis, explicitly UNMEASURED — do not carry it as a result.** M-B's
  harvest ends in `git fetch` from the bundle plus `git fsck
  --connectivity-only` over the quarantine (`harvest-bundle.sh:96-107`). A
  SIGKILL inside that window may leave a partial pack or a `tmp_pack_*` in
  `quarantine/.git/objects`, which the resume's `fsck` then reports. Heavy's
  bundle is large enough for the kill to land inside the write; light's is not,
  and M-A's fetch appears not to expose the same window. **That ordering — big
  bundle, killed mid-write — is a hypothesis about WHY, not the observation.**

  **The question it actually raises, and it is a design reading rather than a rig
  bug — which is why it is a `/consult`.** § 5.6 says a crash "re-runs from the
  same pinned OID". My vehicle resumes **IN PLACE**, reusing the same quarantine.
  A real pipeline restart would call `pipeline_setup` and get a **FRESH**
  quarantine — the run dir is `rm -rf`'d and rebuilt. So either:

  (a) the claim is about a fresh re-run, and H15's vehicle is testing something
      strictly stronger than the design asserts — in which case the vehicle
      re-provisions the quarantine between attempts, and the in-place result
      becomes a recorded finding about M-B rather than a failing row; or
  (b) the claim is about in-place resumption, and this is a REAL asymmetry
      between M-A and M-B at exactly the axis QUE-200 exists to decide — in
      which case it is one of the more valuable results in the matrix.

  **Do not resolve it by re-provisioning the quarantine and moving on.** That is
  the quiet rig edit S2 forbids, and under reading (b) it would erase the
  finding.

  **The next measurement is cheap and is already written**:
  `scratchpad/repro-h15-bundle.sh` — harvest only, no verify, ~1 min on heavy.
  It kills one harvest, then lists what the quarantine holds, runs `fsck`
  standalone, retries `harvest-bundle.sh` in place, and retries it against a
  FRESH quarantine as the control. The fresh-quarantine leg is what separates
  (a) from (b). It was written but **not run** — the session ended first.

  **The defect this exposed in my own row, fixed in the same commit.** `_assert`
  clause 4 called `advance_stage` against the REAL canonical. `advance_stage`
  mutates — precondition, transfer, CAS — and is only harmless while the resume
  landed. With the resume refused, its precondition still held, so **the
  assertion transferred the objects and moved the ref itself**, then
  `assert_outcome` read the canonical the assertion had just written and
  reported `refs unchanged: want 0, got 2` and `+5 objects`. One honest refusal
  presented as seven reds, two of them looking like a canonical-safety failure —
  the single most alarming shape this table can produce, manufactured by the
  observer. Now run against a throwaway clone.

  **The general shape, and it belongs beside F-P05-25's:** an assertion that
  writes to its own subject is invisible while everything passes, and produces
  its most misleading output on the first real failure — exactly when the output
  is being trusted most.

  ### F-P05-28 DIAGNOSED (2026-08-02) — it is a FIXTURE artefact, and BOTH of
  ### the readings the consult was framed around are moot

  **The cause, proven, not hypothesised.** The heavy fixture repo carries a
  **six-file incremental commit-graph chain** (`objects/info/commit-graphs/`,
  mtimes Jun 30 – Aug 1 — it is inherited from the real doctrine repo the
  fixture was cloned from, not written by any run). Provisioning copies
  `objects/` wholesale, so the fixture, `canonical`, `capsule/repo` and
  `quarantine` all carry it. That chain names commits that are **unreachable**
  in the clone — they survive as `dangling` and fsck is content.

  M-B's harvest transfers a 27 MB pack. Across H15's three kills that is enough
  to trip **git's own auto-maintenance in the quarantine**, which repacks and
  **prunes the unreachable objects**. The inherited commit-graph still names
  one. `git fsck` then reads the graph, cannot parse the commit, and exits 16 —
  so `harvest-bundle.sh`'s final leg refuses `fsck-failed`.

  | probe | result |
  |---|---|
  | fixture / `canonical` / `capsule/repo` — chain present, `c89b124a` present | fsck **exit 0** |
  | `quarantine` after H15's three kills — chain present, `c89b124a` **GONE** | fsck **exit 16** |
  | same quarantine, `-c core.commitGraph=false` | fsck **exit 0, zero non-dangling** — sole cause |
  | all 4 packs `verify-pack`; `multi-pack-index verify` | **OK** — no object damage |
  | **M-A arm, same three kills** (`diag-h15-heavy-fetch`) | `c89b124a` **survives**, fsck **0**, in-place resume **exit 0** |

  **So neither (a) nor (b).** It is not a design reading at all. The pinned OID
  is fine, the transferred objects are fine, canonical was never touched (all
  three clauses passed on the failing leg). Reading (b)'s "M-A/M-B asymmetry at
  QUE-200's axis" is *half* true and for the wrong reason — M-B detonates the
  landmine and M-A does not, but the landmine is the fixture's commit-graph, not
  the capsule model. **The prior session's partial-pack / poisoned-quarantine
  hypothesis is FALSIFIED**: zero `tmp_*`, zero `incoming-*`, every pack verifies.

  **In-place resume is NOT broken** — the premise both readings shared. On a
  quarantine killed before any transfer, the in-place resume returns *the same
  OID as a fresh clone* (`d7462447…`, both legs exit 0). § 5.6's word is
  **"idempotent"**, which is a property of re-running against the *same* state;
  re-provisioning between attempts would test a fresh run instead and is the
  quiet rig edit S2 forbids. The vehicle's in-place resume is the right vehicle.

  **Two further findings this turned up, neither fixed (S2 — operator rules).**

  1. **`fsck-failed` is emitted at TWO sites** in `harvest-bundle.sh` — the
     `git fetch` failing (:100) and the final whole-quarantine fsck (:106) —
     and git's stderr is discarded at both. A token that cannot distinguish
     "the ingested objects are bad" (security-relevant) from "this quarantine's
     derived cache is stale" (operational) is what made this cost a session to
     diagnose. I5 says refusals report trusted-side-computed tokens; it does not
     say one token may stand for two causes.
  2. **The harvester fscks the WHOLE quarantine**, not the range it just
     ingested — so any pre-existing or derived-cache damage, from any source,
     lands as a refusal attributed to the capsule.

  **Blast radius beyond H15.** Every heavy M-B cell is one auto-gc away from a
  spurious `harvest/fsck-failed`. H15 merely does enough transfers to reach the
  threshold reliably. This is not H15's finding; it is the heavy fixture's.

  **Evidence and re-run vehicles** (`.doctrine/state/slice/241/drivers/`):
  `F-P05-28-diag-bundle.sh` (+`.log`) · `F-P05-28-diag-fetch.sh` ·
  `F-P05-28-resume-on-killed-quarantine.sh` · `F-P05-28-fsck-persistent.log`.
  Run dirs left standing: `~/capsules/runs/diag-h15-heavy-{bundle,fetch}`,
  `repro-h15-heavy-bundle`.

  **RULED AND CLOSED (2026-08-02).** Operator ruled the provisioning fix sound.
  `quiesce_clone` (`control/pipeline.sh`, `710f94e5`) drops the inherited
  commit-graph and disables `gc.auto` / `maintenance.auto` on `canonical` and
  `quarantine`. The second half was NOT in the reported bug and is kept on its
  own merit: `canonical_objects` counts objects, so a background prune moves the
  number I1's falsifiability rests on — a latent flake in the canonical-unchanged
  clauses independent of the graph. Operator informed of that widening.

  | | before | after |
  |---|---|---|
  | quarantine fsck after the three kills | exit 16, 29,953 errors | **exit 0**, zero non-dangling |
  | in-place resume | refused `harvest/fsck-failed` | **exit 0**, OID `0448d675` |
  | fresh-quarantine control | — | **same OID** — § 5.6's idempotence OBSERVED |
  | object churn across the kills | 725 → 736 (repacked underneath) | 719 → 722 |

  **The row is green IN THE SHAKEOUT DRIVER ONLY — see F-P05-29, which
  supersedes this paragraph.** `shake H15/heavy/bundle/dissolution` scored 15/15,
  all four stages `verdict=pass`, `observed: <no refusal>`. One hour later the
  **harness failed the same cell on the same code.** The shakeout green is real
  as a statement about `shake.sh` and is **not** evidence about the harness.
  F-P05-28's own symptom (`harvest/fsck-failed`) is nonetheless gone and stays
  gone — the harness records `harvest` PASSING on all four H15 cells, which is
  the fix doing exactly what it claimed and nothing more.

- **F-P05-29 — THE SCORED HARNESS RUN IS RED, AND THE SHAKEOUT DRIVER DOES NOT
  PREDICT IT. OPEN, S2, NOT DIAGNOSED.** `rig c3 H6 H9 H12 H15` → **exit 1,
  298 ok / 32 FAIL**. Log: `drivers/F-P05-29-rig-c3.log`.

  | row | altitude | verdict |
  |---|---|---|
  | H6 | `model-level` | clean, four cells |
  | H9 | `model-level` | clean, eight legs |
  | H12 | `n/a` | **heavy/bundle red** — `conform/undeclared-path` |
  | H15 | `n/a` | **all four cells red** — `planted?` red AND `conform/undeclared-path` |

  **The finding is the vehicle, not the cells.** `shake.sh` and the harness
  disagree on identical code, one hour apart, with `quiesce_clone` in place for
  both. So **`quiesce_clone` is ruled out as the cause** (the 15/15 shakeout ran
  after it landed), and — far more important — **every ✓ in T4b's coverage table
  sourced from a shakeout is vehicle-local history, not evidence.** The previous
  packet's "three rows pass everywhere they were run" was a claim about the
  driver. H6's and H9's greens here are the harness and do stand.

  This is the phase's recurring failure class from a third direction: measuring
  through a vehicle that differs from the real one. Cf. F-P05-25 (an assertion
  that writes its own subject) and F-P05-28's driver aborts.

  **Shape of the reds, unexplained.** H15 reds its own `planted?` positive
  control on all four cells while the three canonical-unchanged clauses and both
  same-OID clauses still PASS — so the kills did land and the per-attempt
  snapshots exist, but `H15_planted`'s distinctness check does not hold under the
  harness. Both rows then refuse `conform/undeclared-path`, which is the trap
  F-P05-14 / F-P05-18/21 / F-P05-26 have each been an instance of.

  **Next measurement, cheap and specific.** The harness tears down its run dirs
  (`c3-H15-*` are gone), so: re-run ONE failing cell **through the harness path**
  with teardown suppressed, and find WHICH path `conform` calls undeclared. That
  one fact probably explains both H12's single red and H15's four. Do NOT settle
  it in the shakeout driver — S2, and the driver is precisely the thing under
  suspicion.

  Gates re-run green after the fix: `rig selftest` · `audit-matrix` ·
  `probe-c3 --positive-control` · `shellcheck -x -S style` · `doctrine validate`.

  Generic form carried out of the slice as **ISS-296**, plus memory
  `mem.fact.git.clone-inherits-stale-commit-graph` (`64ca2d8d`) — the signature
  is *fsck fails, every pack verifies, `-c core.commitGraph=false` makes it
  vanish*. **DEC record owed at close** alongside D-P05-6..12.

  ### F-P05-29 DIAGNOSED (2026-08-02) — the vehicle was NEVER the variable. The
  ### `doctrine` BINARY WAS, and conform leg 2 reports its own inability to run
  ### as a refusal by the capsule

  **The cause, proven, not hypothesised.** `/workspace/doctrine/target/debug/doctrine`
  — which `rig_doctrine_bin` resolves to, and which conform **leg 2** invokes —
  **cannot execute in this jail**. Its ELF interpreter is
  `/nix/store/qqiqd3ah10x8hzsif4j1y4xc1miw23nx-glibc-2.42-67/lib/ld-linux-x86-64.so.2`,
  a path that does not exist here. Every invocation exits **127**. Leg 2 is
  `… --strict >/dev/null 2>&1 || { printf 'undeclared-path'; return 1; }`
  (`pipeline.sh:388-393`), so a binary that could not START is reported as a
  **conform refusal attributed to the capsule**.

  | probe | result |
  |---|---|
  | `target/debug/doctrine slice conformance 001 -p <light fixture> --against … --strict` | **exit 127**, `cannot execute: required file not found` |
  | `~/.cargo/bin/doctrine`, same arguments | **exit 1** with a real verdict — `undeclared (1): A tools/format.mjs` |
  | `readelf -l` interpreter path | **absent from this jail's store** |
  | `rig_doctrine_bin` | returns the broken path — `[ -x ]` is TRUE for it |
  | **`rig selftest`** (A-1) | **RED — 17 assertions, EVERY one `conform/undeclared-path`**, including both happy paths (`F-P05-29-selftest-broken-bin.log`) |

  **The timeline closes it.** `target/debug/doctrine` has mtime **13:06**, the
  minute of another agent's commit `b424fd0c8` on the shared tree; the scored run
  ended 13:12. Cells that ran **before** 13:06 pass — H6, H9, **H12/heavy/fetch**.
  Every cell **after** it fails — **H12/heavy/bundle** (the very next cell) and
  all four H15. The red boundary is a wall-clock boundary, not a row property.
  `flake.lock` is modified in the working tree (crane / flake-parts / nixpkgs
  bumped), which is how a build landed against a glibc this jail does not mount.

  **H15's `planted?` red is a CONSEQUENCE, not a second defect.** With conform
  refusing, `stage=conform verdict=pass` is never emitted, so
  `c3_h15_kill_attempt`'s conform trigger never fires, the attempt runs to
  completion, and `H15_planted` reds on `=COMPLETED` and on the missing line in
  `h15-stages.verify`. One root cause explains all 32 failures. It is also why
  clauses 1 and 2 still passed: the kills that *did* land, landed correctly.

  **RETRACTION — the headline claim of F-P05-29 as first written is FALSIFIED.**
  "The shakeout driver does not predict the harness" inferred a difference of
  *vehicle* from a difference of *outcome*, while the environment changed
  underneath both: `shake.sh` ran at ~11:5x against a working binary, the harness
  crossed 13:06 mid-run. The two vehicles were never compared on equal footing.
  **The T4b coverage ✓ marks are rehabilitated as cell measurements** — they
  remain not-a-harness-run (no altitude stamp, no `c3_expected_entries` count),
  which is a statement about coverage bookkeeping, not about their truth.
  `quiesce_clone` stays ruled out; F-P05-28 stays closed and unimplicated.

  **The general shape, and it is this phase's own recurring class from a fourth
  direction:** an absence-shaped result presented as a verdict. F-P05-25 was an
  assertion that wrote its own subject; F-P05-28's `fsck-failed` was one token
  standing for two causes with stderr discarded; F-P05-28 finding 1 named that
  pattern explicitly. **Leg 2 is the same defect at the same altitude** — a
  refusal token cannot distinguish *the verb refused* from *the verb could not
  run*, because `2>&1` throws away the only sentence that says which.

  **TWO FIXES ARE OWED AND BOTH ARE S2 — operator ruling required, no quiet rig
  edit.** They are independent:

  1. **`rig_doctrine_bin` must prove the binary RUNS, not that it is `-x`**
     (`lib/common.sh:160-172`). `[ -x ]` cannot see a missing ELF interpreter, so
     the working PATH fallback — `~/.cargo/bin/doctrine`, which is fine — was
     never reached. A `"$bin" --version >/dev/null 2>&1` probe is the honest test.
  2. **Leg 2 must distinguish a refusal from a failure to invoke.** Exit 127 (and
     126) is a RIG DEFECT, not `undeclared-path`; git's/doctrine's stderr must be
     captured rather than discarded. Without this, the *next* environment change
     produces the same session-costing red. This is I5's territory and it is the
     same finding as F-P05-28's finding 1, now measured twice.

  **Blast radius beyond the rig.** Every `just` recipe resolving
  `${DOCTRINE_BIN:-./target/debug/doctrine}` is equally broken in this jail, so
  any validation gate run here is currently reporting on a binary that cannot
  start. Repairing it by rebuilding in *this* jail would clobber a binary another
  agent may be depending on in *theirs* — the shared-tree hazard (R-E), and an
  operator call rather than mine.

  **Evidence** (`.doctrine/state/slice/241/drivers/`):
  `F-P05-29-rig-c3.log` (the scored run) · `F-P05-29-selftest-broken-bin.log`
  (A-1 red, 17/17 the same token).

  **RULED AND FIXED (2026-08-02) — see D-P05-13.** Both fixes operator-agreed,
  landed with a red/green control that did not exist before. `rig selftest` is
  **GREEN, 57 assertions**, restoring A-1; `audit-matrix`, `probe-c3
  --positive-control`, `audit-dq4`, `audit-nohooks`, `shellcheck -x -S style`
  and `doctrine validate` all clean.

  **CONFIRMED BY CONTROL (2026-08-02) — F-P05-29 is CLOSED on evidence.** The
  re-run landed: `rig c3 H6 H9 H12 H15` exit **0, 330 ok / 0 FAIL**
  (`drivers/T4b-scored-authoritative.log`), against the same four rows and the
  same row code, with a doctrine binary that runs. **32 FAIL → 0.** That is the
  diagnosis' control and it holds: the failures were the rig's own toolchain,
  and neither the capsule model nor the measurement vehicle was ever implicated.
  All four rows scored `pass`; H6/H9/H15 `model-level`, H12
  `unproven-beyond-rust` by construction.

  The re-run pinned `DOCTRINE_BIN` to the bwrap-bound read-only store build
  (0.35.0, version-matched to the tree) rather than `target/debug/doctrine`.
  Version skew was already resolved by the nix fix at `dcc5e280` — the pin buys
  something else: `target/debug/doctrine` is a shared mutable artefact, and a
  co-agent rebuilding it mid-run is precisely the mechanism that voided the
  previous run. **Prefer the immutable rung for any scored run.**

  **T4b is still not closed** — its falsifiability round is owed, and that is now
  the only thing standing between T4b and done.

- **F-P05-27 — `audit-dq4`'s EXEMPTION-STALENESS CHECK IS COUPLED TO THE LIGHT
  DECLARATION'S TOKEN SET, so pointing the audit at another declaration REDS it
  while its actual DQ-4 verdict is clean.** Found while wiring H12's structural
  observable; recorded, not fixed (it is bookkeeping, and the fix is T6/T9
  territory at best).

  `audit-dq4.sh --declaration <heavy>` audits `cargo nix direnv just rustc` and
  prints `clean — no unaccounted trusted-side invocation`. It then reds
  `exemption for control/fixture-light.sh is not stale` — because with heavy's
  tokens that file has **0 sites**, so the exemption covers nothing. The
  staleness check is a fact about which token set was passed in, not about DQ-4.

  **Consequence, and it is why the row is written the way it is:** H12 asserts
  the audit's EMITTED CLAIM rather than its exit status. That is VA-2's
  discipline (`c3_assert_stage_passed` already reads an emitted line rather than
  an exit code) rather than a convenience — and the alternative, asserting exit
  0, would have made the row red for a reason with nothing to do with the
  capsule model. **S3's own re-runs are unaffected**: they use the default
  (light) declaration, where the exemption is live and the check is meaningful.

- **F-P05-26 — H12 CANNOT BE INSTANTIATED AS AUTHORED: BOTH ITS EVAL SURFACES
  ARE UNREACHABLE ON HEAVY, FOR TWO INDEPENDENT REASONS.** S2 — a finding and a
  `/consult`, not a rig edit. Measured before a cell was spent, the same method
  F-P05-14 and F-P05-21 used, and it is **the third recurrence of that same
  trap** (the observation is recorded separately).

  The matrix asks H12/heavy to "modify `.envrc` and `flake.nix` against B".
  Neither is plantable:

  | surface | at heavy's B | obstacle |
  |---|---|---|
  | `flake.nix` | tracked | **undeclared** under SL-241's design-target selectors (`scripts/spike-capsule/**`, `.doctrine/rfc/025/evidence/**`) ⇒ conform leg 2 refuses `undeclared-path` |
  | `.envrc` | ABSENT (`.envrc.example` only) | **gitignored** in this repository — `git add` refuses it without `-f` |

  Measured directly: a range modifying `flake.nix` at heavy's B returns
  `undeclared (1): M flake.nix`, exit 1, from
  `slice conformance 241 -p <clone> --against B..S --strict`.

  **Why that is fatal rather than cosmetic.** H12's re-derived boundary is
  `dissolution`, and `cell_score` reads ANY refusal as falsifying a dissolution
  (R-D, `probe-c3.sh:191-195`). So the row would score `fail` — against a
  `conform/undeclared-path` that is H4's boundary, not H12's — and the results
  table would report a defect of the capsule model where the truth is that the
  fixture's slice does not declare the path the hazard lives at. R4's direction,
  exactly as F-P05-14 described it.

  **The one structural fact that is NOT in doubt**, and it is worth separating
  from the instantiation question: `.envrc` and `flake.nix` are named by the
  heavy declaration's `interpret:` list, and `control/audit-dq4.sh` holds the
  trusted side to the `exec:` list (`cargo nix direnv just rustc`) being ABSENT
  from `control/**`. That audit is green and is H12's actual evidence — § 5.6
  says so in as many words: *"an audit row, not a pipeline row"*. What is
  missing is a **pipeline cell** the harness can score, not the dissolution
  itself.

  Options carried to `/consult`, none of them mine to take mid-execute:

  (a) **Plant the same hazard CLASS at a declared path** — `scripts/spike-
      capsule/.envrc` and `scripts/spike-capsule/flake.nix`. direnv's `.envrc`
      is per-directory and nix evaluates a `flake.nix` wherever it is invoked,
      so the class-2/3 trigger is genuinely preserved; it costs an amendment to
      `matrix.tsv`'s `instantiation` prose (authored, but NOT § 5.6's
      `expected-stage`/`expected-token`, so not S5).
  (b) **Record H12/heavy `n/a` with a structural reason** — refused on the same
      grounds F-P05-14 refused it for H5/light: a selector list is a rig choice,
      not a structural absence (R-C). It would also cap H12 at no altitude at
      all, light already being `n/a`.
  (c) **Re-pin heavy's B** to a commit whose SL-241 selectors declare the root —
      rejected for F-P05-21's reasons (rebuild + a ~6-min `verify:`-green
      re-validation, discarding the pin the whole heavy column is measured
      against).
  (d) **Score H12 as an audit row outside the pipeline harness** — honest to
      § 5.6's own words, but the matrix has no `harness=audit` value and adding
      one is a T2/T3 change with an `audit-matrix` invariant behind it.

  **Recommendation: (a).** It keeps the row live on both mechanisms, preserves
  the hazard class, changes no boundary, and touches nothing § 5.6 owns.

- **F-P05-24 — T4a IS COMPLETE: FIVE ROWS LIVE, ALL model-level** (2026-08-02,
  `4b6ee92b`). H2 and H5 join H1/H3/H4; the result-tree group is done.

  | row | boundary | light M-A/M-B | heavy M-A/M-B | altitude |
  |---|---|---|---|---|
  | H1 | `conform/ancestry-not-descendant` | pass / pass | pass / pass | model-level |
  | H2 | **dissolution** | pass / pass | pass / pass | model-level |
  | H3 | `conform/ancestry-merge-commit` | pass / pass | pass / pass | model-level |
  | H4 | `conform/undeclared-path` | pass / pass | pass / pass | model-level |
  | H5 | `conform/forbidden-path` | pass / pass | pass / pass | model-level |

  **H2 is the first row that must PASS ALL FOUR STAGES**, so it is also the first
  to pay heavy's verify: ~6 min a heavy cell against ~2s for a row refusing at
  `conform`. Worth knowing before T4b — H9/H15's dissolution legs will pay it too.

  **Falsifiable — six mutants, six reds, no survivors.** Each redded its own
  clause and nothing else:

  | mutant | redded by |
  |---|---|
  | `H2_mutate` made a no-op | `planted?` |
  | H2 amended onto a parent off B | `planted?` — the ref DID move off the doorbell's OID, so only the descendant clause can have caught it |
  | `c3_doorbell_oid` echoing the result | `planted?` **and** the I5 assert, independently |
  | H5's non-ASCII form at the spec's literal `.doctrine/naïve.md` | **HEAVY only** — observed `conform/undeclared-path`, scored fail |
  | light dropping the rename payload alone | `planted?` |
  | an undeclared rename destination | **LIGHT only** — observed `conform/undeclared-path`, scored fail |

  The last two are the point: F-P05-18's and F-P05-21's path constraints are now
  **measured rather than argued** — red when violated, green when met, on exactly
  the fixture whose selectors decide it. The mutants also fixed a wrong first
  attempt: a mutant that dropped the rename from BOTH the payload and
  `c3_h5_paths` passes, correctly — that is not a defect, it is heavy's legal
  configuration. A mutant must perturb the payload alone.

- **F-P05-25 — A FAILED `git add` INSIDE `c3_commit` WAS SURVIVABLE, AND IT
  SURFACED AS A `fatal:` IN A GREEN LOG.** Found by reading H5's first passing
  run rather than by a red. Fixed in the same commit; `c3_commit` now
  `rig_die`s.

  H5's rename leg called `c3_commit` with both legs of the rename. `git mv` had
  already staged both, so the source existed in neither worktree nor index and
  `git add -- <source>` matched no pathspec — `fatal: pathspec … did not match
  any files`. The cell passed anyway, because `c3_commit`'s `git commit` carries
  no pathspec and took the whole index, which `git mv` had already populated.

  **Why the luck is not the point.** `git add` and `git commit` are separate
  calls, so a failed `add` leaves the payload OUT of the commit while the commit
  still succeeds on whatever else was staged. The cell then runs against a range
  missing the very thing it planted, and only `planted?` stands between that and
  a scored result. H4 shares `c3_commit` and stays green, so the hardening is
  behaviour-preserving — and M6 observed the new `rig_die` firing on a real
  `git mv` failure, so the guard is not merely asserted.

  **The general shape, which is this phase's recurring one:** an error message
  inside a PASSING log is the cheapest possible warning and the easiest to scroll
  past. Same family as F-P05-15's `*)` arm — a failure that is visible but not
  attributable.

- **F-P05-23 — `core.quotePath=false` IS INERT IN CONFORM LEG 3, BECAUSE `-z` IS
  ALSO PRESENT — so guard (b) is not falsifiable the way F-4 describes.**
  Measured, git 2.54.0, and it corrects a claim the design makes about its own
  belt.

  § 5.2 and F-4 say all three hardenings exist "because all three evasions are
  real". For `core.quotePath=false` that is true of a leg written WITHOUT `-z`,
  and leg 3 has `-z`:

  | invocation | output for `.doctrine/naïve.md` |
  |---|---|
  | `quotePath=true`, **no** `-z` | `".doctrine/na\303\257ve.md"` — quoted AND double-quoted; a `case .doctrine/*` prefix test **does not match** |
  | `quotePath=true`, **`-z`** | `.doctrine/naïve.md` — verbatim |
  | `quotePath=false`, `-z` | `.doctrine/naïve.md` — verbatim |

  `-z` suppresses C-quoting unconditionally, so in leg 3's actual invocation
  form the non-ASCII evasion is already dead and `core.quotePath=false` is
  belt-and-braces. Its only falsifier is removing `-z`, which would also break
  the `read -r -d ''` loop — i.e. not a live evasion path.

  **Consequence for the row, and it is narrow.** Form (b) is still worth
  planting: it observes that leg 3 *fires on a non-ASCII path*, which is what
  § 9 asks of guard (b). What it cannot do is discriminate a leg 3 that kept
  `core.quotePath=false` from one that dropped it. The row must not claim the
  latter. **Not a defect in `pipeline.sh`** — the flag is harmless and the Rust
  belt may read non-`-z` output elsewhere; the defect would be in a results
  table that reported guard (b) as proving something it does not.

- **F-P05-22 — THREE FORMS IN ONE CELL MASK EACH OTHER: leg 3 short-circuits, so
  guard (c) is never OBSERVED REFUSING.** § 9 requires each guard to be observed
  refusing at least once; a combined range cannot deliver that for the
  rename-out.

  `conform_stage` returns on the FIRST `.doctrine/`-matching path
  (`pipeline.sh:377-384`). Measured against a light range carrying all three
  forms — plain edit, non-ASCII, rename-out — under leg 3 with each hardening
  removed in turn:

  | leg 3 variant | paths seen | refuses on |
  |---|---|---|
  | belt-hardened (`quotePath=false`, `--no-renames`) | 4 | `.doctrine/hymns/role/worker.md` (the rename SOURCE) |
  | `quotePath` removed | 4 | same |
  | **`--no-renames` removed** | **3** | `.doctrine/naïve.md` — *the source leg vanished and it refused anyway* |
  | both removed | 3 | same |

  So an unhardened leg 3 scores the combined cell **`pass`**. The rename-out
  evasion is real — `--find-renames` drops the `.doctrine/` source leg, 4 paths
  → 3 — but forms (a) and (b) are unhidden under every variant, so they refuse
  on its behalf. A cell that plants all three proves only that *some*
  `.doctrine/` path is caught, which is H5's boundary but not its guards.

  **Isolation is what makes each form a control**: guard (c) is observed only in
  a range whose ONLY `.doctrine/` path is the rename source. The current shape —
  one pipeline leg per cell (`cell_run`), four cells per row — has nowhere to
  put a second and third leg. This is a shape question, not a rig bug, and it is
  the `/consult` S2 mandates.

- **F-P05-21 — H5's RENAME-OUT HAS NO HEAVY INSTANTIATION: the only design-target
  `.doctrine/` prefix at heavy's B is EMPTY.** Measured, not reasoned — the same
  method F-P05-14 used, and the same class of trap one level deeper.

  A rename-out needs a source that (i) exists **at B**, because leg 3 reads a
  two-dot tree diff `B..S` and a file created then renamed inside the range
  appears in neither tree, and (ii) is **design-target**, or leg 2 refuses
  `undeclared-path` first.

  | fixture | tracked `.doctrine/` files at B | design-target `.doctrine/` prefix | rename-out verdict |
  |---|---|---|---|
  | light | 17 | `.doctrine/**` | **conformant** — reaches leg 3 |
  | heavy | 5, all under `.doctrine/rfc/025/` | `.doctrine/rfc/025/evidence/**`, **0 files** | **undeclared** — leg 2 refuses first |

  Measured directly: `slice conformance 241 -p <heavy clone> --against B..S
  --strict` on a `git mv .doctrine/rfc/025/red-team.md
  scripts/spike-capsule/red-team.md` refuses; the light equivalent
  (`.doctrine/project-orientation.md` → `src/h5-renamed.md`) is conformant.
  Heavy's other four `.doctrine/` files sit under the **scope-relevant**
  `.doctrine/rfc/025/**`, which F-P05-14 already measured does not satisfy
  `--strict`.

  **D-P05-9's remedy does not transfer.** Light was fixed by amending SL-001's
  selectors; heavy's slice is SL-241 **as committed in this repo at B**, and
  `fixture-heavy.sh` asserts `HEAD is at exactly B` with a matching commit count
  (`:93-103`) — the fixture carries no synthetic commits by construction. So
  giving heavy a design-target `.doctrine/` file means a new B in *this* repo
  plus a rebuild, and F-P05-16 requires the new B to be `verify:`-green, which
  is a ~6-minute measurement. Circular for now: the content that would satisfy
  it is T8's evidence output, which T4 precedes.

- **F-P05-20 — EX-1's FLOOR IS FILESYSTEM-ONLY; THE CAPSULE'S NETWORK IS NOT
  CONFINED AT ALL.** Recorded, not fixed — it is outside PHASE-05's subject and
  a v0 profile is allowed to be v0. Named so the claim is not read wider than it
  is.

  `sandbox.sh` defaults `net=1` (`:73`) and **no caller passes `--no-net`** — so
  BOTH capsule kinds get `--share-net`, i.e. the HOST network namespace: same
  routing table, same interfaces, same firewall position as the operator.
  Confirmed against the jail's own design (operator, 2026-08-02): network is
  granted by *namespace omission*, not by a permission, so there is no egress
  policy to inherit and no seccomp layer either.

  Three consequences the profile's prose does not currently bound:

  - **Host loopback is reachable.** Anything on `127.0.0.1` — other capsules'
    services, MCP brokers, dev servers, a cloud metadata endpoint — is inside
    the capsule's reach, and the careful FS/secret hygiene does not cover it.
  - **The capsule is deliberately GIVEN a credential.** `:223` binds the agent
    home "carrying the API credential (EX-1)". A credential plus unrestricted
    egress is an exfiltration path by construction.
  - So EX-1's floor — *the canonical repo, the other capsules, and the git
    credentials must be ABSENT* — is a **visibility** claim about the
    filesystem. It is true as written and it is not a claim about the network.

  **Why this is in scope to RECORD but not to fix.** The spike's question is
  ingestion safety — what a hostile capsule gets INTO canonical — and every P-C3
  row is about that axis. Exfiltration is a different axis and no row covers it.
  But `sandbox.sh` calls itself "the P-C2 v0 **confinement** profile", and a
  reader will take that wider than the filesystem unless the boundary is stated.

  **The lever, if it is ever wanted** (operator): a third option between
  `--unshare-net` and share-host — `--unshare-net` plus a slirp4netns/pasta
  userspace stack, or a net namespace with veth and an nftables egress
  allowlist. `jail.nix` has no combinator for either. The variant that also buys
  *domain-level* allowlisting is `--unshare-net` + `HTTPS_PROXY` to a host-side
  filtering proxy with its MITM CA bound into the jail's `/etc/ssl` — which the
  namespace approach cannot give at all. This is the same lever QUE-204 option 4
  needs, so the two should be settled together if either is.

- **F-P05-19 — THE HEAVY COLUMN REACHES `advance` — STAGE 3 AND 4 PASS FOR THE
  FIRST TIME.** The proof D-P05-7 and the S1 fix jointly owed.

  | leg | stages | wall | capsule |
  |---|---|---|---|
  | heavy happy path, M-A | harvest ✓ conform ✓ **verify ✓** advance ✓ | **376s** | 4.4G |
  | heavy happy path, M-B | harvest ✓ conform ✓ **verify ✓** advance ✓ | **409s** | 4.4G |

  So the four stacked causes in F-P05-15 are all cleared, and **H9/H10/H11/H15/
  H16 are unblocked** — they needed a verify PASS to reach an `advance`
  boundary and on heavy it was unobtainable.

  **The bound sizing is validated by the numbers rather than assumed.** 409s
  clears the old 300s default, which would have failed BOTH cells as
  `verify/verify-timeout` — a LEGAL token, so `assert_outcome` would have
  accepted an honest run as a refusal silently. 900s carries adequate headroom;
  4.4G against the 8G cap likewise.

  **A correction worth carrying, because the misreading is easy to repeat.**
  Mid-run this was read as a STALL: `bun install` sat at ~68M of `node_modules`
  for minutes, 0.1% CPU, sleeping in `do_epoll_wait`, no rustc — while the host
  reached the same registry in 0.17s. Both legs then completed normally. **Low
  CPU and a flat directory are what a network-bound fetch looks like from
  outside**, and a rig whose cells legitimately take ~6 minutes cannot be
  diagnosed by watching one for two.

  **The narrower residual concern stands, and it is QUE-204's.** Every cell
  reaching stage 3 fetches from `registry.npmjs.org` inside the capsule, so a
  cell's verdict depends on a third party: an outage or a throttle lands as
  `verify/suite-failed` or `verify/verify-timeout` with nothing to distinguish
  it from a real verdict. That is a robustness and cost question, **not a
  blocker** — the column works today.

  **What survives of D-P05-7 either way.** Its *reason* is untouched: assets
  must derive from B, not from edge, or the fixture tests an approximation of
  its own base. Only the *timing* is in question — building at fixture-build
  time from the clone's own source at B satisfies the same reason with no
  per-cell network. That is QUE-204 option 1/2.

  **Operator position (2026-08-02):** egress is acceptable *if allowlisted* —
  the objection is to unbounded egress, not to egress — but an allowlist is
  per-project setup, which is recurring toil. And the direction this should
  ultimately grow toward is provisioning from a shared capsule at a known HEAD
  so that N concurrent phases off one base pay setup **once**. Deferred for
  sequencing, not disagreement.

- **F-P05-18 — H5's THREE FORMS EACH HAVE TO CLEAR LEG 2 SEPARATELY, and on
  heavy only ONE prefix does.** Found while writing the row, before it cost a
  cell. D-P05-9 fixed the fixture; this is the same trap recurring per FORM.

  Heavy's actual selector list is sharper than § 5.6's summary of it:

  | selector | intent |
  |---|---|
  | `scripts/spike-capsule/**` | **design-target** |
  | `.doctrine/rfc/025/evidence/**` | **design-target** |
  | `.doctrine/rfc/025/**` | scope-relevant |
  | `.doctrine/knowledge/**` | scope-relevant |

  Only `design-target` clears conformance `--strict`, so on heavy the ONLY
  `.doctrine/` prefix that reaches leg 3 is `.doctrine/rfc/025/evidence/**` —
  which is exactly what F-P05-14 measured (`.doctrine/knowledge/h5-probe.md`
  came back *undeclared*). Consequences for the row:

  - forms (a) plain edit and (b) non-ASCII path must BOTH sit under
    `.doctrine/rfc/025/evidence/` to be common to the two fixtures. Light's new
    `.doctrine/**` covers it; heavy has nothing else that does. A non-ASCII path
    at `.doctrine/naive-with-diaeresis.md` — the spec's literal suggestion —
    would refuse `undeclared-path` on heavy and score the row a defect of the
    MODEL. R4's direction again.
  - form (c) rename-OUT needs its DESTINATION declared design-target too, and
    that differs per fixture: light `src/**`, heavy `scripts/spike-capsule/**`.
    So H5 is the first row needing fixture knowledge inside `mutate`.

  **No signature change is owed** — `probe-c3.sh:569` already calls
  `"${row}_mutate" "${run}" "${fixture}" "${mechanism}" "${alt}"`. H1/H3/H4 take
  only `<run>` because they happened not to need more, not because that is the
  contract.

- **F-P05-15 — S1 WAS FOUR CAUSES STACKED BEHIND ONE TOKEN, and the `*)` arm is
  what stacked them** (2026-08-02, `c4a26004`; operator ruled in `/consult`).

  F-P05-11 diagnosed the heavy verify refusal as the disk cap. That was true and
  it was the FIRST of four. Each was invisible until the one above it was
  cleared, because `verify_stage`'s `*)` swallows every status it does not name
  and prints `suite-failed` — *the project's tests failed* — about runs whose
  tests never started.

  | # | cause | how it presented | fix |
  |---|---|---|---|
  | 1 | 256 MiB cap, sized for the LIGHT fixture | `RIG_EXIT_DISK` → `suite-failed` | per-fixture bound (4.4G observed) |
  | 2 | `web/map/dist/` absent from a clone | compile failure → `suite-failed` | build on site (D-P05-7) |
  | 3 | `/bin/sh` unmounted → hook tests false-RED | 3 tests fail → `suite-failed` | `--ro-bind /bin/sh` (F-P05-17) |
  | 4 | B pinned at a transiently-red commit | 1 test fails → `suite-failed` | re-pin B (F-P05-16) |

  **Three measurement rounds to see past the first**, which is the cost of a
  catch-all that means two things. Resolved per the operator's framing — *stop
  failures being silent as to reason* — which is sharper than "mint a token":
  `*)` → `suite-failed` is HONEST for a command's own nonzero exit, and the
  defect was only that sandbox-injected statuses collided with it. So every
  injected status is now named above `*)`, and `verify/resource-cap` joins the
  closed set as the disk bound on the OTHER capsule kind — `harvest/resource-cap`
  has folded the worker capsule's since PHASE-03, and EX-2's "both kinds, same
  posture" was half-true while only one could say so.

  **Falsifiable, both legs.** 64M of `dd` under a 16M bound emits `resource-cap`;
  the SAME command under a 512M bound passes. `dd` itself always succeeds, so the
  token is attributable to the bound and not to a noisy command.

  **One residue, recorded not closed:** the rig's reserved statuses (`2 3 4 5 6`)
  share a channel with the verify command's own exit code, so a suite exiting `4`
  is indistinguishable from the disk bound. Not live — `cargo` exits `101`,
  `node` exits `1` — and separating them means an out-of-band status channel,
  which is larger than this vocabulary. Named so it stays looked at.

- **F-P05-16 — A FIXTURE'S BASE B MUST BE A COMMIT WHERE THE DECLARED `verify:`
  PASSES, and nobody wrote that down.** The heavy fixture is pinned at
  `31563da55` — *"chore(SL-241): gitignore the capsule-spike raw run logs"* —
  which is the commit that ADDED `.doctrine/rfc/025/evidence/raw/` to
  `.gitignore` without classifying it, reddening
  `worktree::tests::every_runtime_gitignore_glob_is_classified`. This repo
  reverted the entry three commits later (`ba9f4a8a3`).

  So B is a transiently-red commit, and `verify` PASS is unobtainable there for
  a reason that is about the fixture's pin and not about the capsule model. Any
  commit can be transiently red; pinning B without running the declaration
  against it is how this bites. **Belongs in `fixtures.md` as a build-time
  criterion**, not as folklore.

  **Also recorded: the provisioning gap this sat behind.** `fixture-heavy.sh:71`
  provisions by `git clone`, and the capsule provisions by git-object transfer —
  both carry TRACKED content only, by construction. `web/map/dist/` is a
  RustEmbed `#[folder]` root AND gitignored build output, so a clone cannot
  build. This repo already knows: `.worktreeinclude` names it, because
  `worktree fork` hit the same wall first and solved it. The capsule model has
  the dual for *interpretation hazard* (the declaration, DEC-099) and none for
  *provisioning need*. Dissolved for this slice by building on site (D-P05-7),
  which is strictly better than carrying the artefact: the capsule regenerates
  from ITS OWN tracked source, so B stays self-consistent and no
  built-at-edge-vs-B staleness question arises.

- **F-P05-17 — THE CAPSULE'S VISIBILITY FLOOR AND `verify:`'s FIDELITY PULL
  AGAINST EACH OTHER, and the mount list is the register of where.** A
  model-level result, not a rig bug.

  Three of the four heavy verify failures were the capsule's, not the commit's:
  `install_coord_hook_installs_and_chains_the_operator_hook`,
  `install_coord_hook_permits_commit_when_no_chained_hook_exists`, and
  `run_commit_declared_deletion_lands_and_scopes_allowed_deletions_to_child`.
  All three write `#!/bin/sh` git hooks. `sandbox.sh:18-24` rejects the seed's
  `--ro-bind / /` deliberately — EX-1 needs a VISIBILITY floor, not merely a
  writability one — so `/bin` is absent and git cannot exec the hooks.

  **Attributed, not guessed.** Their source is byte-identical at B and at edge:
  ZERO commits touch `src/dispatch/` or `src/worktree/` across the 82 between
  them. Same code, green on the host, red in the capsule ⇒ environment.

  Resolved with `--ro-bind /bin/sh /bin/sh` — the FILE, per the operator: on
  NixOS `/bin` holds `sh` and nothing else so the two forms are identical here,
  and the difference only appears on a host where `/bin` is a few hundred
  binaries. Binding the directory is a posture that silently widens off-NixOS.
  This rides the seam `--ro-bind /usr/bin/env` already cut for F-P02-2, which is
  **the same finding one level down** — F-P02-2 found it for the RIG's own
  runners, this found it for the PROJECT's suite. The bind list is the register
  of shebang dependencies discovered so far, and should grow one entry at a
  time, each naming the run that found it.

  **The general claim, which outlives this fixture:** a capsule running the
  project's declared `verify:` produces false REDs wherever the suite depends on
  ambient filesystem structure the capsule withholds by design. EX-1's floor and
  `verify:`'s fidelity are in tension; every resolution is a named widening of
  the allowlist, and the register is the evidence of how wide fidelity costs.
  A direct QUE-200 input.

  **Control:** the `/bin/sh` bind flips exactly the three hook tests green and
  leaves the base-commit failure RED. Had it moved that one too, the attribution
  would have been wrong.

- **F-P05-12 — THREE ROWS ARE LIVE AND THE HARNESS IS PROVEN END TO END**
  (T4a, 2026-08-02, `ac9b534c`). F-P05-8's open item is closed: `cell_run`,
  `c3_run_row`, the altitude stamping and the completeness count had only unit
  checks; they now have cells.

  | row | boundary | light M-A/M-B | heavy M-A/M-B | altitude |
  |---|---|---|---|---|
  | H1 | `conform/ancestry-not-descendant` | pass / pass | pass / pass | **model-level** |
  | H3 | `conform/ancestry-merge-commit` | pass / pass | pass / pass | **model-level** |
  | H4 | `conform/undeclared-path` | pass / pass | pass / pass | **model-level** |

  Twelve cells, twelve passes, ~5s a row. The wiring T3 predicted would bite did
  not, because F-P05-10 checked the fixture joins first — which is the finding
  about method, not luck.

  **H3 asserts I3's actual claim rather than restating its token.** The row
  observes that `merge-base --is-ancestor B S` SUCCEEDS on the merge commit in
  the quarantine — i.e. a naive single-commit ancestry predicate would have
  passed this result, and only the no-merge clause kills it. That is the whole
  reason conform leg 1 has two clauses, and it is now observed rather than
  commented.

  **Falsifiability — three clauses mutated in a `git init`'d sibling, three
  reds, no survivors.** (The sibling must be a repository: `rig_repo_root` fails
  closed outside one, so an unversioned copy would "red" at I6 for a reason
  that is not the mutation's.)

  | mutant | reddened by |
  |---|---|
  | `c3_publish`'s bundle create redirected to a sink — ref moves, bundle stale | the M-B leg's **boundary comparison** and `_assert`'s trusted-side range. `planted?` passes it, which is the point |
  | the published result never moved to the planted commit | `planted?`, on both mechanisms (F-7) |
  | H1 parented on `HEAD` instead of `B^` — S still a descendant | `planted?` — the payload IS the ancestry relation, so its control is that relation's absence |

  Gate: `shellcheck -x -S style` over every rig file, `audit-matrix` (14
  invariants), `probe-c3 --positive-control` (21), `rig selftest` (6.5s),
  `doctrine validate` — all green.

- **F-P05-13 — H2 HAS NO DETERMINISTIC INSTANTIATION ON M-A, AND THE REASON
  LOOKS LIKE A DISSOLUTION.** S2: a finding and a `/consult`, not a rig edit.

  The matrix asks H2 to "force-move the capsule ref AFTER the doorbell rings"
  and expects `harvest/oid-mismatch`. But read where the pin is actually taken
  (`harvest-fetch.sh:66-79`): the harvester **pins the ref itself**, then
  fetches, then compares. Its `oid-mismatch` guard therefore covers exactly one
  window — between its own `rev-parse` and its own `fetch`, inside one process.

  A mutate running in the A-2 seam is upstream of BOTH reads, so pin and fetch
  agree and no mismatch is producible. Closing that window deterministically
  would need the ref to move *during* the harvester's own fetch, and git offers
  no seam there: `upload-pack` runs no ref-moving hook, and loose/packed refs
  resolve identically for `rev-parse` and for the advertisement. A background
  racer would be nondeterministic, which is not a probe.

  **The interesting part is why, and it is a re-derivation, not a rig gap.**
  Because the pin is taken *inside* the harvester, a rewrite past the doorbell
  is not an attack at all — the OID pinned is whatever the capsule last
  published, and conform, verify and advance all name that same OID
  consistently. The capsule gained nothing by rewriting. H2's hazard is
  **dissolved by the same construction that makes RT-5's pin meaningful**, which
  is the shape § 5.6 already re-derived for H6/H9/H12/H15.

  H2 would only reach `harvest/oid-mismatch` if the pin were taken at DOORBELL
  time and handed to the harvester — and the doorbell carries no authority and
  is never read (I5), so it cannot be the pin's source.

  **Not decided here.** Re-deriving a row's boundary is a design change, and
  § 5.6 is the source of truth (S5). H2 stays unwritten; `rig c3 H2` refuses.

- **F-P05-14 — H5 CANNOT REACH CONFORM LEG 3 ON THE LIGHT FIXTURE: leg 2 gets
  there first.** S2: a finding and a `/consult`.

  Measured, not reasoned — `doctrine slice conformance … --strict` run directly
  against both fixtures:

  | fixture | planted path | verdict |
  |---|---|---|
  | light (SL-001, selectors: `src/**` only) | `.doctrine/project-orientation.md` | **undeclared** ⇒ leg 2 refuses `undeclared-path` |
  | heavy (SL-241) | `.doctrine/rfc/025/evidence/h5-probe.md` | **conformant** ⇒ leg 2 passes, leg 3 refuses `forbidden-path` |
  | heavy (SL-241) | `.doctrine/knowledge/h5-probe.md` | **undeclared** — `scope-relevant` does NOT satisfy `--strict`; only `design-target` does |

  So on light, `cell_score` would read right stage / wrong token ⇒ **fail**, and
  the row would report a defect of the capsule model where the truth is that the
  light fixture's slice declares no `.doctrine/` path for leg 3 to be
  load-bearing over. R4's direction exactly.

  This is `pipeline.sh:352-357`'s own argument arriving as an instantiation
  constraint: leg 3 is load-bearing precisely *"whenever a selector happens to
  declare that path — and this slice's own selectors are exactly that shape"*.
  **That is a description of the heavy fixture.** The light fixture cannot
  express the condition leg 3 exists for.

  Three resolutions, none of them mine to take mid-execute:
  (a) amend F1 to give SL-001 a `.doctrine/**` design-target selector — makes
      light structurally comparable to heavy and is what H5 is *for*; costs an
      F1 rebuild and a `fixtures.md` amendment (T1 territory, marked BUILT);
  (b) record H5/light `n/a` — **refused**: R-C wants a STRUCTURAL absence, and
      a selector list is a rig choice, not an absence;
  (c) re-derive H5's light boundary as `undeclared-path` — a § 5.6 change (S5).

  Note the knock-on: **T6's guards (b) and (c)** (non-ASCII path, rename-out)
  "fall out of T4a" per the sheet — under (a) they fall out on both fixtures,
  under (c) on heavy only. H5 stays unwritten until this settles.

- **F-P05-1 — the PHASE-05 baseline is green, and the run is recorded**
  (T0, 2026-08-02, head `8e962656`, branch `edge`).

  | check | observed |
  |---|---|
  | `./rig selftest` | **all assertions hold**, `6.6s` wall. Happy path both mechanisms; attribution across three stages (`conform/undeclared-path`, `verify/suite-failed`, `verify/verify-timeout`, `advance/stale-base`); bundle hygiene all four legs refusing with the untouched-bundle positive control; **VA-3's falsifiable leg re-observed** — the deliberately canonical-writing harvest REDS `assert_outcome` on the object-count clause while the refs clause stays blind to it |
  | `control/audit-dq4.sh` | clean; exec tokens `node npm tsc` (read from the **light** fixture's declaration); the one exemption printed and asserted **not stale** — `control/fixture-light.sh`, 3 sites |
  | `control/audit-nohooks.sh` | clean; B1–B6 each with a token leg **and** a positive structural witness |

  So A-1 holds: from here a "refused" is distinguishable from "rig broken", and
  a hostile row may claim a kill.

- **F-P05-2 — the light-cell cost estimate for R-B.** `rig selftest` runs eight
  full pipeline scenarios — each with three clones and two sandboxed capsules —
  in **6.6s total**, i.e. well under 1s per light cell. The ~32 light cells are
  therefore not the cost question; the heavy column is, and it is still
  unmeasured. Measure one heavy cell at the top of T4a before committing to 32.

- **F-P05-3 — `audit-dq4` reads the LIGHT declaration, and that matters for
  S3.** Its exec tokens are `node npm tsc`, not the heavy declaration's
  `cargo nix direnv just rustc`. So the audit that guards the inherited DQ-4
  condition is watching exactly the toolchain H11's light instantiation
  (`postinstall`) would invoke. Re-running it after T1 and T4d is not a
  formality — it is the only mechanism that would catch a payload-bearing
  fixture reusing `fixture-light.sh`'s trusted-side `npm` loop.

- **F-P05-4 — F2 and F3 are built, and the base fixture re-greened unchanged**
  (T1, 2026-08-02, `7c134057`). `--variant <base|inrepo|plan>` on the one
  provisioner; each variant adds exactly one commit to the shared
  red→green→install sequence, and the count is asserted rather than floored.

  | check | observed |
  |---|---|
  | `fixture-light.sh --variant base --force` | all assertions hold — the behaviour-preservation leg |
  | `--variant inrepo` | all hold; the in-repo copy is present, **tracked**, byte-identical to the sibling |
  | `--variant plan` | all hold; `slice status 1` reads `phases: 1/1` |
  | `rig selftest` | all assertions hold, `6.5s` — the happy path survives the base rebuild |
  | `audit-dq4` (S3) | clean; the conditional exemption printed and **not stale** |
  | `audit-nohooks` | clean |
  | `shellcheck -x -S style` | clean (`LC_ALL=C.UTF-8`) |

  **Negative controls, observed** — the assertions can fail: an unknown
  (`--variant bogus`) or empty `--variant` refuses at exit 2 **before any
  provisioning**; `cmp` reds on a one-line mutation of the in-repo declaration;
  `ls-files --error-unmatch` reds on an untracked path. The first is a real
  wiring control; the latter two exercise the predicates against mutated data
  rather than the script's dispatch, and are recorded as that and not more.

- **F-P05-5 — F3's phase completion is RUNTIME state, and does not survive a
  clone. T5 inherits this.** `doctrine slice phase … --status completed` writes
  `.doctrine/state/slice/001/phases/phase-01.toml`, which `doctrine install`'s
  own `.gitignore` covers at line 1 (`.doctrine/state/`). So `phases: 1/1` is a
  property of the fixture **directory**, not of the repository: the authored
  `plan.toml` is tracked and clones fine, the completion does not. A sub-probe
  that cloned F3 before running `prepare-review` would meet the gate's refusal
  and could score it as a finding about the incumbent candidate layer — a
  conclusion that would be a fact about the fixture. Both halves are asserted at
  provision (plan tracked; tracking **not** tracked, via `rig_assert_fails`), so
  the constraint is stated by the build rather than discovered by the probe.

  Not chased further at T1: whether `prepare-review` reads the runtime tracking
  or tree-reads `boundaries.toml` off `dispatch/<slice>` (`dispatch.rs:1944`)
  is T5's determination, and EX-15's provisioning criterion is fixed
  independently of it.

- **F-P05-6 — the matrix is authored and gated, and the gate is falsifiable**
  (T2, 2026-08-02, `49fbce82`). 16 rows x 2 fixtures x 2 mechanisms = **64
  cells**, 4 of them `n/a` up front (H12/light — no `.envrc`/`flake.nix`;
  H13/fetch — no capsule-authored artifact exists at all) => **60 live cells**,
  which is the sheet's own predicted shape to the cell.

  `control/audit-matrix.sh` green on 14 invariants; `shellcheck -x -S style`
  clean. **Positive controls, each mutated from the real file and each observed
  reddening its own check** — a token typo (`ancestry-not-descendent`), an
  authored `altitude` value, a `dissolution` cell claiming a token, a deleted
  row, a sixth `vector-class`, a renamed header column, a half-authored `n/a`
  (planted without outcome), and a stray twelfth column. The check the file
  most needs is the token one: without it an authored typo scores as a RIG
  DEFECT at run time, after the cell has run.

  `doctrine slice verify-vt 241` — PHASE-05 VT-1 moved **FAIL -> UNATTRIBUTABLE**
  ("keyword present but ... not modified by this slice"). That is the expected
  reading while the phase is `in_progress`; it resolves on the flip to
  `completed` (F-P04-11). PHASE-06 VT-1 still FAILs on `rfc/025/go-no-go.md`,
  correctly — that file is PHASE-06's.

- **F-P05-7 — `IFS=$'\t' read` SILENTLY COLLAPSES empty TSV fields.** Tab is IFS
  *whitespace*, and a run of IFS whitespace is one delimiter, so
  `...dissolution\t\tn/a\tn/a\t` reads back with the token column holding
  `n/a` and every later column shifted left. Observed at T2 as **two invented
  violations on H12/light** that the file did not have — a validator accusing
  the artefact of the reader's defect, which is the worst direction for this
  error to run: the natural repair is to edit the file until the reader stops
  complaining.

  Fix, and it is the reader's job not the file's: translate tabs to a
  **non-whitespace** separator first, then split on that —
  `IFS=$'\x1f' read -r ... <<<"${line//$'\t'/$'\x1f'}"`. Guarded by a comment
  at `MATRIX_FS` in `lib/matrix.sh`, and recorded in the memory corpus, because
  every generated results file in this rig is a TSV read the same way.

- **F-P05-8 — the harness is landed and its SCORER is falsifiable; the LOOP is
  not yet exercised end to end** (T3, 2026-08-02, `1408f1ae`). Both halves
  matter, and the second is the one to carry forward.

  | check | observed |
  |---|---|
  | `probe-c3.sh --positive-control` | 21 assertions, all hold — the scorer, the altitude computation, the derived outcome, and the planted guard |
  | falsifiability | **each clause mutated in a throwaway sibling and observed reddening its own check**: the planted guard removed; the `partial` clause collapsed into `fail`; `dissolution` made to accept a refusal; a token-less row made to accept a refusal anywhere; `n/a` counted as a hold; `partial` reported without the sole-red condition. Six mutations, six correct reds, zero mutants passing |
  | `rig c3` (no rows) | validates the spec, then **exits 2** naming all sixteen unwritten rows (D-P05-4) |
  | `rig c3 H99` / `SPIKE_C3_LEGS=bogus` | exit 2 on each — the vocabularies refuse before `rig_enter` provisions anything |
  | `audit-matrix.sh` | 14 invariants, green **unchanged** after the `matrix_read` lift — the behaviour-preservation gate on T2's file |
  | `rig selftest` | all assertions hold, 6.5s — A-1 still holds |
  | `shellcheck -x -S style`, `doctrine validate` | clean |

  **What is NOT proven.** No cell has run. `cell_run`, `c3_run_row`, the
  altitude stamping and the completeness count are exercised only by their unit
  checks; the first live cell is T4a's, and it is the integration proof for all
  of them. Expect the first `rig c3 H4` to find something in the wiring — the
  fixture joins (`fixture_slice`/`fixture_stub`) in particular are **asserted
  nowhere yet**: heavy is joined to slice **241** with a stub at
  `scripts/spike-capsule/capsule-stub.txt` (inside SL-241's own
  `scripts/spike-capsule/**` design-target selector) and light to **001** at
  `src/capsule-stub.ts`, and a wrong join makes every cell of that fixture
  refuse at conform leg 2 for a reason about the rig rather than about the
  model. Verify the join on the FIRST heavy cell, before reading anything else
  into a heavy refusal.

- **F-P05-10 — R-B is answered, and the heavy column is CHEAP. The fixture join
  holds** (T4a step 1, 2026-08-02, head `1199ba8a`). Measured with a throwaway
  scratchpad harness sourcing `pipeline.sh` directly — not a rig artefact, and
  the result rather than the script is what survives.

  | heavy cell | setup | capsule | run | TOTAL |
  |---|---|---|---|---|
  | happy path, `verify: true`, M-A | 0.40s | 0.83s | 1.3s | **~2.5s** |
  | happy path, `verify: true`, M-B | 0.40s | 0.69s | 1.47s | **2.55s** |
  | undeclared path (the T4a shape), M-A | 0.40s | 0.76s | 0.27s | **1.43s** |

  So a heavy cell is **~1.5–2.5s**, not the minutes R-B feared, and the 32 heavy
  cells are ≈**1 minute** of wall clock in aggregate. **R-B is resolved: the
  heavy column stays.** A-3's altitude vocabulary is not at risk and no scope
  `/consult` is owed on cost.

  Why the fear was wrong: `git clone --no-hardlinks` of the 169M fixture
  measures **0.2s**, not the tens of seconds a 169M byte-copy implies — the
  filesystem reflinks it. The three-clones-per-cell shape is simply not the cost
  driver anyone expected it to be.

  **The join is PROVEN, and it was the right thing to check first** (F-P05-8's
  open item). A heavy happy path passes **all four stages on both mechanisms**
  with `fixture_slice heavy` = **241** and `fixture_stub heavy` =
  `scripts/spike-capsule/capsule-stub.txt`; the heavy fixture's own
  `.doctrine/slice/241/slice-241.toml` carries `scripts/spike-capsule/**` as a
  **design-target** selector, so conform leg 2 folds clean. And the negative
  control is there too: the same fixture with the stub at `docs/heavy-stub.md`
  refuses at `conform/undeclared-path`. A heavy refusal can now be believed.

  **One latent join hazard, closed by luck rather than by design.** The heavy
  fixture repo sits at a **DETACHED HEAD** (`31563da55`), so
  `git symbolic-ref HEAD` FAILS on the fixture itself. `pipeline_setup` reads it
  off the **clone**, where git recovers the branch by matching the OID against
  `refs/heads/*` — `refs/heads/edge` is the sole match, so `accepted` resolves
  to `refs/heads/edge` and the pipeline works. It would have failed loudly had
  the fixture carried a second branch at that OID. Not repaired (S4-adjacent:
  the fixture is provisioned, and a rebuild is not free); recorded so the next
  reader does not spend the hour.

  Disk, and it is the tighter bound: the heavy **worker** capsule measures
  **195M against the 256 MiB `SPIKE_SANDBOX_DISK_CAP`** — ~24% headroom. A row
  planting more than ~60M into the capsule trips `harvest/resource-cap` for a
  reason about the FIXTURE, not the model. H7 plants deliberately oversized data
  and must therefore set its own cap rather than rely on the default.

- **F-P05-11 — S1 IS LIVE ON THE HEAVY COLUMN, and it fires as the mis-attributed
  token F-P04-10 predicted.** The heavy declaration's real `verify:` is
  `cargo test`. Observed end to end:

  | observed | value |
  |---|---|
  | heavy cell, real `verify: cargo test`, M-A | `verify/suite-failed` in **27.9s** |
  | the verify capsule's own sandbox status | **4** = `RIG_EXIT_DISK` — *not* 124, *not* a suite verdict |
  | verify capsule size, after provision → after `cargo test` | 169M → **2.5G** against a 256 MiB cap |
  | the run directory at its peak | **3.0G** |

  It is **not** a timeout (the 300s bound never fires; it dies at ~27s) and
  **not** an honest suite failure. `sandbox.sh`'s whole-tree `du` leg fires,
  returns `RIG_EXIT_DISK`, and `verify_stage`'s `case` has no arm for it — so it
  falls through `*)` to **`suite-failed`**. That is exactly OQ-a: the disk cap
  has no legal token at `verify` (`verify/resource-cap` is not in the closed
  set), and the row silently acquires a token that says the project's tests
  failed.

  **Not minted, not worked around, not scored** (S1). Recorded as a STATUS.

  **What it does NOT block: T4a.** H1–H5 refuse at `harvest` or `conform`,
  upstream of stage 3, so not one T4a cell reaches the verify capsule — which is
  why the measurement above runs clean. T4a proceeds.

  **What it DOES block, and the shape of the damage.** Every heavy cell that
  reaches stage 3 lands on this: H9's verify leg and H11 directly, and
  H10/H15/H16 transitively — they need verify to PASS to reach `advance`, and on
  heavy it cannot. Those rows would score `fail` against an `advance` boundary
  they never got to, and the table would read that as a defect of the capsule
  model at exactly the altitude the spike exists to claim. R4's direction.
  **This is the `/consult` S1 mandates, and it is owed before T4b.**

- **F-P05-9 — `rig`'s parser owns the flag space, so a probe-specific flag has
  to be an env var.** `rig:127` refuses any unrecognised `-*` before dispatch,
  so `rig c3 --positive-control` cannot reach the probe; the self-check is
  called directly (`control/probe-c3.sh --positive-control`) and the leg
  narrowing travels as `SPIKE_C3_LEGS`, which crosses the `exec` unchanged.
  This is why the self-check ALSO runs at the head of every ordinary run: the
  path that is hard to invoke is the path that stops being invoked.

