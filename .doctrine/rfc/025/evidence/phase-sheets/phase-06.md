# PHASE-06: P-C1b and the scoped go/no-go

Disposable phase sheet — runtime scratch under `.doctrine/state/`, gitignored
and `rm -rf`-able. Expands the plan's phase entry into working detail. Durable
risks / decisions / findings are harvested into the slice audit at close-out;
until then, lift anything that must survive into the slice's `notes.md`.

## Objective

Run the one probe that needs a real agent, fill the measurement table with a
named source per column, and land a go/no-go whose SCOPE IS LOAD-BEARING.
Writing the scope in is what stops the downstream REV over-claiming: the verdict
must not read '16/16', ASM-007 must be recorded strengthened rather than
discharged, and conflict/staleness RESOLUTION must be named as out of evidence.

**What the plan entry does not say, and the sheet must:** this phase still has
**rig to build**. `rig` line 151 dispatches `control/probe-c1b.sh` and that file
**does not exist**; `capsule/worker-agent.sh`, named in design § 9.1's tree, does
not exist either. PHASE-06 is *build a probe, run it once, then write* — not
"write".

## Reading list

- `.doctrine/slice/241/design.md` **§ 9** (`:914–:990`) — the phase's subject.
  The measurement table is `:954–:964`; "Scoped means, precisely" is `:978–:989`.
- `.doctrine/slice/241/design.md` `:535–:549` — order and gating; P-C1b is step 5
  and the only probe needing an LLM (DEC-109).
- `.doctrine/rfc/025/probe-specs.md` **§ P-C1** (`:31–:58`) — the probe spec
  proper: five steps, the pass condition (EX-3), the measure list.
- `.doctrine/rfc/025/evidence/README.md` — the verdict this phase must not
  over-claim from, and its **seven** "what this does NOT establish" items. Read
  before drafting a line of `go-no-go.md`.
- `.doctrine/slice/241/notes.md` § Open (`:1363–:1523`) and §§ PHASE-05 (index
  form; chase `D-P05-n`/`F-P05-n` in `evidence/phase-sheets/phase-05.md`,
  findings newest-first).
- `scripts/spike-capsule/capsule/worker-stub.sh` — the shape `worker-agent.sh`
  must follow: control-plane-named doorbell / result ref / bundle path, all
  `:?`-fail-closed, capsule never names the harvest path (RT-4/F-6).
- `scripts/spike-capsule/control/probe-c1a.sh` — the probe shape, incl. the
  `SANDBOX --capsule … -- /rig/worker-stub.sh` call at `:201` and the
  step-timing/TSV emission this probe reuses.
- `scripts/spike-capsule/control/probe-smoke.sh` — A2's two assertions; the
  proof that `claude -p` authenticates inside the profile.
- `.doctrine/slice/241/handover.md` — session packet; terrain and caveats.

## Assumptions & STOP conditions

**A-P06-1 — `claude -p` runs authenticated inside the sandbox profile.**
Established, not assumed: `~/capsules/probes/smoke/results.tsv` records both A2
legs `pass` (`network/unauthenticated`, `credential/authenticated —
claude -p answered: OK`). If either leg regresses inside the c1b capsule that is
a **profile fault, not a P-C1b result** — probe-smoke.sh's own warning wording.

**A-P06-2 — token usage is machine-readable from the harness.** Verified this
session in-jail: `claude -p --output-format json` emits a `result` message
carrying `usage` (`input_tokens`, `output_tokens`, `cache_creation_input_tokens`,
`cache_read_input_tokens`), `modelUsage` per model id, and `total_cost_usd`. The
`--output-format` flag is **not listed in `claude --help`'s option list** in this
build (only referenced in other options' prose) — it works, but do not cite the
help text as its source; cite the observed run.

**A-P06-3 — the heavy fixture is not rebuilt.** F-P05-21 / D-P05-10 and guard
(b)/H5 rest on `.doctrine/rfc/025/evidence/**` being empty at heavy's pin
`4fc9d32f0`. P-C1b has no reason to touch heavy. Do not "rebuild fixtures" as
hygiene.

### STOP conditions

1. **EX-8 is unsatisfiable as literally written — operator ruling required
   before T6.** `doctrine knowledge inspect ASM-007` returns **`invalidated`**.
   PHASE-04 step 0 falsified it and ASM-008 (the responsibility split, `held`)
   replaced it; design § 9's closure bullet ("ASM-007 is recorded strengthened,
   not discharged, whatever step 0 returns") is **already false**, and notes.md
   § PHASE-04 — FOR RECONCILIATION says so. Writing "ASM-007 strengthened" into
   the go/no-go would plant a false claim in the one document whose whole purpose
   is preventing over-claim. Criteria ids are immutable and edits append, so this
   is a ruling, not an edit-in-place. **Raised to the operator at plan time; T6
   does not start until it is answered.**
2. **S2 — a failed probe is a FINDING and a `/consult`, never a quiet rig edit.**
   Held all through PHASE-05.
3. **S4 — `src/` is not touched.** PHASE-06 has no stated need. If one appears,
   that is a `/consult`, not an improvisation. `src/` was never touched in
   PHASE-01..05.
4. **The retry policy is decided before the first `--agent` run, not after
   seeing it fail** (R-P06-1). Deciding afterwards is how n=1 becomes
   best-of-k wearing n=1's clothes.
5. **A sandbox weakening to make the agent work is a `/consult`, not a fix**
   (R-P06-3). If headless `claude -p` needs a permission escape hatch, that is a
   result about the capsule model and it gets recorded as one.

## Tasks

<!-- [ ] todo · [WIP] · [x] done · [blocked]. Graduates to TOML rows in the
     tracking file when a consumer needs queryable per-task status (D5/Q5). -->

- [ ] **T1 — choose and pin the red→green target.** probe-specs § P-C1 allows
      "one already-completed real phase (re-execution target) **or** a toy phase
      with a genuine red→green test". Recommend the **light fixture** (the
      TypeScript project, `npm test`, `src/**` declared): it is the fixture P-C1a
      already measured, its suite is ~0.25s, and a genuine failing test is cheap
      to plant. Record the choice and its reason as a decision — it bounds every
      number T2 produces. Files: `scripts/spike-capsule/fixtures/light/`.
