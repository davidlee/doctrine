## Decision

**A recording is an occurrence, not a state delta, so it must be emitted
explicitly at the point of record. Invalidation caused by content movement stays
derived.** These are different questions and they take different mechanisms.

## Why derivation cannot work here

`ActInvalidated` is deliberately *not* emitted by the writer. It is computed from
a before/after set difference (`invalidation_rows`, `run.rs:1842`) and the doc
gives the reason: "derived from the before/after difference rather than recorded
by whoever changed the content, so a new way of moving a fingerprint cannot
forget to report what it killed." The set it differences, `live_acts`
(`run.rs:1777`), already chains **both** act record kinds, so a reverse
difference looked like a zero-signature-change fix.

It cannot work, and the proof is short. For a slot already holding `v`:

    no operation:      before = v, after = v
    record same value: before = v, after = v

No function of `(before, after)` distinguishes those histories. Adding the
content fingerprint to the tuple would only detect *unequal* replacement. To
detect every recording by endpoint derivation the state would need an occurrence
identity — a revision, submission id, or generation — persisted into the tuple,
which turns the operation into state and fights the deliberately kind-derived
identity at `run.rs:671`.

The concrete instance, which is `ISS-355`'s own worked example: an act group
replaces **by kind**, and `AgentDeclarationGroup::record` (`snapshot.rs:387`)
says so — "Keyed on the **discriminant**, so a second `BlockingSetDeclared`
displaces the first **however its blocking set differs**." A re-declaration whose
blocking set genuinely changed leaves `(ActKind, DesignId)` identical, so the
difference is empty in both directions and derivation emits nothing. The general
statement: **a derived row can only report changes in the key of the set it
differences.**

## Where the row is built

Not on `RecordedAct` (`attestation.rs:796`). That sum's third arm, `Section`,
carries no record at all — its doc says "carries no attestation, and the absence
is the finding" — and every accessor on it is total on the shape. Giving it
`id() -> Option<&DesignId>` would contaminate an abstraction whose whole point is
totality with a question one arm cannot answer.

Instead a second, narrower sum over the two kinds that *have* an address:

    enum ActRecord { Checkpoint(CheckpointAct), Agent(AgentDeclaration) }
    impl ActRecord {
        fn admission_view(&self) -> RecordedAct<'_>;
        fn id(&self) -> &DesignId;
        fn kind(&self) -> ActKind;
        fn insert(self, next: &mut DesignSnapshot);
    }

This is not parallel modelling. The two sums answer different questions:
`RecordedAct` — can admission and the gate interrogate this act? `ActRecord` —
can this concrete record be stored and named in a change row?

They meet at one seam:

    fn admit_and_record(next, record: ActRecord, rule, derived)
        -> Result<Pending, Refusal>

which admits through `admission_view()`, inserts into the right group, and
returns the mandatory row. **Uniting storage and emission at one seam is the
real cannot-forget property** — stronger than a cleverer set difference, because
a future third record kind cannot be added without going through it.

## Signatures

    fn record_declaration(next, declared: &AgentActDeclaration, derived)
        -> Result<Pending, Refusal>
    fn record_act(next, declared: &CheckpointActDeclaration, derived, payload_digest)
        -> Result<Vec<Pending>, Refusal>

`record_declaration` returns `Pending`, not `Option<Pending>`, and its caller
hoists `request.agent_declaration.as_ref()`. The current early `return Ok(())`
for an absent declaration is `ISS-355` in miniature: "nothing was supplied" and
"something happened" sharing one return value.

`record_act` seeds its vector from the mandatory row, so the `Vec` is non-empty
by construction and no `ActRows` type is needed — the invariant lives at the
seam that owns it. `Vec<Pending>` also matches the house precedent this whole
design follows: `attest` at `run.rs:1370` returns `Result<Vec<Pending>, Refusal>`
and emits `ReviewAttested` unconditionally through a retain-then-push
replacement, which is exactly the case derivation misses.

Construct the optional disposition row **before** calling `admit_and_record`, so
row-construction failure precedes mutation, and order the mandatory row first.

## Provenance

Reached in the `SL-256` design run with GPT-5.5 (codex) as peer advisor. The
occurrence-vs-delta proof is codex's sharpening of a concrete finding made on
this side; the seam design is codex's, accepted after pushback removed an
`ActRows` wrapper type.
## Correction, 2026-08-16 — the cannot-forget property is weaker than claimed

Raised as `RV-360` `F-1` by an external adversarial reviewer during `SL-256`'s
design review. The claim above — *"a future third record kind cannot be added
without going through it"* — is **false**, and it is left standing because it is
what the decision was taken against.

`CheckpointActGroup::record` (`snapshot.rs:361`) and
`AgentDeclarationGroup::record` (`:387`) are `pub(crate)`. They are therefore
reachable from anywhere in the `design_run` module tree, so a future production
caller can store an act without passing through `admit_and_record` and without
producing a row. Wrapping them in `ActRecord::insert` does not remove that route;
it only stops using it.

What the seam actually buys, stated at the strength the evidence supports:

- The three **production** recording paths collapse to one, so the split this
  slice exists to delete cannot re-form by a caller choosing differently.
- A new record kind added *through the seam* cannot omit its row, because the
  return type is `Pending` and not `Option<Pending>`.
- It does **not** prevent a future caller from bypassing the seam entirely. That
  is one obvious route rather than the only route, and it is a convention held by
  visibility, not by the type system.

Making the sink genuinely unreachable would require restricting visibility across
`fixture.rs` and `tests.rs`, neither of which `SL-256` may edit — `tests.rs` is
one of `SL-251`'s design-targets. So the invariant is weakened here rather than
enforced there. The decision's substance is unaffected: explicit emission at a
unified seam is still correct, and the occurrence-vs-delta proof that justifies
it does not rest on this claim.
