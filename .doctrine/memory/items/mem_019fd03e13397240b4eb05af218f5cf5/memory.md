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

**Correction, 2026-08-08 (SL-249 PHASE-02, `ISS-328`).** The claim above that
*"Every inner submission type there carries `deny_unknown_fields`"* is **false**.
Three types in `src/design_run/submission.rs` do — `Declaration`,
`CheckpointActDeclaration`, `AgentActDeclaration` — and the rest do not,
`CreateRecord` among them, nested two levels inside a `Declaration` that does. So
an unknown key inside a `form = "create"` disposition is dropped by the same
mechanism, with none of the structural excuse: nothing prevents the attribute
there. The rest of this memory stands — `flatten` really does forbid the
attribute on the outermost type, and the retirement discipline is unchanged.

**Closed, SL-259 PHASE-03 (2026-09-15) — and the corollary inverts for this
surface.** `design_run::contract_check` now walks the submitted JSON against
`payload_contract`'s key inventory *before* deserialisation, so an unknown key
at any nesting level is a typed refusal (`Refusal::UnknownPayloadKey`) rather
than a silent drop. That reaches both places the attribute cannot: the flatten
envelope on `ApplyRequest`, and the internally tagged enums — `CreateRecord`
among them, which is the 2026-08-08 correction above. All eight contract rows
read `UnknownKeys::Refused`, and the published contract says so to every client.

**What this changes about the retirement discipline.** The grep for stale
fixture claims is no longer the detector; the *suite* is. A retired key that
fixtures keep sending used to keep every test green, which is why the discipline
above exists. It now fails them loudly. So on this surface a green suite IS
positive evidence that no exercised payload carries a stale key — the one place
the general caution *"a green suite is not evidence that a fixture exercises the
mechanism"* is discharged rather than merely unverified.

Two caveats keep the discipline alive. Payloads **no test exercises** are still
dark: the fix for the worst case is to walk the shipped worked example against
the contract it teaches (done at `render/envelope.rs`'s example pin), since that
is what agents copy. And this holds for the *design-run apply payload* only —
any other wire surface still needs the grep and its positive control.

Flipping the strictness is also how you find the stale claims for free: land the
refusal, run everything, and read the failures. SL-259 found exactly four, all
in `e2e_design_state.rs`, all phrasing rather than substance.
