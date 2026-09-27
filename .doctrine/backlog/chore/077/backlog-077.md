# CHR-077: Report the RFC-026 P10 routing trial

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Why this exists

`SL-260` shipped the provisional finding-routing convention (`RFC-026` `P10`:
each blocker or major design-review finding gets one route from a closed set,
and the instrument routes are not repaired in prose). It deliberately does not
run the trial. This chore owns the trial report: once the eligible slices have
run, count their design-review ledgers and write the result into `RFC-026` as a
new evidence entry.

It is sequenced `after` `SL-260`, so it surfaces once the convention is in
effect. Its owner is whoever conducts or supervises the eligible slices' design
passes.

## Where the rules live — cite, do not restate

- **Which slices count:** `SL-260` scope §5 (*Eligibility rules for the three
  trial slices*). Apply it from `SL-260`'s landing commit.
- **What to collect and how to classify it:** `DEC-276` and `SL-260` design
  §9.4. Fixed before any outcome existed; do not adjust it after seeing results.
- **What the convention says:** `install/design-prompts/reviewing.md`,
  *Routing a severe finding*. Unenforced clauses: `CON-006`.

## Checklist

- [ ] **During the trial, at each eligible design pass's conclusion:** capture
  `doctrine review status RV-NNN` verbatim into this item's body, under the
  slice's id. This is the one step that cannot wait for the report. Rounds and
  contests live only in the gitignored runtime baton; once it is gone they are
  unrecoverable, and that ledger's counts are then reported as *unavailable*,
  never estimated. Why: `DEC-276`.

  **NOT DONE — missed for all three ledgers, and left unticked rather than
  quietly closed.** No capture was made during the window. The batons happened to
  survive on this machine, so `RV-374`/`RV-382`/`RV-377` rounds and contests were
  read late and are reported as late-read, not as the specified measure. `E12`
  confound 3 discloses this. It is the register's fourth entry in spirit: an
  obligation whose enforcement is a checklist line no gate reads.
- [x] Record which slices were eligible, and why, against the scope §5 rules.
  `SL-261`, `SL-262`, `SL-263` — next three code-changing by id from `70bd7d1ff`,
  none parked, all `done`. 3 of 3. See `E12`.
- [x] Run the collection procedure (`DEC-276` / design §9.4) over each eligible
  ledger. Every row collected; two qualified (`rounds`/`contests` late-read,
  `RV-377` design growth unavailable) and stated as such.
- [x] **Confound guard (`R4`, `DEC-270`):** before reading a high miss rate on
  adversary clauses or criterion sketches as a routing failure, check the
  surrounding responses for shell truncation. `--response` is one shell
  argument, and backtick spans and dollar signs are eaten before doctrine sees
  them. A miss rate inflated that way is not evidence about routing.

  Checked and **excluded**: all 18 severe responses present, 238–872 chars, none
  truncated. The routes are absent because they were never written.
- [x] Write the `RFC-026` evidence entry. `P10` sets no pass mark; report what
  was observed against its three trial questions and its *would-kill* list.
  Done: **`RFC-026` E12** (`.doctrine/rfc/026/p10-trial/report.md`), split out
  as a separate doc because it carries the rater instrument and both
  classifications. P10's *"no new kind, schema, or tooling"* carries a dated
  correction rather than a rewrite.
- [x] At trial conclusion, settle the questions this chore owns (below).
  Both settled 2026-09-27.

## Outcome

**The window reported a delivery failure, not a mechanism failure.** 18 severe
findings across the three eligible design ledgers; **0 carry a route** in the
field the convention specified, and neither of the two recovered afterwards is an
instrument route — so `P10`'s mechanism is untested by this window rather than
falsified. The cause is documented contemporaneously (`SL-261`, 2026-09-24): the
rule sat in `reviewing.md`'s tail, below where the responder stops reading, and
nothing validated the form.

**Scope was widened at conclusion, on the user's direction, beyond what the chore
originally owed.** `QUE-224` could not be settled by waiting — nothing instructs
an audit or code-review pass to route, so the observational population never
arises. A transferability classification study was therefore run over 47 severe
findings on 24 existing ledgers, with two blind raters, and recorded as **E13**
(`.doctrine/rfc/026/route-transfer/report.md`). It grounds **P11**.

**Residual, not this chore's to fix:** `--route` is not facet-gated, so the
mechanism permits what `install/review-ledger.md` forbids; nothing enforces either
direction. Stated in `E13`.

## Questions this chore owns

Minted from `SL-260` design §6, to be settled at trial conclusion:

- `QUE-224` — should the route axis extend beyond design-review ledgers? (§6 `Q2`)
- `QUE-225` — should the route set become a code-backed closed vocabulary?
  (§6 `Q4`)
