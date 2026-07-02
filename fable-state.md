# fable-state.md — Fable loop working memory

> Your brain across turns. You wake blind; this is continuity. Read first, update last.

OBJECTIVE: RFC-011 friction burn-down — convert the RFC-011 case-notes
(`.doctrine/rfc/011/case-notes.md`) into concrete root-cause fixes. Each
increment: one evidence-backed friction → fix at root (authored doc OR code+TDD)
→ clean self-contained commit on `fable-loop` the User can cherry-pick. Sequence
by value/token, lowest-collision first.

WHY-IT-MATTERS: RFC-011 (token efficiency) is the User's ACTIVE benchmarking
initiative. The case-notes are raw friction logs; turning them into durable root
fixes reduces token waste for every future agent — directly the RFC-011 mission.
Compounds, low-collision (docs + isolated code), doesn't need User decisions.

PLAN (ordered increments, each finishes clean within a turn):
  1. [DONE] ADR-001 gate-edge-model note (SL-180-sess1 friction: wrong tangle
     proof). Verified vs tests/architecture_layering.rs; added subsection.
  2. `memory retrieve <key>` positional rejected but boot Onboarding phrasing
     reads positional (SL-180-sess1). Fix: accept positional key OR fix boot
     doc phrasing. Verify actual CLI shape first.
  3. `worktree land --fork` expects branch not path; `no-such-fork` error
     confusing when the fork DIR exists (ISS-058 note). Fix: clearer error /
     accept path. Code + test.
  4. `arm-spawn --slice` diagnostic-only → wrote arming base to wrong dir
     (sl189 note). HIGHER collision risk (active dispatch/SL-190). Assess late;
     maybe file backlog instead of touching hot code.
  5. Consider: empty IMP body templates forcing reverse-engineering (~7k tokens)
     — process/skill note, not code.

PROGRESS (newest first):
  - Increment 1 DONE: ADR-001 body gained "### Gate edge model" subsection —
    edges are top-level→first-segment, BTreeSet-deduped; sub-classification
    refines direction check only, NOT the tangle ratchet (count_tangle_edges
    over top-level units). Both claims verified against the gate source.
  - Env unblock: fable-loop worktree (hand-created via raw `git worktree add`,
    bypassing .worktreeinclude) lacked `web/map/dist/` → cargo build failed
    (RustEmbed). Copied dist from main worktree (gitignored; no commit impact).
    Tree now green.

NEXT-ACTION: Code increment (fresh, self-contained, TDD) — `doctrine worktree
land --fork` bails with a bare `land-refused: no-such-fork` even when a fork DIR
(not branch) was passed; `--fork` wants a BRANCH name (ISS-058 case-note). Add a
path-vs-branch hint to the error. Find it in `src/worktree/land.rs` (`run_land`,
the `no-such-fork` bail). TDD: red test asserting the hint, green, refactor.
Triage rated collision med (worktree area) but my branch is isolated + User
cherry-picks, so a clean tested error-msg improvement is a fine gift. If it turns
messy/hot, file a backlog item instead and move to another isolated code fix.

TRIAGE WORKLIST (agentId a608c2f9a5f7f8337, done ~03:59) — full table saved in
this turn's reasoning. Top-3 doc-fixes ALL SHIPPED. Remaining actionable:
  - worktree land --fork error hint (cli-behavior, code) → NEXT-ACTION above.
  - arm-spawn --path guard (doc-gap but HOT dispatch/SL-190) → defer or backlog.
  - backlog/SKILL.md body-discipline note (#4, low value, author-dependent) → skip
    unless idle.
  Everything else: OBE (RV-216-contiguity fixed by ISS-058) or by-design
  (worker confinement, marker e2e red, --status flag, clippy denies, References
  role) — no action.

INCREMENT-2 RESULT: OBE. `memory retrieve` is scope/query-based by design (no
positional); `install/boot-footer.md` ALREADY reworded on edge (adds
`doctrine_onboard` MCP path + "use /retrieving-memory skill to retrieve `X`").
Did not edit — no churn on freshly-updated User text.
Side-finding (NOT a bug): boot snapshot Memory index in session context is STALE
— `mem.signpost.*` keys no longer exist in the edge corpus (memories reorganized);
`memory show mem.signpost.doctrine.overview` → "not found" is correct.
LAYERING GATE verified GENUINELY live-tested (`architecture_layering_gate` runs
check_layering→count_tangle_edges); `#[allow(dead_code)]` on count_tangle_edges/
tarjan_scc is a harmless leftover, not theater.

OPEN-QUESTIONS:
  - Q1: Work on `fable-loop` never lands to main/edge. Adoption = User
    cherry-picks clean commits. Keep every commit self-contained + well-scoped.
  - Q2: Increment 4 (arm-spawn) touches hot dispatch code (SL-190 in audit,
    RFC-012 open). Prefer filing a backlog item over editing, to avoid collision?

DECISIONS-LOG (fork → options → choice → why; append-only):
  - D1 [thread choice]: {advance a random backlog IMP} vs {RFC-011 friction
     burn-down} vs {memory /dreaming}. Chose RFC-011 burn-down — directly serves
     the User's stated active initiative, evidence-backed worklist already
     exists (case-notes), low-collision, each increment independently adoptable.
  - D2 [vtgate false-positive]: investigated the RV-216 "keyword false positive"
     note as a candidate correctness fix. Found src/vtgate.rs treats comment-
     keyword matches as an ACCEPTED, POL-002-reasoned weakness (threat model =
     omission, not adversary). NOT a bug. The RV-216 note refers to a different
     keyword mechanism (plan-review VT grep). Did not touch vtgate. Avoided
     "fixing" intended behavior.
  - D3 [dist provisioning]: copy dist from main worktree (reversible, gitignored)
     rather than build the bun frontend (may lack toolchain in jail). Reversible.