- [ ] **T2 — write `capsule/worker-agent.sh`.** Runs INSIDE the sandbox. Follows
      `worker-stub.sh` exactly on the boundary contract: `RIG_DOORBELL`,
      `RIG_RESULT_REF`, `RIG_BUNDLE` all `:?`-fail-closed from the control plane,
      capsule never names the harvest path. Differs only in the middle: instead
      of a scripted commit it invokes `claude -p --output-format json` with cwd
      bound to the clone, writes the raw JSON to a control-plane-named path, and
      leaves the commit to the agent (I1a — total freedom inside). **Interpreter
      is a mount dependency** (`mem.pattern.shell.shebang-interpreter-is-a-mount-dependency`):
      the `claude` binary, its node runtime and `~/.claude` must be ro-bound, and
      the assertion is that it **executes** (exit ≠ 127), not that it resolves.
- [ ] **T3 — write `control/probe-c1b.sh`.** `rig c1b` already dispatches it
      (`rig:151`, arg shape `probe-c1b.sh PHASE-06 "${RIG_ROOT}" [rows…]`).
      Reuses probe-c1a's step-timing and TSV emission; `guard_not_real_repo`
      (I6) before anything. Asserts, in order:
      - **EX-3** — the canonical repo path is **not mounted in the sandbox at
        all** (absent, not read-only). Assert from the sandbox's own mount view,
        with a positive control: a path that IS bound must be found by the same
        probe, or the negative proves only that the probe ran
        (`mem_019fa18161f4…`).
      - **EX-1** — the suite reaches green in-capsule after the agent's work, the
        worker committed, the doorbell rang, and harvest→advance completed.
      - **EX-2** — the usage numbers, extracted from the `result` message.
- [ ] **T4 — gates on the rig change.** `./rig selftest`, `./control/audit-matrix.sh`,
      `./control/audit-dq4.sh`, `./control/audit-nohooks.sh`, `./control/audit-i4a.sh`,
      `doctrine validate`, and `LC_ALL=C.UTF-8 shellcheck -x -S style` over every
      changed file (**without the locale it aborts on the em-dashes** with no
      line and no rule id — F-P02-5). All were green at PHASE-05 close; they stay
      green or the change is wrong.
- [ ] **T5 — run `rig c1b --agent`, once, scored.** `rig selftest` first (A-1);
      pin `DOCTRINE_BIN=~/.cargo/bin/doctrine`; **redirect to a file, never pipe**
      (a pipe reports the reader's exit status). Raw log to
      `.doctrine/state/rfc-025/raw/` — gitignored by the existing `.doctrine/state/`
      entry, **no new `.gitignore` entry** (the v0 entry broke
      `every_runtime_gitignore_glob_is_classified`).
- [WIP] **T5b — the profile fix, the capability leg, and the re-run** (D-P06-5).
      Added after T5's run measured nothing (F-P06-7).
      - [x] **`probe-smoke.sh` — a THIRD A2 leg, written FIRST and shown RED.**
            The agent runs `uname -r > /capsule/exec-proof`; the assertion is
            trusted-side on the file's *content*, never on the agent's narration
            (I5). Kernel release specifically, because a leg a file-writing tool
            could satisfy would pass in exactly F-P06-7's case. Red against the
            old profile — the agent's own reply named the fix — then green
            against the new one. Same leg, same file, one profile change between.
      - [x] **`sandbox.sh` — `--tmpfs /agent/.claude`, credential ro-bound
            inside.** In the shared `mounts` array; `--print-mounts` compares
            equal across both kinds (EX-2 holds). Strictly narrower on
            visibility than what it replaced — the whole of `~/.claude` no
            longer crosses, one file does.
      - [x] **Gates.** `shellcheck`, `rig selftest`, `rig guards`, `audit-matrix`,
            `audit-dq4`, `audit-nohooks`, `audit-i4a`, and **`rig c1a` re-run
            green** (the profile change touches every capsule, so C1a is the
            behaviour-preservation proof).
      - [x] **The handover's open question: P-C2 re-run.** Answered by running
            it — D-P06-7, F-P06-8. `api-cred` went red on a proxy observable;
            consulted per S2, operator ruled **(a) realign the observable**
            (D-P06-8). **All seven rows now pass against the new profile**, so
            P-C2's scored results are re-taken, not carried with a caveat.
      - [x] **Re-run P-C1b scored.** Done, and it is a REAL phase — F-P06-9.
            `agent-committed=yes tree-dirty=no`, the agent's own commit, all
            assertions holding for the right reasons. **EX-1 and EX-2
            discharged.** Attempt 1 stays disclosed with its usage (D-P06-2),
            preserved raw at `~/capsules/probes/c1b/attempt-1/`.
- [x] **T6 — DONE** (`a9924134`), `.doctrine/rfc/025/evidence/measurements.md`.
      All nine rows, each naming both sources or recording after-side-only with
      its reason. **Counting rows 2 and 5 corrected the design** — see F-P06-10.
      Row 8 carries all three non-optional caveats and the prior attempt in full.
- [ ] ~~T6~~ — the measurement table (EX-4). Nine rows from design § 9
      `:954–:964`. Each names **both** sources or says "not measured" (F-10).
      Five are static inspection of the two models and must actually be counted,
      not asserted; wall-clock/disk come from P-C1a's banked TSV (absolutes, not
      deltas — VA-2); tokens from T5. Lands under
      `.doctrine/rfc/025/evidence/` alongside `matrix.md` / `guards.md`.
      **Name which token number is reported** (R-P06-2).
- [x] **T7 — DONE** (`cc90a63a`). Verified this session: `knowledge inspect`
      renders a populated `[evidence]` block on **all four** of ASM-007
      (`contradicts` ×3 + 3 notes), QUE-200, QUE-201 and QUE-202
      (`supports` ×3 + 3 notes). The checkbox below was left unticked; the work
      was done. VA-3 still needs the four to be consistent with the go/no-go.
- [ ] ~~T7~~ — VA-3 prep: the ISS-306 workaround on the two records that lack it.
      `knowledge show`/`inspect` render **no inbound reciprocity**, so a QUE shows
      nothing for the EVDs supporting it. The pattern that discharged VA-5 is to
      populate the record's own `[evidence]` `supports`/`contradicts` block
      (structured tier) plus prose. **Already done for QUE-200 and QUE-201;
      ASM-007 and QUE-202 are not done** and VA-3 needs all four consistent in
      BOTH tiers.
