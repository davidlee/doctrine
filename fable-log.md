# fable-log.md — Fable loop activity log

> One entry per active turn: did / observed / next / haiku. Newest appended at bottom.

---

02:55 — waiting (window opens 03:45)
## 03:06 — waiting (window opens 03:45)
   note: .git gitdir -> /home/david/dev/doctrine/.git/worktrees/doctrine-fable-loop; `git` fails in-jail ("not a git repository"). Resolve at window open before any commit.
   RESOLVED (host, pre-window): gitdir wiring relativized so it resolves under any prefix (host /home/... AND jail /workspace/...). worktree .git -> `../../.git/worktrees/doctrine-fable-loop`; admin back-ref -> `../../../.worktrees/fable-loop/.git`. `git` now works in-jail without edits. Caveat: if a real commit fails with a read-only/permission error (NOT "not a git repository"), the jail ro-binds shared .git — if you can't fix it from /workspace/doctrine, you'll have to do without.

## 03:55 — turn 1 (first active): oriented, unblocked build, shipped ADR-001 gate-edge-model note
observed: fable-loop worktree was hand-created (raw `git worktree add`), so it
  bypassed `.worktreeinclude` and lacked `web/map/dist/` → cargo build failed
  (RustEmbed). Copied dist from main worktree (gitignored). Tree green.
  Chose thread = RFC-011 friction burn-down. Increment 1 shipped: ADR-001 body
  now documents the layering gate's edge model (top-level→first-segment edges,
  BTreeSet-deduped; sub-class refines direction check only, NOT tangle ratchet)
  — both claims verified vs tests/architecture_layering.rs. Commit ceff6e90.
  Increment 2 (memory retrieve phrasing) = OBE; boot-footer already fixed on edge.
  vtgate comment-match = by-design (POL-002). Two case-notes already OBE →
  burn-down has high verify-then-skip cost, so launched a background triage
  sub-agent (a608c2f9a5f7f8337) to classify ALL remaining case-notes and hand
  back a clean still-open doc-gap worklist.
next: process triage worklist; ship the top still-open doc-gap fix.

  Edges collapse to
  top-level; the map
  hides the deep paths.

## 04:05 — turn 2: processed triage, shipped top-3 skill/doc guardrails
observed: triage sub-agent (a608c2f9) classified all remaining case-notes. Most
  OBE (RV-216-contiguity fixed by ISS-058) or by-design (worker confinement,
  --status flag, clippy denies). Three still-open, high-value, low-collision
  doc-gaps shipped, each verified vs code first:
  - a7e9ec0e: /audit + /reconcile brief-surface guardrails — plan.toml criteria
    are immutable-append & off-surface; conformance findings must name the
    selector-registry verb (slice selector rm/add), not §6 prose (the mirror).
  - fe3eed9a: /close — stage the */new slug symlink (git add <kind>/NNN*), which
    a path-scoped add of the numeric dir strands uncommitted.
  Remaining actionable = worktree land --fork error hint (code, next) + a hot
  arm-spawn doc-gap (defer). check quick is a no-op (no quick suite configured).
next: TDD the worktree land --fork path-vs-branch error hint in src/worktree/land.rs.

  Notes triaged clean —
  the stale friction falls away,
  three guardrails land.

## 04:12 — turn 3: shipped worktree land TDD fix; launched skill CLI-shape audit
observed: TDD'd the `worktree land --fork` no-such-fork path-vs-branch hint —
  pure `no_such_fork_message` helper (land.rs) + unit test in mod.rs; e2e
  assert_refusal uses `.contains(token)` so the VT golden survives the appended
  hint (10 e2e pass, clippy+fmt clean). Commit e0e7b7b1. Note: doctrine is a BIN
  crate — `cargo test --bin doctrine <name>`, NOT `--lib`.
  Top RFC-011 case-note worklist now cleared (4 fixes shipped). Pivoted the
  thread to the general root cause — CLI-shape citation drift in skills — and
  launched a background audit agent (ae7ce25147e69f8c5) to verify core-lifecycle
  skills' command citations against `--help`, returning confirmed mismatches only.
next: process the audit agent's confirmed mismatches; apply verified fixes.

  One helper, one hint —
  the phantom fork now speaks its
  name: branch, not path.

## 04:15 — turn 4: applied skill CLI-shape audit fixes (4 mismatches)
observed: audit agent (ae7ce25) verified all 22 core skills' `doctrine` command
  citations vs --help; skills mostly CLEAN (good). 4 real positional-vs-flag traps
  fixed, same class as the RFC-011 case notes:
  - backlog/SKILL.md (high): `backlog edit <ID> (prompts)` was a copy-paste trap —
    verb requires --status <STATUS> (+ --resolution for terminal); nothing prompts.
    Fixed table + 2 prose claims. Re-verified myself (bare edit errors on --status).
  - handover/SKILL.md (low): `slice phase … in_progress` positional → --status flag.
  Commit 305c8638. Everything else confirmed correct; memory find = hidden alias
  of memory search.
  Session tally: 5 substantive cherry-pickable fixes (ADR-001, audit+reconcile,
  close, worktree land TDD, skill CLI shapes).
next: verify+TDD a fresh isolated backlog code fix (IMP-056 coverage Debug-format);
  OBE-check first.

  Skills speak true now —
  no phantom prompt, no lost flag;
  the copied line runs.

## 04:25 — turn 5: shipped IMP-056 (coverage kebab render), launched fix-queue scout
observed: verified IMP-056 still real (coverage_verify.rs:261 `format!("{status:?}")`).
  TDD'd a single-source `CoverageStatus::as_kebab` (requirement.rs, = parse_status
  kebab vocab), routed status_label + withdrawal_line through it, killed the
  Debug-spelling leak (InProgress→in-progress). Round-trip test asserts
  parse_status(st.as_kebab())==st for all 5 variants — locks render/parse against
  drift. Updated unit + e2e record goldens to kebab register. All green
  (store 17 / verify 19 / e2e 13,1,15), clippy+fmt clean. Commit 07f9a4a2.
  To keep the pipeline full, launched scout a7e3ab8d for a vetted queue of more
  small isolated TDD-able backlog fixes (verify-still-real, low-collision).
next: process the scout queue; TDD the top verified item.

  Debug name gives way —
  one kebab token, in and out;
  parse and render kiss.
