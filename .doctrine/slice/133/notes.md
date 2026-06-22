# SL-133 notes — closure harvest

Durable record of the slice. Structured audit findings live in **RV-138**
(`## Synthesis` + `## Reconciliation Brief`); engine-hardening findings in **RV-137**.

## Decisions / outcomes
- Scoring model: `base` (value_dim/risk_dim) + recursive `needs`-leverage (dep-overlay
  condensation DP) + one-hop `ref`-optionality. Mint orders by `base` only (no
  feedback); survey/next display by `score` (I3). policy_version `priority.v3`.
- `next` runs its own induced-frontier Kahn sort over the actionable set with
  surviving-seq precedence and `(score desc, id)` ready-set — NOT cordage `order_key`
  (RV-132 F-3).
- PHASE-05 cutover deleted the old `consequence:u32` path (no parallel impl, EX-2).

## Gotchas → memory
- `mem.pattern.priority.scc-condensation-dp-order` — reverse `ordered()` ≠ reverse-topo
  of the condensed graph; build an explicit component DAG (from RV-137 F-1/F-2).
- `mem.fact.cordage.in-edges-excludes-evicted` — `in_edges(overlay)` already excludes
  Evict-evicted edges; the seq-eviction subtraction is a defensive no-op.

## Deferred follow-ups (already in backlog)
- OQ-3 (collapse the two facet parse paths) → **IMP-109**.
- Worker gate gap (`just check`/lint-js fails in forks, needs gitignored node_modules)
  → **CHR-017**.
- OQ-5 (next fully score-driven / score-on-graph web view) — design §6 open question,
  downstream.

## Reconcile carry (see RV-138 Reconciliation Brief)
- ADR-015 REV: ratify `dep_coeff` domain `[0,1]` with `0` the disable sentinel (F-1);
  add `coefficients.value` to the `value_dim` formula (F-2). No code change.
- design.md §5.1 value_dim formula + §5.2/§7 dep_coeff wording — already edited on
  `dispatch/133`; confirm they land via the candidate→main integrate.
