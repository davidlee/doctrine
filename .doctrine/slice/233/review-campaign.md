# SL-233 — implementation review campaign

Scope decision for the `/audit` stage's code-review leg. Authored because it
outlives the session that wrote it: this is more work than one agent context can
hold, and the next agent needs the ruling, not the re-derivation.

Settled 2026-08-02. Method follows `IMP-024` (large-review funnel), which was
written from `RV-324` — the only prior instance of a review this shape.

---

## 1. The measured surface

Per-phase code delta, from `.doctrine/dispatch/233/boundaries.toml`
(`git diff --shortstat <code_start_oid>..<code_end_oid>`):

> **Corrected 2026-08-02 at S3.** This section originally cited the boundary
> file at `.doctrine/state/slice/233/boundaries.toml`, where it does not exist;
> S2 concluded from that it did not exist at all and derived its ranges from the
> log by hand. It does exist, at `.doctrine/dispatch/233/boundaries.toml`, and it
> is the authoritative record. Three things a later session needs to know:
>
> - **It covers only the eight funnel-dispatched phases** — 01–06, 13, 14. A
>   phase that landed via direct `feat` commits (07, 08, 10, 11, 12, 15, 16) is
>   simply absent, and its range must still be derived from the log. That is not
>   a defect in the file; `provenance = "funnel"` says so on every row.
> - **The churn column below runs 16 low for the two funnel phases S3 measured**
>   (PHASE-05, PHASE-13). The gap is entirely
>   `.doctrine/dispatch/233/funnel.toml`, a running bookkeeping file that sits
>   outside every review pathspec. The **file / src / test counts are exact**, so
>   they — not churn — are the wrong-base alarm worth automating.
> - **S2's PHASE-04 range was a superset of the recorded one.** S2 used
>   `b0e5e9080..cbab2841e` (22 files); boundaries.toml records
>   `b0e5e9080..228a7ed32` (21); this table says 20. This does not overturn any
>   S2 conclusion — a line ruled out of range against the *wider* window is out
>   of range against the narrower one too — but PHASE-04 is the one phase in the
>   campaign whose reviewed window was never reconciled to the record.
>
> S3's ranges were checked against this table's file/src/test counts before its
> raisers were spawned, and that check is now executable rather than manual:
> `campaign/s3/gen.py` refuses to emit a prompt whose range does not reproduce
> the row, and refuses an empty diff explicitly. S4 should reuse it.
>
> **Corrected 2026-08-02 at S4 — the PHASE-14 row was measuring one commit, not
> the phase.** S4 ran the range check over **all eight** `boundaries.toml` rows
> rather than only the ones it needed. Seven reproduce their row exactly.
> PHASE-14 did not: the recorded range `51a74c231..b18b3c157` yields **10 files
> / 7 src / 2 test / 1,217 churn**, against the 4 / 3 / 1 / 205 originally
> tabulated. Those four numbers are precisely the stats of a **single commit** —
> `0c0fe4135` *"PHASE-14 shim live-run arm via the commands tier"*, the phase's
> last `feat`. The whole worker import `c744e58bd` (9 files, 6 `src/`, 2
> `tests/`, 1,014 churn) was absent from the table. The row is corrected below;
> the total, the dark share, and §6's S4 churn are corrected with it.
>
> Two things this does **not** mean. `boundaries.toml`'s PHASE-14 row is
> **right**: the two funnel commits after `b18b3c157` (`befe4934c`, `e93289bd6`)
> touch only `boundaries.toml` and `funnel.toml`, so no code lies outside the
> recorded range. And PHASE-04's row remains the *other* known discrepancy —
> one file, already recorded above — for an unrelated reason.
>
> The lesson generalises past this row. The S3 note told S4 to **trust the
> file/src/test counts and not the churn column**; on this row all four were
> wrong together, and only measuring every row showed which one to disbelieve.
> `check_range` is what turned a silent scope error into a refusal — but its
> refusal would have read as *wrong base*, not *wrong table*, so a session that
> trusted the doc over the measurement would have "fixed" the range instead.
> **When `check_range` refuses, measure the other rows before editing the
> range.**

