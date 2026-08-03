# Review RV-343 — reconciliation of SL-241

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Close-out audit of **SL-241 — Capsule spike rig**, opened at lifecycle `audit`
with 6/6 phases `completed` and every EX / VT / VA / VH criterion recorded
discharged. Facet `reconciliation`, mode **conformance**, self-audit (both roles
driven via `--as`).

**Surface reviewed.** The slice ran **solo on the shared `edge` tree**, not
through `/dispatch` — there is no candidate interaction branch and no
`review/*` / `phase/*` evidence ref. The audit therefore reviews the authored
tree at `edge` head `1cc373ff`, plus the committed evidence corpus under
`.doctrine/rfc/025/evidence/**`.

### What this audit probes

The slice's deliverable is **evidence, not product** — the rig is disposable
scaffolding and nothing in it migrates into dispatch machinery. So the usual
"does the code match the design" question is secondary. The live question is
**does the recorded verdict claim exactly what the evidence supports, no more**,
and design § 9 exists precisely to hold that line. Four lines of attack:

1. **Does the deliverable over-claim?** `.doctrine/rfc/025/go-no-go.md` against
   `evidence/README.md`'s seven limits and `measurements.md`'s closing section.
   The verdict is *scoped*; the scope is the substance, and a reader who takes
   "go" without it has taken something the spike did not measure.
2. **Does canon still tell the truth?** design § 9 is the section written to
   stop over-claiming, and PHASE-06 recorded it wrong in several places
   (F-P06-10, F-P06-12, D-P06-1) without editing in-phase. Every such item is
   confirmed against the artefact it contradicts, not taken on the sheet's word.
3. **Do the plan's criteria and the recorded outcome agree** — including where
   a criterion was discharged by intent rather than by letter, and whether that
   ruling is durably recorded where a later reader will meet the criterion.
4. **Is the durable record complete** — DEC/EVD/QUE/ASM state consistent with
   the verdict in *both* tiers, follow-ups lodged rather than left in prose, and
   the evidence corpus self-describing once the runtime sheets are gone.

### Invariants the slice is held to

- **A criterion's checkbox is not its evidence.** Every discharge claim is
  checked against the artefact, never against the phase sheet's tracking. The
  archived sheets are frozen exhibits and explicitly non-authoritative.
- **The scope is the verdict.** Any bare count, any unqualified portability
  claim, any "16/16" is a defect regardless of where it sits.
- **`n/a` is a structural absence, never a pass**; a guard never observed
  refusing is not known to work; a negative grep without a positive control
  proves only that grep ran.
- **S4 held: no `src/` change in PHASE-01..06.** The rig never bought a green
  by editing the thing under test.
- **The credential refusal is `EROFS` from the mount flag, not `EACCES` from
  mode bits.** Any narrowing that reintroduces the secret on a writable mount is
  a real weakening however tight its permissions look.

### Where the bodies are likely buried

- **Conformance is structurally noisy here and that is not a slice defect.**
  662 undeclared paths, because the recorded source-delta ranges span a shared
  `edge` tree carrying other slices' work — PHASE-05's range alone holds 111
  foreign commits plus two interior merges, which `record-delta --commit` cannot
  excise. `start..end` must not be read as all ours.
- **The largest inherited work item is bookkeeping, not correctness** — DEC
  records owed for D-P05-6..23 and D-P06-1..9, deliberately deferred by sheet
  convention and easy to lose.
- **Already-settled items that must not be reopened**: ISS-307 stays as filed
  (F-P06-13 withdrew the widen-it recommendation — the *start* boundary is
  exclusive and correct by construction); QUE-200's "the EVD does not suffice"
  is a *ruling* under EX-9 (D-P06-9), not unfinished business, and its residual
  is a universal over git's config space that no further probe discharges.
- **No design run exists** — runtime loss; `design.md` is intact and locked.
  `design start --from-design` was deliberately not run: no attestation, receipt
  or gate clearance carries across, so the manufactured run would promise more
  than it holds. Absence of the run is not a finding.
- **`research.md` does not exist** — the pre-design research round was never
  run; the drift advisory reports an absent artefact and is deliberately not
  restamped (rationale in `plan.md` § Notes). Not a finding.

## Synthesis

### The closure story

