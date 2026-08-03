# Notes SL-241: Capsule spike rig

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-03 · **PHASE-04 complete (4/6), PHASE-05 in flight —
T0–T3 done, T4a DONE (five rows), T4b DONE (scored 330/0 + falsified 6/6),
**T4c DONE** (H13 110/0 + 4/4; H14 88/0 + 5/5; **H7 100/0 + falsified 6/6**),
T4d **DONE** (H8 scored 100/0 + falsified 3/3; **H11 scored 80/0 + falsified 9
cases/0 survivors** — F-P05-32 and F-P05-33 both DISPOSED, D-P05-14 and
D-P05-15), **T4e DONE** (H10/H16 scored **128/0**, 8/8 cells `model-level`, +
falsified 6/6 — M25 measures F-14 rather than arguing it).
**ALL SIXTEEN ROWS CARRY SCORED RESULTS: H1–H16. THE MATRIX IS COMPLETE** —
`T4` is done. **T5 DONE** (the conflict sub-probe — H10/H16's owed leg, scored
2/2 `counts-toward-nothing` + falsified 4/4; F-P05-40 and ISS-305 out of it).
**H10/H16 NOW OWN BOTH LEGS.** **T6 DONE** (the five guard probes — scored 9/9
`pass`, 71 assertions, 0 red, + falsified 4/4; F-P05-41/42/43 and D-P05-20..23
out of it). **EX-10, EX-11, VA-2 and VA-3 are discharged.** **T7 DONE** (six EVD
records — EVD-006..011 linked to QUE-200/QUE-201; **EX-13 and VA-5 discharged**;
F-P05-44/45 and ISS-306 out of it). **T8 DONE** (RFC-025 evidence artefacts —
**EX-14 discharged**; the scored results are now IN THE REPO; F-P05-46 out of
it). **T9 DONE — PHASE-05 IS `completed` (5/6 phases).** All fifteen EX and all
five VA criteria discharged; VT-1 flipped UNATTRIBUTABLE → PASS on the
completion, as F-P04-11 predicted. **F-P05-47 and ISS-307 out of the close
itself.** Next is **PHASE-06** (`go-no-go.md`).
· last SL-241 commit — verify before citing (`git log --oneline -1`)
(tree head moves under us — other agents commit to `edge` mid-session; verify a
hash before citing it)
(code ranges — PHASE-01 tip 29c7acf3; PHASE-02 b3ad3eed3..d041f6b39, none
foreign; PHASE-03 acc4b2b34..e14c22cc, one foreign interior; PHASE-04 step 0
b548f65d4..090148ef, **8 of 15 foreign**; PHASE-04 steps 1–4
090148ef..5f42727b, **2 foreign, both INTERIOR** so `record-delta --commit`
cannot excise them; **PHASE-05 `8e962656`..open, 162 commits and 111 FOREIGN
plus two interior merges** — by far the worst, now written up. Every range's
detail is in its own § PHASE-NN boundary — **do not read `start..end` as all
ours**.)

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

**PHASE-04 step 0 (complete; EX-1/2/3/9 + VA-1 discharged):**

- `.doctrine/slice/241/step0-enumeration.md` — the falsifier. 96 npm/TypeScript
  triggers over 11 surfaces (`beb4b665`, CPT-001 unread) + classification
  (`61ea9f08`, append-only). **The whole point is the two-commit gap.**
- ASM-007 → **`invalidated`**, both tiers (`e6dd2e29`)
- minted: **ASM-008** (responsibility split, the replacement claim) · **QUE-203**
  (downstream-interpreter classification, tracked-not-scheduled) — `9e2a7b5e`
- fixture declaration amended to **NO CHANGE with its reason** (`090148ef`)
- CPT-001 deliberately **NOT** amended — see D-P04-4
- RFC-011 friction recorded (`2da32870`); `doctrine validate` clean; `rig
  selftest` green post-change

**PHASE-05 in flight (T1, T2, T3; `3123e96c..1408f1ae`).** Detail —
decisions D-P05-1..4, findings F-P05-1..9, the cell plan and the STOP
conditions — is in the **runtime sheet**,
`.doctrine/state/slice/241/phases/phase-05.md`, which is the working document.
Lifted here so the ids survive the sheet:

- `control/fixture-light.sh --variant <base|inrepo|plan>` — F2 and F3 built
  (`7c134057`); `fixtures.md` F2/F3 amended NOT BUILT → BUILT. **F-P05-5** is
  the one T5 must ride: F3's phase completion is RUNTIME state and gitignored,
  so it does not survive a clone.
- `probes/c3/matrix.tsv` — the authored matrix, 16 rows × 2 fixtures × 2
  mechanisms = 64 cells, 4 `n/a` up front (`49fbce82`). Encoding in **D-P05-2**
  (sixteen ids; a `|` alternation carries a multi-part boundary, so
  `dissolution` stays distinct from empty). PHASE-05 VT-1 FAIL → UNATTRIBUTABLE.
- `lib/matrix.sh` + `control/audit-matrix.sh` — the spec's own gate, run before
  provisioning (**D-P05-3**); 8 positive controls each observed reddening its
  own check.
- `lib/rows.sh` — the row recorder lifted out of `probe-c2.sh` (`3ad74d83`);
  `rig c2` output **byte-identical** after the lift.
- `control/probe-c3.sh` — the matrix harness (`1408f1ae`), § 5.4's loop, reading
  the spec through `lib/matrix.sh` (which gained `matrix_read`, the single
  split). Carries S2's `partial`/`fail` distinction, EX-5's computed altitude
  (`n/a` excluded, a one-fixture hold printed as a divergence finding), and
  D-P05-2's alternations as **one leg per alternative** — one authored line, N
  scored entries. **D-P05-4**: an unwritten row is a usage refusal naming the
  task that provides it, never a recorded `n/a`, because `n/a` costs the row its
  altitude; `SPIKE_C3_LEGS=pipeline` is the sayable narrowing for T4e-before-T5
  and rides the results preamble.
- **F-P05-8** — the scorer is falsifiable: 21 self-check assertions, and six
  mutations each observed reddening its own check. **The LOOP is not yet
  exercised end to end** — no cell has run, and the fixture joins
  (`fixture_slice`/`fixture_stub`; heavy → slice 241 at
  `scripts/spike-capsule/capsule-stub.txt`) are asserted nowhere. T4a's first
  cell is the integration proof.
- memory: `mem.pattern.shell.ifs-tab-read-collapses-empty-fields` (**F-P05-7**)
  — a TSV read with `IFS=$'\t'` silently merges empty columns.
- `lib/instantiations.sh` — the row instantiations (D-P05-5). **T4a COMPLETE**
  (`ac9b534c` H1/H3/H4, `4b6ee92b` H2/H5): five result-tree rows, twenty cells,
  every one `pass`, every row **model-level** (**F-P05-24**). H2 is a
  *dissolution* performed rather than asserted (**D-P05-8**) and is the first row
  that must clear all four stages — so the first to pay heavy's ~6-min verify,
  which re-shapes T4b's cost (H9/H15 pay it too). H5's form set is per fixture
  (**D-P05-10**). Six mutants, six reds, no survivors.
- **T4b — the four DISSOLUTIONS written** (`efbb097a` H6/H9/H15; H12 follows).
  Each performs the hostile thing and shows the pipeline unbothered, which is
  H2's shape; each carries its own positive control, so a mutate that silently
  did nothing reds instead of stamping the table's strongest claim on a probe
  that never ran.
  - **H6** — hostile `.git/config` + executable hooks. Config and hooks are
    repo-local and never objects, so neither mechanism carries them. Its control
    is the CAPSULE firing its own `reference-transaction`: without it, a payload
    with a bad shebang would leave the trusted side silent for the wrong reason.
    `core.hooksPath` is planted ABSOLUTE on purpose — a repo-relative one copied
    into the quarantine would find nothing there and the observable would be
    vacuous. Also observes git's protected-configuration defence of
    `uploadpack.packObjectsHook`, which `upload-pack` would otherwise run as a
    child of the trusted side's own fetch.
  - **H9** — out-of-tree symlink + a `.gitattributes` filter driver, scored as
    two. The `dissolution` leg states I4 as a PAIRING (the payload is in the
    range the trusted side folded, and no candidate tree exists to follow it);
    the `verify` leg observes containment where the tree IS materialised.
  - **H12** — the evaluation surfaces, per **D-P05-11** planted at the slice's
    design-target directory: the matrix's root pair is unreachable on heavy
    (**F-P05-26**, and `/.envrc` is gitignored besides). Asserts `audit-dq4`'s
    EMITTED CLAIM, not its exit status — **F-P05-27**.
  - **H15** — the pipeline killed mid-run, synchronised on its own `stage=`
    emission through a FIFO (bash's `printf` flushes per call, measured) and
    SIGKILLed by process group so no sandbox is orphaned. A timed kill would be
    D-P05-8's racer. **D-P05-12**: stage 4 has no kill of its own because it has
    no interruptible interior — the row observes the consequence instead, a
    repeat advance refusing `stale-base`, i.e. the CAS applied exactly once.
  - shared vehicle grew `c3_execution_log` (a LOG, not a flag — these rows need
    WHERE a payload ran), `c3_design_target_dir` (H9 and H12 both need "somewhere
    leg 2 admits"; one definition, not two), `c3_path_absent` (`-e` is false for
    a dangling symlink and every escape here dangles by design).
- **D-P05-6..10** and **F-P05-10..25** are in the runtime sheet in full. Four
  are operator rulings from one `/consult` (D-P05-6 named sandbox statuses +
  `verify/resource-cap`; D-P05-7 heavy builds assets on site; D-P05-8 H2
  dissolved; D-P05-9 F1 gains `.doctrine/**`), plus **D-P05-10** — guards (b)
  and (c) become isolated T6 probes because leg 3 returns on the first matching
  path, so a combined range cannot observe either (**F-P05-22**).
- **F-P05-19** — the heavy column reaches `advance`: all four stages pass on both
  mechanisms, 376s / 409s, 4.4G. Unblocks H9/H10/H11/H15/H16.
- **F-P05-20** — EX-1's floor is FILESYSTEM-ONLY; both capsule kinds get the host
  network namespace and `:223` binds an API credential. Recorded, not fixed —
  exfiltration is a different axis from the ingestion safety P-C3 tests.
- **F-P05-25** — `c3_commit` tolerated a failed `git add`, surfacing as a
  `fatal:` inside a *green* log; now `rig_die`s. H4 shares it and stayed green.
- **T4c — H13 and H14 DONE; H7 alone remains** (`5c5ed437`, `e3e4f87d`,
  `6a9b7336`).
  - **H13** — the hostile bundle at the harvest path, and the ONLY row whose
    subject is a capsule-authored file. M-A is `n/a` **structurally** (it reads
    no such artifact), which is a QUE-200 verdict input in its own right (F-6).
    Four bundle-hygiene legs shared with `selftest_bundle` through
    `lib/common.sh` (`35431bc6`, selftest unchanged at 57); the size cap stayed
    out as a THRESHOLD, not a mutation. First row to refuse at **stage 1**, so
    `_assert` observes the ingestion boundary itself. Scored 110/0, both
    fixtures `model-level`; falsified 4/4, and **M6 is the one it exists to
    survive** — a zero-byte bundle satisfies an absence-shaped leg, so every leg
    pairs its observable with the honest bundle that stood there first.
  - **H14** — the doorbell duplicated, lost and forged. **Three legs in ONE
    cell**, which is the matrix as authored (one `harvest` stage, no token, so
    one alternative): the doorbell sits UPSTREAM of stage 1, so each leg
    observes at that seam through `rig_wait_doorbell` and the single run that
    follows is the JOIN — the bell is left **forged** for it and the trusted
    side still pins the capsule's own OID. Duplication is observed where it
    costs something, at a second harvest into a **fresh** quarantine, never the
    run's own (F-P05-28's rule). `pipeline_quarantine` / `pipeline_harvester`
    extracted rather than hand-rolled (`e3e4f87d`; selftest assertion list
    byte-identical). Scored 88/0, all four cells `model-level`; falsified 5/5
    with one control, and **M14 changed the row** — see F-P05-30.
