# IMP-065: Positive coordination-tree marker (close OQ-D): replace marker-absence dependence in D2a

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Deferred from ADR-012 / SL-064 as **OQ-D** (RV-023 F-2). ADR-012 v1 ships the
**marker-absence model + the ADR-006 D2b residual fence** (R-5 import belt +
IMP-052 post-spawn check + env-worker-on-main catch + bwrap-no-push). That fence
is honest but leaves the confessed ADR-011 D6/M2 gap: an **unstamped worker**
(stamp-hook failure / matcher drift) is *indistinguishable by absence* from the
legitimate coordination tree — and SL-064 widened that blast radius to the whole
**Orchestrator verb class** (`fork`/`import`/`gc`/sync), since the orchestrator
now also runs in a *linked* worktree.

**The redress:** stamp a **positive** coordination-tree marker (orchestrator
identity) at markerless-creation time, so the identity guard distinguishes
legit-coordination-tree from unstamped-worker by *presence of the right marker*,
not absence of the worker marker.

**Why deferred, not done here:** it touches the **owner-locked D2a
positive-signal model** (SL-056 PHASE-05 VH) — not a seam to redesign inside an
ADR/inquisition. User ruled (a): accept ADR-012 with OQ-D deferred-plus-fence,
split the positive marker to this follow-up.

**Scope when picked up:** a D2a amendment + the creation-time stamp; likely a
dedicated slice (governance-touching). Verify against the §6 fence items so the
positive marker *supersedes* the absence dependence rather than layering on it.

Refs: ADR-012 §Open OQ-D; ADR-006 D2a/D2b; ADR-011 D6/M2; RV-023 F-2;
SL-064 design §2; SL-056 PHASE-05.
