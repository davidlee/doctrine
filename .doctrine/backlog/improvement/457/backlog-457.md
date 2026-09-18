# IMP-457: Design-run verbs occupy the document-render verb slots

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The complaint

`doctrine design show <SLICE>` renders the design **run**'s turn envelope —
stage, traversal, frontier, counts, change rows. It is internal machine state,
and for a reader who wanted the design it is white noise.

Everywhere else in the CLI, `show` renders the entity's **document**. `SPEC-013`
makes that a governed expectation, not a convention: it imposes the uniform
`<kind> <verb>` grammar over a shared verb set of `new`, `list`, `show`,
`status`. `doctrine adr show ADR-004` prints the ADR. `doctrine rfc show
RFC-031` prints the RFC. `doctrine knowledge show DEC-145` prints the record.

Two kinds break it:

- **`design`** — `show` is the run envelope; no verb renders `design.md`.
- **`slice`** — `show` renders the scope body and says so in its own help text
  ("not design/plan/notes"). The slice owns four documents and `show` silently
  picks one; the other three have no reader at all.

## Why it is filed rather than fixed

`SL-246` hit this while siting a composed design read and needed a home for it.
It took the cheap local answer — a document selector on `slice show` — which
leaves the slice's four-document ambiguity explicit and does not add a fourth
near-synonym beside `show` / `inspect` / `design show`. The `design` group's own
naming is untouched.

Unpicking the information architecture properly is a separate piece of work: it
means deciding what the run envelope should be called (`design turn`?
`design envelope`?), whether freeing `design show` for the document is worth the
rename, and whether the slice's documents want a selector or their own verbs.
That is a governed surface — `SPEC-013` conformance rows move with any of it —
so it wants its own scope, not a drive-by.

## The intended shape

Rehome the run envelope and reclaim the verb:

- `doctrine design show <SLICE>` → **`doctrine design state <SLICE>`**. The turn
  envelope is run state — stage, traversal, frontier, counts, change rows — and
  `state` says so. `resume`, `apply`, `materialise` and `contract` are unaffected.
- **`doctrine design show <SLICE>` then renders the design document**, which is
  what `show` means everywhere else in the CLI.

That done, `SL-246`'s `slice design show <SLICE> --knowledge <level>` becomes the
redundant one and should collapse into `design show --knowledge <level>`. Whether
`slice design` survives as an alias or retires entirely is part of this work, not
`SL-246`'s.

Two things to settle when it is scoped. `SPEC-013` conformance rows move with
every one of these renames, so the byte-exact goldens move with them. And the
slice's other three documents — plan, notes, and the scope `slice show` already
renders — want the same treatment or an explicit decision not to give it to them.

## Related

- `SL-246` — the slice that surfaced it and worked around it.
- `SPEC-013` — the CLI surface spec that makes `show` a governed expectation.
- `STD-002` — naming conventions.

## Closed as duplicate of IMP-393 (2026-09-18)

`IMP-393` — *Reader-facing design render for review*, opened 2026-08-03 —
already carries this complaint, six weeks earlier and in the same terms: `show`
means *render the entity's document* everywhere in the CLI, `design show`
renders the writer's turn envelope instead, and the fix is either reverting
`show` to the convention or rehoming the envelope. `IMP-393` also names a
resolution this item did not: `design show`'s own `--format` slot already
spells the writer rendering `prompt`, so the reader render can become the
default without inventing a name for the envelope.

This item was minted during `SL-246`'s drafting without `IMP-393` being found —
the parallel-carrier failure the no-parallel-implementation rule exists to
prevent. `RV-370` `F-6` caught it.

**What happened to the substance.** `SL-246` took `IMP-393`'s route under
`DEC-261`: `doctrine design show <SLICE>` renders the design document, the turn
envelope keeps `prompt`/`json`/`status` under `--format`, and only the default
moves. The rehome this item planned is therefore *done* for the `design` group
rather than deferred, and `SL-246` records `fulfils IMP-393 --degree partial`.

**What remains, and lives on `IMP-393`.** The slice's other three documents —
plan, notes, and the scope `slice show` renders — still have no reader or an
implicit one. The rename of the envelope to `design state` proposed here is
superseded by keeping it on `--format prompt`; whether `design state` is still
wanted as an alias is `IMP-393`'s call. And `IMP-393`'s fuller reader-facing
render — run metadata, attestation standing, open inquiries, the navigable
neighbourhood — is untouched by `SL-246`, which delivers the document plus its
knowledge records and no run state.
