# IMP-174: Split-brain authored state at audit/reconcile/close

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The hazard

When a slice's work lands on a **fork branch** that was cut at an old base and
the **primary tree (edge) advanced concurrently**, the fork's *authored* tier
(`.doctrine/**` — memories, notes, backlog, review ledgers, *other slices'*
files) diverges from edge. At close, there is no guidance on:

1. **Which side is truth** per authored path — the slice's own delta vs edge's
   newer concurrent state.
2. **How to land only the slice's delta** without clobbering authored work that
   accrued on edge after the fork point.

A naive merge/integrate of the fork clobbers concurrent authored state; a naive
"audit on edge" silently drops the fork-only authored artifacts (notes, memory
triage) the auditor needs.

The runtime tier compounds it: review-ledger verbs (the RV baton) refuse to run
in a fork — the baton lives in the **primary tree's gitignored `.doctrine/state/`**
(single-writer coordination, ADR-007). So audit/reconcile/close are *forced* onto
the primary tree, but the slice's authored truth may be on the fork. The two
constraints pull opposite directions with no documented reconciliation.

## Why now (SL-156 surfaced it)

SL-156 (self/auto conduct, **mixed inline + dispatch phases**) is the witness.
`git diff --name-status edge slice/SL-156-cargo-isolation` showed the fork
carrying a tangled authored superset/subset of edge: RFC-005 edits, slice 154/155
notes, review 162/163 add+delete, ~15 memory items churned, backlog 039/051/052 —
**none of it SL-156's own code delta**. Merging would have reverted edge's newer
authored state; auditing on edge alone would miss the fork-only `slice/156/notes.md`
and memory triage. Mixed inline+dispatch + self/auto + a stale fork base is the
worst-case generator.

## What's wanted

**Believable, written guidance (and where cheap, mechanism)** for the close-loop
edge cases:

- A rule for **partitioning the fork delta by tier**: authored doctrine state
  (path-scoped land/cherry-pick to edge) vs code (reviewed from a ref/range, not
  checked out) vs runtime (never crosses).
- A **divergence check** at audit/reconcile entry: detect when the fork's authored
  tier has drifted from edge and refuse-loud rather than clobber (cf. the funnel's
  fail-closed posture; the dispatch path already tree-reads ledgers — solo/inline
  forks have no equivalent guard).
- Conduct/landing-path-aware staging notes in the `/audit` → `/reconcile` →
  `/close` skills for the self/auto + mixed-mode case.

## Relations / neighbours

- **IDE-017** — fork *provisioning* of divergent **untracked** state; sibling on
  the build-input axis, not the authored-tier axis. This item is authored-tier.
- **IMP-024** — review baton lives in primary-tree gitignored state; the runtime
  constraint that forces audit onto the primary tree (the half this item must
  reconcile against).
- **RFC-005 H2 / OQ-5** — checkout-independent integrate; if integrate stops
  riding the live checkout, part of this may simplify. Adjacent, not a substitute.
- **ADR-007** (RV single-writer baton), **ADR-012** (dispatch integration topology),
  the storage-tier rule (authored vs runtime vs derived).