- [x] **T8 — DONE. EX-9 ruled: DOES NOT SUFFICE** (D-P06-9). QUE-200 stays
      `open`, recorded in both tiers of its own record; no status flip. The
      expected answer for a **corrected** reason — F-P06-11 found the stated
      obstacle ("`upload-pack` never exercised") false in four committed places
      and replaced it with the real one: M-A's trusted-side git surface is
      *sampled*, not cleared, while M-B's is enumerated and tested. Corpus
      corrected in place per the ruling: EVD-006 (both tiers), README limit 1,
      `notes.md` `:1079` / `:1647`.
- [ ] ~~T8~~ — EX-9: rule on whether the EVD suffices to settle QUE-200. Six
      records exist (EVD-006..011): four `supports`, one `disputes` (EVD-010),
      one for QUE-201. They establish that the mechanism axis barely matters
      (F-P05-45) and that the one measured asymmetry runs **against** the bundle.
      Against that: QUE-200's own `upload-pack` worry was **never exercised**
      (README limit 1). EX-9 explicitly permits "does not suffice" as the
      recorded answer — **record, do not force.** `doctrine knowledge status`
      is the surface.
- [x] **T9 — DONE** (`0b8ada26`, amended `1e54945f`).
      `.doctrine/rfc/025/go-no-go.md`. **GO, scoped to Linux/bwrap for a client
      of this build shape**, for the ingestion and confinement halves.
      All four anti-over-claim requirements met and verified, not asserted:
      - **the count** reads *"sixteen rows with a capsule-model boundary or a
        recorded dissolution, plus two regression legs"*; the only `16/16` in the
        file is the sentence forbidding it (grepped — VA-2).
      - **scope § 1** splits admission-boundary rows (portable in reasoning,
        unmeasured off Linux) from environment-conditional rows (outstanding for
        macOS), and pins that **`model-level` is an altitude over FIXTURES, not
        operating systems**. Design § 9's *"model-level rows proven portable"*
        reads as OS-portability — **flagged FOR RECONCILIATION** (second § 9 item
        after F-P06-10's row 2).
      - **QUE-202 § 3.1** — out of evidence, **not disproven**; refusal proven
        safe, total and content-blind, admission not designed.
      - **ASM-007 § 3.3** per D-P06-1 — falsified and `invalidated`, replaced by
        ASM-008 which is strengthened-not-discharged, with the divergence from
        EX-8's letter stated FOR RECONCILIATION.
      Carries § 3.2 (QUE-200 does not settle — T8) and § 4's stated model
      properties: the OS boundary IS the boundary, `EROFS`-not-`EACCES`, tokens
      capsule-reported and gating nothing, a degraded agent costing double.
      **VA-3 caught a real gap on the first pass:** the document never mentioned
      **QUE-201**, so § 6's "settles no open question" list named three of four.
      Fixed with § 3.4 (ergonomics-only, EVD-011's byte-identity observed,
      F-P05-43's don't-generalise-the-token warning). All four records now render
      states matching the document — ASM-007 `invalidated`, the three QUEs `open`.
- [ ] ~~T9~~ — write `.doctrine/rfc/025/go-no-go.md` (EX-5/6/7/8). Blocked on
      STOP-1. Scoped: go on **Linux/bwrap**, for a client of this build shape;
      model-level rows portable, env-conditional rows outstanding for **macOS**;
      the count reads *"sixteen rows with a capsule-model boundary or a recorded
      dissolution, plus two **regression legs**"* and **never "16/16"**;
      **QUE-202** named as out of evidence; ASM-007 recorded per STOP-1's ruling.
- [WIP] **T10 — VT-1: keyword leg PASSES; attribution resolves at the flip.**
      `doctrine slice verify-vt 241` reports
      **`≈ UNATTRIBUTABLE — keyword present but go-no-go.md not modified by this
      slice`**. All five keywords (`QUE-202`, `strengthened`, `regression legs`,
      `Linux/bwrap`, `macOS`) are in the file and the file is committed
      (`0b8ada26`). The attribution leg reads the slice's phase-delta registry,
      and **PHASE-06 has no row there yet** — the row is upserted by the
      `completed` flip, which is T13. **Not a failure and not a defect**: it is
      an in-flight phase being asked a question only a closed one can answer.
      **Re-run `verify-vt` immediately after the T13 flip** and confirm PASS.
- [x] **T11 — DONE** (`6037eb44`). EX-10 discharged; `backlog list` checked
      first and no RFC-025 cleanup item existed.
      - **CHR-053** — the RFC-025 cleanup pass: fold in the spike evidence,
        reconcile the census B1 note (`mechanism-census.md:40`, `:179`) against
        **both** dispatch arms since the claude arm's worker self-commits and
        breaks the note's uniform-subprocess assumption, and rule **RT-3**
        (topology, `red-team.md:68`) and **RT-9** (forensic-archive storage tier,
        `:151`).
      - **CHR-054** — REV scoping for the census DELETE rows. The `if go`
        condition is met. Governance routes through a Revision (ADR-013), so this
        is the **scoping** step, not the REV. Its body carries the four scope
        constraints the REV must inherit — Linux/bwrap only, QUE-202's admission
        gap, QUE-200 open, and the count that is not "16/16" — because losing
        them there is precisely the over-claim the go/no-go exists to stop.
      - `CHR-054 after CHR-053`; both `references(originates_from) SL-241`;
        tagged `area:dispatch area:governance cluster:capsule-spike`.
      - **Confirmed not this:** D-P05-17's allowlist / QUE-204 follow-on slice,
        already placed.
- [x] **T12 — DONE. VH-1 DISCHARGED.** Operator, 2026-08-03: *"ok, accepted
      verdict + scope"*. Presented as **one ask** per the criterion — the go on
      Linux/bwrap together with the six scope clauses (platform; `model-level`
      grading fixtures not operating systems; the count that is not "16/16";
      QUE-202 out of evidence; ASM-007 `invalidated` per D-P06-1; QUE-200 and
      QUE-201 still open). Acceptance covers **both**, which is what stops the
      downstream REV over-claiming from a bare verdict.
- [x] **T13 — DONE. PHASE-06 is `completed`; 6/6.**
      - **Harvested into `notes.md`** — the `## Harvest` stamp restamped and
        compressed to pointer-only per convention (it had gone cumulative across
        five phases), a `### PHASE-06 close` section carrying the verdict and the
        two findings the close itself produced, and a
        `### PHASE-06 decisions and findings — INDEX` in PHASE-05's form, since
        this sheet is archived rather than discarded.
      - **Sheet archived** to `.doctrine/rfc/025/evidence/phase-sheets/phase-06.md`,
        replacing the stub copied at PHASE-05's close. `phase-06.toml` and
        `boundaries.toml` re-copied so the archive carries this phase's own close
        rather than PHASE-05's snapshot of it; `phase-sheets/README.md` updated —
        it described `phase-06.md` as *"an unexpanded stub"*.
      - **`code_end_oid` verified, and it WAS wrong** — the flip captured
        `ff0290b5`, SL-244's `ISS-310` commit, exactly as ISS-307 describes.
        Corrected with `record-delta`.
      - **`code_start_oid` verified, and it was RIGHT** — see **F-P06-13**.
        `e43572d4` is genuinely `de0d49a9^`. F-P06-4's recommendation to widen
        ISS-307 to the start boundary is **withdrawn**: an exclusive boundary
        naming a foreign commit is correct by construction.

## T6 — measurement table, working data

Banked while the STOP-5 ruling is open; these rows are static and hold under
either outcome. Each names BOTH sources or says "not measured" (F-10).

| # | metric | before (incumbent) | after (capsule) |
|---|---|---|---|
| 1 | trust-bearing lifecycle states | **7** — `funnel-machine.md` § Transitions: `none` → `spawned` → `worker-committed` → `imported` → `verified` → `concluded` → `reaped`. Each gates verb legality (C7, REQ-384–387), so each is trust-bearing. | **4** — the stages of design § 5.1: harvest, conform, verify, advance |
| 3 | security-significant hooks (target 0) | **2 shipped assets** — `install/git-hooks/pre-commit` (the coord safe-commit / funnel-reversion backstop, `dispatch.rs:1831 install_coord_hook`, ISS-234) and `plugins/doctrine/hooks/hooks.json` (the SubagentStart/Stop matcher set, census B5) | **0** — `audit-nohooks`, and not as a bare grep: rows B1–B6 each carry a token leg *and* a witness leg (subject unrepresentable) |
| 4 | role-detection rules (target 0) | census **B1, B2, B3, B4** — disk worker marker; `DOCTRINE_WORKER` env leg + `worker_mode`; the fail-closed marker-absent-linked-worktree rule (REQ-192); marker recovery/`describe_mode` | **0** — `audit-nohooks` B1/B2/B4 legs: authority is conferred by the sandbox, and there is no marker to stamp or clear |

Rows 2 and 5 (mutable refs per accepted phase; git ops between doorbell and
accepted-ref advance) still need their incumbent enumeration done properly —
the design's parenthetical list is a starting point, not a count, and it must be
checked against the code rather than quoted. Rows 7/8 (wall-clock+disk, tokens)
and 9 (failure states) depend on which run is authoritative.

**Do not fill row 8 from the first run without its caveat** — see F-P06-7.

## Risks

- **R-P06-1 — the agent may not reach green, and n=1 has no slack.** EX-1 is a
  real-agent red→green; DEC-109 calls it non-deterministic by construction. If it
  reds, EX-1 is a **recorded result about the capsule model**, not a rig defect —
  but EX-2's point estimate then has no accepted phase to estimate. **Decide the
  retry rule before T5** (STOP-4): recommend *one scored attempt, with any prior
  attempt disclosed and its usage recorded*, because "best of k reported as n=1"
  is the same over-claim EX-6 exists to prevent, one axis over.
- **R-P06-2 — "tokens per accepted phase" is not one number.** The observed
  `usage` splits four ways and is **dominated by cache**: a trivial in-jail
  prompt recorded `input_tokens: 2`, `output_tokens: 4`,
  `cache_creation_input_tokens: 24384`, `cache_read_input_tokens: 18327`. A bare
  "tokens" figure is ambiguous by an order of magnitude, and a system-prompt
  cache floor rides every headless run regardless of the phase. EX-4's own
  discipline applies: **name the number and its source, or the row is an invented
  incumbent** in miniature.
- **R-P06-3 — headless `claude -p` may need a permissions posture the profile
  does not grant.** The A2 smoke proved *credential + egress*, not *file
  mutation*: `print OK` writes nothing. If the agent cannot edit and commit
  without an escape hatch that widens the sandbox, that is a **finding about the
  capsule model** and a `/consult` — not a profile edit made quietly to get a
  green (STOP-5).
- **R-P06-4 — this phase writes into `.doctrine/rfc/025/evidence/**`, which is
  SL-241's own design-target selector and the only design-target `.doctrine/`
  prefix on the heavy fixture.** Heavy is pinned at `4fc9d32f0` where the prefix
  is empty, so commits cannot reach it — the same reasoning that cleared T8.
  Holds only while heavy is not rebuilt (A-P06-3).
- **R-P06-5 — the shared tree.** Other agents commit to `edge` mid-session
  (SL-244 was in flight at handover). Path-limit **the commit itself**, not just
  the add; `git status --porcelain` first and bail on unexpected staged files.
  This bit twice in PHASE-05 and the pathspec caught it both times.

## Decisions

<!-- D-P06-n — minted here, lifted to notes.md at close (sheet convention). -->

- **D-P06-9 — EX-9 ruled: the EVD does NOT suffice; QUE-200 stays `open`, and
  the corpus is corrected in place (operator ruling, 2026-08-03).** T8. The
  answer matches what PHASE-05 expected; **the reason does not**, and the reason
  is the part that had to be right — F-P06-11.
  - **The ruling.** Candidate 3 is excluded on the question's own grounds
    (EVD-008). Candidates 1 and 2 each carry exactly one trusted-side surface the
    other lacks, and only one was measured: M-B's *parse a capsule-authored file*
    is enumerated and tested (EVD-010, four legs); M-A's *run git inside a
    capsule-authored repository* is sampled at two keys and the sample shows it
    is not uniformly defended. The three measured asymmetries all sit on M-B and
    are all **costs**; the one on M-A is a **safety** asymmetry, and the question
    asks for the minimal *safe* mechanism. "M-B has costs" is not "M-A is safe."
  - **Recorded where EX-9 and VA-3 can both read it** — QUE-200's prose (a new
    `## The EX-9 ruling` section) *and* its `[evidence] notes`, both tiers
    consistent. No status flip: `open` is the answer, not a deferral.
  - **The corpus correction is taken NOW, not left for `/reconcile`** — the
    second half of the ruling, and the divergence from F-P06-10's precedent.
    F-P06-10 corrected the *design*, which `/reconcile` owns by definition. Here
    one wrong record is EVD-006, whose accuracy **VA-3 checks**, and another is
    `evidence/README.md` limit 1 — the document `EX-6`/`VT-1` exist to keep the
    go/no-go honest *against*. Writing the anti-over-claim document while
    knowingly citing a false limit inverts the exercise. Four places corrected:
    EVD-006 (both tiers), README limit 1, `notes.md` `:1079` and `:1647`.
  - Rejected: leaving all four for `/reconcile` with T9 carrying a forward
    reference (T9 would then cite a source it knows to be wrong), and rewriting
    EVD-006's `datum` (the *scored outcome* is unaffected — 8 cells still pass;
    what was wrong was the prose explaining what the row did, so the correction
    belongs in prose and notes).
  - **Follow-on for the go/no-go's outstanding work:** bound or eliminate M-A's
    trusted-side git surface rather than sample it further. The residual is a
    universal over git's config space that no probe can exhaustively discharge;
    the tractable form is whether the parent needs to run git in that repository
    at all. **Not a rig change now** — S2/S4 hold.
