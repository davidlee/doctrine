## The trap

"This contradicts an accepted decision → route it to a Revision" is a natural
reading of ADR-013 ("governance dependency routes through a Revision"), and review
findings and handover packets do say it. **For a knowledge record it is wrong, and
the CLI refuses it:**

```
$ doctrine revision change add REV-044 --action modify --target DEC-092
Error: `revises` target must be one of [SPEC, PRD, REQ, ADR, POL, STD], got a DEC
```

A Revision revises **governance and spec** truth. `DEC` / `QUE` / `ASM` / `CON` /
`EVD` / `HYP` / `CPT` are knowledge records and are not in that set.

## The actual route for a wrong accepted DEC

Knowledge records are append-only in spirit — you do not edit an accepted one to
make it true. Instead:

1. `doctrine knowledge new decision "<title>"` — mint the successor.
2. Author its `[facet]` **by hand**. There is no `knowledge edit` verb; the
   `knowledge` group is `new | list | show | inspect | status | paths`. The facet
   is edit-preserving TOML, so hand-authoring it is the sanctioned route (that is
   how the existing corpus records got theirs).
3. `doctrine supersede <NEW> <OLD>` — writes `supersedes` on the successor and
   `superseded_by` on the predecessor.
4. **`doctrine knowledge status <OLD> superseded` — a SEPARATE step.** `supersede`
   writes only the relation edges; the predecessor keeps whatever status it had.
   `superseded` *is* a valid decision status (`proposed, accepted, rejected,
   superseded`), and `doctrine validate` reports **`corpus clean`** with an
   `accepted` record that is `superseded_by` something — so nothing surfaces the
   contradiction. Skip step 4 and canon asserts two live records for one rule.
5. `doctrine link <NEW> shapes <SL-NNN>` if the predecessor carried that edge —
   relations do not migrate.

## The part that usually IS a Revision

Check whether a **spec** carries the same claim, because that half is a legitimate
`revises` target and is easy to miss. A tech spec's `responsibilities` list often
restates the decision in its own words; correcting the DEC without it leaves the
spec asserting the old rule. See [[mem.fact.revision.spec-prose-modify-target]] —
`modify --target SPEC-NNN` amends spec prose directly, surfaced for manual handling
at `revision apply`.

Worked example: SL-233 RV-324 F-2 contradicted accepted DEC-092. The route that
actually landed was DEC-105 superseding DEC-092 (the knowledge half) **plus**
REV-044 `modify SPEC-029` (the governance half, because SPEC-029's watermark
responsibility carried the same wording). Both were needed; neither alone told the
truth.

## Also worth knowing

- Minting any id in the jail needs `DOCTRINE_RESERVATION_FALLBACK=1`.
- Commit the slug symlink `doctrine <kind> new` mints alongside the entity dir.
