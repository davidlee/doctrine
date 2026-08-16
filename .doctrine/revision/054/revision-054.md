# REV REV-054 — reconcile SL-251 — SPEC-029's design command family is six verbs

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

### Reconcile narrative (SL-251)

`RV-362` `F-7` (major, verified) — `SPEC-029`'s eighth responsibility names the
design command family as four verbs. It is six. `SL-251` shipped the sixth
(`doctrine design contract`, `commands/design.rs:151`), and the line was already
stale by one before this slice was written: `materialise` had been a
`DesignCommand` variant since the design-run's authoring seam landed.

The finding is a **prose staleness**, not a governance change. Nothing about the
spec's intent moves — the responsibility is that the capability is fronted
through *one* command family rather than scattered across the CLI, and that is
exactly as true at six verbs as at four. What the line does is enumerate the
family, and an enumeration that has fallen two behind the enum it enumerates
stops discharging the responsibility it states. `DEC-224` anticipated this and
called it a prose update at reconcile rather than an amendment; `SL-251`'s own
`design.md` §"Governance touchpoints" recorded the obligation and undercounted
it by one, which `RV-362` `F-8` item 5 corrects on the design side.

`SL-251` closed on 2026-08-16 under `RV-361` without this row, because the audit
that raised it (`RV-362`, run first, at 04:31) sat on an unmerged capsule ref
and the second audit never saw it. This REV is that debt discharged, not a new
finding.

### Change row — `modify SPEC-029`

Surfaced-for-manual at apply; landed by hand in `spec-029.toml:27`.

**Before**

> Front the capability through one command family — start, show, apply, resume —
> carrying the sparse mutation contract in which an omitted key persists, an
> explicit null clears a nullable scalar, an empty collection clears a
> collection, a stable-identity object partially updates its subject, and one
> unordered batch refuses duplicate subjects.

**After**

> Front the capability through one command family — start, show, apply, resume,
> materialise, contract — carrying the sparse mutation contract in which an
> omitted key persists, an explicit null clears a nullable scalar, an empty
> collection clears a collection, a stable-identity object partially updates its
> subject, and one unordered batch refuses duplicate subjects.

Two verbs inserted; the sparse-mutation clause is untouched. That clause governs
`apply` alone and always did — widening the enumeration does not widen it, and
narrowing it explicitly would be an amendment this finding does not authorise.

### Not in scope

`RV-362`'s other three delegated findings are per-slice direct edits
(`design.md`, the selector registry, `ISS-333`'s discharge statement) and land
outside any REV. This revision carries the governance leg alone, so a stuck row
here cannot block them and vice versa.