- **D-P06-8 — `api-cred`'s observable is realigned onto its claim (operator
  ruling, 2026-08-03).** F-P06-8 consulted per S2 — a failed probe is a finding,
  never a quiet rig edit. The row now names the **file**: refused on append,
  truncate and unlink, with the credential shown readable first and a **positive
  control writing successfully beside it**, because the agent home is writable by
  design and a refusal proves nothing against a broken write mechanism. The
  unlink leg is new and is the one a writable directory actually requires —
  without it, replacement-by-removal was unasserted. **All seven rows pass.**
  Rejected: narrowing the profile to a tmpfs at `/agent/.claude/session-env` so
  the old row passed unchanged (pins the rig to an undocumented harness internal
  that will move, and buys nothing the capsule-scoped tmpfs does not already
  give), and carrying `api-cred` into the go/no-go as a changed result (leaves a
  sound property unasserted and a red row explained in prose). The realigned row
  is **strictly stronger** than the one it replaces: the old leg could pass while
  the credential was writable.
- **D-P06-7 — P-C2 IS re-run against the changed profile, and the answer was not
  the expected one.** The handover carried this as an open question with a
  predicted answer: the tmpfs "touches none of P-C2's confinement subjects", so
  the scored rows should stand. **Settled by running it rather than by reasoning
  about it, and the reasoning was wrong** — `api-cred` went red (F-P06-8). The
  prediction was right about the *subjects* and wrong about the *observables*: no
  confinement property changed, but one row was asserting on a proxy that only
  coincided with its claim under the old mount posture. Six of seven rows
  (`write-floor`, `canonical`, `git-creds`, `env`, `escape-git`, `resource`) pass
  unchanged and their scored results stand. **The general rule this pays for: a
  probe whose subject is the mount posture is re-run when the mount posture
  changes, not reasoned about — the cost of running it is minutes and the cost of
  a wrong prediction is a silently stale confinement claim in the go/no-go.**

