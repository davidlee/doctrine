# QUE-175: Should claim-surface drift feed retrieve-side staleness ranking

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The question

SL-230 builds a claim-surface constructor for `verify` — the canonicalised uid
directory plus a memory's declared, sanitised scope entries. Should the two
*historical* consumers of a memory's scope — `validate`'s staleness check and
`retrieve::git_facts`'s ranking input — be given an equivalent, **history-stable**
surface?

This is SL-230 design **OQ-2**, promoted to a durable record because it gates
more than one decision:

1. the original: should own-directory drift feed retrieve-side `staleness`, or
   only `validate`?
2. added by RV-307 F-24: should `retrieve::git_facts` (`src/retrieve.rs:556-557`)
   leave the raw scope seam at all?
3. added by RV-307 F-27: `validate` is in the same position, so this is now a
   question about *both* historical consumers, not just `retrieve`.

They are one question by several routes — each reclassifies a large fraction of
the corpus at once and shifts staleness or retrieval ordering broadly.

**Corrected by RV-307 F-27/F-28/F-34.** This record originally described SL-230 as
adopting *one shared constructor* across `verify` and `validate`, and adoption
here as a cheap call-site swap. Both were wrong and the design has been re-cut:

- `verify` asks *is this evidence dirty now*, where canonicalisation is
  mandatory. A historical query asks *what commits touched it since*, where
  canonicalising against today's checkout **erases** a committed symlink retarget
  (measured, git 2.54.0: `rev-list -- link` → 1, over the resolved target → 0).
  So the answer is not "reuse `verify`'s surface" — it is "build a second,
  history-stable one".
- adoption is **not** a call-site swap. Neither consumer has an item directory to
  pass, and `collect_all` (`src/memory.rs:2826-2834`) unions `items/` and
  `shipped/`, so the row's origin is unrecoverable from `uid`. It needs a dataflow
  change through `collect_all` and `memory_health_findings`.

## Why it is open rather than deferred-and-forgotten

`git_facts` today gates on `m.scope.paths.is_empty()` and passes `scope.paths`
raw: no globs, no canonicalisation, no pathspec-magic neutralisation, no uid
path. Every defect RV-307 found in `verify`'s surface is still live there. So
"leave it alone" is not a neutral choice — it means two notions of scoped drift
coexist and **the weaker one drives ranking** (SL-230 R7).

The same is now true of `validate`, which SL-230 round 4 intended to repair and
round 6 returned to the raw seam (RV-307 F-27). So the unrepaired population is
two consumers, not one.

SL-230 declined to answer it inside a body-write slice: converting either
consumer would smuggle a staleness/ordering change in under cover of a bug fix.
The bound is honest about its cost rather than cheap — see the correction above.

## What answering it decides

- **Yes** → implement **IMP-317**; R7 closes.
- **No** → close IMP-317 as `wont-do` and restate R7 as *intended and permanent*
  rather than provisional, with the divergence documented at the `git_facts`
  call site so the next reviewer does not re-raise F-24.

Either answer is fine; leaving it unanswered is the failure mode, because the
gap currently reads as an oversight rather than a decision.

## Evidence to gather before answering

- how many memories change retrieval rank if drift is measured over a repaired
  surface (the SL-230 census machinery answers this: of 417 addressable memories
  and 482 path/glob declarations, 404 are observable and 43 do not resolve);
- whether glob-only memories — invisible to `git_facts` today — are a material
  population;
- whether ranking already treats `Staleness::Unknown` conservatively enough that
  the change is small in practice.

Related: SL-230 (design OQ-2, D11, R7), IMP-317, QUE-173 (the digest-based
alternative, which would make the whole question git-independent).
