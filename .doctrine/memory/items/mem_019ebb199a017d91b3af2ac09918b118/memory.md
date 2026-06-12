# Unified read seam does not deliver a unified write seam

When weighing whether to unify entity storage, the authoring/link **write** verb is
the deciding force, not the reader — a read adapter unifies reads without giving a
shared writer.

Surfaced by the IMP-034 interrogation (→ ADR-010). The first-cut verdict was
"SL-046 already unifies the relation *read* via the per-kind `relation_edges`
accessor, so unifying *storage* is marginal — keep bespoke storage." An external
adversarial pass (codex) refuted it: SL-046 is read-only; relation *writes* stay
per-kind/bespoke. The only thing that yields a shared cross-kind `link`/`relate`
writer (the SL-048 deliverable) is a uniform *storage* block. The write seam — not
the read seam — is what justifies storage unification.

**Why:** a read adapter (accessor + projection) normalises however many bespoke
shapes into one view cheaply; that makes "the read is already uniform" feel like the
whole job, hiding that every kind still has its own *write* path. Generalising over
reads is easy; generalising over writes needs a shared on-disk shape.

**How to apply:** when a "should we unify storage across kinds?" question arises,
ask what the *authoring* surface needs (is a shared write/link verb wanted?), not
just what the read surface looks like. If reads are the only consumer, a per-kind
accessor seam suffices and storage can stay bespoke; if a uniform writer is wanted,
that is the force that pays for unifying storage.

Composes with ADR-004 (outbound-only): a uniform write block must enforce the legal
label vocabulary in code, or it is weaker than the typed fields it replaces. See
[[mem.pattern.architecture.info-flow-wall-at-signature]] for the related
"unify at the right seam" instinct.