- **D-P06-1 — EX-8 is discharged BY ITS INTENT, not its letter (operator ruling,
  2026-08-03).** STOP-1 resolved. `go-no-go.md` records that **ASM-007 was
  falsified and is `invalidated`**, replaced by **ASM-008** (the responsibility
  split, `held`), and that **ASM-008 is strengthened, not discharged**. The
  divergence from EX-8's literal wording is stated in the phase record for
  `/reconcile`, which owns design § 9's stale closure bullet. Rejected: satisfying
  EX-8 literally (a false claim in the anti-over-claim document), and deferring
  the whole ASM line to `/reconcile` (leaves EX-8 undischarged and VA-3 with
  nothing to be consistent with). This is also the only option under which VA-3
  passes — `knowledge show ASM-007` renders `invalidated` in both tiers.
- **D-P06-2 — one scored attempt (operator ruling, 2026-08-03).** STOP-4 and
  R-P06-1 resolved. `rig c1b --agent` runs **once, scored**. Any prior attempt is
  **disclosed and its usage recorded** rather than discarded. Rejected: a stated
  best-of-k, because reporting the best of k as n=1 is EX-6's over-claim one axis
  over, and DEC-109 already names the non-determinism as the thing being
  measured rather than a nuisance to average away.
- **D-P06-6 — `--dangerously-skip-permissions` in the worker is ENDORSED
  (operator ruling, 2026-08-03).** Disclosed rather than discovered: I made this
  call myself in `worker-agent.sh` on the reasoning that it is a harness-level
  flag inside an already-confined capsule, not a change to the bwrap boundary,
  and STOP-5 as written arguably covered it. The operator ruled the reasoning
  sound and the flag stands. **The capsule model's confinement claim is
  therefore explicitly that the OS boundary is the boundary** — a harness inside
  it need not re-litigate confinement per tool. Worth carrying into the go/no-go
  as a stated property rather than an implementation detail.
- **D-P06-5 — the read-only agent home is a SETUP OVERSIGHT, not a weakening;
  fix it and re-run (operator ruling, 2026-08-03).** STOP-5 resolved. The
  profile gets a writable `$HOME` — `--tmpfs /agent/.claude` with
  `~/.claude/.credentials.json` ro-bound **inside** it — so the harness can
  create its per-session working directory. Not a weakening on the operator's
  reading and mine: the credential stays unwritable, and canonical, other
  capsules, `~/.ssh` and `~/.gitconfig` all stay absent. The property that
  matters — a capsule cannot modify the trusted-side credential store — is
  preserved exactly. The first run is **disclosed as a prior attempt with its
  usage** (D-P06-2), never discarded.
