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
- [ ] Record which slices were eligible, and why, against the scope §5 rules.
- [ ] Run the collection procedure (`DEC-276` / design §9.4) over each eligible
  ledger.
- [ ] **Confound guard (`R4`, `DEC-270`):** before reading a high miss rate on
  adversary clauses or criterion sketches as a routing failure, check the
  surrounding responses for shell truncation. `--response` is one shell
  argument, and backtick spans and dollar signs are eaten before doctrine sees
  them. A miss rate inflated that way is not evidence about routing.
- [ ] Write the `RFC-026` evidence entry. `P10` sets no pass mark; report what
  was observed against its three trial questions and its *would-kill* list.
- [ ] At trial conclusion, settle the questions this chore owns (below).

## Questions this chore owns

Minted from `SL-260` design §6, to be settled at trial conclusion:

- `QUE-224` — should the route axis extend beyond design-review ledgers? (§6 `Q2`)
- `QUE-225` — should the route set become a code-backed closed vocabulary?
  (§6 `Q4`)
