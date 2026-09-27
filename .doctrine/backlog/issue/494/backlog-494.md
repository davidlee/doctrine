# ISS-494: capsule-adopt directs audit to a worktree the review verbs refuse

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`capsule-adopt` lands a capsule's work on `capsule/<SL>/<gen>` in
`.worktrees/<SL>-<gen>` and prints "Audit here, not on edge". But
`review::turn::resolve_review_root` classifies any linked worktree whose branch
is not `dispatch/<NNN>` as a fork (`worktree::shared::classify_worktree_role`)
and refuses every guarded review verb (IMP-024). `/audit` cannot open or drive
its ledger there. SL-268's audit (RV-398 `F-5`) had to merge the capsule into
edge first.

Also: the adopted worktree has no gitignored `web/map/dist/`, so
`doctrine check gate` fails to compile there (RustEmbed folder missing) until it
is copied in.

Fix one side: either classify an adopted capsule worktree as a review-capable
locus, or change capsule-adopt's advice (and provision `web/map/dist`).
Related: IMP-240 (solo-fork audit path), ISS-484 (`review new` bypasses the
locus guard). Observation `01a0e021`.

## Resolution (2026-09-27)

Narrowed to the script half; fixed in `scripts/oubliette.sh` `back` (the
`capsule adopt` wrapper — the "Audit here" advice lived there, not in shipped
guidance):

- the landing worktree is provisioned with `doctrine worktree provision`
  (`.worktreeinclude` already lists `web/map/dist/**`);
- the advice now says what works: gather evidence in the worktree (`check gate`,
  `slice verify-vt`, `slice conformance --against`), open the RV ledger on edge
  after the merge.

The review-locus half is not fixed here. Keying the fork guard on the
`capsule/` branch shape would make the engine load-bear on a host convention
(POL-002 facet 1). The real invariant — the tree owns its state tier — needs a
doctrine-owned declaration; that is IMP-240 defect 2, carried there.