SL-241 set out to produce evidence, not product, and it did. The rig ran, the
sixteen hazard rows scored against both ingestion mechanisms on two fixtures,
the seven confinement rows scored with a positive control each, a real agent
executed a real red→green phase inside a capsule and committed its own work, and
a scoped verdict landed. Every EX / VT / VA / VH criterion is discharged;
`slice verify-vt 241` is green across all six phases; `doctrine check gate`
passes clean; S4 held — `src/` was never touched in PHASE-01..06, so the rig
never bought a green by editing the thing under test.

**The audit found no defect in the evidence.** Every claim spot-checked against
its source held: the fifteen-`model-level`-rows-plus-H12 split matches
`matrix.md`'s row-level altitude column; the only `16/16` in the go/no-go is the
sentence forbidding it; the seven P-C2 rows all `pass` in the final scored run
at the timestamp the verdict cites; ASM-007 renders `invalidated` and ASM-008
`held`, both tiers, consistent with the verdict; QUE-200/201/202/204 are all
`open` as the verdict says; CHR-053 and CHR-054 are lodged as EX-10 required;
every path the evidence README claims exists is present and tracked. The
deliverable is unusually disciplined about its own limits, and the audit's
adversarial passes bounced off it.

**What the audit did find is a documentation-integrity story, and it has one
shape.** Ten of eleven findings are the durable record failing to keep up with
what the work learned — and they cluster in the two places that matter most:

- **design § 9 — four findings (F-1..F-4), all inside the section written to
  stop over-claiming.** Row 2's parenthetical counts a file as a ref and mixes
  per-phase with per-slice writes; row 5 names a count where the discriminating
  fact is the kind of git operation; the scope bullet reads a fixture-altitude
  as OS-portability and so asserts fifteen rows proven on macOS when nothing in
  the spike measured a single row off Linux; the ASM-007 bullet forecloses the
  one outcome step 0 was built to be able to produce, and has been false since
  PHASE-04. § 9's own closing line is *"Writing the scope in is what stops the
  REV over-claiming"* — and the scope as written is the over-claim. The
  go/no-go already uses the correct readings throughout and flags two of these
  itself; it is § 9's text alone that is stale.

- **the evidence corpus — F-11, the one finding the handover did not carry.**
  The verdict is GO for the ingestion *and confinement* halves. The ingestion
  half has committed authority: `results-c3.tsv`, with the README declaring it
  wins over any prose. The confinement half had none — P-C2's seven scored rows
  lived in `~/capsules/probes/c2/results.tsv`, outside the repository, in a
  disposable capsule root, plus prose in a phase sheet the corpus explicitly
  declares non-authoritative. Fixed in audit rather than briefed, because the
  source was one `rm -rf` from gone and this slice already owns the standing
  lesson about that (F-P05-39: the T4a–T4e falsification drivers were never
  tracked and are gone). `results-c2.tsv` and `results-smoke.tsv` are now
  committed verbatim.

The general lesson the slice drew for itself generalises to its own artefacts:
*a design's parenthetical enumeration is a starting point, not a count* — the
error is invisible until someone tries to turn the list into a number. Three of
the four § 9 findings surfaced exactly that way, when PHASE-06 tried to fill the
measurement table the design had sketched.

### Standing risks

- **The largest gap is QUE-202 and it outlives the slice.** Conflict/staleness
  *resolution* is out of evidence — not disproven, never designed. The refusal
  is proven safe and total, and it is **content-blind**: stage 4 compares two
  OIDs and never reads a tree, so a genuine conflicting pair and a trunk advance
  a three-way merge would take without a murmur produce the same token. Any
  admission design must supply that discrimination itself; the pipeline will not
  narrow the problem for it.
- **QUE-200's residual is architectural, not another probe.** `upload-pack` was
  exercised in the capsule repo on every M-A cell and git's protected-config
  defence was observed holding — but the surface was **sampled, not cleared**,
  and one of the two keys (`core.fsmonitor`) is honoured from repo-level config
  and stayed silent only because nothing in the M-A harvest path refreshes an
  index. Two keys do not discharge a universal and three will not either. The
  tractable question is whether the parent needs to run git in that repository
  at all.
- **Nothing is measured off Linux.** Every environment-conditional row — the
  whole P-C2 set, `harvest/resource-cap`, H9's containment leg, H12's env-file
  surfaces — is outstanding for macOS, not assumed to hold and not assumed to
  fail.
