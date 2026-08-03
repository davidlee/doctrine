# PHASE-04: Independent enumeration, P-C1a, P-C2, and the audits

Disposable phase sheet — runtime scratch under `.doctrine/state/`, gitignored
and `rm -rf`-able. Expands the plan's phase entry into working detail. Durable
risks / decisions / findings are harvested into the slice audit at close-out;
until then, lift anything that must survive into the slice's `notes.md`.

## Objective

Run the gating probes in the design's order (§ 5.4). Step 0 comes first and is
the reason this phase leads: it is the ONLY mechanism in the whole rig that can
falsify an exhaustiveness claim. Every other mechanism — the sixteen rows, the
DQ-4 audit — is confirmatory, since rows instantiate already-classified triggers
and the audit greps for tokens the declaration already names. A trigger no class
describes is invisible to both. Cheap now, expensive once it is shipped
enforcement.

## STATUS AT SHEET-AUTHORING TIME — read first

**T0a is already DONE and COMMITTED** (`beb4b665`,
`.doctrine/slice/241/step0-enumeration.md`). It was done *before* this sheet was
written, deliberately and out of the handover's stated order, because it is the
one artifact in this phase whose validity is destroyed by the reading that
phase-planning itself requires (EX-9). Everything below T0a assumes that commit
exists and that CPT-001 was unread at it.

The context that authored T0a had read only `handover.md` (authored to name no
trigger), `plan.toml` PHASE-04, and ASM-007. **Declared leak:** ASM-007's prose
names the five class *labels*. Recorded in the artifact as a bounded leak rather
than claimed clean — do not let a later reader find it and treat it as a
concealed defect.

## Reading list

| what | where | why |
|---|---|---|
| PHASE-04 criteria | `plan.toml:145-176` (EN-1, EX-1..9, VT-1/2, VA-1..3) | authoritative; `slice phase` is a WRITER, no read-back verb (`mem_019facc24ac5`) |
| order and gating | `design.md:468-499` (§ 5.4 steps 0-5) | step 0 → P-C1a → A2 smoke → P-C2 |
| measurement table | `design.md:878-900` (§ 9) | wall-clock/disk row is **not measured** incumbent-side |
| audit discipline | `design.md:851-854` | positive controls on both audits |
| P-C1a spec | `.doctrine/rfc/025/probe-specs.md:31-57` | the five steps + what to measure |
| P-C2 spec | `.doctrine/rfc/025/probe-specs.md:59-86` | the six-row table + the no-hooks clause |
| census B1–B6 | `.doctrine/rfc/025/mechanism-census.md:40-45` | EX-8's subjects |
| PHASE-03 terrain | `notes.md:200-311` (decisions, findings, evidence, OQ) | ride `pipeline.sh`, don't refork |
| CPT-001, fixture `interpret:` | `knowledge show CPT-001`, `fixtures/light/interpretation-surface.txt` | **T0b onward only** — never before T0a |

**Research advisory**: `slice research SL-241` reports drift, but the four
drifted paths are `slice-241.md`, `design.md`, `plan.md`, `plan.toml` — this
slice's own authored artifacts, added *after* the baseline stamp. That is
in-slice churn, not upstream drift; no research thread's subject moved. Do not
refresh-and-restamp on this signal. (Same advisory PHASE-01/02/03 ran under.)

## Assumptions & STOP conditions

**Carried assumptions**

- A1 — EN-1 met: PHASE-03 `completed`, VT-1..4 PASS, `rig selftest` re-runnable,
  so a refusal is distinguishable from a broken rig.
- A2 — `control/pipeline.sh` is sourceable; `pipeline_setup` / `scenario`
  **publish** (`PIPELINE_RUN`, `SCENARIO_RUN`), they do not print. Forcing them
  into `$( … )` swallows `guard_not_real_repo`'s `exit` (F-P03-4). Don't.
- A3 — `nix` and `direnv` are ABSENT in this jail; the toolchain arrives by
  ro-bind. P-C1a's "nix env ready" step is `n/a` **with its reason**, never
  dropped (EX-4).
- A4 — no incumbent wall-clock/disk column exists and none is to be invented
  (§ 9 names it not-measured). P-C1a banks **absolutes** (VA-2).
- A5 — no `src/` changes in scope. Rust gates therefore don't apply; corpus gate
  is `doctrine validate`, shell gate is `LC_ALL=C.UTF-8 shellcheck -x -S style`.

**STOP conditions — `/consult`, do not improvise past**

- S1 — **OQ-a is live in this phase.** EX-6 records the resource-bound row as a
  P-C2 row in its own right, and a *verify* capsule overrunning the DISK cap has
  no legal token: `verify/resource-cap` is not in the closed set. **Do not mint
  one.** The set being closed is what `assert_outcome` keys off. Get an operator
  ruling or `/consult`. Same for OQ-b/OQ-c if a row reaches them.
