# RFC-026 route re-rate — pre-registration

Fixed **2026-09-27**, committed before any rater ran. Governs `SL-270` step 1
(*settle the taxonomy before the window*). Nothing here may be adjusted after a
rater's output exists; a deviation is disclosed in the report, never absorbed.

## Why

`E13` (`route-transfer/report.md`) found `owner-fix` modal at 21 of 47 severe
audit and code-review findings under both raters, and grounded `P11` on it. A
review on 2026-09-27 (`SL-270` context, findings 5–7) qualified that result:

- its rater prompt carried a steering rule — *"Where a finding is about shipped
  code not matching what the design or a governance record says … let that
  decide the route"* — that points at `owner-fix`;
- its "both at 21" is equal totals over 72% row agreement (34 of 47);
- its population is the facets `QUE-224` excludes, so `owner-fix`'s load where
  routing applies — design review — is unmeasured.

## Populations

| id | population | findings | source |
|---|---|---|---|
| **T** (transfer) | `E13`'s 24 reconciliation and code-review ledgers, every blocker and major | 47 | `route-transfer/rater-prompt.txt` |
| **D** (design) | `E11`'s 11 design ledgers plus `E12`'s 3, every blocker and major | 155 | `E11` 137 + `E12` 18 |

Counts verified 2026-09-27 against `review show --json` (T by `E13`'s raters;
D: RV-325 16, RV-370 18, RV-307 30, RV-314 37, RV-349 16, RV-358 6, RV-365 4,
RV-359 0, RV-355 5, RV-347 0, RV-344 5, RV-374 2, RV-382 5, RV-377 11).

## Raters

Four runs, blind to each other, to `E11`–`E13`'s classifications, and to this
document's reading rules.

| thread | population | runner | prompt |
|---|---|---|---|
| A | T | `pi-research` (deepseek-v4-pro) | `rater-prompt-transfer.txt` |
| B | T | codex `gpt-6-sol`, read-only sandbox | `rater-prompt-transfer.txt` |
| C | D | `pi-research` | `rater-prompt-design.txt` |
| D | D | codex `gpt-6-sol`, read-only sandbox | `rater-prompt-design.txt` |

Different model families per pair, so agreement is not shared-model bias.
The route definitions are verbatim from `install/design-prompts/reviewing.md`.
The only instrument change from `E13` is: the steering rule is removed, and a
second label is added.

**Second label** (on every row routed `owner-fix` or `none`):

- `duplicate` — two live accounts of one fact; settled by deleting one.
- `stale` — a record lags the thing it describes (design / decision /
  governance says X, code does Y); settled by editing the record.
- `neither`.

## Reading rules

Computed by the analyst from the raters' flat tables, joined on `ledger/F-N`.

- **R1 Agreement.** Per population: row-level route agreement (count and %) and
  Cohen's κ. Equal marginals are not reported as agreement.
- **R2 Discrimination.** Per population and per rater: *discriminating* iff ≥3
  routes used **and** the modal route ≤50% of rows. The population *passes* if
  both raters pass, *fails* if both fail, else *split*.
- **R3 Steering.** `E13`'s steering is **material** iff the mean `owner-fix`
  share over A and B is at least 10 points below `E13`'s 44.7% (i.e. ≤34.7%).
  Otherwise the headline *survives*. Either way, `E13` is read through this.
- **R4 P11 decision — population D only.** Over rows both C and D route
  `owner-fix`:
  - second-label agreement < 80% → **indeterminate**; `P11` is not decided from
    this run.
  - else, jointly-`stale` rows ≥ 5 **and** ≥ ⅓ of the jointly-`owner-fix` rows
    → **split** `owner-fix` (recommend narrowing it to duplicates and naming the
    stale class).
  - else → **keep** five routes.
  Population T's second labels are reported and are not decisive: routing does
  not apply there (`QUE-224`).
- **R5 `none`.** Reported per rater with the raters' stated questions. Not
  decisive for any route change in this run; a recurrent shape shared by both
  raters in D is recorded as a candidate for the window's protocol.
- **R6 Failed run.** A rater that skips rows, reads excluded material, or
  exceeds its backstop is re-run once from the same prompt; a second failure is
  reported as missing, never replaced by a third runner.

## What this run does not decide

It does not validate `P10` (only a use trial can), sets no pass mark for the
second window, and does not change any count in `E11`–`E13`.