| phase | files | src | test | churn | code-review status |
|---|---|---|---|---|---|
| PHASE-01 | 87 | 0 | 0 | 1,587 | dark — **not code** |
| PHASE-02 | 13 | 11 | 0 | 1,781 | dark · design-gated `RV-320` |
| PHASE-03 | 19 | 17 | 1 | 4,247 | **covered** `RV-321` |
| PHASE-04 | 20 | 15 | 4 | 3,279 | dark |
| PHASE-05 | 10 | 8 | 1 | 1,845 | dark |
| PHASE-06 | 9 | 6 | 2 | 837 | dark · design-gated `RV-323` |
| PHASE-07 | 19 | 4 | 3 | 1,250 | dark |
| PHASE-08 | 22 | 4 | 4 | 1,034 | dark · design-gated `RV-325` |
| PHASE-09 | 10 | 0 | 1 | 1,513 | dark — **not code** |
| PHASE-10 | 12 | 10 | 2 | 1,489 | **covered** `RV-324` |
| PHASE-11 | 5 | 3 | 2 | 698 | **covered** `RV-324` |
| PHASE-12 | 12 | 10 | 2 | 1,332 | **covered** `RV-324` |
| PHASE-13 | 10 | 7 | 2 | 1,168 | dark |
| PHASE-14 | 10 | 7 | 2 | 1,217 | dark · **row corrected at S4** |
| PHASE-15 | 18 | 6 | 4 | 1,092 | dark |
| PHASE-16 | 27 | 15 | 9 | 3,087 | dark |
| | | | | **27,456** | |

Covered by an implementation review: 7,766 churn (28%). Dark: 19,690 (72%).

**The design surface is larger than the code surface, and that is the campaign's
defining fact.** For the three phases whose `EN-2` sketch gate was adversarially
reviewed:

| gate | sketch | sketch B | ledger | ledger B | findings | rounds |
|---|---|---|---|---|---|---|
| PHASE-02 | `projection-bounds.md` | 63,643 | `RV-320` | 24,788 | 7 | 33 |
| PHASE-06 | `marker-grammar.md` | 103,996 | `RV-323` | 25,417 | 6 | 30 |
| PHASE-08 | `thin-adapter.md` | 113,349 | `RV-325` | 104,618 | 17 | 59 |

PHASE-08 carries 218 KB of settled design against 1,034 lines of code.

---

## 2. The ruling: two review kinds, and they do not mix

**`RV-320` / `RV-323` / `RV-325` are DESIGN reviews. They gated sketches, not
implementations.** They therefore buy this campaign **zero** code coverage — but
they buy something better, and it is the leverage the campaign is built on.

### Kind A — conformance to a reviewed design (PHASE-02, 06, 08)

Not "is this code good" but **"does the landed code honour what 33 / 30 / 59
turns of adversarial review actually settled"**. Highest value per line, and
priority: the design bar was set deliberately high here and nothing has checked
the code against it.

The leverage: **the 30 verified findings across `RV-320` + `RV-323` + `RV-325`
are already a conformance checklist.** Each is an adjudicated, specific claim
about what the implementation must do — pre-argued, pre-disposed, and *stated*
rather than inferred. Checking landed code against a stated claim is
mechanically re-verifiable work, which is exactly what `IMP-024` §1 says the
cheap tier may safely be given.

Beyond the findings, each sketch carries commitments no finding contested; those
need a reading pass, not a checklist.

- churn 3,652 · 21 src files · 30 pre-adjudicated assertions
- ledger: **one**, facet `implementation`

### Kind B — ordinary code review of the dark phases (04, 05, 07, 13, 14, 15, 16)

No design gate, no code review. Ordinary correctness / quality review.

- churn 11,926 · 58 src files
- ledger: **one**, facet `code-review`

### Neither — PHASE-01 and PHASE-09 are not code

PHASE-01 is 87 files and **zero** `src/`: governance descent. PHASE-09 is the
evaluation kit, also zero `src/`. Both are authored prose. They belong to the
audit's spec-coherence and conformance legs, **not** to the review funnel, and
sending them to a code reviewer would burn a tier on the wrong question.

### Why two ledgers and not one

`IMP-024` §3: dedup is a whole-set operation and is not parallelisable, so
splitting a ledger multiplies duplicates. That argues for one. Against it: the
two kinds ask different questions, rest on different evidence, and dispose
differently — a Kind A non-conformance may be the *criterion's* problem, and
**criteria are the owner's** (cf. the standing `VA-4`/`EX-8` F-10 tension). Two
ledgers keeps dedup whole *within* each kind while stopping a design dispute
from being dispositioned as a code defect. Cross-kind dedup is handled by the
shared corpus in §3.

---

## 3. Session 0 is the dedup corpus, and it is the highest-ROI pass

