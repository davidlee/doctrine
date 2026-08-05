`#[serde(deny_unknown_fields)]` is what turns a **removed** field into a refusal
rather than a key serde quietly swallows. A struct carrying `#[serde(flatten)]`
**cannot have it** — serde would refuse the flattened field's own keys as
unknown. So the outermost request type, which is usually the one with the
envelope, is precisely the one that cannot tell a caller their payload is stale.

Observed on `design_run::submission::ApplyRequest` (SL-244 PHASE-05 `T13`,
finding `F57`). Every *inner* submission type there carries
`deny_unknown_fields`; the request cannot, and says so in its own doc. SL-244
`T11` deleted the `evidence` field, and three e2e fixtures went on sending
`"evidence": […]` for two more tasks. Every suite stayed green, because the key
had simply stopped meaning anything.

**What to do when you retire a wire field.** Deleting the slot is half the job.
Grep the fixtures and any hand-rolled JSON for the key and delete the claims too
— with a positive control on the grep, since a clean negative proves nothing
(`mem_019fa18161f47651af7687d8dccbbc67`). Then decide whether the absence needs
guarding: a one-line assertion at the fixture's single payload-building
chokepoint quantifies over every test in the file and every test added later,
where a single test asserting the absence once cannot stop it coming back.

**The corollary that bites hardest.** A green suite is not evidence that a
fixture exercises the mechanism it appears to. If the claims a fixture makes
have been silently ignored for a while, the suite has been proving something
narrower than it reads. That is the same class as
[[mem.pattern.harness.grep-negative-needs-positive-control]] — an absence you did
not verify is not a fact.

Related: [[mem.fact.design-run.snapshot-outlives-the-binary]] — the read side of
the same question, where a *stored* vocabulary must keep parsing what earlier
binaries wrote.
