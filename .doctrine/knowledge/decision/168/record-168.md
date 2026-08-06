# DEC-168: Filled records are written at mint step 5

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The two candidate sites

`create_record` is the single fresh-creation path for all seven record kinds,
and its doc says why it must stay that way — *"a second reservation+scaffold
path is exactly the parallel implementation that drifts."* Pre-filling would
thread the payload through it into `entity::materialise_fresh_hooked`.

`apply_record_effects` is DEC-086 step 5, reached once the bytes are on disk.
It already applies the acceptance→status move (the sole route to `accepted`,
per DEC-088) and the `shapes` edge back to the governing slice.

## Why step 5

The decisive argument is not preference but the resume path. On a crash between
reserve and materialise, recovery calls `materialise_record_at`, which
re-scaffolds from the journalled reservation — id, title, slug. A pre-filled
scaffold would resume hollow unless the whole facet-and-prose payload were
journalled alongside, which widens the journal to carry content it otherwise
never holds.

Step 5 needs none of that, because the intent journal already models it:
`IntentState::Applied` is a distinct state, and step 5 is written to be
re-runnable. Both existing effects are idempotent — `set_authored_status` writes
only on a change, `append_edge` returns `Noop` when the edge is present — and a
facet-and-prose write is idempotent on the same terms.

The coupling argument stands independently. `entity::Inputs` is
`{ slug, title, date }` and is shared by every entity kind; knowledge-specific
facet content has no business travelling through it, and the alternative —
per-field placeholders across seven scaffold templates — is worse.

## Interaction with the phase order

DEC-165 puts the prose half of objective 2 in phase 1 and the facet half later.
Both land at the same site, which is what makes the split cheap: phase 1 calls
`entity::write_body` (already used by `memory edit`), and the later phase adds
the facet write beside it.
