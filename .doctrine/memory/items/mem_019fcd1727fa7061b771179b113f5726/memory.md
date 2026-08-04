The storage model classes `.doctrine/state/` as **runtime state: gitignored,
disposable, `rm -rf`-able**. That is true of the *tier* and false as a licence
for any individual file in it.

`.doctrine/state/slice/NNN/design.toml` is a design run's snapshot. A run is
opened at `design start` and stays open until the design locks — **weeks**, over
many `cargo build`s and often several released versions. It is also not
`rm -rf`-able in practice: it carries the run's attestations, receipts, gate
clearances and change log, none of which any authored artefact reproduces, and
`design start --from-design` explicitly cannot rebuild them.

So every type serialised into that snapshot has a **wire form that outlives the
binary that wrote it**, exactly like an authored entity does:

- `RecoveryIntent` — `[[checkpoint.intent]]`. `DEC-125` re-keyed `checkpoint` →
  `subject`; the `#[serde(alias = "checkpoint")]` is what keeps live runs
  readable. Deleting it made `design show` fail outright on this repo's own
  `SL-243` (9 rows) and `SL-244` (16 rows) runs.
- `ChangeEvent` — `[[change_log.row]]`. It deserialises **strictly**, so one
  unrecognised `event` fails the whole file, not just that row. `SL-244`
  PHASE-04 retired `IntegratedReviewRecorded` and broke `design show 244` that
  way — see `ISS-315`.

Before changing a field name, an enum member, or a variant's representation on
anything reachable from `DesignSnapshot`, ask what live runs already hold. Two
cheap probes:

    grep -c '^checkpoint = ' .doctrine/state/slice/*/design.toml
    grep -rn '<the-token-you-are-retiring>' .doctrine/state/slice/*/design.toml

Additive-and-defaulted is safe. A rename needs a serde alias. A **retirement**
needs a decision about what the reader owes the history the writer can no longer
produce — the two vocabularies are not the same set.

Pin the compat at the tier that breaks: `snapshot::parse` over a literal legacy
fragment, not a unit round-trip over the inner type. The precedents are
`a_snapshot_written_before_the_policy_reads_as_human_only` and
`a_snapshot_written_before_the_intent_subject_key_still_parses`
(`src/design_run/snapshot.rs`).

See [[mem.fact.doctrine.storage-tiers]] for the tier the framing comes from.
