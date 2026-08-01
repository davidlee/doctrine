# Review RV-341 — implementation of SL-233

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Kind A of a two-ledger campaign.** Scope, method, tiering, and venue are
settled in `.doctrine/slice/233/review-campaign.md` on `dispatch/233`; this brief
carries only what a reader of the ledger needs. The sibling ledger is `RV-342`
(Kind B, the dark phases).

**Subject.** The landed code of SL-233 PHASE-02, PHASE-06, and PHASE-08 on
`dispatch/233`, in the coordination worktree `/workspace/doctrine/.dispatch/SL-233`.
3,652 lines of churn across 21 `src/` files.

**The question, and it is not "is this code good".** Each of these three phases
had its `EN-2` sketch gated by an adversarial design review — `RV-320`
(projection bounds, 7 findings, 33 rounds), `RV-323` (marker grammar, 6 findings,
30 rounds), `RV-325` (thin adapter, 17 findings, 59 rounds). Those are **design**
reviews: they gated sketches, not implementations, so they buy this campaign zero
code coverage. The question here is **whether the landed code honours what 33 /
30 / 59 turns of adversarial review actually settled.**

PHASE-08 carries 218 KB of settled design against 1,034 lines of code. The design
surface is larger than the code surface, and that is the campaign's defining
fact.

**Lines of attack.** The 30 verified findings across `RV-320` + `RV-323` +
`RV-325` are already an adjudicated conformance checklist: each is a specific,
pre-argued, *stated* claim about what the implementation must do. Checking landed
code against a stated claim is mechanically re-verifiable work, which is exactly
what `IMP-024` §1 says the cheap tier may safely be given. Beyond the findings,
each sketch carries commitments no finding contested; those need a reading pass,
not a checklist.

**Method.** Two tiers, split on one rule: *the cheap tier is given only work
whose output can be mechanically re-verified* (`IMP-024` §1).

- Cheap tier — `scripts/pi-review.sh`, confined and read-only (`--ro-bind / /`),
  no worktree fork. Not a cheap *model*: pi resolves `deepseek-v4-pro` and the
  knob is `PI_THINKING`, raised above `low` for conformance judgement.
- Top tier — codex MCP, for adjudication and anything turning on intent. It
  receives the cheap tier's tables framed explicitly as *evidence to verify, not
  conclusions to trust*.

Two enforcement rules, both non-negotiable, both proven on `RV-324`: every claim
ships the command that reproduces it, or is discarded unread; every negative
result ships a positive control. The second bit twice more during this slice's
own PHASE-08/09 sessions.

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

**Venue.** This ledger is minted in the **primary** tree because the corpus is
split while `dispatch/233` is live and an id minted in the wrong tree collides
silently (`ISS-279`) — and `ISS-277` means an RV collision cannot be reseated at
all, so it is unrepairable by tooling. Reading and evidence run in the coord
tree. See the campaign doc §5.

**Standing.** Findings are raised and open; dispositions are the owner's call. A
Kind A non-conformance may be the *criterion's* problem rather than the code's —
criteria are the owner's (cf. the standing PHASE-09 `F-P09.3` `VA-4`/`EX-8`
tension). That is precisely why this is a separate ledger from `RV-342`: it stops
a design dispute from being dispositioned as a code defect. All 16 phases are
`completed`, so no landed phase is edited by this review.
