# ISS-310: sections_attested ignores reviewer identity

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The defect

`DesignSnapshot::review_standing` derives `sections_attested` by matching each
section's current fingerprint against the held attestations
(`src/design_run/snapshot.rs:425-431`):

```rust
let sections_attested = !current.is_empty()
    && current.iter().all(|(id, fingerprint)| {
        self.review.attestations.iter()
            .any(|held| held.subject() == id && held.fingerprint() == fingerprint)
    });
```

Subject and fingerprint only. **`Reviewer` is never consulted.**

`Reviewer` distinguishes `Human` (the v1 default) from `Adversarial` (opt-in per
section) and is recorded on every attestation
(`src/design_run/attestation.rs:23-28, 36-40`). It is stored, and then not read
by the condition whose whole subject it is.

## Why it matters

A design can satisfy `section-attestations-current` — and, cumulatively, reach
`Locked` — with **every section attested only adversarially and no human having
reviewed anything**. The condition reports that the sections have been attested;
what it actually checked is that attestations of *some* kind exist at the current
digest.

`Reviewer::Adversarial`'s own doc comment says the opt-in is per section while
"integrated adversarial review stays mandatory", which reads as a design in which
the two kinds are not interchangeable. Nothing enforces that reading.

This is the same defect shape as the gate's claimed conditions — a check that
looks stronger than it is because the stored data would support the stronger check
and nobody wrote it. The difference is that this one sits on a condition already
classified `is_derived() == true`, so it has never looked suspect.

## Why it is separate from SL-244

`SL-244` is settling what the gate checks, and `DEC-126`'s discriminator is
precisely *does the actor's identity matter* — under which
`section-attestations-current` becomes an **Attested** condition whose rule must
name the required authority. So `SL-244` specifies the model that makes this
expressible.

It does not, on its own, fix the incumbent derivation. Raised so the existing
defect is not silently carried on the assumption that the new model absorbs it:
if `SL-244` ships without repairing `review_standing`, the bug survives the slice
that named it.

Sequencing is genuinely open. If `SL-244` implements authority-aware attestation
rules, the natural fix is to route this derivation through them and this item
closes inside that work. If it does not reach that far, this stands alone and
needs its own answer to the question the fix implies: **what authority does
`section-attestations-current` actually require** — human, or either, or human
plus adversarial where the section opted in? That is a decision, not a
refactoring, which is why it is an issue rather than a chore.

## Origin

Surfaced by the adversarial review of `SL-244`'s design section 1 (finding 2),
which observed that the proposed attestation model could not express required
actors — and that the incumbent already does not.
