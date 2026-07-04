
[reconcile→close; SL-198-recon-c63c1397]
Coord/primary state-split cost the reconcile stage real tokens. Slice lifecycle
status is AUTHORED (slice-198.toml), yet the dispatch-executed+audited slice
advanced its status only in the coord tree's copy; the primary tree sat stale at
`ready`, so /reconcile had to walk ready→started→audit→reconcile by hand before
the closure seam would accept the reconcile transition (audit→reconcile). Same
class as the phase-completion split hit last session. Signpost: a dispatched
slice's authored lifecycle status + phase completion both need explicit
primary-tree catch-up before audit/reconcile/close verbs behave. A
`dispatch sync --lifecycle` (fold coord-tree authored status/phase flips back to
primary) would erase this recurring hand-walk. Also: candidate `admit` needs a
`candidate create`-recorded candidates.toml row — a hand-built merge+branch -f
(as done for the review surface last session) leaves `candidates(none recorded)`,
so admit refuses on provenance; close must run a fresh `candidate create`.

[/plan; SL-199-prelock-review-a1]
Pre-lock hostile review (codex GPT-5.5) over the transactional surface cost real
tokens on two verification detours the plan text should have pre-empted:
- Judging finding severity required disambiguating a two-tier storage subtlety the
  plan/design never state outright: boundaries.toml is COMMITTED on the coord branch
  (`dispatch/<NNN>`) yet GITIGNORED on trunk (.gitignore `.doctrine/dispatch/`), so
  "atomic commit" residue is poison, not disposable runtime. Had to read
  run_record_boundary + .gitignore + storage model to rank it. Plan naming the
  storage tier of each committed output up front would save this.
- The plan named the WRONG reuse seam for the coord resolver (worktree_for_ref /
  live_worktree_for_ref — single-hit first-match) vs its own design's enumerate
  intent (list_worktrees). Verifying the mismatch cost a source read. A plan that
  cites the exact fn it reuses (and its arity/return) is cheaper to review.
- Net: both "blocker" findings collapsed to ONE root cause (server commit must be
  working-tree-free via commit_tree/scratch-index, which the repo already has) once
  the seams were read. The design asserting the mechanism (not just "commits
  server-side") would have pre-answered the reviewer.
