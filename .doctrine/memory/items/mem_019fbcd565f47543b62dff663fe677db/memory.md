Knowledge records are two-tier like every doctrine entity: structured/queried
data in `record-NNN.toml`, prose in `record-NNN.md`. **`doctrine knowledge new`
seeds `[facet]` with every field present and empty**, and authors routinely
write the whole record as prose and never fill it. So the tier that surveys,
queries, and downstream tooling actually read stays blank while the `.md` looks
complete.

The failure this produces: a ruling lands in the artifact that *argues* it (a
slice design, a review response) and not in the record that *governs* it — and
even when the governing record is amended, the amendment goes into the `.md`
only. Half-invisible: `grep` finds it, `knowledge show` shows it, but nothing
structured does.

Observed on DEC-099 during RV-323 (SL-241): the record's entire `[facet]`
(`context` / `choice` / `alternatives` / `rationale` / `consequences` /
`decided_on`) was empty while its `.md` carried a full decision plus an
amendment. ASM-007 and QUE-201 were the same. An assumption's
`validation_plan` — the field that says what would falsify it — was empty on a
record whose whole point was that it is falsifiable.

## What to do

- Amending a knowledge record = **both tiers**. Ask which facet field the
  ruling belongs in: a decision's `consequences`, an assumption's `claim` /
  `validation_plan`, a question's `question` / `why_matters`.
- **Verify via `doctrine knowledge show <ID>`, never by reading the `.md`.**
  `show` synthesizes both tiers, so an empty facet is visible; opening the file
  is how the gap survives.
  (`knowledge inspect` is the opposite — metadata only, no body.)
- When a slice design rules on something a governing record owns, amend the
  record in the same pass. The design argues; the record governs.

Related: [[mem.concept.doctrine.reading-entities]],
[[mem.concept.doctrine.storage-model]], [[mem.fact.doctrine.storage-tiers]],
[[mem.signpost.doctrine.knowledge]].