- S2 — if step 0's residue is **non-empty**, that is ASM-007 *falsified*, not a
  bookkeeping detail. Stop and surface it before writing the amendment; it
  changes what the post-spike REV can claim.
- S3 — if a positive control cannot be made to fail (EX-7), the audit is
  decoration. Stop rather than record the audit green.
- S4 — any `src/` change, any new refusal token, any design-level gap → `/consult`.

## Tasks

<!-- [ ] todo · [WIP] · [x] done · [blocked]. Graduates to TOML rows in the
     tracking file when a consumer needs queryable per-task status (D5/Q5). -->

### Step 0 — the falsifier (EX-1, EX-2, EX-3, EX-9 · VA-1)

- [x] **T0a — independent enumeration, pre-classification.** DONE at `beb4b665`.
      96 triggers over 11 surfaces, walked by ecosystem surface rather than by
      recalling the taxonomy. No classification in that commit; CPT-001 unread.
      The commit gap is VA-1's independence evidence.
- [x] **T0b — read CPT-001 and the light fixture's `interpret:` list.** Done.
- [x] **T0c — classify all 96 against CPT-001.** Appended below the sentinel at
      `61ea9f08`. 84 clean · 6 strain · 2 out-of-scope · **4 residue**.
- [x] **T0d — residue stated: NON-EMPTY.** R1 LLM/unbounded carrier · R2 terminal
      escapes · R3 metadata interpolation · R4 non-inert parsing. R2 and R4 are
      individually sufficient. Tripped STOP condition S2 → escalated, ruled.
- [x] **T0e — ASM-007 amended in BOTH TIERS.** `invalidated` (operator ruling:
      not `obsolete` — the claim was posed, tested, and failed, and the status
      should say so). Verified via `knowledge show`. Replacement carried as
      **ASM-008**; residue tracked as **QUE-203**; CPT-001 deliberately untouched.
- [x] **T0f — separate commits.** `beb4b665` → `61ea9f08`, append-only diff.
- [x] **T0g — declaration amendment is NO CHANGE, with its reason** (`090148ef`).
      No residue item is path-shaped, so none is declarable — which is itself the
      sharp form of the finding. Non-regression verified, not assumed: all three
      fields parse byte-identically and `rig selftest` is green.

**STEP 0 IS COMPLETE.** EX-1, EX-2, EX-3, EX-9 and VA-1 discharged. The
contamination-sensitive, one-shot part of this phase is done and committed;
everything remaining (P-C1a, P-C2, the audits) is confirmatory and re-runnable.

### Step 1 — P-C1a, the cost baseline (EX-4 · VA-2)

- [x] **T1a — `control/probe-c1a.sh`**, dispatched by `rig c1a`. Riding
      `pipeline.sh`'s existing setup/teardown, not a second forked pipeline.
- [x] **T1b — bank wall-clock per step**: clone · provision · **nix env ready
      (`n/a` + reason)** · build · test · harvest. The `n/a` row is *emitted*,
      with its reason string, in the same shape as an `n/a` matrix cell.
- [x] **T1c — bank peak disk per capsule.** Absolute. No delta column, no
      incumbent column (VA-2).
- [x] **T1d — results to `probes/c1a/results.tsv`**, same append shape as
      `probes/selftest/results.tsv`.

### Step 3 — P-C2, the confinement matrix (EX-5, EX-6 · VA-3)

Design § 5.4 step 2 is the A2 smoke; it is PHASE-04-adjacent but not named in
any EX. Run it if cheap (it is), record it, but it gates nothing here.

- [x] **T2a — `control/probe-c2.sh`.** Every row a `bash -c` **INSIDE** the
      sandbox (DQ-2 — a probe "contained" by a worker politely declining is
      void). Note `mem_019fbd3cb782`: the shebang interpreter is a mount
      dependency; a row that fails to exec is not a row that passed.
- [x] **T2b — the six spec rows**, each asserting on a **named observable**
      (DQ-3): write floor · canonical invisibility · git creds · API cred
      presence · env probe · escape-via-`.git`.
- [x] **T2c — reuse `absent_inside`, do not re-derive it** (F-P03-2). It proves
      the subject reachable OUTSIDE before asserting it does not resolve INSIDE,
      and records `n/a` with reason where it is not. Under nested confinement the
      OUTER jail may be doing the hiding, which reads as the profile working
      (`mem_019fbd70c924`). **`~/.ssh` is `n/a` here** — it exists on the host
      but the outer jail hides it (operator, 2026-08-01). Load-bearing subject is
      the host home ROOT.
