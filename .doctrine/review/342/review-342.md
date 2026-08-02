# Review RV-342 — code-review of SL-233

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Kind B of a two-ledger campaign.** Scope, method, tiering, and venue are
settled in `.doctrine/slice/233/review-campaign.md` on `dispatch/233`; this brief
carries only what a reader of the ledger needs. The sibling ledger is `RV-341`
(Kind A, conformance to reviewed design).

**Subject.** The landed code of SL-233 PHASE-04, PHASE-05, PHASE-07, PHASE-13,
PHASE-14, PHASE-15, and PHASE-16 on `dispatch/233`, in the coordination worktree
`/workspace/doctrine/.dispatch/SL-233`. 11,926 lines of churn across 58 `src/`
files.

**Why these seven.** SL-233 landed 26,444 lines of churn across 16 phases. Only
29% carries an implementation review — PHASE-03 (`RV-321`) and PHASE-10/11/12
(`RV-324`). These seven phases have **no design gate and no code review**: they
are simply dark. This is ordinary correctness and quality review, which is what
separates it from `RV-341`.

Two phases are excluded and are not oversights. PHASE-01 (87 files, zero `src/`
— governance descent) and PHASE-09 (the evaluation kit, also zero `src/`) are
authored prose. They belong to the audit's spec-coherence and conformance legs,
and sending them to a code reviewer would burn a tier on the wrong question.

**Method.** Two tiers, split on one rule: *the cheap tier is given only work
whose output can be mechanically re-verified* (`IMP-024` §1).

- Cheap tier — `scripts/pi-review.sh`, confined and read-only (`--ro-bind / /`),
  no worktree fork. Not a cheap *model*: pi resolves `deepseek-v4-pro` and the
  knob is `PI_THINKING`, left at `low` for census and correspondence work.
- Top tier — codex MCP, for adjudication and anything turning on intent,
  receiving the cheap tier's tables framed explicitly as *evidence to verify, not
  conclusions to trust*.

Two enforcement rules, both non-negotiable, both proven on `RV-324`: every claim
ships the command that reproduces it, or is discarded unread; every negative
result ships a positive control.

Evidence and findings are different kinds (`IMP-024` §2). Cheap raisers emit
*candidate* findings to a staging directory; promotion to this ledger is a
judgement act by the orchestrator, performed **serially** — the ledger is
append-only with derived status, and N concurrent `review raise` calls contend.

**Dedup.** Every finding is checked against
`.doctrine/slice/233/prior-findings.md` on `dispatch/233` — the campaign's dedup
corpus, built once at S0 from the 60 verified findings across the six terminal
ledgers, 112 items censused from the 16 runtime phase sheets, 67 from `notes.md`
`### Learned` + `### Open`, and the open backlog. It is injected verbatim into
every raiser. A finding *adjacent* to a corpus entry names the entry in its
detail.

This slice has an unusual number of deliberately-unrepaired conditions, and they
are the dedup corpus's main earning: `ISS-289` (PHASE-15 `VT-2` unmet, an audit
judgement — any raiser that runs the suite will find it), `ISS-290`, `ISS-291`,
`RV-324` F-2's tolerated lost-update window, and a `VA-NC` partial disclosed in
seven separate phases.

**One carried obligation is in scope.** `RV-321` F-1 / F-2 / F-4 were accepted
and explicitly **carried forward to PHASE-04**. Whether PHASE-04 discharged them
is a question for this ledger; the `RV-321` findings themselves are not.

**Venue.** This ledger is minted in the **primary** tree because the corpus is
split while `dispatch/233` is live and an id minted in the wrong tree collides
silently (`ISS-279`) — and `ISS-277` means an RV collision cannot be reseated at
all. Reading and evidence run in the coord tree. See the campaign doc §5.

**Standing.** Findings are raised and open; dispositions are the owner's call.
All 16 phases are `completed` with `record-delta` recorded, so no landed phase is
edited by this review.

## Reconciliation Outcome

All 8 findings terminal (`verified`) and accepted by the raiser; ledger `done`,
`await=none`, 24 rounds. **No finding carried a `reconcile-action`** — five were
repaired inside the campaign and three were deferred to backlog items, so this
ledger contributed no write to the reconcile pass.

### Repaired in-campaign (`4f2816efa`, `dispatch/233`)

- **F-3** — the observed exit code now reaches the discharge record, in place of
  a hardcoded `0`.
- **F-5** — the PHASE-07 receipt assertion is line-anchored, and is proven by the
  finding's own mutation, which now **fails** where it previously measured ok.
- **F-6** — the acceptance attestation digest refuses rather than digesting `""`.
- **F-7** — the PHASE-15 comment citing `DEC-105` retargeted to `DEC-100`, the
  decision that actually survives.
- **F-8** — the doc comment naming `entry_text` corrected.

`doctrine check gate` exit 0 after the repairs.

### Deferred to backlog (`229b98205`, `edge`)

- **F-1** → **ISS-290**, appended rather than given a new item, because one
  repair closes F-1, ISS-290 and `RV-341` F-2 together and three parallel items
  would be three parallel implementations of one fix. The append records that the
  gap has a *demonstrated* escape, not a hypothetical one: at `63b348de6` three
  numbers disagreed and a step id refused at admission had passed validation.
- **F-2** → **IMP-379**. New test surface against an `EX-16` guarantee, not a
  defect in landed code: the clause's rendering half is exercised ten ways, its
  evaluation half survives only as the shape of an `if` plus a comment, so a
  refactor folding it back into `cumulative_conditions` for uniformity would go
  green while silently changing the machine.
- **F-4** → **ISS-295**, owner ruling **repair, not tolerate**. Worse than the
  `DEC-100` window lawfully tolerated for v0.0.1 on two counts: deterministically
  unrecoverable without hand-repair of the journal or tree, and reachable by an
  ordinary Ctrl-C, since the two writes are separated by user-interruptible work
  with no handler installed. Not repaired in-slice — all 16 phases are
  `completed`.

Also minted from this leg: **IMP-380** (the `explore.scope` residual).

### REVs completed

None. No item on this ledger touched governance or spec truth, so no REV was
authored and no half-applied row blocks the close gate.
