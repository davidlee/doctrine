# Doctrine lifecycle: route → slice → design → plan → phase → audit → reconcile → close

The ordering an intentional change moves through. The routing table in
`.doctrine/state/boot.md` is the authority; the per-stage skills carry the
detail (see [[signpost.doctrine.skill-map]]). No code without an approved plan.

1. **/route** — the mandatory gate. Picks the governing skill before you touch
   anything.
2. **slice** (`doctrine slice new`) — scope the change into `slice-nnn.{toml,md}`
   under `.doctrine/slice/nnn/`.
3. **design** (`doctrine slice design`) — author `design.md`, then adversarial
   review (`/inquisition`) until the decisions lock.
4. **plan** (`doctrine slice plan`) — `plan.toml` (phases + EN/EX/VT criteria) +
   `plan.md` (rationale). Then `doctrine slice phases` materialises the runtime
   tracking sheets.
5. **phase-plan** — expand the next phase's authored entry into its disposable
   runtime sheet, just before executing.
6. **execute** — flip the phase `in_progress` (`doctrine slice phase`), implement
   TDD red/green/refactor (see [[pattern.doctrine.tdd-loop]]), end green, flip
   `completed`.
7. **audit** — evidence gathering, conformance checking, and reconciliation
   against the design. Uses the review ledger (RV kind). See
   [[signpost.doctrine.audit]].
8. **reconcile** — (ADR-009 closure seam) formal reconciliation of findings,
   coverage, and lifecycle status. Resolves blockers and drift before the close
   gate.
9. **close** — final commit, harvest durable findings, reconcile lifecycle
   status, and land a clean final commit.

Authored artifacts land under `.doctrine/slice/nnn/`; runtime phase sheets land
gitignored under `.doctrine/state/` (see [[concept.doctrine.storage-model]] and
[[signpost.doctrine.file-map]]). The whole loop in one line:
[[pattern.doctrine.core-loop]]. Phase ids (`PHASE-NN`) and criteria ids
(`EN-/EX-/VT-`) are immutable — edits append, never renumber.