- **T4d — H8 DONE; H11 remains** (`d1e67fb7`). Conform leg 4's two arms, one
  scored leg each, planted **separately**: leg 4 returns on the first matching
  entry, so a tree carrying both refuses once and the second alternative would
  score its sibling's refusal (F-P05-22 from a second direction). `_assert`
  MEASURES F-2 rather than asserting it — leg 2's own verb is run over the same
  range and observed to ACCEPT, so the kill is leg 4's alone; the hole was real
  because `reject_submodules` is index-scoped and unreachable from an
  object-only pipeline. Both forms sit under the design-target directory
  (D-P05-11), so `.gitmodules` is a NESTED one — the row does **not** show a
  root `.gitmodules` reachable under these selectors, and says so. Scored 100/0,
  both rows `model-level`; falsified 3/3 with 2 controls, and **M16 is the
  F-P05-14/18/26 trap made reproducible**: planted at an undeclared path the
  cell refuses `conform/undeclared-path` while `planted?` stays LIVE, so the
  positive control cannot see it and only the observed token can.

- **T4d — H11's light half BUILT AND FALSIFIED, NOT SCORED** (`aba9268f`,
  `f4c61fa4`). The first row whose subject is EXECUTION: the payload has to pass
  all four conform legs and then be RUN by the fixture's own `verify:` command.
  Light plants a `src/*.test.ts` that `node --test src/*.test.ts` executes in
  the verify capsule. **Filesystem escape CONTAINED** — the host path is not in
  the namespace at all, asserted with the payload's own `escape=denied` beside
  it so the absence is not an absence of anything tried. **Network canary
  REACHED, and deliberately unscored** (F-P05-32): `--unshare-all` then
  `--share-net` is the profile, `--kind` never reaches the mount array, and EX-2
  says both capsule kinds get the same posture — so "canary unreached" is an
  inherited expectation no mechanism makes, and asserting it either way would
  settle a design question from inside a test. **Heavy is refused at the plant**
  (F-P05-33): its payload would have to be both admitted by conform and executed
  by `just web-build && cargo test`, and those sets are disjoint under SL-241's
  selectors. Shakeout 12/0 on both mechanisms; falsified 5/5, no survivors, 2
  controls — **M22 is the row's reason for existing** (an inert payload leaves
  the containment clause passing anyway) and **M18 found a real defect**
  (F-P05-34). `audit-dq4` refused the first canary, which ran `node`
  trusted-side; the listener is `socat` now — guard kept, dependency dropped.
  **Not scored**: `rig c3 H11` runs the heavy cells too.
- **`expect_assert` — a third entry point in the falsifiability round**
  (`drivers/falsify-lib.sh`, runtime). H11's reds land inside `_assert`, which
  neither `expect_planted` nor `expect_refusal` reaches. It runs the full leg,
  then calls `_assert` with its verdicts captured and its failures rolled back
  out of the round's own count, and **the red count doubles as the isolation
  control** — "exactly one clause red, and it is this one" says in one line what
  a separate isolation function says in several.

**PHASE-04 steps 1–4 (complete; EX-4/5/6/7/8 + VA-1/2/3 discharged;
`090148ef..5f42727b`):**

- `control/probe-c1a.sh` (`rig c1a`) — the cost baseline, absolutes only,
  banner-stamped with rig state because `probes/c1a/results.tsv` is append-only
  **across the F-P04-7 fix boundary**; earlier rows must not be quoted
- `control/probe-c2.sh` (`rig c2`) — the confinement matrix, 7 rows, each with
  a named observable in a COLUMN and a positive control; outcome DERIVED from
  the assertions the row made
- `control/audit-dq4.sh` · `control/audit-nohooks.sh` — both with re-runnable
  `--positive-control`, both directions demonstrated
- `lib/sandbox-probe.sh` — the in-sandbox observation helpers EXTRACTED, not
  copied (D-P04-6); `declaration_field` moved `pipeline.sh` → `lib/common.sh`
- `capsule/provision.sh` pins the capsule git identity **on the clone** (F-P04-7)
- `ba9f4a8a3` — an operator ruling found UNCOMMITTED in the shared tree and
  preserved; SL-241 authored state, not PHASE-04 code (see § PHASE-04 boundary)

**PHASE-05 (in flight):**

- runtime sheet expanded — T0..T9, three STOP conditions live (S1 verify-disk
  token, S2 failed-row-is-a-consult, S3 the inherited DQ-4 condition)
