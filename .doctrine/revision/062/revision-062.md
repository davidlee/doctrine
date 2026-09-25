# REV REV-062 — Add the kind-blind show router to SPEC-013

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

`SPEC-013` owns the two-level `<kind> <verb>` grammar (`REQ-197`, `FR-001`) and
the kind-blind **list** spine (`REQ-198`, `FR-002`). A kind-blind **show** — one
top-level `doctrine show <REF>` that resolves a canonical ref to its kind and
delegates to that kind's own `show` — is neither: it is not the per-kind grammar,
and it is not the list spine. It therefore takes a new requirement, `FR-006`,
sibling to `REQ-198` (design sec-5; `RV-384` `F-2`).

The change is additive and clarifying. The spec prose that says "A top-level
`Command` enum names each entity kind" reads as exhaustive; a top-level
kind-blind `show` falsifies that reading, so the grammar section names it. The
`Responsibilities` paragraph and the structured `responsibilities` list gain the
router so the spec's own account of what it owns is complete. No existing
requirement is altered.

Precedent: `REV-038`, which amended `SPEC-013`'s own member requirements.

## Before / after

### `spec-013.md` § *Uniform command grammar*

**Before**

> The verbs within are the shared set — `new`, `list`, `show`, `paths`, and (for
> lifecycle kinds) `status` — so the invocation shape is identical across kinds:
> `doctrine <kind> <verb>`. …

**After**

> The verbs within are the shared set — `new`, `list`, `show`, `paths`, and (for
> lifecycle kinds) `status` — so the invocation shape is identical across kinds:
> `doctrine <kind> <verb>`. Alongside that per-kind grammar, one **kind-blind**
> read verb is top-level: `doctrine show <REF>` resolves a canonical ref to its
> kind (the prefix names the kind) and delegates to that kind's own `show`, so a
> reader need not restate a kind the id already carries. …

### `spec-013.md` § *Responsibilities*

**Before**

> Mirrors the structured `responsibilities` list: impose the uniform
> `<kind> <verb>` grammar; own the kind-blind read spine; carry the shared
> `CommonListArgs` flatten; own the `--columns` projection model; fix the
> canonical id form; and pin the surface with the conformance matrix and
> black-box goldens.

**After**

> Mirrors the structured `responsibilities` list: impose the uniform
> `<kind> <verb>` grammar; own the kind-blind read spine — the `list` spine and
> the top-level `show` router; carry the shared `CommonListArgs` flatten; own the
> `--columns` projection model; fix the canonical id form; and pin the surface
> with the conformance matrix and black-box goldens.
