# record-delta before verify-vt, or every VT row reads UNATTRIBUTABLE

`doctrine slice verify-vt <slice>` reports a phase's VT criteria as
**UNATTRIBUTABLE** unless `doctrine slice record-delta <slice> PHASE-NN` has been
run for that phase. The keywords, patterns and `test_file` can all match
perfectly and it will still refuse to attribute them. Nothing in a plan, a
skill, or `verify-vt`'s own help says so.

## Why

`verify-vt` builds its `modified_files` set from the slice's **source-delta
registry** and from nothing else — for each recorded boundary row it runs
`git diff --name-only <code_start_oid>..<code_end_oid>` and unions the result
(`src/slice.rs:880-899`). No rows means an empty set.

The pure judge then tests file-in-delta **before** it tests the keyword
(`src/vtgate.rs:85-135`, and the order is load-bearing by `EX-2`): a mandated
`test_file` that exists but is absent from `modified_files` short-circuits to
`Unattributable { "keyword present but <path> not modified by this slice" }`,
so a phase's work is invisible to the gate no matter how well it was done.

`Unattributable` is non-halting, so `verify-vt` still exits 0 — the signal is
easy to read as noise and skip. The registry is **runtime state**
(`.doctrine/state/slice/<nnn>/boundaries.toml`, gitignored): it does not travel
with a clone, a fresh worktree, or a machine move, so the rows can also
disappear from under a slice that once verified clean.

The only incidental hint is the disclosure `doctrine slice phase … --status
completed` prints; there is no error and no advisory from `verify-vt` itself.

## How to apply

Immediately after committing a phase, before flipping it `completed`:

```
doctrine slice record-delta <slice> PHASE-NN --start <sha>^ --end <sha>
doctrine slice verify-vt <slice>          # confirm the rows flip to PASS
```

- `<sha>^..<sha>` is the **single-commit** case, not the rule. When a phase
  spans more than one commit — a follow-up fix, an incidental commit that landed
  inside the span — pass the real range (`--start <before-first> --end <last>`).
  SL-251's PHASE-02 spans two commits plus an unrelated lockfile refresh.
- The row is keyed by phase and upserts, so re-recording a phase corrects it
  rather than duplicating it.
- Re-running `verify-vt` after recording is the check that the range was right:
  rows that stay `UNATTRIBUTABLE` mean the mandated file is outside the range
  you gave.
- Moving a slice to another machine or worktree? Carry
  `.doctrine/state/slice/<nnn>/boundaries.toml` with it, or re-record every
  completed phase on arrival.

## Neighbours

Same machinery, different failure cells:

- [[mem_019f0342741174e3b0b5c293b181fac1]] — a fork-landed solo `/execute` never
  stamps the boundary on the primary tree, so `slice conformance` reads
  **undelivered**. That one is about a fork; this one bites in the primary tree
  too, whenever the flips are driven by hand rather than by `/execute`.
- [[mem_019fd109f42a7fb2b1d030f77332ad17]] — conformance reads the recorded
  boundary row, not the diff.