- [x] **T2d — the resource-bound row (F-11) as a P-C2 row in its own right**
      (EX-6): wall-clock timeout + disk cap. **S1 applies** — if the row lands on
      a verify capsule overrunning disk, stop.
- [x] **T2e — record every row with its observable named** (VA-3), to
      `probes/c2/results.tsv`.

### Step 4 — the two audits (EX-7, EX-8 · VT-1, VT-2, VA-3)

- [x] **T3a — `control/audit-dq4.sh`** (VT-1). Greps `control/**` for the active
      declaration's `exec:` tokens; they must be ABSENT. VT-1 keywords: `control/`,
      `interpretation-surface`.
- [x] **T3b — `control/audit-nohooks.sh`** (VT-2). Names the exact machinery the
      census marks DELETE so its absence is *witnessed*: `SubagentStart`,
      `WorktreeCreate`, marker (`.doctrine/state/dispatch/worker`),
      `DOCTRINE_WORKER`, `worker_mode`. VT-2 keywords: `SubagentStart`,
      `WorktreeCreate`, `worker_mode`.
- [x] **T3c — a POSITIVE CONTROL on each, DEMONSTRATED FAILING** (EX-7, VA-3).
      Use `rig_assert_fails`. **Never `rig_assert '…' ! cmd`** — `!` arrives as
      the command name and the assertion reds on its own invocation (F-P02-3).
      "Demonstrated" means observed red with the target token removed, then green
      with it restored — not merely coded.
- [x] **T3d — EX-8: witness B1–B6 UNREPRESENTABLE, not merely unused.** This is
      the criterion most likely to be satisfied cheaply and wrongly. "Absent from
      the rig" is *unused*. *Unrepresentable* needs the structural reason stated
      per row — B1/B4 no marker exists to stamp or clear; B2 no coord tree for a
      worker to be mis-placed onto; B3 no linked worktree and no stamp; B5 no
      in-session worker for a hook to fire on; B6 base is pinned in the
      provisioning contract, so placement cannot determine it. Record the reason
      alongside the grep, or EX-8 is met in name only.

### Wrap

- [x] **T4a — `verify-vt`**: PHASE-04 VT-1 and VT-2 flip green (two of the four
      still-FAIL mandates).
- [x] **T4b — gates**: `doctrine validate` (corpus) + `LC_ALL=C.UTF-8 shellcheck
      -x -S style` on every rig file. The locale is not optional — without it
      shellcheck aborts on em-dashes with no line and no rule id (F-P02-5).
- [x] **T4c — lift durable material into `notes.md`** (decisions / findings /
      evidence / open questions / boundary) before this sheet is `rm -rf`'d.
- [x] **T4d — flip PHASE-04 `completed`.**

## Risks

- **R1 — step 0's classification is back-fittable even now.** T0a is banked, but
  T0c is still authored by a context that will by then have read CPT-001. The
  temptation is to classify generously and return an empty residue, which reads
  as ASM-007 surviving. Mitigation: T0d's named watch-list, and recording
  *strain* rather than rounding to the nearest bucket.
- **R2 — VT-1/VT-2 prove PRESENCE, not BEHAVIOUR.** They grep the audit scripts
  for keywords. This slice has three times found that evidence shape to be
  decoration (F-P02-\*, F-P03-\*). VT green must not be read by PHASE-05/06 as
  "the audits work" — EX-7's demonstrated positive controls are what carry that,
  and they are VA-3's business, not VT's. Say so in `notes.md`.
- **R3 — nine of seventeen refusal tokens remain unobserved**, five of them
  named by rows this phase does not run. Criteria-conformant (PHASE-05 observes
  them), but a phase that touches token plumbing must not assume they work.
- **R4 — `assert_outcome`'s pass arm asserts `changed == 2`** (`comm -3` emits
  the old and new line of one moved ref). Would also read 2 for two unrelated
  refs appearing once each; the pinned-OID assertion covers it in practice. If
  this phase touches `assert_outcome`, strengthen rather than inherit.

## Decisions

- **D-P04-1 — T0a ran before the sheet existed, and before the phase flipped
  `in_progress`.** Deliberate inversion of the handover's stated order. The
  handover carried two instructions that conflict: "`/phase-plan` first" and
  "do step 0 in a context that has not read the [reading list]". Phase-planning
  requires reading design § 5.4 and `notes.md`, so honouring the first destroys
  the second — silently, and in the direction that manufactures a false green
  (empty residue reads as ASM-007 surviving). Cost of inverting: a lifecycle
  order blemish, visible and recorded here. Cost of not inverting: the phase's
  only falsifier is void and nothing says so. Took the visible cost.

## Findings

<!-- populated during execution -->
