# DEC-100: Capsule admission via derived single commit

**Decision.** A capsule worker commits freely (multi-commit). Harvest pins the
result tip OID and preserves the **full capsule history** in the forensic
archive. Admission then derives a **single commit on the contracted base** from
the pinned tip's tree, and feeds that to `worktree import --fork`.

## The problem

Two existing verbs can land a branch onto coordination, and neither fits a
multi-commit capsule result:

- `worktree import --fork` runs the belt — `doctrine-touch`, `claude-touch`,
  `undeclared-scope` — but requires **exactly one non-merge commit** with
  `S^ == B` (`src/worktree/import.rs`).
- `worktree land` takes multi-commit branches but is **explicitly beltless**
  (`src/worktree/land.rs`: "land's beltless `--no-ff` merge is a different verb
  from import's belted apply").

So today there is no belted admission path for a multi-commit result. That gap
is a **finding to report**, not something the rig papers over.

## Why derive rather than contract-to-squash

Contracting the worker to end at one commit was the cheaper option, but it
discards worker history — which RT-11 wants as forensic evidence and which is
one of QUE-200's mechanism-verdict inputs ("forensic completeness: worker
history preserved?"). Deriving keeps both: the belt applies, and the history
survives outside the admission path.

## Why this is not a parallel implementation

DQ-1 / RT-2 disqualify a rig that re-derives merge, admission, or CAS logic. The
derivation is `git commit-tree` over an already-pinned tree — **transport, not
trust**. Every admission decision still happens inside an existing verb: the
belt at `import`, the 3-way and `Conflicted` handling at `candidate create`,
pinning at `admit`, CAS at `integrate`.

## Ordering constraint

The ancestry check must run **before** the derivation. A merge commit `S` with
parents `(B, X)` satisfies import's `single_commit` predicate (`S^ == B`), and
derivation would then launder it into a clean single commit. Probe row H3 only
gets killed if ancestry precedes derive.

## Related

- RFC-025 `red-team.md` RT-2 (reuse the candidate verbs), RT-11 (forensics).
- `probe-specs.md` DQ-1, row H3.
- QUE-200 — the ingestion-mechanism question this feeds.
