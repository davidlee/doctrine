# IMP-010: SL-032 review cleanups: write_class test rewrite + reseat atomicity

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Rollup of the two SL-032 review findings (`.doctrine/slice/032/review.md`,
`audit.md`) that are **independent of the SL-031 funnel** — distinct from F-2/F-5,
which ride SL-031's kind-registry rework. Both are code-quality remediation on
SL-032 surfaces, expected to become **one near-term cleanup slice** (small enough
that this item is its scope seed; promote with `slice new`).

## F-3 — `write_class_tests` is struct-literal theatre (`src/main.rs:1361`)

~330 lines hand-build every `Command` variant with all fields, so any field
addition to any command breaks the table for reasons unrelated to classification.
It also bypasses clap (constructs `Command` directly), so it cannot catch a
CLI-wiring regression the way the argv-driven `e2e_worker_guard.rs` does — it
tests implementation shape, not behaviour.

**Fix:** rewrite as argv-driven assertions (`Cli::try_parse_from([...])` →
`write_class`), covering the Read/Write split + verb labels per the existing
black-box-cli-golden pattern (`mem.pattern.testing.black-box-cli-golden`). Drop
the redundancy with the e2e WRITE_VERBS table. The compiler's exhaustiveness
already proves totality; the test only needs to pin the split + labels.

## F-4 atomicity — reseat is non-transactional (`src/integrity.rs` `run_reseat`)

The six post-guard fs ops (rename dir → rename toml → rename md → rewrite toml →
drop alias → plant alias) have no rollback; a mid-sequence failure leaves a
half-reseated entity `validate` will flag. The non-zero-on-success dangler exit
is **accepted** (design R-3, now documented at the verb) and is *not* in scope —
only the atomicity is.

**Fix options to weigh at design:** stage into a temp dir + atomic swap; or
order the ops so any partial failure is self-healing / re-runnable; or wrap with a
best-effort unwind on error. Reseat targets pre-execution collisions where the
current blast radius is tolerable, hence backlog not blocking.

## Not here
- F-2 (shared kind registry + escape-guard test) and F-5 (`KindRef` carries the
  state dir) → **SL-031** (it is the second consumer that shapes the registry).
- F-1 trunk-mint *wiring* → already **SL-031** scope (D6/§5.4).
