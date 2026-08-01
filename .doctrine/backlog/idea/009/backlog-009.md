# IDE-009: Knowledge read-path validation / knowledge lint verb

Source: SL-059 `/code-review` findings C4 (🟡), C6 (🔵), plus the C3 structural
fix. Deferred out of SL-059 scope (the slice shipped tolerant-read per R2,
accepted) and captured here so it is not lost.

## The gap

The knowledge hand-edited tier (`record-NNN.toml`) silently swallows two classes
of bad input on read:

- **C4 — typo'd / foreign facet keys.** The 24-field `RawFacet` superset
  (`knowledge.rs`, the kind-blind tolerant-read) accepts any facet key; a typo
  (`claimm = …`) or a key belonging to another kind's facet is dropped silently
  rather than flagged. Same gap exists corpus-wide for other authored kinds.
- **C6 — unvalidated record `status` on read.** A record's `status` is not checked
  against the kind's vocabulary on read (only the `status` *transition* verb
  validates). A hand-edited out-of-vocab status passes through.

Both are the cost of the deliberately tolerant read (R2). The fix is a *separate*
validation surface, not tightening the read.

## Proposed shape

A `doctrine knowledge lint` verb (or a shared corpus-wide lint) that, over the
authored tier, reports:
- facet keys not in the kind's known facet schema (catch typos / foreign keys),
- record `status` not in `statuses(kind)`,
without mutating anything — a read-only drift/typo canary the tolerant read
cannot raise.

## Third canary — the wholly-empty facet (SL-241 / RV-323, 2026-08-01)

Neither C4 nor C6 catches the commonest real defect: a record whose `[facet]` is
**entirely unfilled** while its `.md` reads complete. `knowledge new` seeds every
facet field present-and-empty, so prose-only authoring is the path of least
resistance and leaves nothing for a query to find.

Observed on DEC-099: full decision prose plus two amendments in the `.md`, and
`context` / `choice` / `alternatives` / `rationale` / `consequences` / `decided_on`
all `""`. ASM-007 and QUE-201 the same — ASM-007's `validation_plan`, the field
naming what would falsify it, blank on a record whose entire purpose is
falsifiability.

A lint leg: flag any non-draft record whose kind-required facet fields are all
empty. Cheap, and it catches the half-invisible amendment before the ruling is
needed. See `mem.pattern.doctrine.amend-knowledge-both-tiers`.

## Also fold in (C3 structural)

The `knowledge list` reveal rule (`list_rows`) reproduces `listing::retain`'s
status-keyed reveal because the kind-aware `is_hidden` cannot ride retain's
status-keyed closure. SL-059 left a DRIFT comment only. The structural fix is a
**kind-aware `retain` closure** in `listing.rs` so the per-item hide-set is
expressed once, killing the duplication.

## Links

SL-059 (origin), `src/knowledge.rs` (RawFacet, list_rows), `src/listing.rs`
(retain closure).
