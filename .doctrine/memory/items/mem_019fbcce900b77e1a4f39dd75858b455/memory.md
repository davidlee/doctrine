## The footgun

Any policy check that asks "did this change touch a forbidden path?" by parsing
`git diff --name-only` **in shell** is evadable two ways, both silent:

1. **Non-ASCII.** With git's default `core.quotePath=true`, a path containing a
   non-ASCII byte is emitted C-quoted *and wrapped in double quotes*:
   `".doctrine/na\303\257ve.md"`. A prefix test for `.doctrine/` does not match
   the leading `"`, so the forbidden touch passes.
2. **Rename.** With rename detection on (the default), `--name-only` prints only
   the **destination**. Move a file *out of* `.doctrine/` into a declared path
   and the source leg vanishes from the diff entirely — the governance file is
   deleted and nothing refuses.

## The reference form

The Rust belt already carries all three hardenings, and carries them *because*
all three evasions are real. Copy this invocation, do not re-derive it:

```
git -c core.quotePath=false diff --name-only --no-renames -z <B>..<S>
```

- `core.quotePath=false` — non-ASCII paths emit verbatim UTF-8.
- `--no-renames` — a rename reads as delete+add, so the source leg is visible.
- `-z` — NUL-delimited is the only byte-safe form to parse (a path may contain
  a newline).

Reference sites, both commented with the reasoning:

- `src/mcp_server/dispatch.rs:487` — the import belt's `B..fork` gather.
- `src/slice.rs:2890` — `actual_from_range`, behind
  `slice conformance --against A..B --strict` (SL-180 PHASE-01, EX-4).
- `src/git.rs:1257` — the `-z` NUL-delimited diff helper, which exists for
  exactly this reason.

**So: prefer the existing verb.** `doctrine slice conformance <id> --against
A..B --strict` folds a rev-range against a slice's `design-target` selectors,
fail-closed, with no worktree and no index — and it is already hardened. Reach
for shell only for the legs no verb covers, and then use the invocation above.

## Why it bites harder than it looks

The failure direction is the dangerous one. A missed forbidden path does not
error — the hostile change is *admitted*, or a hostile probe row scores "no kill
= fail" and reads in a results table as a defect of the **model** rather than of
the checker. That is the finding least likely to be re-examined.

A prefix check that has never been *observed refusing* is not known to work:
plant a non-ASCII forbidden path and a rename-out-of-forbidden-prefix as
positive controls. See [[mem.pattern.harness.grep-negative-needs-positive-control]].

Surfaced by RV-340 F-4 (SL-241 capsule spike rig design), where the scope leg
correctly rode the existing verb and the forbidden-path leg beside it was
hand-rolled shell without any of the three.
