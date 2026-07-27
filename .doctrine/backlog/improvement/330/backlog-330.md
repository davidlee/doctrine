# IMP-330: Share one flatten struct across observation list and search args

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Raised out of RV-317's F-6 remediation (SL-231, `d4a042e39`). F-6 is **closed and
verified** — this is the residue its prescribed fix shape deliberately left, not
a reopening.

## What F-6 fixed, and what it left

F-6 found `run_list` and `run_search` were 43 near-verbatim duplicated lines. Its
fix shape specified one helper taking `(projection, filter, limit, search,
empty_msg, json, format)`, and that is what landed: both now delegate to
`query_and_render`.

Because the prescribed signature takes `projection` and `filter` **already
built**, each caller still constructs them, so the two functions remain identical
for their first 12 lines:

    let root = resolve_root(args.path)?;
    let projection = if args.history { History } else { Active };
    let filter = Filter {
        kind: args.kind.map(KwKind::to_kind),
        time_from: args.time_from.clone(),
        time_to: args.time_to.clone(),
    };

## The duplication upstream of that

The residue is a symptom; the source is the clap layer. `ObservationListArgs` (33
lines) and `ObservationSearchArgs` (36 lines) share **eight identical fields** —
`history`, `kind`, `time_from`, `time_to`, `limit`, `format`, `json`, `path` —
each with its own duplicated doc comment and `#[arg(...)]` attributes. Search adds
exactly one field, `query`.

So the same eight flags are declared twice, and the code deriving a projection and
filter from them is written twice. Adding a listing filter today means four edits.

## Shape

Extract the shared eight into one `#[derive(clap::Args)]` struct and
`#[command(flatten)]` it into both, then hang the projection/filter derivation off
that struct as a method — at which point the two run functions reduce to genuine
argument adaptation and the F-6 residue disappears with it.

## Why this is not urgent, and why it was not folded into the F-6 fix

- No live incorrectness. The two paths are byte-identical in behaviour and both
  route through the service.
- It touches the **CLI surface**. Flattening changes where clap sources each
  argument and can reorder `--help` output, so it needs its own verification
  against the help goldens rather than riding a security-fix remediation turn.
- SL-231 has PHASE-04 and PHASE-05 outstanding. A CLI-surface change mid-slice
  competes with them for the same files.

## Watch for

This is the fourth parallel-implementation instance in SL-231, after PHASE-02's
`ensure_dir_components`, PHASE-03's `filter_and_order`, and F-6/F-7. The pattern
each time is that a fix removes the *instance* it was pointed at and leaves the
sibling standing — see `mem.pattern.review.sweep-defect-class-not-instance`. When
this is picked up, sweep for the shape rather than fixing the two structs named
here.
