# QUE-190: CLI namespace for managed design runs

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

Where should v1 expose start, inspect/project, sparse-apply, resume, and
materialise operations for the managed `/design` run?

1. `doctrine design …` — a first-class, deliberately design-specific command
   family whose internals retain extraction-friendly seams.
2. `doctrine slice design-run …` — explicit nesting beneath the owning slice,
   at the cost of a deeper and less natural agent-facing command.
3. `doctrine workflow design …` — reserves a generic workflow namespace now,
   despite DEC-056 rejecting a generic workflow platform for v1.

The run remains slice-scoped in all three options; this question is about the
public product vocabulary, not storage ownership.

## Answer

DEC-075 chooses option 1: managed design runs use the first-class
`doctrine design …` command family. The public vocabulary remains
design-specific while its internal mechanisms preserve extraction-friendly
seams.
