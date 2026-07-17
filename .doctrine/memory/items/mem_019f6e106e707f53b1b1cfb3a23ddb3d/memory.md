# Audit surface discipline for dispatched slices

Auditing a dispatched slice: verify VTs and suites from the admitted candidate surface, not the primary tree — and expect belt-declared selectors to be missing from the primary registry

Pre-integration, a dispatched slice's code lives only on its evidence refs
(`review/NNN`, `phase/NNN-*`) and the candidate interaction branch — the
primary tree does not have it. Consequences at `/audit`:

- **`slice verify-vt` from the primary tree fails wholesale** (files absent /
  keywords absent). Not a defect: create the candidate (`dispatch candidate
  create --role review_surface`), cd into its worktree, and run verify-vt
  there — SL-220's battery went 14/14 PASS cleanly attributed from the
  candidate after failing 14/14 from primary.
- **Run the full suite + gate in the candidate worktree** (own in-tree
  `target/`), never against primary.
- **`slice conformance` will report adjudicated extensions as `undeclared`**:
  belt design-target declarations made in the coord tree never project back
  into the primary selector registry (IMP-294). Map undeclared cells against
  phase-sheet adjudications before treating any as scope creep; promote via
  `slice selector add` at reconcile.
- **Audit repairs land on the candidate branch** (fix-now commits in its
  worktree; the worktree is detached, so advance the branch ref with a CAS
  `git update-ref` and re-`dispatch candidate admit --review RV-NNN`).
- Run `dispatch candidate admit` from the primary root — inside the candidate
  worktree, root auto-detect finds no recorded candidate.

Related: [[mem_019ec912f7fd746284bfaef00717443e]] (close via admitted
close_target), [[mem_019f4c64e65574238b7026f7301c8a2c]] (evidence-ref worktree
needs web/map/dist), [[mem_019ee33f591d77f18144b8f76fa1021f]] (green-but-
incomplete phases).
