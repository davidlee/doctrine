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

OBJECTIVE (evolved): started as RFC-011 case-note burn-down (top items now all
shipped); PIVOTED to the general root pattern behind them — CLI-shape citation
drift in skills (every wrong flag = token waste for every agent). Same mission
(token efficiency), broader leverage.

PLAN / PROGRESS:
  1. [DONE ceff6e90] ADR-001 gate-edge-model note (verified vs layering gate).
  2. [OBE] memory retrieve phrasing — boot-footer already fixed on edge.
  3. [DONE e0e7b7b1] worktree land --fork no-such-fork path-vs-branch hint —
     pure no_such_fork_message + TDD unit test; e2e goldens preserved (10 pass).
  3b.[DONE a7e9ec0e] /audit + /reconcile brief-surface guardrails (plan criteria
     off-surface; conformance findings name the selector-registry verb).
  3c.[DONE fe3eed9a] /close stage-the-slug-symlink guidance.
  4. arm-spawn --slice doc-gap — HOT (SL-190/dispatch). Still deferred; file
     backlog OR skill note only if the CLI audit surfaces it cleanly.
  5. [NEXT] SKILL CLI-shape drift audit — background agent ae7ce25147e69f8c5
     (launched ~04:12) verifies core-lifecycle skills' `doctrine` command
     citations against `--help` ground truth, returns confirmed mismatches only.
     Apply the verified high/med fixes, each a clean commit. This is the current
     compounding thread for the rest of the window.

PROGRESS (newest first):
  - Increment 1 DONE: ADR-001 body gained "### Gate edge model" subsection —
    edges are top-level→first-segment, BTreeSet-deduped; sub-classification
    refines direction check only, NOT the tangle ratchet (count_tangle_edges
    over top-level units). Both claims verified against the gate source.
  - Env unblock: fable-loop worktree (hand-created via raw `git worktree add`,
    bypassing .worktreeinclude) lacked `web/map/dist/` → cargo build failed
    (RustEmbed). Copied dist from main worktree (gitignored; no commit impact).
    Tree now green.

NEXT-ACTION (WIND-DOWN): full bin test suite running (bg b5h1hxzlo) to verify all
11 fixes collectively green pre-handover. On green: session is complete — hold at
low cadence (window open till 08:45) unless a genuinely safe+valuable item appears.
Do NOT force the scout #3 items (all brush design/zone lines — see below). On any
red: investigate + fix before anything else.

SCOUT #3 FINDINGS (real issues, but NOT autonomous-safe — surfaced for the User):
  - IMP-235 (vtgate): line-anchored `patterns` false-Fail a formatter-wrapped
    multi-line assert (src/vtgate.rs:139). Real, but the card lists options
    "not yet chosen" → a DESIGN CHOICE on a governance gate. Needs a decision.
  - IMP-107 (review engine): ReviewError::DanglingRef / LockContention declared
    but never constructed (#[expect(dead_code)] at review.rs:908 reason="PHASE-02
    verb handlers"); run_new (1246) leaks a generic anyhow so MCP returns -32603
    Internal not -32000 DANGLING_REF. Real + fully-specified (5 steps), HIGH conf
    — but it's COMPLETING PLANNED PHASE-02 WIRING in the review avoid-zone. Best
    candidate if the User wants it done; not autonomous-clean (planned work + zone).
  - IMP-192 (state.rs:563/577): spurious `phase-binding capture skipped` warning
    on a legitimate solo no-binding → completed flip (stderr noise). Real, but
    suppress-vs-downgrade is a judgement call + the full card is a /close recipe.

ISS-059 DEFERRED (design choice, not autonomous): contentset::compute (src/
contentset.rs:129) fs::read aborts IsADirectory on a directory/symlink-to-dir
selector (e.g. a memory-master dir) → `review prime` fails. Root: resolve_
selectors_to_fileset (review.rs:2592) passes a LITERAL selector through
unexpanded, so a symlink→dir reaches compute. NO existing dir-content-hash
pattern to reuse (grep found none). Two candidate fixes, each a real drift-
semantics DECISION (belongs in a slice, not autonomous invention):
  (a) expand a dir/symlink→dir selector to its tracked files in resolve_selectors_
      _to_fileset (canonicalize symlink → git ls-files the real dir → per-file
      entries; drift stays per-file, consistent with globs). Touches review-zone
      selector expansion.
  (b) hash the directory's contents in compute (walk+combine) — defines new
      per-dir drift semantics (walk order, symlink depth, removals).
Recommend (a). Left for a design decision / slice.

SCOUT #2 QUEUE remainder: IMP-135 (help-text, low), IMP-137 (needs --remove =
capability, skip for autonomous). IMP-139 + IMP-140 DONE.

IMP-183 DEFERRED: rendering estimate/value in backlog show needs config units
threaded through format_metadata/format_show/format_inspect (+ tests + JSON
parity + goldens) — signature churn across the whole backlog render surface,
borderline slice-worthy. Not a single clean autonomous increment. Left for a
proper slice.

FIX QUEUE (re-verify each before building — OBE risk real):
  - [DONE 1dacc7a8] ISS-003 cordage explain() foreign-node empty cone.
  - [DONE e30e482e] IMP-211 next value cell shows effective default (1.0*) not
    ABSENT for value-bearing kinds; DEFAULT_VALUE now pub(crate) single-source;
    marker named const; golden updated (it had encoded the bug).
  - IMP-183 (M, MED) estimate/value write-only on non-slice kinds → NEXT-ACTION.
  - IMP-019 (M) cordage golden_net value oracle — test hardening only, no runtime
    bug. Lower urgency; do if code fixes dry up.
  - ISS-205 defer (no clean local red).
  - IMP-183 (M, MED) estimate/value facets writable to any kind but only
    slice show renders them (write-only metadata). Add rows to backlog show via
    existing estimate::display::format_estimate_confidence / value::format_value_normal
    (mirror slice.rs:1945/1958). Keep to backlog show (≤2 files); knowledge = FU.
  - IMP-019 (M) cordage golden_net has no independent value oracle (proof gap,
    test hardening only — no runtime bug). Lower urgency.
  - ISS-205 (S, MED-LOW) cordage denylist.rs env!(CARGO_MANIFEST_DIR) baked path
    — but passes locally (compiled in place); no clean red. Defensive; defer/skip.

[DONE 07f9a4a2] IMP-056 — CoverageStatus rendered via as_kebab (single source,
= parse_status vocab) instead of `{:?}` Debug; round-trip test locks the pair;
unit+e2e goldens updated to kebab. 6th fix. TDD, clippy+fmt clean.

CAPSTONE (reserve for ~08:00+): RFC-011 friction taxonomy synthesis — a durable
artifact categorizing the friction classes + this session's remediation status
(fixed / OBE / by-design / open) as input for the User's RFC-011 writeup. Only
if code threads dry up or window is closing.

[DONE 305c8638] Skill CLI-shape audit applied: backlog edit (prompts→--status
required; high-sev trap) + handover slice-phase positional→--status flag. Audit
verified all other slice/worktree/dispatch/revision/rec/spec verb shapes correct;
`memory find` is a live hidden alias of `memory search`.

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