- **T0 baseline green** — `rig selftest` all assertions hold in 6.6 s (incl.
  VA-3's falsifiable object-count leg re-observed); `audit-dq4` and
  `audit-nohooks` clean with the DQ-4 exemption asserted not-stale. So a
  "refused" is now distinguishable from "rig broken" and hostile rows may run

**Cross-phase amendment (2026-08-02, operator ruling) — OQ-1 v0 relocated:**

- Raw run logs move from the bespoke gitignored dir inside the authored tree
  (`.doctrine/rfc/025/evidence/raw/`) to the runtime tier at
  `.doctrine/state/rfc-025/raw/`. The `.gitignore` entry is reverted and the
  `.gitignore` design-target selector dropped from scope — this supersedes the
  two `.gitignore` bullets above (§ Produced, § PHASE-01).
- Trigger: the entry reddened `every_runtime_gitignore_glob_is_classified`
  (`src/worktree/mod.rs`), the parity test requiring every runtime-tier
  `.doctrine/` glob to be classified in `WITHHELD` / `DERIVED_RUNTIME`.
- Why not classify it: a one-off spike path in the *shipped* engine's hazard
  authority is POL-002 facet 2 — permanent library surface bought for transient
  project-local state. The state tier already carries the semantics wanted
  (gitignored, fork-withheld as `Tier::State`, disposable), so the fix is
  out-of-band and the test stays strict.
- Landed in: design § 5.3 + § 6 OQ-1 + § 9.1, `slice-241.md`, plan EX-7/EX-14.

- **T5 — THE CONFLICT SUB-PROBE DONE** (`616877a5`, `1b2d60a7`). `lib/conflict.sh`
  supplies the `conflict_subprobe` body `c3_run_conflict` has been calling since
  T2; H10/H16 now own BOTH legs, and their pipeline block stays stamped
  `legs=pipeline`. Scored 2/2, both `pass`, both `counts-toward-nothing`.
  - **The two rows PARTITION § 5.1's sequence** rather than each driving all of
    it. H10 refuses at `candidate create` (pair classified Conflicted, parked
    for `ingest`); H16 gets through create AND admit — **neither reads trunk** —
    and refuses at the integrate fast-forward CAS, which guides to supersede on
    the new base. So on this layer staleness is caught by the CAS and by nothing
    earlier: the incumbent's analogue of the ordering `pipeline.sh:560-566`
    holds for stage 4.
  - **F-9 is enforced in the FILE, not in prose.** `altitude` reads
    `counts-toward-nothing` in the column a reader would sum, and
    `expected-stage`/`expected-token` carry values deliberately outside
    `MATRIX_STAGES` / `token_legal`, so a tally folding these into the
    pipeline's vocabulary fails loudly instead of silently counting an incumbent
    refusal as coverage.
  - **F3 is COPIED, never cloned** — `prepare-review`'s phase-completion gate
    reads gitignored runtime state, so `phases: 1/1` is a property of the
    fixture DIRECTORY. EX-15 is made observable: the leg asserts the gate
    CLEARED rather than merely not dying.
  - Falsified 4/4 before scoring (`drivers/falsify-t5.sh`). **M34/M35 are a
    deliberate pair** — M34 deletes the hazard, M35 keeps it and moves it in
    time — so a leg asserting only "it moved and integrate refused" scores M35
    green. Two named seams (`conflict_peer_half`, `conflict_move_trunk`) exist
    because a mutant can only wrap what has a name.
  - **F-P05-40 / ISS-305** out of it: `candidate create` exits ZERO on a
    conflict — the refusal is LEDGERED, not status-borne — while integrate's
    staleness refusal is non-zero. The clap help asserts the opposite, and for a
    `review_surface` the arm it describes is unreachable. A QUE-202 input.

- **T6 — THE FIVE GUARD PROBES DONE** (`1de56f9c`, `fdebae1e`).
  `control/probe-guards.sh` + `rig guards [a…e]`; **EX-10, EX-11, VA-2, VA-3
  discharged**. Scored **9/9 `pass`, 71 assertions, 0 red** —
  `~/capsules/probes/guards/results.tsv`. Falsified 4/4 first
  (`drivers/falsify-t6.sh`, m36–m39). Order per F-P05-31 honoured throughout.
  - **Its OWN executable and its OWN results file** (D-P05-20). The guards have
    no `Hnn` trio, no fixture × mechanism cross-product and no altitude, so they
    are not matrix rows and must never be summed with them. `lib/conflict.sh`
    could sit on the matrix's axes and take a `counts-toward-nothing` altitude;
    these cannot, and a discriminator column a reader must *notice* is weaker
    than a separate file.
  - **`lib/fixtures.sh`** lifts the fixture table out of `probe-c3.sh` (plus the
    `light-inrepo` arm) so both probes share one set of joins — the `lib/rows.sh`
    move at T3, on the same second-real-caller bar. Behaviour-preserving.
  - **(a) is a CITATION, and it can fail** (D-P05-21). H8 *is* guard (a); the leg
    counts its `conform/gitlink` and `conform/gitmodules` entries out of the
    committed `results.tsv`, with a negative control on a token H8 never
    produces. A citation that cannot fail is prose.
  - **(b)/(c) isolation is ASSERTED, not assumed** — exactly one governance path
    in the range. Leg 3 returns on the first match, so without the count guard
    (b) is a differently-named re-run of H5 (F-P05-22). **m36 proves it**: the
    refusal and the ingestion stay green while the observation quietly becomes
    about another path.
  - **(c) both directions** — with `--find-renames` the `.doctrine/` source leg
    vanishes and the same capsule passes, so `--no-renames` is *shown*
    load-bearing rather than asserted.
  - **(d) needed a genuinely broken suite to observe anything** (F-P05-42) — and
    `audit-i4a` ran as the static half in both directions alongside it.
  - **(e) is three legs** (D-P05-22): F1 baseline · F2 worktree rewrite
    (byte-identical trusted-side — **QUE-201's evidence input**) · F2 committed
    rewrite. The trusted-side observable excludes the OIDs deliberately; those
    are fixture identity, not behaviour.
  - **F-P05-41 / F-P05-42 / F-P05-43** out of it, the first two now memories.
    F-P05-43 — the committed-rewrite leg's `conform/undeclared-path` is
    **fixture-specific** (F2 puts its declaration at the repo root, which SL-001
    declares no selector for) and is recorded as an observation, not generalised.

- **T7 — THE SIX EVD RECORDS DONE** (`0b9581b1`). **EX-13 and VA-5 discharged.**
  QUE-200's five verdict inputs plus QUE-201's one, derived from
  `probes/c3/results.tsv` and the PHASE-05 findings — no new probe run.
  - **EVD-006** `supports` QUE-200 — H6/H7 are **mechanism-neutral**: 8 cells,
    all `pass` at `model-level`. H6 dissolves on both because config and hooks
    are repo-local and both mechanisms transfer objects; the plant was live
    (a reference-transaction hook observed firing in the capsule). Carries
    F-P05-37/38's caveats — H7's cap is enforced *in-capsule*, and its observable
    is the refused write's signature, not its product.
  - **EVD-007** `supports` QUE-200 — every scored refusal token is produced by
    the **mechanism-independent** downstream pipeline and computed trusted-side
    from object identity. The mechanism decides the transfer, not the admission
    boundary. States the one defective line rather than smoothing it: M-B's
    `harvest/fsck-failed` is emitted at two sites with stderr discarded, and the
    harvester fscks the *whole* quarantine (F-P05-28).
  - **EVD-008** `supports` QUE-200 — forensic completeness, **scoped explicitly**:
    no row *scores* retention, so it opens by saying so. What the rows do show is
    that both mechanisms carry the worker's commit into quarantine as a
    first-class object (H1/H3's trusted-side-computed `S`, H15's clean
    `verify-pack`). Discriminates only against candidate 3, which was never
    rigged.
  - **EVD-009** `supports` QUE-200 — operational friction. Cost is **equal**
    (heavy cell: M-A 2.50s, M-B 2.55s), so cost is not the discriminator. The
    asymmetry is fragility: M-B's 27 MB transfer trips git auto-maintenance in
    the quarantine and produced a spurious refusal M-A did not. Bounded honestly
    at N=1 and fixture-caused — **not** generalised to "bundles are fragile".
  - **EVD-010 `disputes`** QUE-200 — the previously banked input, and the only
    `disputes` edge. H13's four legs exist on M-B only; on M-A the row is
    **structurally** `n/a`. It disputes a claim QUE-200's own body makes for
    candidate 2 ("cleanest trust story"), not the question — the bundle **moves**
    the trusted-side surface rather than removing it.
  - **EVD-011** `supports` QUE-201 — guard (e)'s three legs; byte-identical
    trusted-side behaviour. Carries F-P05-43 in full: the committed leg's
    `conform/undeclared-path` is fixture-specific and the generalisable claim is
    the byte-identity, not the token.
  - **VA-5 resolved differently than the sheet implied** (F-P05-44). `knowledge
    show`/`inspect` render **no inbound reciprocity**, so the Tier-1 edges are
    invisible from the question's side. "Both tiers" is satisfied on the
    *question's own* record: its `[evidence]` citation block (structured) plus an
    `## Evidence` section (prose), both verified via `show`. Filed as **ISS-306**
    — pre-existing, `src/` untouched (S4).
  - **F-P05-45** — the phase's actual result: **the mechanism axis barely
    matters.** Equal on config/hooks, resource bounds, forensic retention and
    cost; the one measured asymmetry runs against the bundle. And the gap that
    must ride with it — QUE-200's `upload-pack`-in-hostile-context worry was
    **never exercised** and stays argument, not evidence.

- **T8 — THE RFC-025 EVIDENCE ARTEFACTS DONE** (`d730ee50`). **EX-14
  discharged.** 28 files under `.doctrine/rfc/025/evidence/` — SL-241's declared
  design-target selector.
  - **THE SCORED RESULTS ARE NOW IN THE REPO.** `results-c3.tsv` (98 rows) and
    `results-guards.tsv` were the **only** copies and lived outside git under
    `~/capsules`. A wiped capsule root meant re-running every probe. That is
    over; this was the durability step, not a write-up.
  - **Three summaries, citational-plus** — `README.md` (the verdict, plus
    **seven things the evidence does NOT establish**, led by the untested
    `upload-pack` claim), `matrix.md` (the sixteen rows per-mechanism),
    `guards.md` (five guards + the conflict sub-probe).
  - **Raw logs → `.doctrine/state/rfc-025/raw/`** (80 files, 488K): runtime
    tier, gitignored by the existing `.doctrine/state/` entry, **no new
    `.gitignore` entry** — § 5.3 as amended, since the v0 entry broke
    `every_runtime_gitignore_glob_is_classified`. Verified with `git
    check-ignore -v`, not assumed.
  - **The phase sheets are archived, copied straight** (`phase-sheets/`, with a
    README framing them as frozen exhibits and naming what is authoritative
    instead). **Operator ruling**, and the reasoning is durable: a citational
    summary is only worth anything while its citations resolve, distilling
    spends context on reasoning that may never be read, and the loss is
    irreversible where the cost is not — cf. **F-P05-39**, which is this exact
    failure already paid for once.
  - **`drivers/` committed too** (T5's/T6's falsification + diagnostic vehicles)
    — beyond the brief, on that same F-P05-39 precedent. The copy is free; the
    loss is not.
  - **F-P05-46** — the falsification round is recorded **in-band**, so four
    `outcome=fail` rows in `results-c3.tsv` are four *successes*, discriminated
    only by a `MUTATED=` stamp on the **preceding preamble line**. A naive count
    of fails misreports a matrix that has none. Documented in both places a
    reader arrives from rather than repaired (rewriting scored evidence
    post-hoc is worse). **The generalisation: a discriminator a reader must
    *notice* is weaker than a separation they cannot miss** — the guards got the
    separation (D-P05-20), the sub-probe's mutants did not.

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
  DEC-107/DEC-108's claim (a known class has a TypeScript instance) corroborated
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

### PHASE-04 steps 1–4 — decisions (durable)

- **D-P04-5 — P-C1a keeps `provision` in the step list as a COMPOSITE of
  `clone`, with no number of its own.** probe-specs § P-C1 names six steps and
  this rig does not provide all six separably: step 2 is `.envrc` + `direnv
  allow`, neither binary exists in this jail, and the provisioning that remains
  (detach at B, capsule identity, remote strip) is the same runner invocation as
  the clone. The three options were to drop the step (a shorter list that looks
  complete), to repeat the clone's number under both names (a column a reader
  sums wrongly), or to carry it with its reason and no number. Took the third —
  it is the `n/a` matrix-cell discipline applied to a second kind of
  unmeasurability, and both rows are ASSERTED present rather than trusted.
- **D-P04-6 — the in-sandbox observation helpers were EXTRACTED, not copied**
  (`lib/sandbox-probe.sh`). `absent_inside` encodes a rule that was wrong in its
  first two writings (F-P02-1, then F-P03-2); a second copy would have carried
  its own version of that rule, and the two would have disagreed silently.
- **D-P04-7 — `audit-dq4` reports SITES under a stated predicate, not string
  hits.** See F-P04-9. Comments, quoted-only occurrences and sandbox-routed
  lines are not execution; everything else must be exempted WITH A REASON or the
  audit refuses. The blind spot (a trusted-side `sh -c '…'`) is closed by
  treating an unrouted `sh -c`/`eval` as a site, and is stated in the file.
- **D-P04-8 — `audit-dq4`'s scope is the trusted side: `control/`, `lib/`, and
  the `rig` entry point.** `capsule/` is excluded because those runners execute
  INSIDE a capsule by mount posture (I4a) — running the toolchain is their job.
  `fixtures/` is excluded because `package.json` naming `node --test` is the
  fixture being a TypeScript project, not the control plane running one. Note
  `lib/` is IN scope and EX-7's wording (`control/**`) is narrower than the
  trusted side actually is.
- **D-P04-9 — both audits' positive controls plant into a COPY of the rig under
  the capsule root, never into this repository.** A rig that wrote payloads into
  its own source tree, in a repo several agents share, would be a worse hazard
  than the one it audits. `audit-i4a`'s precedent (plant into a capsule) applied
  to an audit whose subject is the rig itself, via `--root`.

### PHASE-04 steps 1–4 — findings (durable)

- **F-P04-7 — the headline P-C1a number was a resolver timeout, not capsule
  cost.** First run: `clone` 7.74s, `harvest` 7.69s, on a 24-file fixture. Cause:
  `provision.sh` pinned the capsule identity AFTER the clone, so the clone's own
  reflog writes had no committer ident and git GUESSED one — which means
  resolving the hostname, and under `--unshare-all --share-net` that is a DNS
  query for an unshared UTS name that blocks until the resolver gives up.
  Measured in isolation: **3905ms ident unset, 40ms ident set, 35ms unset with
  `--no-net`**. Every capsule paid it twice (worker and verify). Fixed at the
  clone (`git clone -c user.name/-c user.email`, effective before the fetch and
  persisting into the new repo's config, asserted); re-measured **clone 0.047s,
  harvest 0.348s** — the accepted-phase round trip ~16s → ~1s. Two durable
  points: a capsule with network but no identity has a multi-second per-git-op
  tax that looks exactly like "capsules are slow"; and results files that are
  appended across a rig fix must be banner-stamped with rig state, which
  `probe-c1a` now is.
- **F-P04-8 — "no mount resolves under this repository" is FALSE, and the wrong
  assertion nearly shipped as an EX-8 witness.** The profile ro-binds the
  control-plane runners at `/rig`, and they live under
  `scripts/spike-capsule/capsule/`. That is I4a working as designed. The witness
  is now the EXACT SET (one repo-derived mount, carrying no `.git`), which states
  the admitted exception instead of hiding it behind an emptiness claim that
  reddened. Generalises: an EX-8-style "unrepresentable" witness written as
  *nothing crosses* is the shape most likely to be both wrong and green-looking.
- **F-P04-9 — DQ-4's literal form is unimplementable, and the exemption it
  forces is the real finding.** "The exec tokens must be ABSENT from
  `control/**`" (light declaration, EX-7) forbids the WORD: `pipeline.sh` must
  quote `node -e "process.exit(1)"` in a comment to explain why `verify:` runs
  through `sh -c`. Under a predicate that means *invocation*, exactly one site
  survives: **`fixture-light.sh` runs `npm` trusted-side at fixture build time**
  (3 sites). That is outside DQ-4's stated subject — capsule content and
  harvested trees — because the tree is assembled from this repo's own authored
  sources and no capsule has touched it. **The exemption is CONDITIONAL and
  PHASE-05 inherits the condition**: a payload-bearing fixture variant (H11
  `postinstall`) that reuses that build loop would execute the planted payload
  trusted-side and break DQ-4 for real. The audit prints the exemption and its
  site count every clean run and reds if it goes stale.
- **F-P04-10 — the OQ-a residue, located precisely (by inspection, not
  observed).** `verify_stage`'s status→token map has no arm for `RIG_EXIT_DISK`,
  so a verify capsule that overruns the disk cap is reported as
  `verify/suite-failed` — a wrong attribution, not a missing token. **No token
  was minted** (STOP condition S1 honoured): P-C2's resource row records
  STATUSES on the worker kind, as PHASE-02 does. This sharpens OQ-a from "there
  is no legal token" to "the mis-attribution is at `pipeline.sh` `verify_stage`,
  and it is one `case` arm".
- **F-P04-11 — `verify-vt` reports UNATTRIBUTABLE, not PASS, while the phase is
  `in_progress`.** Both PHASE-04 VTs read *"keyword present but <file> not
  modified by this slice"* with the files committed. Attribution reads the
  phase's conformance boundary, and `code_end_oid` is captured by the flip to
  `completed`. Expected to resolve at the flip; a future phase should not read
  UNATTRIBUTABLE mid-phase as a real failure.
- **F-P04-12 — `/tmp` inside the capsule is a tmpfs, so the write-floor row must
  not assert that the write failed.** It succeeds inside and is simply invisible
  outside. The observable is the HOST path holding no sentinel. Asserting on the
  write status would have scored that row wrong in both directions — the DQ-3
  trap in a place it is easy to miss, because the other two targets in the same
  row do fail the write.

### PHASE-04 steps 1–4 — evidence (what was OBSERVED, not merely coded)

| criterion | observation |
|---|---|
| EX-4 / VA-2 | `rig c1a` green. Absolutes only; header asserted to carry no before/incumbent/delta column. `nix env ready` present as `n/a` **with its reason**, and `provision` present as a composite **with its reason** — both asserted, not trusted |
| EX-5 / VA-3 | `rig c2` — all seven rows pass, each with a named observable in a COLUMN, each with a positive control. Row outcome is DERIVED from the assertions the row made, never supplied. Two legs recorded `n/a` with reasons (`~/.ssh`, hidden by the outer jail; `DOCTRINE_BIN` unset out here) |
| EX-6 | resource row recorded in its own right: status 124 for a 600s sleep under a 3s bound, `RIG_EXIT_DISK` for 64MiB under an 8MiB cap, both positive controls green |
| EX-7 | **both positive controls DEMONSTRATED**, not coded: each audit observed clean → refusing on a plant → clean again, on a copy of the rig. `audit-dq4` refused a planted `(cd … && npm test)` over a harvested tree; `audit-nohooks` refused a planted `worker_mode` branch |
| EX-8 | B1–B6 each carry a token leg **and** a positive structural witness. B2's witness reddened on first writing and was corrected (F-P04-8) — the census table is the observation, not the prose |
| VT-1 / VT-2 | keywords present; **UNATTRIBUTABLE while the phase is `in_progress`** (F-P04-11) |
| behaviour preservation | `rig selftest`, `probe-capsule`, `audit-i4a` re-run green after the `provision.sh` and `pipeline.sh` changes; rig-wide `shellcheck -x -S style` clean |

**R2 discharged, and it must not be read the other way.** PHASE-04's VT-1 and
VT-2 grep the audit scripts for keywords: they prove PRESENCE, never BEHAVIOUR.
This slice has three times found that evidence shape to be decoration
(F-P02-\*, F-P03-\*). **PHASE-05/06 must not read VT green as "the audits
work."** What carries that claim is EX-7's demonstrated positive controls, which
are VA-3's business and are re-runnable as `control/audit-dq4.sh
--positive-control` and `control/audit-nohooks.sh --positive-control`.

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

### PHASE-05 close (T9) — the gates, and the one thing the close itself found

**Gates, all observed rather than assumed:** `shellcheck -x -S style` exit 0
across `rig` and every `*.sh` outside `probes/` · `doctrine validate` clean ·
`./rig selftest` **all assertions hold** (not owed — no rig change since T6 —
but "still green" was an assertion, and this phase's discipline is that an
observation beats one) · `slice verify-vt 241` PHASE-05 VT-1 **UNATTRIBUTABLE →
PASS** on the flip, exactly as F-P04-11 predicted. **No `doctrine check`**: S4
held and no Rust appeared in PHASE-05 at all.

**F-P05-47 — the completion flip wrote another slice's commit as this phase's
`code_end_oid`.** `slice phase --status completed` captures `HEAD`, and `HEAD`
at that instant was `5c78c892` — `design(SL-244): DEC-121`, a different agent's
commit. PHASE-05's real code tip is `fdebae1e`; T7–T9 touched no source.
Corrected by hand in `boundaries.toml` (runtime tier). Filed as **ISS-307**.

Two things make this worth carrying rather than filing and forgetting:

1. **The warning does not have the shape of the defect.** It says the boundary
   "spans 12+ commits — any that are not this phase's are attributed to it",
   which describes *width*. Width is expected here and documented three sections
   below (111 foreign commits). It does **not** say the endpoint belongs to
   another slice. I nearly filed it as known.
2. **PHASE-01..04's `code_end_oid`s were captured the same way and are
   unverified.** Worth checking at audit before any of those ranges is believed.

The `record-delta` registry was correct throughout — it takes explicit
`--start`/`--end` and got `8e962656..fdebae1e`. **Two stores, one of which asks
and one of which guesses**; they disagreed, and the guessing one is the one audit
reads.

### PHASE-05 decisions (durable) — INDEX, because the sheet is archived

**Read this differently from §§ PHASE-01..04.** Those phases' sections restate
their decisions because the sheets that held them were discarded. **PHASE-05's
sheet is not discarded** — it is committed verbatim at
`.doctrine/rfc/025/evidence/phase-sheets/phase-05.md` (T8, operator ruling). So
this is an **index with the binding ones stated**, not a transcription. Chasing
any `D-P05-n` means grepping `## Decisions` in that file; the id is stable and
the reasoning is intact.

| id | decided |
|---|---|
| D-P05-1 | `matrix.tsv` is the **authored input** (VT-1's subject); `probes/c3/results.tsv` is the generated output |
| D-P05-2 | sixteen row ids; a `\|` alternation carries a multi-part boundary |
| D-P05-3 | the spec's validator lives beside its reader and runs before it |
| D-P05-4 | **an unwritten row is a REFUSAL, never a recorded `n/a`** |
| D-P05-5 | the rows live beside the harness, not inside it; the rig plays adversary trusted-side for convenience |
| D-P05-6 | every sandbox-injected status is named; `*)` means the command's own |
| D-P05-7 | the heavy capsule builds its assets on site |
| D-P05-8 | **H2 is re-derived in § 5.6 as DISSOLVED** (operator-ruled) |
| D-P05-9 | F1 gains a `.doctrine/**` design-target selector so H5 can reach leg 3 |
| D-P05-10 | guards (b) and (c) become their own T6 probes; H5 plants per fixture |
| D-P05-11 | H12 plants its evaluation surfaces under the slice's selectors |
| D-P05-12 | H15 interrupts three stages; stage 4's indivisibility is asserted |
| D-P05-13 | **a tool the rig cannot invoke is a defect of the rig** |
| D-P05-14 | **egress is allowlisted, not binary, and the allowlist is per capsule** |
| D-P05-15 | H11 scores at a different boundary per fixture, and that is correct |
| D-P05-16 | H7's disk bound is 20G (operator-ruled) |
| D-P05-17 | **the allowlist work lands in a follow-on slice, not here** |
| D-P05-18 | the 20G bounds the blast radius, not the cap H7 crosses |
| D-P05-19 | the capsule-time seam is a declarative per-cell lookup |
| D-P05-20 | **the guard probes get their own executable and their own results file** |
| D-P05-21 | guard (a) is a mechanical citation of H8, not a fifth script |
| D-P05-22 | guard (e) is three legs, and the baseline is one of them |
| D-P05-23 | `fx_case` lifted to `falsify-lib.sh` |

**The four that bind work outside this phase:**

- **D-P05-17 — the allowlist work is a FOLLOW-ON SLICE.** D-P05-14 established
  that egress must be allowlisted per capsule rather than on/off. Implementing
  that is explicitly *not* SL-241's, and the follow-on has not been sliced. This
  is the largest piece of forward work the phase created.
- **D-P05-8 — H2 is DISSOLVED in § 5.6**, operator-ruled. A future reader who
  finds H2 scored as a dissolution and reaches for "the rig didn't try" has it
  backwards; F-P05-13 carries why no deterministic instantiation exists on M-A.
- **D-P05-4 — an unwritten row is a REFUSAL.** The rule that kept the matrix
  honest, and the one most likely to be quietly relaxed by a future probe under
  time pressure. Its sibling is R-C: every `n/a` names a *structural* absence.
- **D-P05-20 — separate executable, separate results file** for anything that is
  not a matrix row. F-P05-46 is what it costs when a discriminator rides a
  preamble instead of a separation.

**DEC records are owed at close for D-P05-6..23.** Sheet convention is to lift at
close and they were deliberately not minted mid-phase. That obligation now falls
to `/reconcile` or `/close`, not to PHASE-05.

### PHASE-05 findings (durable) — INDEX, same reason

Forty-six findings, F-P05-1..46, all intact in the archived sheet under
`## Findings` (**newest first**). Indexed here by what they establish; expanded
only where the lesson generalises past SL-241.

| ids | about |
|---|---|
| F-P05-1, 2, 4, 6, 8, 12 | baseline, fixtures, the matrix gate, the harness landing — the build-out |
| F-P05-3, 27 | `audit-dq4` is coupled to the light declaration, and its exemption-staleness check with it |
| F-P05-5 | F3's phase completion is runtime state and does not survive a rebuild |
| F-P05-7 | **`IFS=$'\t' read` silently collapses empty TSV fields** |
| F-P05-9 | `rig`'s parser owns the flag space; a probe-specific flag goes to the probe |
| F-P05-10, 19 | the heavy column is cheap and reaches `advance`; R-B answered |
| F-P05-11, 15 | S1 was four causes behind one token |
| F-P05-13, 14, 18, 21, 22, 23 | H2's and H5's instantiation constraints — the fixture-specific ones |
| F-P05-16, 17 | a fixture's base B must be a commit where `verify:` passes; the visibility floor |
| F-P05-20 | EX-1's floor is filesystem-only; the capsule's network is not |
| F-P05-24, 25 | T4a complete; a failed `git add` was survivable and surfaced as `fatal:` in a green log |
| F-P05-26, 33, 34, 35 | H12's and H11's re-homing, and where class-2 triggers live |
| F-P05-28, 29 | the H15/heavy/bundle red — fixture commit-graph, not the capsule model; and the shakeout green that is not evidence about the harness |
| F-P05-30, 31, 38, 41, 42 | **the falsification discipline** — a clause that cannot fail is not a control |
| F-P05-32 | the verify capsule shares the network namespace by construction |
| F-P05-36 | `model-level` is earned by two different boundaries and the table cannot say so |
| F-P05-37 | the trusted-side plant seam cannot reach a hazard the sandbox evaluates |
| F-P05-39 | **T4a–T4e's falsification drivers were never tracked and are gone** |
| F-P05-40, 43 | two tokens that must be stated, not smoothed — ISS-305, and guard (e)'s fixture-specific refusal |
| F-P05-44 | `knowledge show`/`inspect` render no inbound reciprocity — ISS-306 |
| F-P05-45 | **the phase's actual result** — the mechanism axis barely matters |
| F-P05-46 | four `fail` rows in the scored table are four successes |

**The five that generalise past this slice:**

- **F-P05-39 — losing the vehicles loses re-runnability even when the results
  survive.** T4a–T4e's drivers were never git-tracked. The scored data is whole,
  so the evidence stands; what is gone is the ability to re-run those mutants.
  **Do not reconstruct them from prose and re-run them under the old claims'
  names** — an inferred contract's green is new evidence wearing an old label.
  This finding is why T8 committed `drivers/` and the phase sheets.
- **F-P05-38 / F-P05-41 / F-P05-42 — a clause that cannot fail is not a
  control.** Paid for three times in one phase: an assert that counted files over
  a bound where none can exist; "refuses nowhere" satisfied by a run that skipped
  a stage; a guard asserting an unaffected verdict against a suite that passes
  anyway. Every assertion needs a positive control that reds it.
- **F-P05-28 — an assertion that writes to its own subject** is invisible while
  everything passes and produces its most misleading output on the first real
  failure, exactly when it is trusted most. One honest refusal presented as seven
  reds, two looking like a canonical-safety failure — manufactured by the
  observer.
- **F-P05-31 — a scored run started before the row was final is VOID.** Not
  "probably still fine". The ordering discipline held for the rest of the phase.
- **F-P05-46 — a discriminator a reader must *notice* is weaker than a
  separation they cannot miss.** The guards got their own file (D-P05-20); the
  sub-probe's mutants ride a `MUTATED=` preamble stamp, and a naive count of
  `outcome=fail` misreports a matrix that has none.

### PHASE-05 evidence (what was OBSERVED, not merely coded)

- **All sixteen hazard rows carry scored results; every scored cell is `pass`.**
  Two `n/a` cells (H13/M-A, H12/light) are structural, and one altitude is capped
  (`unproven-beyond-rust`, H12). `~/capsules/probes/c3/results.tsv` — **now
  committed** at `.doctrine/rfc/025/evidence/results-c3.tsv`.
- **The five guard probes each OBSERVED refusing at least once** — 9/9 `pass`,
  71 assertions, 0 red, falsified 4/4 (EX-10, VA-2). A guard never seen to fire
  is not known to work.
- **Guard (e): trusted-side behaviour is BYTE-IDENTICAL** between a capsule that
  rewrote its own in-repo declaration and one that had nothing to rewrite (EX-11,
  VA-3) — QUE-201's only evidence input, now **EVD-011**.
- **The conflict sub-probe scored 2/2** at `counts-toward-nothing` (VA-4),
  falsified 4/4, against the **real** candidate layer.
- **Six EVD records carry the verdict inputs** — EVD-006..011, linked to QUE-200
  and QUE-201 (EX-13, VA-5). Four `supports`, one `disputes`, one for QUE-201.
- **Cost measured, not estimated**: a heavy cell is ~1.5–2.5s (M-A 2.50s / M-B
  2.55s), because `git clone --no-hardlinks` of a 169 MB fixture reflinks in
  0.2s. R-B is answered and the heavy column stays.
- **What was NOT observed, and must ride with all of the above**: QUE-200's
  `upload-pack`-in-hostile-context worry was never exercised. H6 shows the
  hazard class does not reach the parent under either mechanism — weaker and
  different. **"Fetch is proven safe against hostile config" is not what this
  matrix says.**

### PHASE-06 — IN FLIGHT, lifted early because the phase found something

Not the close-time lift (PHASE-06 is `in_progress`). These two survive the sheet
because they bind work outside SL-241 and because the second one is the kind of
result that gets quietly smoothed away by a later green.

- **P-C1b's first scored run reached green and the phase never executed.** The
  agent had **no shell at all**: the profile ro-binds the agent home
  (`sandbox.sh:225-228`, `HOME=/agent`) and the harness must create a
  per-session working directory under `$HOME` before any Bash tool call runs, so
  every shell path died on `EROFS … mkdir '/agent/.claude/session-env/<id>'`. It
  wrote `src/split.ts` by reading the tests and reasoning through the cases by
  hand, was **correct**, and the suite passed when the probe ran it trusted-side
  afterwards. `agent-committed=no tree-dirty=yes` — the worker's residue sweep is
  the only reason there was anything to harvest. **EX-1's assertion passed for a
  reason unrelated to EX-1.** Recording the ritual separately from the commit is
  what kept this visible; folded together it would have read as a clean green.
- **The A2 smoke certified a profile that cannot carry the workload** — the
  generalisable one. `probe-smoke.sh` proved *credential* and *egress* with
  `claude -p 'print OK'`, a prompt needing **no tools**, and both legs are green
  in `probes/smoke/results.tsv`. It never proved the agent can **work**. The
  smoke already split credential from network on exactly this reasoning (A8:
  "distinct failure modes and a single test conflates them"); the same argument
  extends one step further, to *can it work*, and was not taken. **A dependency
  smoke that exercises the happy path without exercising the capability the real
  workload needs will certify a configuration that cannot carry it, and the
  failure surfaces at the most expensive possible moment.** A third A2 leg is the
  cheap fix and belongs in the rig regardless of P-C1b's fate.
- **Operator ruling (2026-08-03): the read-only agent home is a SETUP OVERSIGHT,
  not a weakening.** The profile gains a writable `$HOME` (`--tmpfs
  /agent/.claude`, credential ro-bound inside) and P-C1b re-runs; the first
  attempt is disclosed with its usage, never discarded.
  `--dangerously-skip-permissions` inside the worker is endorsed — **the capsule
  model's claim is that the OS boundary IS the boundary**, and a harness inside
  it need not re-litigate confinement per tool.

**The fix landed, and the re-run is a real phase** (`e4aac673`, `1a1383b0`).
Three more that survive the sheet:

- **The capability leg was written FIRST and shown RED.** `probe-smoke.sh` gains
  a third assertion: the agent runs `uname -r > /capsule/exec-proof` and the
  assertion is trusted-side on the file's **content**, never on the agent's
  account of itself (I5). The kernel release specifically — a leg a
  file-writing tool could satisfy would pass in exactly the case that produced
  this whole detour. Red against the old profile (the agent's own reply named
  the fix), green against the new one, same leg, one profile change between.
  **That ordering is the point**: a capability leg added after the fix would
  prove only that the fix works, never that the leg can detect its absence.
- **Re-running P-C2 answered the handover's open question with a RED, and the
  prediction was wrong.** The handover reasoned the tmpfs "touches none of
  P-C2's confinement subjects". Right about the *subjects*, wrong about the
  *observables*: `api-cred` claimed *"the capsule cannot rewrite the
  credential"* but asserted a write to a **different file in the credential's
  directory**. Those coincided only while the whole of `~/.claude` was ro-bound.
  **The proxy was never sound in the direction that mattered** — a read-only
  directory with the secret bind-mounted rw over it passes the old leg while the
  secret is writable. Operator ruled (a): realign the observable onto the file
  (refused on append, truncate **and unlink**, credential shown readable first,
  against a **positive control writing successfully beside it**, because the
  home is writable by design and a refusal proves nothing against a broken write
  mechanism). All seven rows pass; P-C2's results are **re-taken, not carried
  with a caveat**. Generalised into
  `mem.pattern.review.guard-test-asserts-property-not-proxy`. **The rule this
  pays for: a probe whose subject is the mount posture is re-run when the mount
  posture changes, not reasoned about.**
- **A degraded agent is not a cheap one — it cost roughly DOUBLE.** The void
  attempt burned 8 131 output tokens / 388 449 cache reads / \$0.657 across 18
  turns of failing tool calls and 142 s, for a phase it never executed. The run
  that actually did the work: 3 415 / 195 816 / **\$0.335** in 51 s, with
  `agent-committed=yes tree-dirty=no` and the agent's own commit
  (`9872a712`, `src/split.ts` +25, `Co-Authored-By` trailer — not the worker's
  residue-sweep wording). Worth carrying into the go/no-go, because it points
  against the intuition that a blocked agent gives up early. And note the
  circularity that closes the loop: **`git commit` IS shell execution**, so the
  fact discharging EX-1's ritual clause is the same fact the new smoke leg
  tests for — at a fraction of the cost.

### PHASE-05 boundary — 111 FOREIGN COMMITS, and two interior merges

Written 2026-08-03, mid-phase and deliberately so: this range is the most
contended in the slice by an order of magnitude, and it is the kind of thing
that cannot be reconstructed at close. **The range is still OPEN** — PHASE-05 is
`in_progress`; the end below is HEAD at writing, not the phase's end.

**The start is bound, and it is not in `boundaries.toml`.**
`code_start_oid = 8e962656` lives in the runtime `phases/phase-05.toml`;
`boundaries.toml` only receives a phase's row when the completion flip captures
`code_end_oid` and upserts it. So the four rows there are the four completed
phases and PHASE-05's absence is correct, not a missed binding. A reader who
checks only `boundaries.toml` will conclude the opposite — which is why this
paragraph exists.

Range `8e962656..HEAD`. **162 commits, 51 mine, 111 FOREIGN.** For scale,
PHASE-04's most contended portion was 15 commits with 8 foreign. The foreign
work is not noise from one neighbour — it is a dozen concurrent efforts:

| scope | commits | scope | commits |
|---|---|---|---|
| SL-233 | 36 | CHR-051 | 5 |
| SL-243 | 16 | SL-244 | 3 |
| CHR-049 | 16 | IMP-104 | 3 |
| RFC-027 | 6 | *(12 further scopes)* | 1–2 each |

**Two INTERIOR MERGES**, which is new for this slice and matters to the
mechanism rather than only to the reading: `e2c2f4ab` (`Merge branch 'main' into
edge`) and `a45ce30b` (`candidate(233/close-001)`). `record-delta`'s guard
requires a **non-merge end** — satisfied, since both are interior — but a range
containing merges is not a linear patch series, so `--start/--end` over it folds
in everything those merges brought. The phase's own code tip is what should be
recorded, and `--commit <S>` per commit is the safe default the CLI already
prefers.

**Do not read `slice conformance 241` as this slice's footprint.** The same
warning PHASE-04's section gives applies here with 111 foreign commits behind
it rather than 8.

Operating rule inherited from PHASE-04 and vindicated again: **every commit in
this phase was path-limited**, and the tree was shared throughout with other
agents' uncommitted work sitting in it.

### PHASE-04 boundary — EIGHT foreign commits, and a hostile shared tree

Range `b548f65d4..090148ef` (step 0 portion). **15 commits, 7 mine, 8 foreign** —
SL-233 ×3, SL-242 ×3, IMP-315, plus two unscoped (`chore: skills`, `doctrine`).
The most contended range in the slice; `record-delta` at phase close, do not
assume `start..end` is all ours.

**Steps 1–4 portion: `090148ef..5f42727b`, 10 commits, 8 mine, 2 foreign** —
`9d4f18959 chore: v0.34.4` and `a3c5ec0f2 obs(SL-233)`. Both are INTERIOR, not
trailing, so `record-delta --commit <tip>` cannot excise them: the phase's own
code tip *is* the range tip (`5f42727b`). Recorded here instead, which is what
the earlier boundary sections do. `doctrine slice phase --status completed`
emitted its 12+-commit warning on exactly this.

One of the eight is not phase work in the ordinary sense: **`ba9f4a8a3` was
found UNCOMMITTED in the shared tree at the start of this session** — a
coherent, self-documenting operator ruling (OQ-1's raw logs to the runtime
tier, dated 2026-08-02) touching design.md, plan.toml, slice-241.md/.toml,
notes.md and `.gitignore`. Committed rather than left exposed, given this
range's two recorded destructive incidents (F-P04-6). It is SL-241 authored
state, not PHASE-04 code, and `/reconcile` should treat it as such.

**Whole-slice conformance is heavily commingled** — `doctrine slice conformance
241` reports 65 undeclared paths, most of them other agents' (backlog IMP-378,
ISS-290, ISS-291, `.claude-plugin/`, `plugins/`). Step 0's own ASM-008 and
QUE-203 also read undeclared. Do not read that count as this slice's footprint;
it is an audit-time reconciliation input, not a finding.

Beyond commit interleaving, the tree was **actively destructive** this phase
(F-P04-6). Two separate incidents, both from other agents, both mid-step:

1. `git reset --hard` discarded uncommitted ASM-007 edits;
2. `git stash` / `git stash pop` wrote **conflict markers into ASM-007's `.md`**.

Both recovered with nothing lost, but the operating rule changed as a result:
**commit each coherent unit the moment it is coherent.** AGENTS.md's "do not
stash / never discard uncommitted work" is a rule *other* agents can break on
your behalf, and the working tree is not safe storage between two steps of your
own task.

Left alone deliberately, for their owners to resolve: `stash@{0}` (8 entries in
`git stash list`), the conflicted `.doctrine/slice/242/slice-242.md`, and staged
`Cargo.toml` / `Cargo.lock` / `plugins/**` / `.claude-plugin/**` sitting in the
shared index. **A pathless `git commit` by anyone sweeps those up** — every
commit in this phase was path-limited.

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
- mem.pattern.sandbox.git-ident-unset-dns-stall — F-P04-7 made reusable
- mem.pattern.evidence.witness-the-exact-set-not-the-emptiness — F-P04-8 ditto
- mem.pattern.tests.assert-the-refused-writes-signature — F-P05-38 made reusable
- mem.pattern.handover.cite-artifacts-by-tier-not-just-path — F-P05-39 ditto
- mem.pattern.tests.guard-needs-a-discriminating-difference — F-P05-42 ditto
- mem.pattern.tests.baseline-needs-its-own-positive-control — F-P05-41 ditto
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
- **an absence-shaped result is not a verdict** — three instances in one session,
  all in throwaway harnesses rather than in the rig: a run piped to `tail`
  reporting `tail`'s exit status (a timeout kill read as a pass); a hand-rolled
  leg-3 emulation whose erroring command emitted nothing, read as "guard
  evaded"; a mutant driver scoring SURVIVED where zero cells had run. Every
  absence-shaped verdict needs a control proving the measurement ran. Committed
  as friction observations (`d8e59952`); generalises
  `mem_019fa18161f47651af7687d8dccbbc67`. **Candidate memory.**
- mem.fact.tooling.x-bit-is-not-runnability — F-P05-29 made reusable
- mem.fact.tooling.shellcheck-source-dev-null-hides-cross-file (`bd3aa0d9`)
- mem.fact.git.clone-inherits-stale-commit-graph — F-P05-28 made reusable
- **a falsifiability round need not pay the row's own cost** — `planted?` is
  evaluated BEFORE the pipeline leg, and the harness's own positive control
  already proves a forced-false `planted?` reds the cell, so the two halves of
  the composition are established at opposite ends and a `planted?` mutant needs
  no verify leg. T4b's round ran in seconds against a ~75-min row sweep. Only a
  mutant whose red is a *refusal* needs a pipeline. **Candidate memory.**
- **a mutation must WRAP the real function, never restate it** — a hand-copied
  function body in the mutant driver is the same
  vehicle-differs-from-the-real-one defect as F-P05-25 and F-P05-29, arriving
  from a third direction. `rebind` + wrapper keeps the driver honest.
- **an isolation control is what turns "it red" into "it red for this reason"** —
  each mutant asserts the clauses it did NOT target still hold. Without it a
  mutant that breaks everything proves nothing about the clause it was written
  for. Both rounds carry one per mutant.
- **an observable of a REPEAT must count the repeat** (F-P05-30) — H14's
  duplicate-ring leg observed that the waiter answered the same twice and the
  published ref had not moved. Both hold when there is NO second ring: sameness
  is exactly what a single occurrence also produces, so the mutant that no-op'd
  the duplication survived until `rings` / `rings-distinct` were added. When the
  hostile input is *that something happened again*, the positive control has to
  be the occurrence count. Same family as the absence-shaped observable, one
  step over. **Candidate memory.**
- **an isolation control must assert the state the mutation actually LEAVES** —
  M12's first control claimed the honest ring was intact at the bell; in fact
  the leg that ran before it had destroyed the bell and the no-op'd forgery
  never rewrote it. A no-op composed with the legs preceding it does not restore
  the un-mutated state. A control that reds is about the control until shown
  otherwise.
- **shakeout → falsify → THEN score** (F-P05-31) — the falsifiability round
  CHANGES the row, and `probe-c3.sh` sources `lib/instantiations.sh` once at
  process start, so a scored run launched before the round is void by
  F-P05-29's mechanism with the co-agent replaced by oneself. Cheap consolation:
  `rows_write` writes the whole file at the end, so a killed run leaves
  `results.tsv` untouched.
- **a falsifiability sibling isolates the rig's CODE but not its RESULTS** —
  `SPIKE_CAPSULE_ROOT` still defaults to `~/capsules` in the copy, so mutant
  cells append rows indistinguishable from real ones. Point it at a scratch root
  **outside the sibling repo** (inside, I6 correctly refuses) with `fixtures`
  symlinked in.
- **a `${x:-DEFAULT}` is only inert if the default is inert IN THE COMMAND BELOW
  IT** (F-P05-34) — H11's arming check used `kill -0 "${PID:-0}"` to keep
  `set -u` quiet, and `kill -0 0` addresses the caller's own process group and
  SUCCEEDS. The guard passed for exactly the state it existed to catch, and the
  mutant that no-op'd the arming survived its first sweep. Every `:-` wants the
  question *what does this value mean to the command that consumes it?*
  **Candidate memory.**
- **the audit that fires on your own new code is the audit working** — the first
  H11 canary was six lines of `node`, trusted-side, and `audit-dq4` refused it
  because `node` is a light `exec:` token. The listener read no project content,
  which is precisely the argument an exemption would have been built on. Drop
  the dependency, keep the bright line: `socat` is in neither fixture's list.
  F-P05-3 predicted this audit would catch a payload-bearing row reaching for
  the project's toolchain, and it did, first run, against the named row.
- **a clause a mechanism does not make is not the row's to assert** — H11's
  matrix observable asks for `canary unreached`; the profile shares the network
  namespace by EX-2's construction. Asserting `unreached` reds a profile
  behaving as designed; asserting `reached` enshrines egress as intended. The
  row MEASURES it, names the finding, and refuses to launder either into a
  verdict (F-P05-32). Scoring a design question from inside a test is the
  failure mode; `--no-net` would have greened the row in one line and retired
  EX-2's uniformity claim silently.
- **two rows scoring the SAME token is a result, not a redundancy** (T4e) —
  H10's genuinely conflicting pair and H16's trivially mergeable trunk advance
  both refuse `advance/stale-base`, because stage 4 compares two OIDs and never
  reads a tree. The instinct is to collapse them or to differentiate the
  expectation; both would destroy the finding. § 5.1's *safety yes, resolution
  no* IS that indistinguishability, so what the token cannot carry the rows
  carry in `_planted` / `_assert` instead. When two cells agree, ask whether the
  agreement is the measurement before treating it as duplication.
- **a mutant can INVERT an ordering by wrapping, without restating the body** —
  T4e's M25 had to test stage 4 transferring before its precondition. Rewriting
  `advance_stage` inverted would have measured the copy. Instead the wrapper
  performs the transfer itself and then calls through, so the real precondition
  runs against the forbidden state. Composition reaches orderings that
  substitution would have to fork the subject to reach. **Candidate memory.**
- **the isolation control is where a falsification round earns its keep** — M25
  reds only the object-count clause; the refusal token, canonical's REFS, and
  all seven of the row's own clauses hold. Stating that is what turns "the
  mutant died" into the finding that a refs-only `stale-base` clause would have
  scored an inverted stage 4 GREEN. The red proves the row works; the control
  proves *which* part of the design is load-bearing.
- **a rule with a genuine exception should be SCOPED in place, not silently
  broken** (T4e) — `lib/instantiations.sh` forbade rows touching `canonical`,
  and H10/H16 cannot exist without it. Editing the row to sneak past, or
  deleting the rule, both lose the reason it was written. The preamble now names
  the carve-out, says why the protected property survives (both re-snapshot),
  and confirms `quarantine` is still untouched by every row.

### Open

- **The agent home's write scope is coarser than it wants to be — FOLLOW-UP,
  deliberately not a backlog item yet.** Operator, 2026-08-03, on seeing the
  posture PHASE-06 landed: *"probably some things to tune in future here but
  nothing showstopping … not in this slice."* Deliberately **not** lodged via
  `backlog new`: the whole capsule model is pre-go/no-go, and a standalone
  improvement would presume it ships. It rides the slice until the verdict
  decides whether there is anything to improve.
  - **What shipped (D-P06-5):** a blanket `--tmpfs /agent/.claude`, empty, plus
    the credential and `~/.claude.json` ro-bound in. Writable-but-blank, and
    everything the harness writes there dies with the capsule unobserved.
  - **Where it wants to go**, all three worth carrying into the go/no-go's
    outstanding-work section rather than losing:
    1. **Scope the writes**, rather than granting the whole directory — the
       session folder and `~/.claude.json`, not a blanket tmpfs. Note the
       tension already recorded: `session-env` is an **undocumented harness
       internal** that will move (D-P06-8 rejected pinning to it for exactly
       this reason), so scoping needs a stable seam or a harness-version canary,
       not a hardcoded path.
    2. **Populate from fixtures in a known state** — `~/.claude.json`
       especially. Today it is the control plane's live 90 KB file, ro-bound;
       a fixture makes the capsule's starting state declared and reproducible
       instead of inherited, which is the same argument the pinned fixtures
       already won elsewhere in this rig.
    3. **Salvage it for forensics, attached to the run.** A tmpfs that dies with
       the capsule discards exactly the evidence that would have diagnosed
       F-P06-7 in seconds — the harness's own session state. Harvesting it to
       the run directory alongside `worker-stdout` is the shape.
  - **Constraint any design must keep:** the refusal is `EROFS` from the **mount
    flag**, not `EACCES` from mode bits. The capsule runs as `uid=1000` and
    *owns* `.credentials.json` (`-rw-------`), so permission bits would not stop
    it — only the read-only mount does, and `--unshare-all` denies the
    `CAP_SYS_ADMIN` needed to remount. A narrower scheme that reintroduced the
    secret on a writable mount would be a real weakening however tight its mode
    bits looked.
  - **Live edge, not yet bitten:** `~/.claude.json` is ro and the harness
    normally writes it (cost, project history). Non-fatal in the scored run. If
    a later harness version needs to persist there it will present as a partial
    failure, not an error — and `probe-smoke.sh`'s capability leg is what catches
    it. Argument for that leg being permanent rather than a PHASE-06 one-off.
- **QUE-204** — how a capsule obtains build inputs git cannot carry. The one open
  decision PHASE-05 carries and **not a rig edit**: heavy's web assets built on
  site per cell (current, measured poor — every stage-3 cell fetches from
  `registry.npmjs.org`, so an outage lands as `verify/suite-failed` with nothing
  to distinguish it from a real verdict) vs at fixture-build time from B's own
  source. D-P05-7's *reason* survives either way; only its timing is open.
  Operator: egress acceptable **if allowlisted**, but that is per-project toil —
  and the shared-capsule-at-a-known-HEAD direction is the real target. Shares a
  lever with F-P05-20 option 4; settle together if either is settled.
  **PLACED 2026-08-03 (D-P05-17): a FOLLOW-ON SLICE, settled together with
  QUE-204**, the two being one lever. PHASE-05 builds none of it.
- **~~OQ-a~~ RESOLVED** (`c4a26004`, D-P05-6) — every sandbox-injected status is
  now named above `*)`, and `verify/resource-cap` joined the closed set.
  **OQ-b / OQ-c still live** (§ PHASE-03 open questions): worker wall-clock
  overrun is H15's business; M-A has no absent-result token
- **ISS-296** — any local clone of this repo inherits the stale commit-graph
  landmine (F-P05-28). Carried OUT of the slice; `git gc` on the primary was
  assessed and deliberately rejected.
- **~~T4c / H7~~ SETTLED AND SCORED 2026-08-03 (`6312f519`) — T4c is DONE and
  the matrix is COMPLETE.** `rig c3 H7` exit 0, 100 assertions, 0 FAIL, 4/4
  cells `model-level`, 4s; falsified 6/6, no survivors. `worker-hostile.sh`
  written. Both open decisions ruled:
  - **D-P05-18** — the 20G bounds the **blast radius**, not the cap H7 crosses.
    Neither reading D-P05-16 named was taken. The worker cap is raised
    row-locally and modestly (512 MiB, against a measured 201M heavy capsule);
    attribution is earned by a control, not a margin; **no host-level ceiling is
    built here**.
  - **D-P05-19** — the capsule-time seam is a declarative per-cell lookup, not a
    hook. `pipeline_capsule` takes the vehicle and worker cap as parameters;
    `cell_run` sets them beside the verify bounds. D-P05-5 is honoured on its own
    stated reason — the harness learns a parameter, not a row.
  - **F-P05-37** disposed by D-P05-19; **F-P05-38** (a refused write leaves no
    oversized artifact) and **F-P05-39** (the T4a–T4e falsification drivers were
    never tracked and are gone) recorded.
- **F-P05-39 carries forward into T7/T8** — the falsification evidence trail is
  **asymmetric** and the evidence artefacts must say so rather than smooth it:
  H7's round is committed and re-runnable (`scripts/spike-capsule/drivers/`);
  T4a–T4e's rounds are **attested only**, their drivers gone and their sweep
  logs surviving for T4e alone. Reconstructing them was deliberately not
  attempted — see the finding.
- **~~H11's two operator calls~~ BOTH LANDED 2026-08-02, and H11 is SCORED** —
  `rig c3 H11` exit 0, 80 assertions, 0 FAIL, all four cells `pass`, row
  `model-level`. Thirteen of sixteen rows now carry scored results.
  - **F-P05-32 → D-P05-14 — egress is ALLOWLISTED, not binary.** The framing was
    `--share-net` vs `--no-net`; the answer is a trusted-side allowlist naming
    exactly the build hosts, with **content per capsule kind** so the agent
    hosts are absent (deny-all) when no agent runs. The coupling that creates —
    the worker kind's list names the model/harness endpoints — is **ADR-011 D3's
    per-harness capability layering**, not a new axis, and per-kind lists are
    what stop it becoming that ADR's "uniform altitude lie". EX-2 restates as
    *same mount posture, same egress MECHANISM, allowlist content per kind*.
    H11's canary clause stays measured-and-unscored **gated on QUE-204**, not
    unclaimed indefinitely: the two share one lever and settling them apart is
    how the earlier recommendation came to contradict QUE-204's own measurement.
    Feasibility is established, not assumed — `tinyproxy` and `iproute2` are
    installed, `socat`/`python3` were already there, and all are DQ-4-clean
    (in neither declaration's `exec:` list). Topology: no route at all, a unix
    socket bound in, the proxy the sole chokepoint — which forces an EXPLICIT
    proxy and is fail-closed by construction. **Not built. PLACED 2026-08-03 —
    D-P05-17 sends the work to a follow-on slice with QUE-204.**
  - **F-P05-33 → D-P05-15 — heavy is NOT `n/a`; it scores at `conform`.** The
    trigger is live on heavy (cargo really runs a root `build.rs`) and
    undeclared, so conform refuses it at leg 2 — one boundary EARLIER than § 5.6
    predicted, cheap (no verify capsule), and a result rather than an absence.
    Generalised in F-P05-35: class-2 triggers live at repo ROOTS, slice design
    targets are SUBDIRECTORIES, so conform dissolves the build-evaluation hazard
    unless a slice declares a root build file — true on BOTH fixtures. The light
    instantiation was stale in the same way (`postinstall` never runs under `npm
    test`); the row plants a test file, so light is vector-class **1** and heavy
    **2**, now stated per fixture in `matrix.tsv`.
  - **What it cost, recorded** (F-P05-36): H11's `model-level` is earned by two
    DIFFERENT boundaries, so the sandbox-containment claim is exercised on light
    only. The row asserts that explicitly rather than leaving the table to imply
    otherwise. A fixture-local slice for heavy declaring `build.rs` would prove
    containment there — rejected on cost, not principle (light's SL-001 is
    itself rig-fabricated, so a fixture slice is not the governance edit it
    looks like). Latent and not amended: `cell_altitude`'s `client-local` is
    overloaded — it means both "one fixture FAILED" and "the other had nothing
    to plant", with contradictory labels asserted three lines apart.
- **QUE-202 gains its first EVIDENCE input** (T4e, 2026-08-03) — H10 and H16 are
  scored, 8/8 cells `model-level`, and what they measure is the *shape* of the
  admission gap rather than only its existence. The refusal is safe and total;
  it is also **content-blind**, since stage 4 compares two OIDs and never reads
  a tree. So the capsule model cannot distinguish a genuine conflicting pair
  from a trunk advance a three-way merge would take without a murmur — both are
  `advance/stale-base`. Any admission design QUE-202 eventually reaches has to
  supply that discrimination itself; the pipeline will not be narrowing the
  problem for it. The sub-probe (T5) measures the incumbent's recovery path and
  **counts toward nothing** (F-9).
- **QUE-202 gains its SECOND evidence input** (T5, 2026-08-03) — the sub-probe
  is scored, and what it adds is the shape of the *recovery* path the capsule
  model would have to replace. Three facts, none of them derivable from the
  pipeline legs: (1) the incumbent's discrimination is real — a genuine
  conflicting pair is classified `Conflicted` and parked for hand-resolution,
  which is precisely the content-blindness the pipeline lacks; (2) **admission
  never reads trunk**, so `create` and `admit` both accept a moved trunk and the
  fast-forward CAS at integrate is the sole staleness gate; (3) the two paths
  **disagree about how a refusal is signalled** — conflict exits ZERO with the
  verdict in `candidates.toml`, staleness exits non-zero (F-P05-40, ISS-305).
  An admission design that reuses conflict semantics per DEC-110 inherits (3)
  unless it is named, and a scripted caller reading exit status alone cannot see
  a conflict at all.
- **F-14 is MEASURED, not argued** (T4e falsification M25) — inverting stage 4
  so the transfer precedes the precondition leaves the token (`stale-base`),
  canonical's REFS, and all seven of the row's own clauses intact; **only the
  object count reds.** So § 5.5's strict-clause carve-out is confirmed as a
  scope correction rather than a weakening, by measurement: had `stale-base`
  carried the refs-only clause that `cas-lost` legitimately carries, that defect
  would have scored GREEN, and H10/H16 are the only place in the matrix where it
  shows. Worth citing at close wherever F-14 is defended.
- **DEC records are OWED at close for D-P05-6..23** — the sheet's convention is
  "lift to notes.md at close", so they were deliberately not minted mid-phase.
  The range has extended repeatedly: D-P05-16 (H7's 20G bound), D-P05-17
  (allowlist work → follow-on slice), D-P05-18/19 (T4c/T4e), and **T6 added
  D-P05-20..23** (guard-probe placement · guard (a) as citation · guard (e)'s
  three legs and its OID-excluding observable · the `fx_case` lift).
- **the DQ-4 exemption is CONDITIONAL and PHASE-05 holds the condition** —
  F-P04-9. A payload-bearing fixture variant reusing `fixture-light.sh`'s
  trusted-side `npm` build loop breaks DQ-4 for real, and the audit cannot see
  that the fixture's provenance changed
- QUE-200 — ingestion mechanism M-A vs M-B; the rig's whole point. **All five
  evidence inputs are now recorded and linked** (T7, `0b9581b1`): EVD-006
  (H6/H7 mechanism-neutral) · EVD-007 (every refusal token trusted-side,
  downstream of the mechanism) · EVD-008 (both preserve worker commit identity)
  · EVD-009 (cost equal; M-B trips git auto-maintenance) · **EVD-010
  `disputes`** (M-B's trusted-side file-ingestion boundary — the previously
  banked input, now counter-evidence to candidate 2's "cleanest trust story").
  **The result is that the mechanism axis barely matters** and the one measured
  asymmetry runs against the bundle — F-P05-45. **Do not read the matrix as
  "fetch is proven safe against hostile config"**: QUE-200's own `upload-pack`
  worry is untested and stays argument. The question remains `open` — it settles
  in the post-spike REV, not here.
- QUE-201 — declaration home; now ergonomics-only. **Its evidence input is
  OBSERVED as of T6** (guard (e), three legs): a capsule that rewrites its own
  in-repo declaration leaves trusted-side behaviour byte-identical, worktree-side
  and committed alike. **EVD-011 recorded and linked at T7**, carrying F-P05-43
  — *where* the committed rewrite refuses is fixture-specific and must not be
  generalised. Question remains `open`; settles post-spike.
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
- DEC-099/107/108/109/110 — settled, carried
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
  the four-stage capsule pipeline (CON-004, DEC-110) depends on plan-gate
  mechanics, so the revisions should land orthogonally to this rig.
