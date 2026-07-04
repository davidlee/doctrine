# IMP-254: memory validate should enforce the master scope-floor (advisory vs test-gate parity)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

An embedded **master** memory needs ≥1 of `scope.paths` / `globs` / `commands`
(never tag-only). That floor is enforced **only** by a unit test
(`corpus::tests::every_embedded_master_lints_clean` in `src/corpus.rs`). The
author-facing advisory verb, `doctrine memory validate`, does **not** check it —
it covers dangling relations, stale verification, and draft expiry.

So a tag-only master passes `memory validate` clean, syncs, and **commits
clean**, then surfaces a full session later as an out-of-context test panic
("scope floor unmet — a master needs >=1 of paths/globs/commands") that a human
has to relay back. The advisory surface and the enforced surface disagree on
what a valid master is.

Witnessed authoring `mem.signpost.doctrine.dispatch` under CHR-036: shipped
tag-only, validated clean, committed, broke `cargo test corpus::` later. Fix was
mechanical (add a scope floor) but the round trip — diagnose panic → edit toml →
force re-embed (`touch src/corpus.rs`) → rebuild → re-sync → re-commit — was
pure waste a record-time or validate-time check would have eliminated.

## Fix (shape, not designed)

Make the advisory surface agree with the test gate. Options:

1. **Absorb the master-lint into `memory validate`** — validate runs the same
   master-scope-floor predicate the corpus test does, so the author gets the
   verdict at authoring time, not from an unrelated test run. Single source of
   truth for "is this master well-formed"; the unit test becomes a thin caller.
2. **Check at record/sync time** — refuse (or warn hard) when a shipped master
   is authored/synced tag-only.

Prefer (1): one predicate, two callers (advisory + test), no drift. Ride the
existing lint seam in `src/corpus.rs`; do not reimplement the floor.

## Related

- IMP-217 — retiring a local-capture memory to a shipped master needs a
  first-class verb. Sibling memory-authoring footgun; both are gaps in the
  shipped-master authoring path.
- RFC-011 case-notes `[backlog CHR-036; shipped-master-lint-blindspot]` — the
  originating instrumentation entry.