`IMP-024` §5 names this the single highest-return step: on `RV-324` it
compressed 81 KB of phase sheets into 36 lines of prior findings that every
downstream raiser reused. Here the input is much larger — 16 runtime phase
sheets, `notes.md` (167 KB) `### Learned` + `### Open`, and 293 KB across six RV
ledgers.

Build **once**, inject into **every** raiser. Without it each raiser
re-discovers, and re-reports, defects that are already found, already disposed,
or already deliberately left unrepaired — of which this slice has an unusual
number:

- `ISS-289` — PHASE-15 `VT-2` unmet, an audit judgement left unrepaired. Any
  raiser running the suite will find it. Pre-empt it.
- `ISS-290` — `StepDischarged` declares `Outcome` as `Token`, `run.rs:1442`
  builds a `label`. Deliberately unrepaired pending a ruling.
- `ISS-291` — `SKILL.compact.md` still teaches the eight-stage machine.
- The `VA-4` / `EX-8` round-4 (F-10) tension — flagged, not amended.
- `IMP-373` / `IMP-374` / `IMP-376`.

**These six resolve only from `/workspace/doctrine`** — see §5.

Output: one file, prior-findings list, `<40` lines, cited by id.

---

## 4. Tiering and spawn

Per `IMP-024` §1, the load-bearing rule is **give the cheap tier only work whose
output can be mechanically re-verified** — not the easy parts, the parts where
being wrong is *detectable*.

| tier | vehicle | gets |
|---|---|---|
| cheap | `scripts/pi-review.sh` — confined, read-only (`--ro-bind / /`), no worktree fork | census, enumeration, correspondence tables, Kind A's finding-by-finding checklist |
| top | **codex MCP** | adjudication, subtle Kind A judgement, anything about intent |

`pi-review.sh` passes no `--model`, so pi's configured default is the *only*
tier reachable through it — by design. Top-shelf never goes through pi.

Two enforcement rules, both non-negotiable and both proven on `RV-324`:

- **every claim ships the command that reproduces it**; a claim without one is
  discarded unread;
- **every negative result ships a positive control.** Four of four zero-hit
  results were wrong before this rule; it bit twice more in this slice's own
  PHASE-08/09 sessions (`mem.pattern.harness.grep-negative-needs-positive-control`).

**Evidence and findings are different kinds** (`IMP-024` §2). Cheap raisers emit
*candidate* findings to a staging dir; promotion to the ledger is a judgement
act by the orchestrator. The top tier receives the cheap tier's tables framed
explicitly as *evidence to verify, not conclusions to trust*.

**Raising is serial**, by the orchestrator, after raisers return — the ledger is
append-only with derived status and N concurrent `review raise` calls contend.

Spawn shape (the script lives in the primary tree; `REVIEW_ROOT` points it at
the code):

```bash
REVIEW_ROOT=/workspace/doctrine/.dispatch/SL-233 \
  /workspace/doctrine/scripts/pi-review.sh <LABEL> <PROMPT_FILE> <OUT_DIR>
```

Do **not** use harness subagents. Note `pi-review.sh` is not on `dispatch/233` —
invoke it by absolute path.

### 4.1 What the cheap tier actually is, and the `CHR-051` caveats

**It is not a cheap model.** `pi-review.sh` passes no `--model`, so pi resolves
`~/.pi/agent/settings.json` → `deepseek-v4-pro`, and it passes
`--thinking "${PI_THINKING:-low}"`. So the tier knob is **thinking level, not
model**: the cheap tier is *pro at low thinking*. Verified 2026-08-02, not read
off the script's header comment — `CHR-051` §3 exists because two header
comments named the wrong model.

Raise `PI_THINKING` for Kind A's conformance judgements; leave it at `low` for
census and correspondence work.

`CHR-051` records three defects in the pi spawn surface. **`pi-review.sh` is
clean of all three** — it carries the poll and reap fixes (`a69274c8`,
`299b6a55`) and uses no agent profile. Three things still bite the operator:

- **Log size is not a progress signal.** `pi --mode rpc` re-serializes
  accumulated state per event, so 50–150 MB for one ordinary turn is normal.
  Reading that as a runaway killed four in-flight raisers during `RV-324`, two
  of which had already written their findings. Watch for the `<LABEL>.md` file
  and the script's own `terminated reason=` line.
- **Do not reach for `setsid`** if the reap ever needs touching. It is the
  obvious fix, it is absent from this jail, and it breaks the spawn outright.
  `set -m` + `kill -9 -"$PI"` is the working form and is already in place.