- **The `EROFS`-not-`EACCES` constraint is live and the follow-up work points
  straight at it.** The capsule runs as `uid=1000` and *owns* the credential
  (`-rw-------`), so mode bits would not stop it; only the read-only mount does,
  and `--unshare-all` denies the `CAP_SYS_ADMIN` needed to remount. The
  outstanding "scope the agent home's writes" direction is precisely the one
  that could reintroduce the secret on a writable mount — a real weakening
  however tight its permissions look.
- **27 sheet decisions are citable only as doc-local ids** until F-6 is
  actioned. Not urgent, since the sheets are committed, but several of them bind
  work outside SL-241 where a doc-local id is the wrong currency.
- **The spike's biggest process near-miss is worth carrying forward.** P-C1b's
  first scored run reached green while the phase never executed — the agent had
  no shell at all, wrote correct code by reading the tests and reasoning by hand,
  and EX-1's assertion passed for a reason unrelated to EX-1. What caught it was
  recording the ritual separately from the commit. What should have caught it
  earlier was a smoke that exercised the *capability* rather than the
  *dependency*; go/no-go § 5 item 8 argues that leg should be permanent, and it
  is right.

### Tradeoffs consciously accepted

- **PHASE-06 EX-8's letter is unmet** (F-5, `tolerated`). Writing it would have
  planted a false claim in the anti-over-claim document. The ruling is right and
  is recorded in three committed places; what is accepted is that the
  authoritative surface — `plan.toml` — is the one place the divergence is
  invisible, because plan criteria are immutable-append and off reconcile's
  write surface. F-4 removes the mirror a downstream REV would actually read.
- **Conformance cannot give this slice a clean signal** (F-8, `aligned`). 662
  undeclared paths, dominated by other slices' work on the shared `edge` tree —
  PHASE-05's range alone holds 111 foreign commits and two interior merges that
  `record-delta --commit` cannot excise. Already lodged as IMP-292 and IMP-282;
  no new item warranted. ISS-307 stays as filed.
- **`n = 1` on the agent axis, two fixtures on the project axis, one OS.** All
  three are structural and all three are stated in the verdict. The token
  measurement is capsule-reported and cannot be otherwise without dissolving
  EX-3; it is recorded, never asserted, and no stage or outcome reads it.
- **`cas-lost` is reachable but unexercised** — owned by no matrix row, because
  producing it means racing the rig's own scheduling rather than a hostile
  capsule. Recorded as reachable-but-unexercised rather than impossible, which
  is the weaker and more accurate claim.

### Verdict

**No blockers.** Eleven findings, all terminal: two fixed in audit, one aligned,
one tolerated with rationale, seven delegated to `/reconcile`. The evidence
stands as recorded; what needs writing is canon catching up to it. SL-241 is
clear to move to `reconcile`.

## Reconciliation Brief

Seven delegated findings across three write surfaces. The design § 9 group
(F-1..F-4) is the substance; the rest is registry and corpus bookkeeping.

### Per-slice (direct edit)

**`design.md` § 9 — the measurement table.**

- **F-1 · row 2, the `before (incumbent) source` cell.** Replace the
  parenthetical *"static — coord branch, projected refs, candidate branch,
  `candidates.toml`, trunk"*. `candidates.toml` is a file and must leave a count
  of refs. The replacement must keep the per-phase / per-slice split visible —
  `dispatch/<N>`, the worker fork branch and `phase/<N>-NN` attributable to one
  accepted phase; `review/<N>`, `candidate/<N>/<label>`, trunk and optionally
  `edge` amortised over *k* phases. `measurements.md` § 2 carries the corrected
  enumeration and its REQ-311 (SPEC-022 FR-001) derivation — mirror it, do not
  re-derive it.
