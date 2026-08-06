# ASM-009: Git is the assumed and privileged version control system

Doctrine assumes Git, and privileges it structurally rather than incidentally.
Entity storage is a Git tree; provenance is commits; the capsule programme's
immutable inputs, candidate identity, and admission journal are objects and refs
(`SPEC-030`). Nothing states this as a dependency, and v0 contemplates no
alternative.

## Why record it

Every other mechanism doctrine rests on is deliberately substitutable, and the
contrast is what makes this worth writing down. The confinement backend is
defined by observable properties with no mechanism named, so bubblewrap,
Landlock supplementation, Docker, LXC, and virtual machines are all admissible
on the same terms (`SPEC-030` § Platform backend contract, its `D8`, `REQ-459`
criterion 3, and [[DEC-156]]). The harness axis has the same shape one level up
— one contract, per-harness concessions at a declared altitude ([[ADR-011]]).
Network posture is specified as a property, not as a technology.

Git is the exception, and it is currently an invisible one. That is a reasonable
v0 choice; the cost of leaving it unwritten is that a future substitution
conversation would begin with archaeology rather than with a record.

## What rests on it

- **Entity storage and history** — the authored tier is a Git tree, and
  diffability is the review model.
- **Immutable inputs** — the per-base export and clone-inside provisioning
  ([[DEC-157]]) are Git objects; a capsule's contracted base is a commit OID.
- **Interpretation provenance** — `REQ-449` resolves policy from a blob at the
  contracted base OID, and [[DEC-136]]'s read-once invariant is stated in those
  terms.
- **Identity, provenance, and admission** — candidate identity and the admission
  journal in `SPEC-030` are ref and object identities.
- **Confinement's denial half** — `REQ-448` denies writes to canonical refs and
  shared object storage, which are Git nouns.

## What would invalidate it

A proposal to back doctrine with something other than Git. This record is not a
proposition awaiting evidence; it is held, and the list above is the inventory a
substitution would have to work through. Note that the export half of
[[DEC-157]] is already stated mechanism-neutrally with respect to *confinement*
— it is not neutral with respect to *version control*, and this record is where
that distinction is kept.