- **`scripts/pi-spawn-confined.sh` and `pi-spawn.sh` are NOT patched.** If any
  part of the campaign reaches for those instead, it inherits defects 1 and 2.

**`pi-scout` / `pi-research` are model-inverted and must not be used for
campaign threads.** Confirmed still live 2026-08-02 — and the inversion is
**project-local**: the scripts parse `$PROJECT_ROOT/.pi/agents/*.md`, not
`~/.pi/agents/`, and only the project copies are wrong. A judgement thread sent
to `pi-research` silently gets flash. Detail and the one-line fix are in
`CHR-051`.

---

## 5. Venue — which tree, per artefact kind

The corpus is split and an id minted in the wrong tree collides silently
(`ISS-279`). Measured high-water marks, 2026-08-02:

| artefact | tree | why |
|---|---|---|
| reading / evidence | **coord** (`.dispatch/SL-233`) | the code is here; review is read-only |
| **RV ledgers** | **primary** | coord's next RV is 326; `edge` holds 326–342 (S0 minted 341/342 there). And `ISS-277`: `reseat` **cannot** renumber a review, so an RV collision is unrepairable by tooling — this is the `RV-320` incident, which the owner fixed by hand. **This rule is a convention you must hold yourself, and nothing enforces it.** `resolve_review_root` (`src/review.rs:2014`) refuses only a worktree *fork* and otherwise returns the local root — a coordination tree is an explicitly *legitimate* review venue (`ISS-275`/`DEC-094` made the test the role, so ADR-012's design-gate reviews can live there). So a review verb run from coord writes to **coord**. Pass `-p /workspace/doctrine` on every campaign `review` verb, and check which tree you are in first |
| backlog items | **primary** | coord's next issue is 289; `edge` holds 289–292 |
| knowledge records | **primary** | already collided — `DEC-099`…`DEC-104` |
| slice artefacts | **coord** | they are the slice |

Mint in the primary tree with `-p /workspace/doctrine`; commit by exact leaf
path, `git status --porcelain` first. Primary stays on `edge`.

---

## 6. Session plan

| # | scope | churn | output |
|---|---|---|---|
| **S0** | dedup corpus (§3) + open both ledgers | — | `prior-findings.md`; Kind A = `RV-341`, Kind B = `RV-342` |
| **S1** | **Kind A** — PHASE-02/06/08 vs `RV-320`/`323`/`325` + their sketches | 3,652 | Kind A ledger |
| **S2** | Kind B — PHASE-04 + PHASE-16 (the heavy pair, 30 src files) | 6,366 | Kind B ledger |
| **S3** | Kind B — PHASE-05 + PHASE-07 + PHASE-13 | 4,263 | Kind B ledger |
| **S4** | Kind B — PHASE-14 + PHASE-15, then the 13 undeclared / 11 undelivered conformance entries (`src/commands/verify.rs`, `tests/runbook_fixture/mod.rs`) | 2,309 | Kind B ledger + dispositions |

Calibration: `RV-324` covered 4,200 churn / 19 files with 8 raisers in one
session. S2 is the largest at 6,366 and may want splitting; S4 is light and
folds naturally into the audit's conformance leg.

S1 first — it is the priority, and it is the session most likely to change what
the others look for.

### 6.1 The two-pass method, from S2 onward

S2 changed the method and `IMP-024` §4a/§4b now carry it: **every raiser runs
twice, concurrently, and the two passes are read for DISAGREEMENT.** Two passes
from one model are correlated samples — they buy variance, not bias — so
disagreement is strong signal and agreement is weak evidence. A second pass is a
triage instrument that says where to spend expensive attention, not a
confirmation device. Read `IMP-024` §4a and §4b before running a session; the
body lives in the **primary** tree (the `dispatch/233` copy of
`backlog-024.md` is empty).

### 6.2 Ranges as run

| session | phase | range | source |
|---|---|---|---|
| S2 | PHASE-04 | `b0e5e9080..cbab2841e` | derived by hand; a superset of the recorded row (see §1) |
| S2 | PHASE-16 | `894045566..63b348de6` | derived by hand; absent from boundaries.toml |
| S3 | PHASE-05 | `cb45bc958..ba2d9ba9a` | `boundaries.toml`, verified 10/8/1 against §1 |
| S3 | PHASE-07 | `4435904cd..35ef61cac` | derived; absent from boundaries.toml; reproduces §1 exactly (19/4/3/1250) |
| S3 | PHASE-13 | `03339c62a..5acb8d304` | `boundaries.toml`, verified 10/7/2 against §1 |
| S4 | PHASE-14 | `51a74c231..b18b3c157` | `boundaries.toml`; verified 10/7/2 — and it is §1's row that was wrong, not the range (see §1) |
| S4 | PHASE-15 | `7a8a7fe52..540aed959` | derived; absent from boundaries.toml; reproduces §1 **exactly**, all four numbers (18/6/4/1,092) |

