# QUE-175: Should claim-surface drift feed retrieve-side staleness ranking

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The question

SL-230 builds a single **claim-surface constructor** — the canonicalised uid
directory plus a memory's declared, sanitised scope entries — and adopts it in
`verify` and `validate`. Should `retrieve` adopt it too, so drift over that
surface feeds retrieval **ranking**?

This is SL-230 design **OQ-2**, promoted to a durable record because it now gates
two decisions rather than one:

1. the original: should own-directory drift feed retrieve-side `staleness`, or
   only `validate`?
2. added by RV-307 F-24: should `retrieve::git_facts` (`src/retrieve.rs:556-557`)
   route through the shared constructor at all?

They are the same question by two routes — both reclassify a large fraction of
the corpus at once and shift retrieval ordering broadly.

## Why it is open rather than deferred-and-forgotten

`git_facts` today gates on `m.scope.paths.is_empty()` and passes `scope.paths`
raw: no globs, no canonicalisation, no pathspec-magic neutralisation, no uid
path. Every defect RV-307 found in `verify`'s surface is still live there. So
"leave it alone" is not a neutral choice — it means two notions of scoped drift
coexist and **the weaker one drives ranking** (SL-230 R7).

SL-230 declined to answer it inside a body-write slice: converting `git_facts`
would smuggle a retrieval-ordering change in under cover of a bug fix. The bound
was drawn so answering later is cheap — the constructor takes `(root, memory,
dir)` and borrows nothing from `verify`'s command context, so adoption is a
call-site swap.

## What answering it decides

- **Yes** → implement **IMP-317**; R7 closes.
- **No** → close IMP-317 as `wont-do` and restate R7 as *intended and permanent*
  rather than provisional, with the divergence documented at the `git_facts`
  call site so the next reviewer does not re-raise F-24.

Either answer is fine; leaving it unanswered is the failure mode, because the
gap currently reads as an oversight rather than a decision.

## Evidence to gather before answering

- how many memories change retrieval rank if drift is measured over the repaired
  surface (the SL-230 census machinery answers this: 236 memories declare a path
  or glob scope; 55 carry non-contributing entries);
- whether glob-only memories — invisible to `git_facts` today — are a material
  population;
- whether ranking already treats `Staleness::Unknown` conservatively enough that
  the change is small in practice.

Related: SL-230 (design OQ-2, D11, R7), IMP-317, QUE-173 (the digest-based
alternative, which would make the whole question git-independent).
