# Review-capable worktree locus

## Context

This slice is RFC-032's roadmap slice 4 ("identity & locus"): decisions `D9`
then `D7` of `.doctrine/rfc/032/decision-frontier.md`. It was first scoped as a
doctrine-owned "this tree owns its state" declaration; that approach is close to
the "record a home tree" option `D7` rejects (trees are ephemeral, branch names
do not survive landing), so it was re-scoped on 2026-09-27.

The review verbs refuse any linked worktree that is not a dispatch
coordination tree. `resolve_review_root` (`src/review/turn.rs:169`) asks
`classify_worktree_role` (`src/worktree/shared.rs:77`), which decides by branch
shape alone: `dispatch/<NNN>` is `coord`, every other linked tree is `fork`.
Two kinds of tree that could safely write a review fail that proxy:

- a **solo fork** — a `/slice` branch worked in a `/worktree` (IMP-240
  defect 2, RV-238);
- an **adopted capsule worktree** (RV-398 `F-5`, ISS-494).

Neither can open an audit ledger where the code and phase state are, so the
audit lands the code on the parent tree first. SL-268 did exactly that.

Widening admission widens the id-collision window: under `reach = "local"`, a
reservation is a per-tree `mkdir`, so two trees of one clone can mint the same
id (ISS-279). `D7` therefore ships **with or after** `D9`, never before.

Already done outside this slice: `review new` now runs the locus guard before
allocating (ISS-484, commit `120b8b321`).

## Scope & Objectives

1. **D9 — clone-wide local reservation.** The `local` claim becomes a zero-oid
   `update-ref` CAS in the clone's common git dir, under its own prefix
   `refs/doctrine/reservation-local/<PREFIX>/<id>`. Allocation scans both the
   local and shared namespaces whatever reach is configured. Fixes the class
   for every kind (ISS-279). The scan also reads every live worktree's entity
   dirs, covering ids minted before D9 (DEC-337).
2. **D9 alongside — `reseat`** reads the alias slug through the lenient reader
   (ISS-277), and claims its destination id through the reservation backend so
   its pick sees sibling trees.
2a. **Reservation hygiene.** The fallback env opt-in is read once at the shell
   edge and passed in, so reserve tests are hermetic (ISS-483); the TTY
   fallback prompt names the persistent levers and says "this clone only".
3. **D7 — three locus tiers, one table, evaluated once** in the review verb
   dispatcher:
   - read (`show`, `list`, `findings`): any tree, including a confined worker;
   - runtime (`prime`, `status`, `unlock`): any tree with a writable state tier;
   - authored (`new`, `raise`, `dispose`, `contest`, `verify`, `withdraw`,
     `amend`, `reopen`, `conclude`): any tree that can durably write
     `.doctrine/review/`; refused in a confined dispatch worker (the
     `DOCTRINE_WORKER` marker / read-only authored tier).
   Replaces the branch-shape test; no host branch format is recognised
   (POL-002). Simplified by DEC-338: the single test is worker mode
   (`DOCTRINE_WORKER`), refusing the whole guarded set in a worker.
4. **Every RV mint path obeys the guard.** The design run's `mint_review` call
   (`src/commands/design.rs:1487`) is covered by the CLI worker guard, since it
   has no MCP surface (ASM-012); no code change.
5. **Guidance.** `/audit` and `review-ledger.md` say where an audit can run
   (IMP-190), and state the rule "one writer per RV at a time; git merge is a
   best-effort backstop".
6. **Consumer.** `scripts/oubliette.sh back` advice returns to "audit here"
   (project-local; validates the change).
7. **Governance.** A REV amends ADR-007 D-C1/D-C7 (tiers; "one tree" → "one
   writer per RV at a time") and revises PRD-005 / SPEC-008 (`reach = local`
   means "this clone").

## Non-Goals

- Recording a home tree or branch on the RV (`D7` rejected).
- Concurrent multi-tree editing of one RV, or a semantic journal merge.
- Worker forks writing reviews, or parallel raisers (IMP-024).
- Pushing local claims when a clone moves to `shared` reach.
- Provisioning embed assets in general audit worktrees (IMP-291).

## Summary

Make id reservation clone-wide (`D9`), then admit review writes by capability
rather than branch shape (`D7`), so solo forks and adopted capsules can be
audited before landing.

## Risks & Open Questions

- **R1 — D1 not landed (retired).** The baton is already a pure cache
  (`src/review/turn.rs:35-58`; SL-268 put turns in the ledger), so a review
  worked in a linked tree loses nothing on landing (DEC-338).
- **R2 — reservation contract change.** `reach = local` changes meaning; clones
  holding existing per-tree claims must not re-mint them.
- **OQ-1 — landing (answered).** Audit and close in the linked tree, then
  land; phase status is not needed after close.

## Verification

- VT: two linked trees of one clone allocating concurrently get distinct ids.
- VT: allocation sees claims in both reservation namespaces, and ids in a
  sibling live worktree's entity dirs.
- VT: `reseat` renumbers a review; an explicit `--to` onto a claimed id refuses.
- VT: reserve tests pass with `DOCTRINE_RESERVATION_FALLBACK` set (ISS-483).
- VT: the review guard admits primary, coord and solo linked trees and refuses
  a worker; read verbs work in a worker.
- VT: an RV opened in a solo linked tree runs raise → dispose → verify →
  conclude to `done`, and lands.
- VA: audit skill and `review-ledger.md` state the rule; REV recorded.

## Follow-Ups