- **D-P06-4 — a rig defect is not an "attempt", and the boundary is stated
  BEFORE the first run.** D-P06-2 disclosest prior *attempts*; it governs agent
  non-determinism, not a probe that does not work. The line: a run that fails
  **before the agent leg** — provisioning, cold baseline, EX-3, the plant, the
  observed red — is a **rig defect**, fixed and not counted, because no model
  was reached and nothing about the capsule model was measured. A run in which
  `worker-agent.sh` **invoked `claude`** is an **attempt** and is disclosed with
  its usage whatever it returned, including a red one. Decided in advance
  precisely because deciding it after seeing a result is how "best of k" gets
  reported as n = 1.
- **D-P06-3 — the token accounting is reported in full, with the
  phase-attributable subset named (operator ruling, 2026-08-03).** R-P06-2
  resolved. The record carries all four `usage` figures (`input_tokens`,
  `output_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens`) plus
  `total_cost_usd`, and states explicitly which subset is attributable to the
  phase as against the system-prompt cache floor that rides every headless run.
  A single headline "tokens" figure is not published — it would be ambiguous by
  orders of magnitude (F-P06-2).

## Findings

<!-- F-P06-n — newest first, per the PHASE-05 convention. -->

- **F-P06-13 — F-P06-4 was half wrong: the START boundary was fine all along.
  ISS-307's scope should NOT be widened to cover it.** F-P06-4 recorded that
  flipping PHASE-06 to `in_progress` captured `code_start_oid = e43572d4`,
  *"SL-244's commit … with no SL-241 content in it"*, and concluded ISS-307
  bites at both boundaries. Checked at close, which is when the phase's first
  commit is known: **`git rev-parse de0d49a9^` IS `e43572d4`.** The parent of
  PHASE-06's own first commit is that SL-244 commit, because SL-244 committed
  immediately before PHASE-06 started.
  - **A start boundary is exclusive** — `record-delta --start` is defined as
    *"commit-ish for HEAD before the phase's code landed"*. Naming a foreign
    commit there is **correct by construction**, not a defect: the honest start
    is whatever HEAD was, whoever wrote it.
  - **The end boundary is the real one.** `--status completed` captured
    `code_end_oid = ff0290b5` — SL-244's `ISS-310` commit, landed while this
    session worked. That endpoint *is* wrong and *is* ISS-307.
  - **So ISS-307 stays as filed.** F-P06-4's "widen it" recommendation is
    withdrawn. The generalisable shape: *a foreign commit at an **exclusive**
    boundary is not the same defect as a foreign commit at an **inclusive** one*,
    and F-P06-4 reached for the symmetry without checking whether the two
    endpoints have the same semantics. They do not.
  - Foreign **interior** commits remain unexcisable (PHASE-04's situation), so
    the corrected range still contains SL-244 work. That is a separate and known
    limitation, not something the endpoint correction addresses.

- **F-P06-12 — "model-level" grades FIXTURES; design § 9 reads it as grading
  OPERATING SYSTEMS. FOR RECONCILIATION.** Writing the go/no-go's scope section
  forced the altitude vocabulary to be stated precisely, and the design's closure
  bullet does not survive it. § 9 reads *"model-level rows proven portable, env-
  conditional rows outstanding for macOS"*, which pairs `model-level` against
  macOS and so reads as a claim about **host platforms**. It is not one:
  `model-level` means **the row held on both fixtures** (design § 5.4 / A-3), a
  claim about **client-project shape**. `evidence/README.md` limit 5 and
  `measurements.md`'s closing line both state the narrow meaning — *"the rig ran
  in-jail on Linux/bwrap throughout; nothing here is portable to macOS without
  re-measurement"* — so the design is the odd one out, not the evidence.
  - **Why it matters rather than being pedantry:** taken literally, § 9 says
    fifteen of sixteen rows are proven portable to macOS. Nothing in the spike
    measured a single row off Linux. This is the exact over-claim § 9's own
    closing sentence — *"writing the scope in is what stops the REV
    over-claiming"* — exists to prevent, sitting inside the instruction.
  - **The go/no-go uses the narrow reading throughout** and says so in § 1, so
    the artefact is correct; it is design § 9's wording that needs the edit.
    `/reconcile` owns it. Third § 9 item, after F-P06-10's row 2 and row 5.

- **F-P06-11 — the `upload-pack` worry WAS exercised, and the corpus says four
  times over that it was not. What is actually untested is a different and
  sharper thing. FOR RECONCILIATION.** T8 went to rule on EX-9 and found the
  stated obstacle is not the real one.

  **(a) The corpus contradicts itself.** `instantiations.sh:635–641` plants
  `uploadpack.packObjectsHook` in the capsule repo *deliberately*, and says so:
  *"`upload-pack` runs this INSTEAD of `pack-objects` when serving a fetch, i.e.
  as a child of the trusted side's own `git fetch` on M-A … the row OBSERVES
  that defence rather than trusting the documentation for it."* `notes.md:150`
  records it correctly. But **EVD-006, `evidence/README.md` limit 1,
  `notes.md:1079`, `notes.md:1647` and `phase-sheets/phase-05.md:1667` all state
  that `upload-pack` running in the capsule repo's context *"was never
  exercised"*.** It ran on every M-A cell — verified, not inferred:
  `GIT_TRACE=1 git fetch -- <local-path>` spawns `git-upload-pack '<path>'`
  which spawns `pack-objects` (git 2.54.0, in-jail). Four scored H6 cells, live
  payload, positive control.

  **(b) The two planted keys did not hold for the same reason, and only one of
  them is a defence.** Measured directly this session, not reasoned about
  (D-P06-7's rule):

  | key | honoured from repo config? | why it did not fire trusted-side |
  |---|---|---|
  | `uploadpack.packObjectsHook` | **no** — protected-config only | **git defended it.** QUE-200's named worry, observed holding |
  | `core.fsmonitor` | **YES** — fires on index refresh | **nothing triggered it.** `rev-parse` does not refresh an index; `git status` fires it (observed, twice) |

  So H6's green is one part defence and one part *the M-A harvester happening not
  to run an index-touching command in the hostile repo*. `harvest-fetch.sh` runs
  exactly three git operations against the capsule's clone — `rev-parse` (`:57`,
  `:66`) and the `fetch` that spawns `upload-pack` (`:73`). Add a `status`,
  `diff`, `add` or `checkout` there and the same passing plant becomes trusted-side
  arbitrary execution. **H6 would catch it** — `c3_assert_never_ran_in` reads the
  execution log — so the row is a live regression detector for the hazard, which
  is the mitigation worth recording alongside the fragility.

  **(c) The dissolution's stated reasoning is sound but incomplete.** *"Config
  and hooks are repo-local, never objects"* covers what **travels**. It does not
  cover the trusted side **going to** the hostile config: on M-A the parent runs
  git three times with the capsule's `.git/config` on its effective cascade. On
  M-B it runs git in the capsule repo **zero** times (`harvest-bundle.sh` reads a
  flat file; grep for `git -C "${clone}"` returns nothing). The asymmetry is
  real, it is M-A's, and it is the mirror of EVD-010's.

  **What the limit should say** — narrower than the current wording and no longer
  false: the one documented lethal vector was exercised and git's protected-config
  defence was **observed** holding (git 2.54.0); the universal QUE-200 states —
  *"git's protected-config rules cover everything a hostile repo config can do to
  upload-pack"* — is **not** discharged by two keys, one of which turns out not to
  be protected at all. `/reconcile` owns EVD-006, README limit 1, and the two
  `notes.md` passages.

- **F-P06-10 — counting the measurement rows corrected the design's own list.
  FOR RECONCILIATION.** Design § 9's row 2 parenthetical reads *"coord branch,
  projected refs, candidate branch, `candidates.toml`, trunk"*. Counted against
  REQ-311's ref taxonomy and the write sites, it is wrong twice:
  - **`candidates.toml` is a FILE, not a ref.** It cannot appear in a count of
    "mutable refs written".
  - **It conflates per-phase with per-slice.** `dispatch/<N>`, the worker fork
    branch and `phase/<N>-NN` are attributable to one accepted phase;
    `review/<N>`, `candidate/<N>/<label>`, trunk and `edge` are slice-level and
    amortise over *k* phases. The difference is 3 vs 7 depending on how *k*
    falls, so quoting the list would have published a number with no fixed
    meaning.
  The sheet anticipated exactly this — *"the design's parenthetical list is a
  starting point, not a count, and it must be checked against the code rather
  than quoted"*. **`/reconcile` owns design § 9's row-2 wording**; the table
  states the corrected enumeration and its derivation.
  - Row 5 carries a smaller version of the same: the honest contrast is the
    **kind** of git operation, not the count. The incumbent materialises a tree
    and stages into a shared index (ISS-234's hazard class); the capsule model
    never materialises one. A row reporting "4 vs 2" describes the wrong axis.

- **F-P06-9 — the re-run is a REAL phase. EX-1 and EX-2 are discharged, and the
  discriminator that exposed attempt 1 is what certifies attempt 2.**
  `probe-c1b.sh --keep`, `SPIKE_WORKER_MODE=agent`, 2026-08-03, after the
  D-P06-5 profile fix. The one line that separates a phase from a void green:

  | | attempt 1 (void, F-P06-7) | attempt 2 (SCORED) |
  |---|---|---|
  | `p-c1b-ritual` | `agent-committed=no tree-dirty=yes` | **`agent-committed=yes tree-dirty=no`** |
  | phase step | 142.2 s, 18 turns of failing tool calls | **51.1 s** |
  | `output_tokens` | 8 131 | **3 415** |
  | `cache_creation_input_tokens` | 25 872 | **15 079** |
  | `cache_read_input_tokens` | 388 449 | **195 816** |
  | `input_tokens` | 24 | **15** |
  | `total_cost_usd` | 0.6571435 | **0.33495199999999997** |

  - **The agent committed its own work.** `9872a712` — *"[add] even cent
    splitting, remainder to the earliest parts"*, `src/split.ts`, +25, with a
    `Co-Authored-By` trailer. Exactly one commit beyond the plant, and its
    message is the agent's, **not** the worker's residue-sweep wording. The
    residue sweep did not fire: the tree was clean.
  - **`git commit` IS shell execution**, so the fact that discharges EX-1's
    ritual clause is the same fact that proves the harness had a working shell.
    The leg added to `probe-smoke.sh` predicted this run and would have caught
    its absence for a fraction of the cost.
  - **A degraded agent is not a cheap one — it cost roughly DOUBLE.** Attempt 1
    burned 2.4× the output tokens and 2.0× the cache reads of the run that
    actually did the work, for a phase it never executed. Worth carrying into
    the go/no-go: failure has a cost curve of its own, and it points the wrong
    way from the intuition that a blocked agent gives up early.
  - Pipeline clean: all four stages `pass`, no token minted, exactly one
    canonical ref changed, accepted ref at the pinned OID. Peak disk 757 760 B
    worker / 667 648 B verify (absolutes, VA-2). Tokens carry
    **`capsule-reported`** in the row itself, per F-P06-5, and the n = 1 caveat
    travels with the number.

- **F-P06-8 — P-C2's `api-cred` row asserted a PROXY, and the tmpfs pulled the
  proxy apart from the claim.** Re-running P-C2 after the profile fix (D-P06-7)
  returns `api-cred FAIL`. The row's stated claim is *"the capsule cannot rewrite
  the credential"*; its observable (`probe-c2.sh:260-261`) is
  `rig_assert_fails … 'printf x >>/agent/.claude/.capsule-write-probe'` — a write
  to a **different file, in the credential's directory**. Read-only-directory was
  standing in for read-only-credential, and the two were indistinguishable only
  because the whole of `~/.claude` was ro-bound.
  - **Directly measured under the new profile, before any edit:** appending to
    `.credentials.json` is **refused**; truncating it is **refused**; unlinking it
    is **refused**; reading it **succeeds**. Creating an unrelated file in
    `/agent/.claude` succeeds — by design, that is the harness's session dir.
    **D-P06-5's premise holds exactly**; the property the operator ruled on is
    intact and is now, for the first time, directly observable.
  - **So the red is the row's, not the profile's.** The old row could pass while
    the credential was writable (a directory can be ro with the file bind-mounted
    rw over it), and now fails while it is not. It never asserted its own claim.
  - Same class as **F-P06-6**, one probe over: an assertion on a *dependency* of
    the property standing in for the property. The rig found its own instance of
    the pattern it had just been bitten by.
- **F-P06-7 — the scored run reached green, but NOT by executing a phase. The
  agent had no shell at all.** `rig c1b --agent`, 2026-08-03, exit 0, every
  assertion holding — and the result is not what it reads like. The agent's own
  final message: *"shell execution is unavailable in this session … `EROFS:
  read-only file system, mkdir '/agent/.claude/session-env/<session-id>'` … I
  could write the file, but I could **not** run `npm test`, `npm run lint`, or
  `git commit`."* It wrote `src/split.ts` by reading the tests and reasoning
  through them by hand, traced each case in prose, and was **correct** — so
  `npm test` passed when the probe ran it trusted-side afterwards.
  - **EX-1's assertion passed for a reason unrelated to EX-1.** "The suite
    reaches green in-capsule" is literally true. "A real agent executes a real
    red→green phase, the worker commits freely" — the plan's actual words — did
    not happen: nothing was executed by the agent, and it committed nothing.
  - **`agent-committed=no tree-dirty=yes`.** The `worker_commit`-style residue
    sweep in `worker-agent.sh` is the only reason there was anything to harvest.
    Recording the ritual separately from the commit is what kept this visible
    instead of being absorbed into a green — the one design choice in the worker
    that earned its keep.
  - **Not a rig defect in the D-P06-4 sense, and not agent non-determinism
    either.** `claude` was invoked, so it is an attempt and is disclosed. But
    what it measured is a **degraded agent**, and the numbers describe 18 turns
    of failing tool calls, not a phase.
- **F-P06-6 — the A2 smoke certified a profile that cannot carry the
  workload.** `probe-smoke.sh` proved *credential* and *egress* with
  `claude -p 'print OK'` — a prompt that needs no tools — and both legs are
  green in `probes/smoke/results.tsv`. It never proved the agent can **work**.
  The gap is structural, not incidental: the harness creates a per-session
  working directory under `$HOME` before any Bash tool call runs, and the
  profile ro-binds the agent home (`sandbox.sh:225-228`, `HOME=/agent`).
  R-P06-3 predicted exactly this at plan time — *"the A2 smoke proved credential
  + egress, not file mutation: `print OK` writes nothing"* — and it landed.
  **`probe-smoke.sh` already split credential from network on this very
  reasoning** (A8: "distinct failure modes and a single test conflates them").
  The same argument extends one step further, to *can it work*, and was not
  taken. A third A2 leg is the cheap fix and it belongs in the rig regardless of
  what happens to P-C1b.

- **F-P06-5 — the token measurement is the ONE number with no trusted-side
  source, and the rig's own discipline says so.** `probe-c1a.sh:82–85`:
  *"Wall clock is taken TRUSTED-SIDE … and never read out of anything the capsule
  wrote … a measurement parsed from it would be the one place in the rig where
  capsule content decides a recorded value."* Token usage **cannot** meet that
  bar: the harness that counts tokens is `claude`, `claude` runs inside the
  capsule (that is EX-3's entire point), and no trusted-side observer exists.
  Two mitigations, both taken, neither a fix:
  1. **Stream, don't file.** The JSON leaves over the worker's **stdout** and is
     captured trusted-side by the probe. It never becomes a file in the capsule's
     rw root, so the capsule cannot revise it after the fact — the parent holds
     the only copy from the moment it is written.
  2. **Nothing gates on it.** It is recorded, never asserted. No stage, token,
     outcome or altitude reads it, so I5 ("nothing the capsule says decides
     anything") is intact — the number is evidence about cost, not a verdict.
  The residue is real and goes in the measurement table verbatim: **tokens are
  capsule-reported**. A capsule that wanted to lie about its cost could. It is
  the honest column header, and it is what EX-4's "name both sources" rule
  produces when one source is inside the boundary.

- **F-P06-4 — ISS-307 bites at the START boundary too, not only the completion
  flip.** Flipping PHASE-06 to `in_progress` captured
  `code_start_oid = e43572d4` — **SL-244's `design(SL-244): DEC-124` commit**,
  the shared tree's HEAD at that instant, with no SL-241 content in it. ISS-307
  was filed against `--status completed` capturing a foreign `code_end_oid`; the
  same `HEAD`-capture applies to `in_progress`, and there is no correction
  available *at* the flip because the phase has no commit of its own yet. Fix at
  close, when the phase's first commit is known: the honest start is that
  commit's parent. Widens ISS-307's scope — worth adding to it rather than filing
  anew.

- **F-P06-1 — the phase entry understates the phase: `probe-c1b.sh` and
  `worker-agent.sh` do not exist.** `rig:151` dispatches `probe-c1b.sh`;
  `ls scripts/spike-capsule/control/` has no such file, and `capsule/` holds
  `worker-{stub,hostile}.sh` but not `worker-agent.sh` though design § 9.1's tree
  names all three. The plan's objective reads as a writing phase ("fill the
  table, land a go/no-go"); the entrance criterion (EN-1) is met and nothing is
  blocked, but the effort shape is rig-build-then-write. Recorded so the estimate
  is not inherited from the objective's wording.
- **F-P06-2 — `claude -p` usage is cache-dominated, so "tokens" is
  under-specified.** See R-P06-2 for the measured numbers. The design's
  measurement row says "tokens per accepted phase … n = 1"; it does not say
  *which* tokens, and the four available numbers differ by ~4 orders of
  magnitude between `input_tokens` and `cache_creation_input_tokens`. EX-4's
  "name the source" rule is the answer, applied one level finer than the row.
- **F-P06-3 — `--output-format` is undocumented in this `claude --help`.** The
  flag works (verified by an in-jail run returning a `result` message with
  `usage`/`modelUsage`/`total_cost_usd`), but it appears in the help text only
  inside *other* options' prose (`--include-partial-messages`, `--input-format`),
  never as its own option row. Cite the observed run, not the help.