- **F-2 · row 5.** Re-state the axis as the **kind** of git operation, not the
  count: the incumbent materialises a tree and stages into a shared index
  (ISS-234's hazard class); the capsule model never materialises a tree — it
  moves objects and one ref. `measurements.md` § 5 is the source, and also
  discloses that the incumbent count is the subprocess arm.

**`design.md` § 9 — the "Scoped means, precisely" list.**

- **F-3 · bullet 2.** *"model-level rows proven portable, env-conditional rows
  outstanding for macOS"* → the altitude and the OS axis are **independent** and
  the replacement must state both, not merely soften one. `model-level` means
  the row held on both fixtures (design § 5.4 / A-3) — a claim about client
  shape. Separately, **every** row is unmeasured off Linux. go-no-go.md § 1's
  two-class table is the model to follow.
- **F-4 · final bullet.** *"ASM-007 is recorded strengthened, not discharged,
  whatever step 0 returns"* → ASM-007 was **falsified and is `invalidated`**;
  ASM-008 replaced it and is the record that is strengthened-not-discharged.
  Drop *"whatever step 0 returns"* — foreclosing step 0's outcome is the defect,
  not just the stale state. go-no-go.md § 3.3 is the wording to mirror.

**`design.md` § 9.1 — the code-impact table** (mirrors only; the load-bearing
change for both is the selector registry below):

- **F-7 ·** add `.doctrine/rfc/025/go-no-go.md` — the deliverable § 9's own
  Closure paragraph requires, currently unnamed in the table that lists
  `.doctrine/rfc/025/evidence/**`.
- **F-9 ·** add `flake.nix` — modified by PHASE-01 (29d842dc, shellcheck into
  the jail, D-P01-4). Also revisit the table's closing *"No `src/` changes"*,
  which reads as *"and nothing else structural either"*.

### Selector registry (`doctrine slice selector`) — load-bearing

This is what `slice conformance` reads; the § 9.1 rows above are the human
mirror and do not move the signal.

- **F-7 ·** `doctrine slice selector add` — `.doctrine/rfc/025/go-no-go.md` as
  **`design-target`**. Scope to the file, **not** to `.doctrine/rfc/025/**`:
  RFC-025 prose edits are an explicit slice non-goal (CHR-053 owns them), so
  promoting the directory would declare intent the slice deliberately did not
  have.
- **F-9 ·** `doctrine slice selector add` — `flake.nix` as `design-target`.
- Both are expected to clear from `undeclared` on the next
  `doctrine slice conformance 241`; verify rather than assume.
- **Not in scope:** `.doctrine/slice/241/**`. That is slice-own process
  artefacts and a known tooling gap (IMP-282). Declaring it here would paper
  over the gap in one slice rather than fix it.

### Knowledge corpus (`doctrine knowledge`)

- **F-6 · DEC records for D-P05-6..23 and D-P06-1..9.** The notes carry a
  standing *"owed at close"* for all 27 and no record exists for any.
  **Recommendation — mint the outward-binding subset, not all 27**, and record
  the decision about the remainder so it reads as decided rather than dropped:
  - **Mint:** D-P06-9 (QUE-200's residual is architectural — the guard against a
    future agent planting a third config key), D-P05-17 (the allowlist / QUE-204
    lever, still unsliced and the largest forward work), D-P06-6 (the OS
    boundary *is* the boundary — a downstream design that weakened the sandbox
    to compensate for a harness setting would invert it), D-P06-5 and D-P06-8
    (the agent-home posture and the `EROFS`-not-`EACCES` constraint, which
    go/no-go § 5 item 4's follow-up points straight at).
  - **Leave indexed:** the remainder, whose reach is inside SL-241 and which
    resolve through notes.md's per-phase INDEX into the committed sheets. State
    that in notes.md so the "owed" line stops reading as outstanding.
  - This is a judgement call with real cost either way; the operator may prefer
    a different cut. What is not acceptable is leaving it silent.

### Governance / spec (REV)

**None.** No finding touches an ADR, policy, standard, spec or requirement. The
slice's non-goals held: no RFC-025 prose edits (CHR-053), no census REV scoping
(CHR-054), no dispatch machinery changes, no `src/`.

### Not for reconcile — recorded so it is not re-raised

- **F-5** (`tolerated`) — PHASE-06 EX-8's letter. `plan.toml` is off the
  reconcile write surface. The one-sentence amendment following PHASE-01
  EX-7/EX-8's own precedent is the operator's call at `/close`.
- **F-8** (`aligned`) — conformance's 662 undeclared. Tooling gap, already
  lodged (IMP-292, IMP-282). ISS-307 stays as filed.
- **F-10, F-11** (`fix-now`) — landed in audit: `results-c2.tsv` and
  `results-smoke.tsv` committed to the evidence corpus, README manifest and
  authority sentence updated.

## Reconciliation Outcome

Reconcile pass complete, 2026-08-03. Every brief item is resolved. **No REV was
authored** — the brief's governance/spec section was empty and nothing found
touched an ADR, policy, standard, spec or requirement.

### Direct edits applied — `design.md`

- **§ 9 measurement table, row 2 before-cell** (RV-343 F-1) — the false
  parenthetical replaced with the counted enumeration: 3 attributable per phase
  (`dispatch/<N>`, the worker's fork branch, `phase/<N>-NN`) plus a share of 3–4
  slice-level refs amortised over *k*. `candidates.toml` is gone from a count of
  refs; the per-phase / per-slice split is now visible. Mirrors
  `measurements.md` § 2 and its REQ-311 derivation.
- **§ 9 measurement table, row 5** (F-2) — the metric now names the **kind** of
  git operation as load-bearing, not the count: the incumbent materialises a
  tree and stages into a shared index (ISS-234's hazard class); the capsule
  model moves objects and one ref. The before-cell now discloses that the count
  is the **subprocess arm**.
- **§ 9 "Scoped means, precisely", bullet 2** (F-3) — replaced with both axes
  stated independently: `model-level` grades *fixtures* (client-project shape,
  § 5.4 / A-3), and separately **nothing was measured off Linux at any
  altitude**. Points at `go-no-go.md` § 1's two-class table.
- **§ 9 "Scoped means, precisely", final bullet** (F-4) — now states that
  ASM-007 was **falsified and is `invalidated`**, and that **ASM-008** is the
  record that is strengthened-not-discharged. *"whatever step 0 returns"* is
  deleted: foreclosing step 0's outcome was the defect, not merely the stale
  state.
- **§ 9.1 code-impact table** (F-7, F-9) — two rows added
  (`.doctrine/rfc/025/go-no-go.md`, `flake.nix`), and the closing *"No `src/`
  changes"* qualified so it no longer reads as *"and nothing else structural
  either"*.

### Selector registry — the load-bearing half of F-7 / F-9

- `doctrine slice selector add 241 .doctrine/rfc/025/go-no-go.md --intent design-target`
- `doctrine slice selector add 241 flake.nix --intent design-target`

Both file-scoped, **not** `.doctrine/rfc/025/**` — promoting the directory would
declare intent the slice deliberately did not have (RFC-025 prose edits are
CHR-053's). **Verified, not assumed:** `doctrine slice conformance 241` moved
662 → **660 undeclared**, both paths now appear under `conformant` against their
new selectors, undelivered stays 0. The residual 660 is F-8's shared-tree noise.

### Knowledge corpus — F-6, on an operator ruling

Ruling 2026-08-03: **mint the outward-binding five, index the remainder in the
fifth.**

| record | lifts | what it binds outside SL-241 |
|---|---|---|
| **DEC-128** | D-P06-9 | QUE-200's residual is architectural — `open` is the answer, not a deferral |
| **DEC-129** | D-P05-17 | egress allowlist + QUE-204 → follow-on slice; largest forward work, still unsliced |
| **DEC-130** | D-P06-6 | the OS boundary **is** the confinement boundary; names the inversion risk |
| **DEC-131** | D-P06-5 | writable agent home, unwritable credential — the property, not the proxy |
| **DEC-132** | D-P06-8 | the `EROFS`-not-`EACCES` refusal, **and the index of the remaining 22** |

All five `accepted`, both tiers populated (typed `[facet]` in TOML, reasoning in
prose). The remaining **22** sheet decisions are indexed one line each in
DEC-132's prose body, resolving into the archived phase sheets. `notes.md`'s
four standing *"DEC records owed at close"* markers are struck through and
restated as settled, so nothing reads as outstanding into `/close`.

### Carried to `/close` — not reconcile's to write

- **F-5** — PHASE-06 EX-8's unmet letter. `plan.toml` is off the reconcile write
  surface and stayed untouched. DEC-132's index flags D-P06-1 as the live tail.
- **New, out of brief — `design.md` § 9 row 2's *after*-cell** reads `static —
  one, asserted by I1`, but `measurements.md` § 2 states the after-side is
  **measured**, not static (the `assert_outcome` leg passed in both P-C1a and
  P-C1b). F-1 scoped itself to the before-cell, so under D9 this was **not**
  fixed here. It is a one-word accuracy nit, raised for a ruling at `/close`
  rather than silently absorbed.

### Withdrawn / tolerated / already landed

- **F-8** (`aligned`) — no write needed; IMP-292 / IMP-282 / ISS-307 stand.
- **F-10, F-11** (`fix-now`) — landed during audit. A follow-on pre-disposal
  sweep found F-11 was one instance of a class and archived four further scored
  tables and two proof objects (`results-c1a`, `results-c1b`, `results-r2`,
  `results-selftest`, `exec-proof-smoke`, `run-c1b/`); recorded in `notes.md`
  § Harvest, outside this brief.

Reconcile pass complete — handoff to `/close`.
