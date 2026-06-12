# Unified read seam does not deliver a unified write seam

When weighing whether to unify entity storage, the authoring/link **write** verb is
the deciding force, not the reader — a read adapter unifies reads without giving a
shared writer.

Surfaced by the IMP-034 interrogation (→ ADR-010). The first-cut verdict was
"SL-046 already unifies the relation *read* via the per-kind `relation_edges`
accessor, so unifying *storage* is marginal — keep bespoke storage." An external
adversarial pass (codex) refuted *that*: SL-046 is read-only; relation *writes* stay
per-kind/bespoke, so the authoring surface, not the reader, is the live question.

**The correction (ADR-010 revised, second pass).** Codex then overshot — "the only
thing that yields a shared `link`/`relate` writer is a uniform *storage* block" is
**false**. A shared writer needs a shared *write accessor* seam (a per-kind
`append_edge(label, target)` mirror of SL-046's read accessor, dispatched by the
same `outbound_for(kind, id)` over `integrity::KINDS` and gated by a
code-authoritative label vocabulary) — **not** a shared on-disk shape. ADR-010
delivers SL-048's cross-kind writer over *bespoke* storage that way. Unifying the
on-disk shape buys only **append-only writes** (a new TOML table at EOF) instead of
edit-preserving array-splice across four schemas — a writer-*safety* gain, not
writer *feasibility*, and it costs a corpus migration. So storage unification is a
local, opportunistic call, not the price of the writer.

**Why:** a read adapter (accessor + projection) normalises however many bespoke
shapes into one view cheaply; "the read is already uniform" then *feels* like the
whole job, hiding that every kind still has its own *write* path. But the symmetry
runs both ways: a write *accessor* generalises writes just as a read accessor
generalises reads. What a shared on-disk shape adds over that is append-only-write
safety, nothing more.

**How to apply:** when a "should we unify storage across kinds?" question arises,
ask what the *authoring* surface needs. If reads are the only consumer, a per-kind
read-accessor seam suffices. If a uniform writer is wanted, build a per-kind *write*
accessor + one vocabulary-gated dispatch verb — that alone delivers it over bespoke
storage. Unify the on-disk shape only where append-only-write safety (or one parser)
clears the migration cost, and only for the clean payload-free / arity-free /
validated-label subset — never the typed-guarantee cases (arity, required fields,
free-text type).

Composes with ADR-004 (outbound-only): a uniform write block must enforce the legal
label vocabulary in code, or it is weaker than the typed fields it replaces. See
[[mem.pattern.architecture.info-flow-wall-at-signature]] for the related
"unify at the right seam" instinct.
