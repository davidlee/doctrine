# Dispatched-slice audit: primary tree answers the algebra, candidate tree the code

Candidate worktrees lack phase runtime state (conformance reads incomplete there) and check out detached; run registry/ledger/lifecycle verbs from the primary tree, advance the candidate branch ref after repair commits, and route selector edits to post-integration reconcile.

When auditing a dispatched slice (observed SL-222 / RV-284), two roots answer
different questions:

- **Candidate worktree** (`dispatch candidate create --worktree`) — the code
  surface only: build, suite, gate, greps, fix-now repairs. It is provisioned
  WITHOUT phase runtime state, so there `slice status` reads `phases: —` and
  `slice conformance` reports "incomplete — recorded row for PHASE-NN, which
  is not a completed phase". That is not registry corruption; ask the primary
  tree instead.
- **Primary tree** — the conformance algebra, the delta registry
  (`record-delta` resolves here even from linked worktrees), review-ledger
  verbs (they refuse worktree roots), lifecycle transitions.

Three wrinkles:

1. The authoritative **selector registry** may ride the impl bundle's
   `slice-NNN.toml` (workers run `selector add` mid-flight) while the primary
   copy is stale. Pre-integration `selector rm/add` on the primary forks the
   file and conflicts at landing — route those edits through the
   reconciliation brief for post-integration execution.
2. An audit **fix-now commit on the candidate branch** sits outside every
   phase's recorded delta, so its design-target cell stays "undelivered"
   until integration. Record that honestly in the finding response; don't
   force-fit a phase range.
3. The candidate worktree checks out **detached**. After committing a repair,
   advance the branch ref (`git update-ref refs/heads/candidate/NNN/… <new>
   <old>`) or the commit dangles and `candidate admit` pins the stale tip.
