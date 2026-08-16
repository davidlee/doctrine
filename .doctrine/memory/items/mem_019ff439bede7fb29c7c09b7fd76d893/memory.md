`doctrine design apply` takes a JSON payload whose schema is **not** discoverable
from `--help` or from `design show`. `design show`'s `declaration_example` shows
only node creation (`declare` with `subject`/`question`/`parent`/`needs`) and
`traversal`. The real vocabulary is `ApplyRequest` in
`src/design_run/submission.rs` — read that struct, not the envelope.

Every payload carries the flattened envelope:

    {"run_uid":"dr-…","known_revision":<current>,"submission_id":"<unique>", …}

Run-level keys beside it: `declare`, `traversal`, `stage`, `acceptance`,
`adopt_authored`, `delegation`, `discharge`, `review_policy`, `checkpoint_act`,
`agent_declaration`.

## Disposing an inquiry node

Not a verb and not a run-level key — a `declare` entry whose subject is a `cp-`
checkpoint id:

    {"subject":"cp-1","disposes":"inq-1",
     "dispose":{"form":"adopt","record":"DEC-195"}}

`dispose` is the only spelling of a disposition (EX-12). Four forms, tagged by
`form`: `create` (`kind`,`title`,`body?`,`facet?` — Doctrine mints the record and
claims the id), `adopt` (`record` — validated against the corpus), `unresolved`
(`note`), `non-durable` (`note`). Several checkpoints may ride one batch, and a
blocked node may be disposed — `needs` gates advancement, not disposition.
Resolving a node unblocks its dependents with no second act.

## The trap

`ApplyRequest` does not deny unknown fields, so a guessed or misspelled key **at
the top level** is **dropped silently**: the revision bumps, a receipt is
written, no event is emitted, no state changes — and the CLI prints the same
`revision N stage <stage>` a real mutation prints. A run can therefore sit
several revisions ahead of its own decisions with `resolved=0`. If an apply
reports no event rows, treat that as a refusal, not a success. ISS-333.

The asymmetry matters, and this memory had it wrong until 2026-08-16 (RV-362
`F-10` — SL-251's reconciliation review): `Declaration` **does** deny unknown
fields (`submission.rs:123`), as do `CheckpointActDeclaration` and
`AgentActDeclaration`. Only those three of the twelve wire structs refuse; the
other nine discard. So a misspelling *inside* a `declare` entry is refused with
serde's own message, while the same misspelling one level up is swallowed —
which is the whole of ISS-333. (`ISS-346` recorded the same defect and was
closed as a duplicate of `ISS-333` at that audit.)