**How PHASE-15 was derived**, since nothing records it. PHASE-15 is *"RV-324
remediation"* — it has no `feat(SL-233): PHASE-15` commits at all, which is why
a subject-line search for it comes back empty and looks like absence. Its work
is the `fix(SL-233): RV-324 F-n` / `RV-324 F-2 re-dispositioned` run. The base
`7a8a7fe52` is `plan(SL-233): settle PHASE-15 EX-5`, the last commit before the
phase opens; the tip `540aed959` is `fix(SL-233): F-1 — an accepted proposal's
checkpoints run the same protocol`, its last code commit. Everything after it up
to the `0c57cbea0` completion harvest is `review`/`doc`, which is why widening
to the harvest inflates the row to 22 files.

### 6.3 The rig

`.doctrine/state/slice/233/campaign/s3/{template.md,gen.py}` supersedes S2's.
Two S2 traps are dissolved in it rather than restated:

- **The pass-B output-path trap.** The template hardcodes its own output path
  from `__LABEL__`, so S2's practice of `sed`-ing a B copy from the A prompt
  meant a raiser spawned `foo-b` silently overwrote `foo.md`. `gen.py` now
  renders **both** passes from the template directly, each with its own label, so
  the path is right by construction; `verify_pair()` then asserts the pair
  differs on exactly one line and that the line is the output path.
- **The wrong-base trap.** `check_range()` refuses to emit a prompt whose range
  does not reproduce §1's file/src/test counts, and refuses an empty diff by
  name.

The template also gained, from S2's post-mortem: an explicit **scope gate** (a
finding must ship the `git diff <range> -- <file>` proving its line is in range),
an instruction that a **later-repaired defect is history, not a severity**, and a
warning that `grep` with `|` is a literal pipe in a basic regular expression —
two of S2's *clean* verdicts rested on evidence that could not have run.

### 6.4 Outcome — the campaign is complete

All five sessions ran. **`RV-341`** (Kind A) carries 3 findings; **`RV-342`**
(Kind B) carries 8 — F-1/F-2/F-3 from S2, F-4/F-5/F-6 from S3, F-7/F-8 from S4.
Both ledgers are `await=responder`. S4's conformance half is dispositioned in
`conformance.md`.

Two debts are deliberate and still open, and neither is closed by the campaign:

1. **`RV-341` has had no top-tier pass.** Its two cheap "clean" reports are
   unconfirmed and one was already overturned by hand. S2, S3 and S4 all argue a
   second *cheap* pass is the cheaper first move.
2. **Eight of `RV-325`'s seventeen findings are uncheckable by a code funnel** —
   they promised criterion amendments or sketch text. They route to the audit's
   spec-coherence leg (`conformance.md` §4).

**What S4 adds to the method**, beyond `IMP-024` §4a–§4d:

- **A wrong table and a wrong base produce the identical `check_range` refusal.**
  S4's PHASE-14 row was wrong in all four numbers at once, against a standing
  instruction to trust three of them. Measuring *every* row — seven reproduced,
  one did not — is what identified the table rather than the range. §1 records it.
- **"Later-repaired is history, not a severity" cuts both ways.** S3 had a pass
  rate a repaired defect `major`. S4 had a pass rate a **live** defect `history`
  (`RV-342` F-7), reading a renumbering sweep as a deletion. The settling move is
  identical in both directions: read current state, do not reason about it.
- **A dedup corpus can suppress a real finding class.** `prior-findings.md`'s
  `XQUE-177` entry was false at head; a raiser obeying it would have stayed
  silent. Corrected, with the measurement. A corpus needs re-testing, not just
  re-reading.
- **An undelivered selector is not a defect.** All six of SL-233's real ones
  dissolved on measurement (`conformance.md` §2b). Four of the six had a
  plausible-sounding defect story that measurement refuted.

## 7. Out of scope

- Re-opening the six terminal RV ledgers. All are `done · await=none`.
- Fixing `ISS-289` / the `VA-4`-`EX-8` tension. Dispositions, not defects.
- Integration. Blocked on the five outstanding `DEC` moves — see `notes.md`
  `## Harvest` → `### Open`.
