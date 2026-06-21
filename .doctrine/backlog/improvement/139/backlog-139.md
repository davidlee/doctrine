# IMP-139: Estimate set dual-optional bounds UX

## Source

IMP-133 UX review, first pass (F-3).
See `.doctrine/backlog/improvement/133/ux-review-findings.md`.

## Problem

`doctrine estimate set <ID> [LOWER] [UPPER]` — both bounds are marked
optional but are only optional when `-x/--exact` is given. If neither
`-x` nor both bounds are supplied, the command fails at runtime:

```
estimate set: must supply both lower and upper, or -x/--exact
```

The arg descriptions say "omit with -x" but the framing is easily missed.

## Options

1. Make `-x`/`--exact` the default and add `--range` for lower/upper mode
2. Add a usage-line note: `estimate set <ID> <LOWER> <UPPER> | -x <N>`
3. Better error message: "with -x, omit LOWER/UPPER; without -x, both are
   required" (a more specific error than the current "must supply both")
