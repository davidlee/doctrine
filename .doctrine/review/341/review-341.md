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

## Reconciliation Outcome

All 3 findings terminal (`verified`) and accepted by the raiser; ledger `done`,
`await=none`, 9 rounds. One finding carried a `reconcile-action`.

### Direct edits applied

- `sketches/projection-bounds.md:133` — **F-3**. The settled budgeted-rendering
  section stated a *single-byte-marked* elision; the code ships `U+2026`
  HORIZONTAL ELLIPSIS at three UTF-8 bytes. Re-measured before writing: line 133
  as cited, `ELISION_MARKER` at `src/design_run/render/mod.rs:159`, and the
  positive control at `:305` confirming `cap.saturating_sub(ELISION_MARKER.len())`
  consults the length rather than assuming 1. No invariant broke — the divergence
  was documentation-versus-code on a *stated byte property*, which is exactly the
  kind of thing later arithmetic leans on. Prose corrected to state the marker
  and that its encoded length is subtracted; the code is unchanged, because the
  glyph is the better choice.

### Withdrawn / tolerated

- **F-1** — `not-a-defect`. The finding argued the ROUND-10 gate, which
  `RV-325` F-17's round-11 owner ruling overturned using those same four steps;
  PHASE-08's entire diff to `exploring.toml` (`78d00a074`) is the one line that
  ruling prescribed. Rationale in the finding disposition.
- **F-2** — `deferred`. Rationale in the finding disposition.

### REVs completed

None. No item on this ledger touched governance or spec truth, so no REV was
authored and no half-applied row blocks the close gate.
