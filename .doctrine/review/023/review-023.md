# Review RV-023 — design of SL-064

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Inquisition of the SL-064 design **spine** (`design.md` §1–§3 + §5 amendments),
not the already-deferred OQs. Lines of interrogation (external reviewer: codex /
GPT-5.2, adversarial, read-only): (1) is D7 single-branch recovery genuinely
preserved once projection routes code to `phase/*` and intent to trunk? (2) does
markerless coordination-tree creation open a privilege hole via the ADR-006
D2a/ADR-011 D6 marker-absence semantics? (3) concurrency races across runs
converging on `edge`/trunk; (4) does the baked topology violate ADR-006 D1
policy-agnosticism; (5) provisioning completeness vs the full D9 contract; (6)
scope/altitude — is this an ADR's worth of framework smuggled under a D8 amendment?

## Synthesis

**Verdict: heretical but redeemable — the spine holds after reconciliation.** Six
charges, all terminal. The design's *structure* (isolated coordination worktree;
projection-not-funnel-change; preserved code branches; contention-unreachable-by-
construction) survived; its *claims* did not — it overclaimed safety in three
load-bearing places, and smuggled an ADR's worth of topology under a "D8 amend".

**Gravest (blockers, both reconciled fix-now):**
- **F-1** — the design claimed D7 crash-recovery survives projection because it is
  "downstream and re-derivable". False: a crash mid-sync (intent on trunk, code
  not yet integrated) orphans state the coordination branch never recorded. No
  journal, no replay contract. **Penance:** claim recanted (§1/§4/§5 downgraded to
  "recovery preserved for the funnel only"); the projection journal + idempotent-
  replay contract now **binds OQ-A**.
- **F-2** — markerless creation sold **marker-absence as positive identity**.
  ADR-011 D6/M2 already confesses an unstamped worker is indistinguishable by
  absence; SL-064 widened the blast radius to the Orchestrator verb class.
  **Penance:** absence-as-identity recanted; the ADR-006 **D2b residual is
  honestly inherited** with its fence (R-5 belt + IMP-052 + env-catch + bwrap-no-
  push); a §6 verify-item pins the fence against Orchestrator-verb impersonation;
  the positive-coordination-marker redress is **OQ-D** (owner-locked D2a → not
  redesigned in the dock).

**Altitude sins (major, ruled by the User):**
- **F-5 / F-4** — an ADR's worth of branch ontology + projection + class-routing
  smuggled under a D8 amendment, with D1 falsely claimed "preserved via config".
  **Ruled (a):** promote to **ADR-012 (proposed)**; SL-064 is its implementing
  slice; ADR-006 keeps only the pure-placement amendments (D8/D2a/D9-addendum);
  the **D1 tightening is owned honestly** in ADR-012. **ADR-012 acceptance gates
  SL-064's plan.**

**Lesser taints (reconciled):**
- **F-3** (major) — branch names aliased across runs. Corrected: `dispatch/<slice>`
  stable-across-resume by design; concurrent same-slice dispatch refused at
  creation.
- **F-6** (minor) — §2 dropped the D9 copy axis. Corrected: full two-axis
  provision stated.

**Standing risks carried (not defects — conscious deferrals):** OQ-A (intent
routing + projection recovery), OQ-B (delta-class temporal boundary), OQ-C (audit
vs sync ordering), OQ-D (positive coordination marker). All home in ADR-012; all
must close before ADR-012 acceptance, which gates the SL-064 plan.

**Tolerated:** none. Every charge reconciled fix-now or design-wrong; nothing
normalised away.

HERESIS URITOR; DOCTRINA MANET
