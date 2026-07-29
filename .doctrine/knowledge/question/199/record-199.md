# QUE-199: Is SPEC-012 REQ-304 'by construction' accurate given cooperative confinement?

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The question

PRD-015 NF-003 (REQ-304) promises tier exclusion **"by construction rather than a
trusted check"**, and SPEC-012 describes the worker-mode guard as
**fail-closed-on-ambiguity**. Two observations sit awkwardly against that:

1. **Confinement is cooperative by design.** `mem.fact.dispatch.worker-confinement-is-actor-based`
   records the settled position: the marker answers *"is the process running me
   confined?"*, not *"is this tree protected?"* — every protected tree is
   markerless by construction, `WriteClass::MarkerClear` is deliberately
   unguarded, and a worker can stand itself down explicitly. It is an
   accident-fence, not a security boundary.
2. **`worker_mode` is permissive when the marker is missing.**
   `describe_mode` (`src/worktree/marker.rs:88-101`) computes
   `refused = (is_linked && marker_present) || env_set`. A linked worktree with
   **no** marker yields `refused = false` — full write privilege. SPEC-012
   (`spec-012.md:273-274`) describes this case as *"refused, not trusted — so a
   stamp-failure or a self-clear loses privilege rather than gaining it"*, which
   reads as the opposite of what the predicate computes.

Whether (2) is a defect or a misreading turns on what "refused" denotes. Under the
actor-based model in (1) the behaviour is coherent — an unmarked process is simply
not a worker — but then the spec sentence's *"loses privilege"* framing is
misleading, and REQ-304's *"by construction"* is doing more work than the
mechanism supports.

## Why it is not settled here

Raised by RV-322 round 5 (F-1, second limb) during SL-237's design review. It is
**not caused by SL-237** — the predicate and the confinement posture both predate
it, and single-homing phase state does not change either. REV-043 originally
adjudicated REQ-304 as "preserved, construction relocated to the OS floor"; the
split removed that adjudication rather than resolving it, because the answer
belongs to the dispatch confinement model, not to where phase sheets live.

## What settling it would require

- Decide whether "by construction" is claimed against *provisioning* (no copy
  reaches the fork — true, and untouched) or against *write exclusion* (enforced
  cooperatively — weaker than the words imply).
- If the latter, either revise REQ-304's wording or accept it with the
  cooperative posture stated explicitly, as ADR-006 does elsewhere.
- Reconcile `spec-012.md:273-274`'s "loses privilege" sentence with
  `describe_mode`'s actual truth table either way.

Related: [[IMP-354]] (platform-agnostic confinement wrapper) raises the
subprocess arm's floor but does not answer this — a stronger floor still is not
the same claim as "by construction".
