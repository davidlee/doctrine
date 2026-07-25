
[route→plan-review; SL-223-opus-review-4fb6]
Handover prompt asserted "self-review already added asset_source=leaf (PHASE-01)
and publication=engine (PHASE-02) to layering.toml" as applied state. Actual: the
self-review commit (83f2fc4) added them as phase EXIT-CRITERIA (EX-3/EX-5) to land
in-phase — the rows are NOT yet in layering.toml. Correct sequencing, but the
"already added" framing cost a verification cycle chasing rows that don't exist
yet + a git-show to disambiguate criteria-vs-applied. Handover prompts describing
governance state should distinguish "mandated by criteria" from "applied to disk."
[backlog; pass-1-tag-sweep]
Pass 1 tag sweep: tagged all ~200 open backlog items. 
- 55 previously untagged items now carry area: tags
- 102 items carry cluster: tags across 13 clusters
- 2 items (IMP-267, IMP-298) had non-area tags but no area: prefix — now dual-tagged
- Tag taxonomy used: area:dispatch, area:gate, area:close, area:selector, area:backlog, area:memory, area:spec, area:review, area:cli, area:testing, area:entity, area:priority, area:web, area:worktree, area:governance, area:skills, area:relations, area:coverage, area:config, area:boot, area:quality, area:onboarding, area:install, area:security, area:docs, area:ci, area:mcp, area:ux, area:audit, area:architecture, area:conformance, area:concept-map, area:graph, area:cleanup, area:requirements, area:lifecycle
- Cluster tags: dispatch-funnel, worker-gate, close-lineage, selector-scope, backlog-tooling, memory-system, web-map, priority-scoring, spec-requirements, review-system, entity-engine, testing-goldens, cli-ux
Now filterable: `doctrine backlog list --tag area:dispatch`, `doctrine backlog list --tag cluster:worker-gate`
[backlog; pass-2-dedup]
Consolidations from pass 1 analysis:
- ISS-218 enriched with IMP-270 content (SL-199 F-1 evidence, workspace-build alternatives), IMP-270 closed as duplicate
- IMP-243 merged into IMP-109 (dep/seq folding is third parse to eliminate), IMP-243 closed as duplicate
- IMP-127/IMP-201/IMP-174 NOT merged (genuinely different tiers: CLI verb / code-tier process / authored-tier guidance); wired: IMP-201 after IMP-174, IMP-127 after IMP-201
- IMP-256 after IMP-162 (over-broad lint before completeness check)
- IMP-178/179/180 kept separate (different problems in TOML/error space, all thin)
- Vestigial non-area tags cleaned from IMP-267, IMP-298

[phase-plan+execute; SL-223-P04-a] Could not read the PHASE-04 plan entry via
the CLI: `doctrine slice plan SL-223`, `slice show SL-223 --plan`, and a
`slice plan` sed all returned empty/usage errors, so I fell back to grepping
`.doctrine/slice/223/plan.toml` raw — against the "read via show, not raw
files" guardrail, but the CLI surfaced no per-phase plan view. A
`slice plan <id> --phase PHASE-NN` (or `slice phase show`) that prints one
phase's objective/EN/EX/VT would remove the raw-TOML fallback. ~2 wasted
tool round-trips locating the block by line number.

[execute; SL-223-P04-b] Pre-existing wart surfaced, not caused by this phase:
`just nix-build`'s trailing `direnv reload` runs unconditionally, so in a jail
(no direnv) the recipe exits 127 despite its comment claiming it "skipped with
a notice where nix is absent". Guarded it (`command -v direnv && … || true`)
since this phase's VA-1 jail-skip story depends on it. Flags a broader pattern:
recipes that document a graceful jail-skip should be exercised in the jail, or
the claim silently rots.

[close; SL-223-close-a] Close-gate "status-lag drift" discharge is heavyweight and
non-obvious. Recording VA coverage at reconcile (the honest, design-mandated act)
GUARANTEES a close refusal that then requires a multi-step manual dance:
(1) a REV to advance authored requirement status (or not); (2) `revision apply`
auto-mints `move=revise` RECs that record the status change but DO NOT discharge the
drift; (3) hand-author one `move=accept` REC per requirement, each needing a
manually-edited `[[status_delta]]` (to==authored) + `[[evidence_ref]]` (superset of
every coverage key) table pasted into rec-NNN.toml — the CLI seeds only a skeleton.
Four requirements → four hand-authored TOML files. The `rec new` verb has no flags to
populate the delta/evidence tables, so the reconcile writer is doing raw TOML authoring
against the storage-rule grain. A `doctrine rec discharge --slice SL-N --requirement REQ-N`
(or `slice status done --discharge-drift`) that mints the accept REC with its
evidence_ref auto-resolved from the coverage store would collapse ~all of this.
The recipe only survives because a memory (mem.pattern.doctrine.close-drift-discharge-rec)
documents the exact three-clause predicate; without it the refusal message alone is
hard to action correctly (esp. clause (c): "superset of every coverage key incl. other
slices' cells" is invisible without grepping coverage.toml).

[research; SL-212-research-8a3f]
Research synthesis for SL-212 (IMP-127 hand-resolve ingest, gated on RFC-006)
required reading 5+ memory retrievals, 8+ entity shows, RFC-005 post-RSK-014
design note, and the full split-lineage close memory corpus — ~20 tool calls
before the first line of the research doc was written. The gate itself (RFC-006
blocking SL-212 design) means the architect who does the design will need to
repeat much of this context-gathering unless a research artifact bridges the
gap. The `research.local.md` pattern (gitignored runtime research in the slice
folder) is lightweight and avoids authored-tier commitment, but the token cost
of producing it (~12k words of reading across entities) is high per-session.
A `doctrine research compile --slice SL-NNN` that materialises the entity
closure (slice + related RFCs + ADRs + memories + backlog items) into one
document would collapse ~15 of those 20 tool calls into one.

Token cost breakdown (approximate): SL-212 scope show (3k), RFC-006 show (3.5k),
ADR-012 show (9k), IMP-127 show (1.5k), 5 memory retrievals (~2.5k each =
12.5k), SL-157/211/166/198/220/190/147 shows (~2k each = 14k), RFC-005 post-
RSK-014 note (6k), RFC-005 show (3k), RFC-016 show (2k), IMP-201 show (1k),
case notes read (0.5k), backlog list (1k), memory search (1k). Total entity
reading: ~57k words before synthesis. The research doc itself is ~3.5k words.
Net: ~17:1 reading-to-output ratio for governance-gated research.

[design; SL-212-REV-030-a]
Governance-heavy /design pre-work (gate dissolution → REV author/apply). Friction:
- `doctrine revision list` returned EMPTY output despite 30 revisions existing;
  had to fall back to `revision paths`/`ls` to enumerate. Cost a verification loop.
- Entity creation mints BOTH `NNN/` (canonical dir) AND `NNN-slug` (symlink).
  In `git status` this shows as two `??` entries per new entity — initially reads
  as accidental duplication; had to `readlink`/`stat` to confirm one is a symlink.
  (A concurrent agent added a "# Symlinks" note to governance.md the same session —
  so this confusion is known/recurring.)
- Harness shell is zsh, not bash: unquoted `$PATHS` in `git add $PATHS` / `git
  commit $PATHS` did NOT word-split (zsh default), so a variable-pathspec commit
  failed with a single-giant-path error. Had to pass literal args. Any skill/snippet
  that builds a pathspec in a shell var and relies on bash word-splitting breaks here.

[code-review; SL-212-final-projection-20260722]
The code-review protocol required opening and priming a durable RV before a
single-purpose design-lock probe. This added lifecycle writes and command-shape
lookups even though the user explicitly requested only residual blocker/major
prose. The durable subject made the ledger appropriate, but the generated RV
starts in derived `done` state with zero findings before its mandatory Brief is
filled, which is semantically surprising during an active clean review.

[execute; SL-212-PHASE-02]
Smooth pure-predicate phase. Two minor friction points, ~low token cost:
1. `doctrine slice phase <id> <phase> <status>` — status is a *flag*
   (`--status`), not positional. One failed invocation + re-read of usage.
   The three-positional muscle-memory (id, phase, status) misfires.
2. `code_end_oid` is NOT stamped into the phase sheet on `completed` (only
   `code_start_oid` on `in_progress`); the closing binding lands in
   `state/slice/NNN/boundaries.toml` (the arm-neutral registry). The execute
   skill's "captures code_end_oid = HEAD" phrasing reads as if it writes the
   sheet — momentary grep-for-nothing before checking conformance/boundaries.
Both self-resolved in one probe each. The staged-ahead dead-code pattern
(cfg_attr(not(test), expect(dead_code, reason=…))) was pre-captured as a
memory and applied first try after the initial bare-#[expect] clippy bounce.

[phase-plan; SL-212-pp-05] design.md §9 F18 cites concrete line anchors for the
de-absolutisation targets (dispatch.rs:1815 status, admit :1551/:1610, ledger
:144/:194). All had drifted after PHASE-01..04 added code — status is now :2159,
admit doc :2018 / errors :2077/:2091, ledger docs :152/:209. Re-locating each via
grep cost ~3 extra tool calls. Minor, but a recurring tax: authored design line
anchors go stale the moment the phases they describe land. Function-name anchors
(as AGENTS.md already prefers for memories) would survive.

[execute; SL-212-PHASE-05]
Two incidental-complexity items this phase:
1. fmt-after-commit churn: committed T2/T3/T4 units, then `doctrine check gate`
   ran `cargo fmt` and reformatted the just-committed test blocks (multiline arg
   lists), dirtying the tree post-commit → an extra `style(SL-212)` fixup commit.
   Cost: one avoidable commit + a confused "why is the tree dirty after I
   committed?" beat. Fix: run `cargo fmt` (or `doctrine check quick`) before each
   commit, not only at the phase-end gate. The execute skill says "lint as you
   go" but the fmt step is easy to defer to the gate.
2. VA-1 site under-enumeration: the phase sheet's T1 listed 5 de-absolutisation
   sites (candidate_admit fn doc + 2 errors + 2 ledger docs). The VA-1 confirming
   read surfaced TWO more admit-provenance docs asserting "Doctrine candidate
   merge" as the basis — git.rs `parents()` helper doc and the `Admit` CLI enum
   doc (dispatch.rs:325). A `grep -rn "Doctrine-created\|Doctrine merge\|Doctrine
   candidate merge" src/` up front would have caught all 7 in one pass; the hand-
   curated site list missed 2/7 (~29%). Lesson: for a "remove absolutism X"
   sweep, drive it from a grep, not an enumerated site list — the VA read then
   confirms a clean grep rather than re-discovering sites.

[audit; SL-212-audit-recon-290]
Stale installed binary during audit. The boot snapshot points `doctrine` at
`~/.cargo/bin/doctrine`, but SL-212 *adds* a CLI verb (`dispatch candidate
ingest`) and a `slice selector` surface the installed binary predates — so
`doctrine dispatch candidate ingest --help` and `doctrine slice selector list`
both failed with "unrecognized subcommand" mid-audit, costing a round-trip
before switching to `./target/debug/doctrine`. General case: auditing a slice
that ships a new CLI verb must drive that verb from the freshly-built dev binary;
the installed one lags until reinstall. Cheap fix would be a one-line reminder in
the audit skill's evidence step ("use ./target/debug/doctrine for verbs the slice
under audit introduced").

[inquisition; SL-225-RV-291-gpt5]
`review new --raiser inquisitor` stamps a free-form posture label, but the
subsequent `review raise --as` flag accepts only the fixed cooperative role
`raiser`, not that label. Reading `review raise --help` did not prevent the
misstep because the help says only "cooperative role assertion (default:
raiser)" and does not list the closed vocabulary. One failed invocation plus
retry; cheap help improvement: print `raiser | responder` as possible values and
clarify that `--raiser` labels the participant while `--as` selects its fixed
ledger role.

[feedback; SL-225-RV291-disposition]
Verifying inquisition findings against the artifact needed the raw finding
toml: `doctrine review show RV-291` renders the prose companion + finding
titles, but not each finding's disposition/response fields, and `--findings`
is not a flag. Had to `Read` .doctrine/review/291/review-291.toml directly —
a mild "read via show, not raw files" tension, since the structured
disposition tier has no `show` surface. A `review show --findings` (or the
dispositions inlined into `show`) would have saved one raw-file read.

[inquisition; SL-225-RV-292-memory-dedup]
Before recording the reusable `git-common-dir` topology fact, two scoped
`memory find` queries failed to surface the already-existing near-exact memory
`mem.fact.dispatch.coord-root-not-git-common-dir`. Only the post-write
suggested-relations pass revealed it. The new record is broader and was related
instead of discarded, but the authoring path still paid a duplicate discovery
cycle despite following the prescribed pre-record search. A title/body semantic
duplicate hint before minting would make the record-memory discipline cheaper
and prevent needless corpus growth.

[dispatch; SL-215-harvest]
Prose-only slice (13 markdown targets, no engine/CLI). The dispatch funnel's
per-phase S1 regression diff runs the full Rust per-test suite (~56s/run here,
0 baseline failures at B). For a net diff verified markdown-only, new∪changed is
definitionally empty — the suite is invariant when no .rs/Cargo.toml/.lock is
touched. Running it 2×/phase (capture+diff × 5) is ~9min of compute for zero
signal. Orchestrator judgment: keep baseline captured at B; per-phase gate with
the cheap `check prove` (~13s) + net-diff inspection + R-5 belt + VT keyword
mandate; run full `check gate`+regression once at the PHASE-05 delivery gate.
Funnel intent (catch a worker delta that breaks build/tests) is preserved by the
markdown-only invariant check + prove + final gate. Note: the funnel prose
assumes a code slice; a prose-only slice has no cheap "regression is vacuous"
escape hatch documented — the orchestrator has to reason it out each time.

[dispatch; SL-215-harvest]
Working-tree-free funnel leaves the coord working tree stale. After
dispatch_import + dispatch_conclude_phase the coord HEAD ref advances (object-db
compose), but the coord tree's index+worktree are never checked out — so
`git status` shows the landed delta as staged "reversions" and the working-tree
files still hold the pre-phase content (grep for the new section returns 0).
This matters because `slice verify-vt` at conclude AND the orchestrator's
per-phase inspections read the coord WORKING TREE, not the ref graph — a stale
tree would false-red verify-vt. The dispatch/dispatch-agent funnel step list does
not mention resyncing; the operator must `git reset --hard HEAD` (same branch,
no switch — safe on the dedicated sole-writer coord tree) after each land to
restore INV-6 ("coord working tree == committed graph"). Worth a documented
funnel beat, or the tooling updating the working tree on import.

[dispatch-conclude; SL-215-ph05-conclude]
Conclude cadence for an orchestrator-run final phase (delivery gate, no worker)
surfaced four avoidable token-sinks — each cost a diagnostic detour:

1. verify-vt UNATTRIBUTABLE pre-prepare-review. Ran `slice verify-vt` first per
   the documented cadence (verify-vt → prepare-review); it reported ALL VTs
   `UNATTRIBUTABLE` because the conformance registry attribution reads is only
   *derived* at prepare-review. So the documented order guarantees a scary
   all-red-looking (but exit-0) first pass. The mechanics doc explains derivation
   happens "at prepare-review" but the conclude cadence lists verify-vt first
   without noting its attribution is necessarily blank until the next step.
   Re-running verify-vt AFTER prepare-review showed 18/19 PASS. Fix candidate:
   verify-vt could print "registry not yet derived — run prepare-review" instead
   of N×UNATTRIBUTABLE, or the cadence doc could note the two-pass expectation.

2. record-boundary empty-delta refusal for a no-code phase. PHASE-05 landed no
   source (re-embed touch = no git delta; only a knowledge-record dogfood commit).
   Manual `dispatch record-boundary --code-start X --code-end X` refused with
   `empty-delta`. This is correct, but I only learned it's fine — that
   prepare-review auto-heals an empty PHASE-05 row itself — by reading the source
   behaviour. An empty final phase is common (delivery/dogfood); the refusal
   reads as an error when it's a no-op-by-design.

3. prepare-review leaves a STAGED reversal of its own commits. After a clean
   exit (5 refs), the coord index held staged deletions of journal.toml AND the
   PHASE-05 boundary row — i.e. exactly reversing prepare-review's own auto-heal
   commit. Committing them would delete the journal that `sync --integrate`
   replays, breaking close. Nothing flags this as disposable; only tracing what
   each commit did revealed the staged state must NOT be committed and is dropped
   by the (documented) worktree teardown. High footgun potential for an agent that
   reflexively commits a dirty tree.

4. slice-status flip didn't visibly persist on first call. `slice status SL-215
   audit` from the coord tree returned "ready → audit [skip] [self/auto]" but a
   subsequent `slice status` still showed `ready`; a second identical call from
   the primary tree stuck it to `audit`. Unclear whether the coord-tree call was
   discarded at teardown or the first transition silently no-op'd. Cost a re-run
   + re-verify.

## [audit; SL-215-audit-0724]

1. `slice verify-vt` false-reds ALL VTs when run from edge on a dispatched slice.
   The impl bundle (incl. the dogfood notes.md) lives only on `review/*` +
   `dispatch/*` pre-integrate; verify-vt reads the working tree, so on edge every
   `test_file` is absent → 19/19 FAIL (not UNATTRIBUTABLE — a hard "file not
   found"). The handover's "18/19 PASS" was against the bundle. An auditor who
   trusts an edge verify-vt run reads a totally-green slice as totally-broken.
   The verb has no guard/hint that the slice's deliverables aren't on the current
   branch; you must know to worktree the bundle first. Cost: one confused re-run
   until the topology clicked.

2. `doctrine slice notes SL-NNN` silently MINTS an empty stub. Called to locate
   notes.md (expecting a path, read-only), it instead CREATED a fresh 256-byte
   stub on edge because notes.md wasn't on the branch (it's a dispatch-side
   knowledge-record commit). Left uncorrected, that empty stub would collide with
   the dogfood notes.md at `sync --integrate`. `slice notes` reads as a locator
   but is a create-if-absent — a read verb with a write side-effect on the
   authored tree. Had to detect (untracked `??`) and `rm` it.

3. PHASE-05 VT-1 UNATTRIBUTABLE is structural, not incidental: a `/notes`
   (knowledge-record) commit is never in the code-delta registry (`record-delta`
   tracks phase-cut code only), so any dogfood-in-notes.md VT can only ever reach
   UNATTRIBUTABLE, never PASS. The verify beat can confirm keyword substance but
   not attribution — the two are conflated in one status line. A slice that
   dogfoods its own artifact into notes.md structurally cannot get a clean VT row
   for it; the auditor must know UNATTRIBUTABLE-on-a-notes-target is the ceiling.

[close; SL-215-iss030-fp]
/close step 3a's ISS-030 tree-true verifier (a) `git diff --quiet HEAD` is a
WHOLE-TREE check. In a multi-agent shared primary tree it false-positives on
work that has nothing to do with the integrate: (i) the deliberately-uncommitted
RFC-011 case-notes.md itself, and (ii) a concurrent agent's SL-225 authoring
(uncommitted, then committed mid-close — edge advanced under me from 0d752778 to
cb5fd6960 while I closed SL-215). The verifier reported FAIL though the integrate
was clean. Cost ~4 extra diagnostic steps to prove it a false positive: had to
re-scope the check to the bundle domain (`git diff --stat HEAD -- install/
plugins/ .doctrine/slice/215/` → empty) and confirm main==admitted close_target
OID + journal trunk row (check (b) passed). Suggested firming: scope (a) to the
projected fileset, or exclude paths outside the slice's delta, so a
concurrent/uncommitted unrelated change can't mask or mimic a real reverse-diff.
Also: dispatched-close-on-main + concurrent edge authoring => edge/main diverge
(2-and-2 here); the next `git fetch . edge:main` promotion is non-FF and needs a
merge. Not a defect, but a recurring convergence cost worth a documented recipe.
[preflight; imp306-consolidate-capture] Severity-gated footgun hook fired on
`doctrine backlog show IMP-306` and `slice show SL-215` with dispatch-arm
memories (worker markers, seatbelt write-floor, branch-prefix) — zero relevance
to a skills-consolidation preflight. Hook keys on command shape, not task
context; ~3 irrelevant memory summaries injected across 2 calls.

[execute; SL-225-P01-a] `cargo clippy --tests` false-floods 8330 denials
(unwrap/expect/print_stdout enabled only in the test profile — AGENTS.md warns
this; use plain `cargo clippy` / `doctrine check`). Cost one detour before
recalling the documented trap. Minor: `just -n` writes its dry-run trace to
stderr, not stdout — cost one red-for-wrong-reason cycle in a belt-order e2e.
[harvest/handover consolidation; imp306-consolidate-capture] SKILL.md
frontmatter YAML footguns cost two fix cycles: ': ' in a description is a loud
parse error, but ' #' silently truncates (YAML comment) — tests green, damage
visible only in the rendered skill list. No authoring-time lint; each variant
was discovered downstream (test failure / skill-list inspection). A frontmatter
lint in `install::tests_skills` (or `doctrine check`) rejecting `: `, `"`, and
' #' in description values would have saved both cycles.

[dispatch; SL-208 sess-a]
PHASE-01 import refused `undeclared-scope` 3× before landing. Root causes, in order:
1. Selector authored at plan time omitted Cargo.toml/Cargo.lock, but EX-1 mandates
   the textwrap dep in Cargo.toml (→ Cargo.lock resolver churn). The worker delta
   legitimately touched both. worker_commit itself only *warns* `undeclared:[...]`
   (soft tier) and commits; dispatch_import *hard*-rejects — so the gap surfaces
   only at the funnel, one full worker round in.
2. classify_import's scope leg honours **design-target** intent ONLY. First fix
   declared Cargo.lock as `scope-relevant` (semantically "artifact, not target") →
   still undeclared → second refusal. Had to re-declare as design-target.
3. The belt reads the **committed** selector on the coord branch, not the working
   tree — writing the selector via CLI wasn't enough; needed a coord commit
   (then amend) before each retry.
Cost: ~4 import probes + 2 selector commits + source spelunking into import.rs /
conformance.rs to discover the design-target-only rule. A pre-spawn selector-vs-EX
reconciliation (does every mandated file appear as a design-target selector?) would
have caught this before the worker ran. The empty `detail:""` on the Refused
payload also forced manual `git diff --name-only` + source reading to identify the
offending paths — the CLI arm prints them (report_undeclared_scope) but the MCP
tool returns them empty.

[dispatch-agent; SL204-a15d-P01]
- dispatch_import is object-db-only: it advances the coord branch ref to S but
  leaves the coord WORKING TREE at B (grep confirmed old `Kind.scaffold` field
  still in src/entity.rs post-import, git status showed all 15 files as staged
  S→B reversions). The funnel's verify beat (`check regression diff`) runs the
  suite in the coord working dir, so it would build/test STALE B content and
  false-green unless the operator first syncs the tree to HEAD. Neither the
  /dispatch router nor /dispatch-agent SKILL documents an explicit "sync worktree
  to HEAD after import" step; I had to infer `git reset --hard HEAD` (safe here —
  all delta committed at S). This is a load-bearing, undocumented funnel step.
- Stale LSP `new-diagnostics` fired against the worker's MID-EDIT worktree
  snapshot (E0560 "Kind has no field scaffold" / E0061 arg-count) and directly
  contradicted the worker's committed-green hand-back. Cost a full investigation
  round (inspect flagged lines AT the fork tip via `git show <fork>:<file>`) to
  establish the diagnostics were stale and the committed fork was consistent.
  The funnel's own regression-diff later confirmed green — but the false alarm
  forced early disambiguation of "broken fork vs stale diagnostic".

[audit; SL-225-RV294]
- Conformance registry was polluted (55 undeclared, mostly foreign IMP-306/chore)
  because PHASE-01's solo auto-binding recorded a HEAD-before→HEAD-after range that
  swallowed 8 sibling commits interleaved on shared `edge` (PHASE-01 landed as two
  non-contiguous commits). The mechanical audit signal (`slice conformance`) was thus
  unusable as-is; recovering the true verdict cost a manual per-commit `--against`
  reconstruction across three ranges + inspecting `boundaries.toml`. Known hazard
  (IMP-175/IMP-292) but it recurred and taxed this audit directly. A per-commit solo
  binding (the safe `--commit` mode already exists) would have avoided the whole detour.

[dispatch-agent; SL204-a15d-P02-falsehalt]
- PHASE-02 worker HALLUCINATED a base-guard failure and bailed (0 files, no
  commit) on a FALSE premise. It forked correctly at B=4d3936e9 (verified: its
  worktree's src/entity.rs has the Kind.scaffold field ABSENT, rec.rs has 0
  scaffold_unused — PHASE-01 code fully present). Root cause: the worker ran
  `git show 4d3936e9 --stat`, saw the conclude/boundary commit touches ONLY
  .doctrine/dispatch/204/boundaries.toml, and (wrongly) concluded "zero changes
  to src/entity.rs → PHASE-01 not landed" — confusing a commit's own diff with
  its inherited tree. It then FABRICATED matching file contents (claimed the
  field, the initializers, the stubs were all still present) to justify the
  halt. Cost: a full re-dispatch round.
- Contributing factor: my base-guard check #4 embedded an EDITORIALIZING semantic
  claim ("proves PHASE-01 landed; the Kind struct is already pure identity") on
  top of the literal greps. The greps PASSED; the prose invited the worker to
  re-derive landing from commit history, where it tripped. Lesson: base-guard
  seams must be PURELY literal/positive grep assertions — no "this proves prior
  phase X" narration for the worker to re-litigate. The funnel's two-commit
  cadence (code commit + separate boundary-row commit inheriting the code) is a
  predictable confusion surface for a worker reasoning from `--stat`.

[dispatch-agent; SL204-a15d-P02-undeclared]
- PHASE-02 worker committed a clean, green delta but included ONE undeclared-scope
  path (src/lazyspec.rs) → dispatch_import refused `undeclared-scope` (landed
  nothing; coord tip unchanged — the belt worked as designed). The change is a
  1-line cosmetic deref (`&crate::adr::ADR_KIND.kind` → `crate::adr::ADR_KIND.kind`)
  forced/encouraged by the sanctioned Step-5 `GovKind.kind: &'static Kind` retype.
  The pre-existing `&…` would still auto-deref (`&&Kind` → field access works), so
  the edit was NOT strictly required — an optional cleanup that breached the
  declared selector set. worker_commit only WARNED on undeclared (it blocks on
  forbidden-zone, not undeclared), so the worker landed it on its fork; the harder
  gate is the import belt. Net: workers should be told NOT to make optional edits
  outside the declared selector set — "undeclared" at worker_commit is a soft warn
  but a HARD import refusal, so a cosmetic out-of-scope touch costs a full
  re-land/fixup cycle. Design §5's selector list also under-enumerated lazyspec.rs
  as an ADR-identity consumer.

[dispatch; SL204-a15d-P02-scope-fix]
Resolving the PHASE-02 undeclared-scope refusal (src/lazyspec.rs) surfaced a
non-obvious mechanic worth a token cost: a mid-dispatch design-target selector
correction CANNOT be made on the session-root/edge tree (where the skill says
authored writes go). dispatch_import reads selectors from the COORD tree's
working copy (crate::slice::selectors(&coord.root, ...)), and the funnel's
post-import `git reset --hard` re-syncs the coord tree to the phase commit — so a
merely-uncommitted coord-tree edit is wiped, and audit-time `slice conformance`
(VA-2) re-reads coord-tree selectors and would re-flag the path. Net: the
correction must be a committed orchestrator authored commit ON dispatch/<slice>.
That advances coord HEAD off the captured B, forcing the 3-way import path
(merge-tree(B, B', fork)) instead of the trivial coord.tip==B fold, and requires
conclude_phase code_start=B' (the selector commit) rather than the original B, so
the authored .doctrine edit stays outside the phase's recorded conformance range
(conformance folds only recorded per-phase [start,end] source ranges). None of
this — coord-tree-is-the-selector-source, reset-wipes-uncommitted, code_start
must exclude the authored commit — is stated in the dispatch skill; it took source
reading (dispatch.rs import_compose + conformance.rs) to derive. A one-line
"mid-dispatch selector/authored corrections: commit on the coord branch, set the
next phase's code_start past them" note in the funnel section would save it.

[dispatch; SL-208 sess-b]
Two token-costly friction points surfaced while distilling the PHASE-02 worker prompt (orchestrator side):

1. VT keyword encoded as a raw control char. plan.toml VT-2/VT-3 encoded the
   "no ANSI escapes" mandate as the TOML unicode escape for U+001B, which a TOML
   parser decodes to a RAW ESC byte. verify-vt (vtgate) matches keywords as a raw
   substring over host source (POL-002, no language-syntax interpretation). A raw
   ESC byte cannot sanely live in Rust source (tests assert via the '\u{1b}' char
   LITERAL — six ASCII bytes, not a raw ESC). Net effect: the mandate is either a
   silent false-FAIL (once the source-delta registry attributes the file) or
   toothless (UNATTRIBUTABLE, before attribution). Cost: had to hexdump plan.toml,
   cross-check tomllib vs disk bytes, read vtgate.rs + the attribution memory to
   confirm the failure mode, then byte-patch the plan (the Edit tool could not
   match the control/escape bytes; had to fall back to a Python byte replace, and
   the Bash inline heredoc tripped a control-char input guard twice). Root cause:
   nothing validates that a VT keyword is expressible as a raw substring of
   plausible source; a control-char keyword is accepted silently at plan-author
   time and only bites at conclude/audit. Candidate: `slice plan` / plan validate
   could warn on a keyword containing control chars (or normalize \uXXXX intent).

2. Renderer design-conformance gaps invisible to the VT gate. PHASE-01's
   render_subcommand_help/render_options_section shipped three deviations from
   design D2's info-contract (usage drops the `doctrine <path>` prefix; the global
   --color option vanishes entirely from every subcommand's Options because the
   cloned unbuilt clap node loses ancestor globals; local args drop
   [default:]/[possible values:] annotations). None are caught by any VT keyword —
   the VTs assert structural tokens ("│", "Usage:", function names), not fidelity
   to clap's rendered contract. Only an empirical render-probe (throwaway test)
   surfaced them. Cost: a throwaway probe test + git restore cycle to see actual
   output. Root cause: VT keyword mandates are a presence floor, not a
   contract-conformance check; a renderer can satisfy every keyword while silently
   dropping semantics the design requires. Folded the fixes into PHASE-02 rather
   than shipping and reopening at audit.

[dispatch; SL-224 PHASE-01 orchestrator]
Plan-authored wiring un-satisfiable within declared scope — a self-referential
selector-completeness miss. PHASE-01 EX-2 mandated threading `slice: u32` through
`run_import`, but `run_import`'s sole non-test caller is `src/worktree/mod.rs:494`,
which is NOT a design-target selector (selectors: import.rs, conformance.rs, plan.rs,
dispatch.rs, check.rs + scope-relevant slice.rs/vtgate.rs; no glob over mod.rs).
Threading forces editing mod.rs → the slice's own `dispatch_import` scope belt would
refuse the delta as `undeclared-scope`. Worker correctly held scope and shipped
EX-1/EX-3/EX-4/EX-5 (MCP refusal detail, the objective's primary target), deferring
EX-2. Additionally, EX-3 as literally written ("match Refusal::UndeclaredScope in prod
dispatch.rs") was un-buildable: `Refusal` is re-exported from `worktree` only under
`#[cfg(test)]` (mod.rs:109-111); worker dissolved it via a value-method
`refusal.scope_detail(..)` needing no type name — strictly in-scope, same result.
Token cost: source-diving mod.rs visibility + re-export gating to discover the
declared wiring was un-buildable/out-of-scope as specified. This is *exactly* the
defect class SL-224 PHASE-02's plan-time selector-completeness lint is built to catch
— the slice's own plan would have been flagged pre-dispatch had the lint existed.

[dispatch; SL204-a15d-P03-d2-deviation]
PHASE-03 (table+resolvers relocation) hit a forced locked-design deviation. Design
D2 mandated resolve.rs route id-formatting through `listing::canonical_id`. That
edge is infeasible: listing→tag (listing.rs:222 calls tag::fold_filter_tag) and
tag→kinds (tag.rs:15 re-exports kinds::TAGGABLE), so kinds→listing closes a fresh
leaf 3-cycle and fails the very architecture_layering_gate SL-204 exists to satisfy.
Worker inlined the byte-identical `format!("{prefix}-{id:03}")` at the two sites
(behaviour-neutral, tangle stays 0), documented in a resolve.rs module comment, and
flagged it as a mandatory-review tripwire.

Cost accounting: the deviation was caught by ME (orchestrator) reading the worker's
worktree source at the conclude gate, NOT surfaced structurally — the funnel's
regression diff was green (behaviour-preserving inline passes), so a green phase
still carried a design-premise falsification. The token cost was a full consult
round-trip + independent cycle re-verification (listing.rs:222 / tag.rs:15) before
concluding. Signal: a locked-design premise that turns out false is invisible to
the behaviour-preservation gate; only prose-level design/worker cross-read catches
it. D2's "listing import is cycle-free" assumption was never tested at design time.
Resolution: user accepted the inline (option A); canonical_id single-sourcing
(relocate into kinds, which sits below tag) folded into PHASE-04's layering pass.

[audit; SL-208-RV295]
The sess-b prediction held: PHASE-01's D2 info-contract deviations (G1/G2/G3) were
invisible to the VT gate — all 14 VTs PASS mechanically and the full test suite is
green, yet fidelity to clap's rendered contract is only observable by an empirical
render probe (run the built candidate binary, eyeball `worktree --help` /
`onboard --help` output). This audit had to build the candidate and probe rendered
output to confirm the folded-into-PHASE-02 fixes; a keyword-only audit would have
rubber-stamped it. Confirms the standing RFC-011 gap: VT keyword mandates are a
presence floor, not a contract-conformance check — a renderer can satisfy every
keyword while dropping semantics the design requires. Otherwise low-friction: the
dispatched-slice audit path was well-signposted (`dispatch candidate status`
emitted the exact `candidate create` command; the impl was cleanly on the
`review/208` impl-bundle evidence ref). Conformance's undeclared cell was pure
false-positive (designed-but-unselected Cargo.toml / e2e test) — a where-to-look
aid working as intended, routed to a `slice selector add` at reconcile, not code.

[none — ad-hoc backlog capture; sess-graph-cli]
Boot-snapshot command spine lists `explore` as a group heading
(`explore search inspect relation concept-map map onboard`), which reads as a
literal subcommand path; `doctrine explore concept-map …` fails
(`unrecognized subcommand 'explore'`) — the members are top-level verbs.
Cost: one wasted 4-command probe round. The spine's two-level indentation is
ambiguous between "group verb with subcommands" (real: `relation list`) and
"display-only heading" (`explore`, `facets`, `governance`…).

[dispatch; SL-224 PHASE-01 EX-2 worker false-halt]
A dispatch worker burned a full cycle (~50k tokens) on a PHANTOM base-guard
failure. Its own base-guard `grep` CORRECTLY reported the prerequisite seam
(`undeclared_detail` at conformance.rs:149); the worker then Read the file,
concluded (wrongly) the function was absent, declared its own correct grep
"fabricated by the rtk hook", and halted. Orchestrator ground-truthed via BOTH
Read and grep on the identical worktree — the function is present exactly where
grep said; no tooling corruption occurred. Root cause: worker over-trusted a
mistaken Read over a correct grep and manufactured a tooling-corruption narrative
rather than reconciling. Cost: one wasted worker spawn + orchestrator diagnosis
(read both trees, verify fork base, clean the aborted worktree, re-arm, re-dispatch
with ground truth asserted). Mitigation applied: re-dispatch prompt states the
verified ground truth and instructs "trust a successful grep; reconcile, don't
halt-and-blame-tooling". Possible systemic fix: base-guard seam checks should be
self-verifying / less prone to worker second-guessing, and worker prompts should
not invite tooling-distrust spirals.

[slice; sess-graph-cli]
Relation authoring for a new slice took 3 error rounds of vocabulary
trial-and-error: `originates_from` illegal for SL (it's a `references` role,
not a label), `specs` not link-authorable, `governed_by` target-kinds
[ADR|POL|STD] only, `references` requires `--role`. Root causes: (a) the
slice skill's own examples say "`specs` a spec" and suggest provenance
labels that SL cannot author; (b) the legal-labels error message prints
`references, references, references` (once per role?) — duplicate, and
doesn't mention the `--role` requirement until you try it. Error messages
were otherwise good (each rejection named the fix).

[dispatch; SL204-a15d-P04-vt9-gate-falsered]
PHASE-04's canonical_id worker produced a correct, fmt/clippy-clean, behaviour-
preserving 2-file src delta — but worker_commit's gate refused commit-gate-red on
`memory::ambient_surface_tests::vt9_no_discoverable_root_emits_nothing`, a test
100% orthogonal to the delta (canonical-id format vs memory root discovery).

Root cause: vt9 asserts "no discoverable root ⇒ emit nothing" using a fake non-
resolving cwd, but discover_surface_root (memory.rs:9613) falls back to ambient
CLAUDE_PROJECT_DIR, then surfaces from that root's LIVE memory corpus, deduped by
the runtime seen-set. So vt9's pass/fail depends on (a) whether the gate's env has
CLAUDE_PROJECT_DIR set to a real root and (b) whether that root's corpus+seen-set
happens to surface a memory for vt9's input (session "s9", changed "src/x.rs").
Non-hermetic w.r.t. ambient env + mutable runtime state. Verified: vt9 GREEN in
the coord tree (regression capture at B = 0 failures) both with and without
CLAUDE_PROJECT_DIR=/workspace/doctrine; only reds inside the deeply-nested worker
fork whose env/corpus surfaces something. Explains why PHASE-03's worker_commit
passed the same gate but PHASE-04's did not — a flake on fork corpus/seen state.

Token cost: the worker_commit gate is a BINARY pass/fail running the full cargo
test suite; a single non-hermetic ambient test false-reds the whole landing with
no differential. The worker cannot diagnose or fix (memory.rs is out of scope; no
retry allowed), so it halts and hands back. The orchestrator then spends a full
investigation (read the test, trace discover_surface_root, reproduce with/without
env) to establish the red is not the delta — cost the funnel's own coord-tree
`regression diff --base B` would have absorbed for free (it cancels persistent/
ambient reds by construction), but the skill forbids routing around a commit-gate-
red via the fallback import. Signal: worker_commit's gate should run the funnel's
B-vs-S differential (or at least tolerate a persistent/ambient red the coord-tree
baseline also carries), not a bare full-suite pass/fail — else any non-hermetic
test in the corpus can block an unrelated, correct worker delta. Latent defect:
vt9 (and siblings) should unset/guard CLAUDE_PROJECT_DIR + pin an empty corpus.

[dispatch; SL-224 conclude — verify-vt UNATTRIBUTABLE false-negative]
`slice verify-vt 224` at conclude (coord tree) reported ALL FOUR VTs as
`≈ UNATTRIBUTABLE — keyword present but <file> not modified by this slice`
(exit 0, non-halting). This is a FALSE negative of IMP-228's source-delta
intersection attribution: the four VT test_files (conformance.rs, dispatch.rs,
plan.rs, check.rs) DO appear in their phase boundary-range diffs
(36d0fe266..d08a6d51 and e8f3a3e8..23456005) and the tests genuinely exist with
the mandated keywords (workers reported them passing). The attribution heuristic
cannot see the slice-modification in the dispatch-coord-tree topology (dispatch/224
forked from main, changes committed working-tree-free via merge-tree), so it
mis-reports genuinely-attributable tests as inert. Token cost: orchestrator must
diagnose (confirm range diffs contain each test_file, confirm exit 0 non-halting,
confirm IMP-228 is the closed feature responsible) to distinguish a real gap from
a tooling false-negative before trusting the handover signal. IMP-228 is closed/
fixed but evidently untested against the dispatch-coord conclude path. Candidate
follow-up: verify-vt attribution should consume the conformance boundary registry
(code_start..code_end per phase) it already has, rather than recomputing a
source-delta that misfires under the coord topology.

[dispatch; SL204-a15d-P04-vt9-resolution]
Resolution of the vt9 gate false-red above: fixed the test's hermeticity (option 4)
rather than routing around the commit-gate-red (skill forbids the fallback detour)
or deferring. The fix is 3 lines (rootless tempdir cwd so discovery never reaches
the ambient CLAUDE_PROJECT_DIR fallback); the COST was the dispatch plumbing to land
a memory.rs touch through the belts — mint ISS-235, declare src/memory.rs as a
collateral design-target selector (worker_commit's SOFT scope tier is non-blocking
for src, but dispatch_import's HARD tier requires declaration), append PHASE-04 EX-6,
then SendMessage the SAME worker to fold the fix into its existing delta (reusing its
context + worktree with the canonical_id change already staged). worker_commit's gate
then ran the FIXED vt9 → green → landed. Signal: a one-line test fix, when it lives
in a file outside the active phase's selectors, incurs full selector/plan/backlog
ceremony to pass the import belt — the belt is right to be strict, but the friction
of an in-flight collateral fix is real. Reusing the blocked worker via SendMessage
(vs a fresh spawn) was the token-saver: the canonical_id delta persisted in its
worktree, so only the vt9 edit + re-commit were needed.

[audit; RV-296 / SL-224 audit 2026-07-24]
Minor friction during a dispatched-slice audit (coord tree removed):
- `doctrine slice design SL-224` intending to READ the design refuses with
  "Refusing to overwrite existing design.md" — `design` is the authoring verb,
  no read affordance; fell back to the Read tool on the raw path. A reader
  reasonably expects `slice design <id>` with no body to surface it (like other
  `show`-ish verbs). One wasted round-trip.
- `doctrine reports next` → "unrecognized subcommand 'reports'" despite the boot
  snapshot Commands index listing `reports status next blockers survey explain
  findings`. Doc-vs-CLI shape mismatch — guessed the wrong invocation; had to
  drop it. (Used `dispatch status --slice` instead, which worked.)
- ToolSearch `select:review_new,...` (bare names, as one might read them in prose)
  returned "No matching deferred tools"; the MCP tools only resolve under their
  full `mcp__doctrine__review_new` names. Cost one empty search round-trip before
  re-querying with the prefix.
- Dispatched-slice audit surface split cost the usual setup tax but was smooth
  once oriented: canonical selectors from `dispatch/224` tip (edge copy stale by
  design), code leg on a detached `review/224` worktree with `web/map/dist`
  seeded first (RustEmbed). Both already covered by memories — no new discovery.

[design; sess-graph-cli]
Knowledge-record body file is `record-NNN.md`, but I appended prose to a
guessed `knowledge-NNN.md` (pattern-matched from `slice-NNN.md` / entity-kind
naming) — three stray files committed before `knowledge show` exposed the
empty body. `doctrine knowledge paths` would have answered it; the per-kind
body filename convention is inconsistent across kinds (slice-NNN.md vs
backlog-NNN.md vs record-NNN.md), which invites exactly this guess.

[preflight; rfc021-next-slice-4a1c]
Three small CLI-shape misfires cost round-trips while scoping RFC-021's next slice:
- `doctrine memory retrieve <key>` rejects a positional key (`unexpected argument`);
  the working read is `memory show <key>`. `retrieve` wants flags, not an id.
- `doctrine spec req show/get REQ-NNN` do not exist — only `spec req list <SPEC>`.
  To read a single requirement's statement I had to grep the `spec show` body for
  `FR-00N (REQ-NNN)` headings; the `req list` table's prose column is `—` for all
  rows, so the roster gives status but not the requirement text. Reading "which
  pending requirement is which" is a two-command dance (list for status → grep
  body for statement) where one `req show <REQ>` would do.

[audit; SL204-recon-297]
Auditing a dispatched-but-unintegrated slice cost several orientation probes the
skill doesn't anticipate:
- `/audit`'s dispatched-slice note says "audit the candidate surface published by
  `dispatch candidate create`" — but here NO candidate existed and trunk had moved
  22 commits past the prepared base. The skill assumes a candidate; the real entry
  state was "sync prepared, never integrated". Had to reconstruct via
  `dispatch status` + `dispatch candidate status` + `git log --grep` that the impl
  lived only on `review/204`, not edge HEAD. A one-line "if no candidate: the impl
  is on the review/* evidence ref; audit there" would save the archaeology.
- **`cmd | tail -N; echo $?` masks build failure.** Ran `cargo build 2>&1 | tail`
  and `cargo test 2>&1 | tail` with `echo EXIT $?` — `$?`/last-PIPESTATUS is
  `tail`'s exit (0), so a compile failure read as "exit 0". Twice mistook a failed
  build for a pass; only caught it when `grep -c "test result: ok"` returned 0.
  Cost a diagnosis round-trip. Lesson: capture `${PIPESTATUS[0]}` or grep for the
  failure token, never trust `$?` after a pipe.
- web/map/dist seeding into the fresh worktree (known, mem_019f4c64) was the actual
  root cause of the first "compile error" — worth the skill cross-linking that memory
  from the dispatched-slice audit note, since it presents identically to a regression
  (`Assets::get not found`).

[plan; sess-graph-cli]
plan.toml scaffold's inline comment renders the mandatory VT-mandate example
as a MULTI-LINE inline table ({ id = "VT-1",\n expects = ...) — invalid TOML.
Authoring the plan by following the scaffold's own example produced a parse
failure at `slice phases` (cost: one failed round + a rewrite to single-line
rows). The scaffold example should be valid TOML as displayed, or the plan
schema should accept [[phase.verification]] array-of-tables for multi-line
mandates (they are far more readable at this length).

[close; SL-224-split-lineage]
Close of a dispatched slice cost heavy read-tokens before any mutation, because the
correct route was NOT what either tool surface advised. `dispatch status` said
"refresh-base + re-prepare"; `dispatch candidate status` said "candidate create
--role review_surface". Both are the naive path that projects code-only and strands
the reconcile truth stranded on edge (design.md/RV-296/harvest) — silently wrong for
this topology. The right route (split-lineage-reconcile-on-edge: unite edge⊕review,
FF main, no-op close_target, integrate, reunite edge) is only discoverable from
memory, not from the status machine's "next" hint. Compounding cost: coord tree was
removed (refresh-base unavailable without `dispatch setup` resume); trunk had moved 9
commits past the prepared base; the canonical selector lived on a *third* branch
(dispatch/224), not edge. ~15 read-only probes (git topology, 3-way blob compares,
merge-tree previews, 4 memory retrievals) to ground a plan before touching a ref.
Signal: a pre-close check that diffs the admitted close_target's tree against edge and
flags reconcile/authored divergence would collapse this to one command (the memory
itself calls for it). The status machine being lineage-blind is the root token sink.

[preflight; IMP-308-scope]
Minor: `doctrine backlog show RFC-016` rejected (`unknown backlog prefix RFC`) —
RFC is its own kind (`doctrine rfc show`), not a backlog prefix, but the search
listing renders RFC-016 in the same column as IMP/ISS rows, inviting the wrong
verb. One failed call. Cross-kind `search` results don't hint which `show` verb
each id needs.

[preflight; RFC-016-cluster2-preflight-20260724]
Read-only research subagents (Explore) spawned at repo root to read entities via
`doctrine <kind> show` could not run the CLI at all: no `./target/debug/doctrine`
build present, and `~/.cargo/bin/doctrine` unusable because the bash
`worktree-jail: cwd-not-a-worktree` hook refuses commands from a cwd-pinned
subagent that is not inside a worktree. Both spec/backlog readers fell back to
reading raw `.toml`/`.md` facets directly — exactly the "read via show, not raw
files" guardrail agents are told to avoid. Cost: subagents burned tool calls
discovering the block before falling back; the guardrail-preferred read path was
structurally unavailable to the delegated readers. Friction = preflight
delegation to read-only subagents collides with the worktree-jail hook + absent
prebuilt binary combination. Mitigation candidates: (a) ensure a debug build
exists before spawning read-only research fan-out, or (b) relax the worktree-jail
hook for read-only `doctrine ... show/paths` invocations.

[spec-tech/revision; RFC-016-cluster2-preflight-20260724]
`doctrine revision list` returns empty (exit 0, no rows) despite 31 revisions on
disk under `.doctrine/revision/`. Had to fall back to `ls .doctrine/revision/`
to enumerate and pick an exemplar (REV-030) — the guardrail-preferred `list`
verb gave nothing, costing a debug round-trip. Either a default status filter is
hiding done/approved revisions with no hint, or a genuine bug. Token cost: two
extra bash calls + reasoning to notice the list was lying rather than truly empty.

[dispatch; SL-227-drive]
Pre-dispatch base-promotion raced a concurrent agent twice: `dispatch setup`
fails closed when the fork base predates edge's `.doctrine` corpus tip, and edge
advanced (bd3582c4d, then 3875d1a40) between each `git fetch . edge:main` and the
setup retry. Cost: two extra promote+retry cycles + inspection to confirm each was
a clean fast-forward (not a diverge). The corpus-tip guard is correct, but in a
multi-agent tree the promote→setup window is inherently racy; a `setup
--promote-base` that atomically fast-forwards-and-forks (or a retry-with-refresh)
would collapse the loop. Also incidental: the shared index surfaced another
agent's *staged* CLAUDE.md mid-drive — required a path-limited-commit posture
throughout to avoid laundering it.

[corpus-survey; RFC-009-D2-latent-taxonomy]
- Per-entity `doctrine slice show SL-NNN` in a loop timed out at 2m across 227
  slices (build a 40-doc index this way = dead). Fell back to slug symlinks +
  raw `wc -l`/`head -1` for the index tier; reserved `show` for the 2 governing
  entities. A cheap `doctrine <kind> index --oneline` (id+title, no MD synth)
  would remove the tax on any corpus-wide census. ~2m + one wasted call.
- zsh word-splitting footgun: shell is zsh, not bash. `grep ... $VAR` with a
  newline-joined file list does NOT split in zsh (no `SH_WORD_SPLIT`) → matched
  the whole blob as one filename → silent 0 hits on the first census pass. Cost
  one full re-run. `grep -r --include=` sidesteps it (and dedupes slug symlinks
  for free since `-r` won't follow symlinked dirs). Worth a note in any harness
  doc that assumes bash splitting.

[dispatch+phase-plan; fable-sl226-a]
- Handover's literal `doctrine dispatch setup --slice 226` omits the required
  `--dir` flag — cost a `--help` round-trip. Handover templates should carry
  the full invocation incl. `--dir .dispatch/SL-<n>`.
- edge→main promote blocked by a stale prunable worktree registration
  (/workspace/sl204-close, dir already deleted) — cost a worktree-list +
  inspect + prune detour. A `worktree prune` beat in the pre-dispatch ritual
  (or in `dispatch setup` itself) would absorb this.
- Boot snapshot's SPINE lists command names only; dispatch router says check
  `doctrine.toml → [dispatch]` but the file lives at `.doctrine/doctrine.toml`
  — first grep at repo root missed it.

[close; SL-204-close-recovery]
- RV-297's reconcile brief AND the handover both prescribed the bare
  `dispatch sync --integrate --trunk main` for landing — omitting the
  close_target create/admit step that /close skill 3a (and mem_019ec912)
  require. Following the underspecified brief projected the code-class
  `phase/*` chain onto main, stripping the ENTIRE authored `.doctrine/` corpus
  (7246 files, -400674). Lineage-blind status still reaches `done`. A high-cost
  multi-reset recovery. The slice-local playbook (RV brief/handover) should not
  restate the integrate command in a form that diverges from the skill; or the
  skill's 3a should be the single source and the brief should point to it.
- A candidate-less integrate bakes a **verified** journal trunk row at the
  phase-04 code tip. That row is STICKY: admitting a close_target afterward does
  NOT re-target it — integrate idempotently REPLAYS the baked row (main went
  code-only twice, no CAS refusal). Recovery: `git branch -D review/204
  phase/204-*` + `sync --prepare-review` regenerates the journal with ZERO trunk
  rows; THEN create+admit close_target BEFORE the single integrate. Cost: ~1hr,
  many careful read-only probes to avoid a third misfire. A guard — integrate
  refusing `--trunk` when no close_target is admitted, or warning that it will
  project the raw phase chain — would have prevented the whole incident.
- `--payload code` (close skill 3a template) vs `--payload impl_bundle`
  (mem_019ec912): only impl_bundle carries authored `.doctrine/` onto trunk.
  The skill template shows `code`, which strips authored state. Skill/memory
  disagree on the load-bearing flag.
- prepare-review CAS-refuses stale review/phase refs with no force flag → a
  `git branch -D` dance on every re-cut (2× this session).
- Fresh landed worktree shows `phases: —` (phase status is per-tree runtime, held
  on the primary tree); `slice conformance` complains, though the `done` gate did
  not actually require it — a false alarm that cost an investigation detour.

[dispatch; SL-227-drive-scope]
PHASE-02 stalled at the funnel on an `undeclared-scope` import refusal. Root
cause: the slice selector (authored scope) AND design §5.2 both predicted a
top-level `src/library.rs` wired via `main.rs`; the real codebase houses command
modules under `src/commands/` (wired in `commands/cli.rs`), and ADR-001 layering
forbids an unclassified top-level module. The worker correctly discovered this and
placed the veneer at `src/commands/library.rs`, but `worker_commit` (advisory on
undeclared non-forbidden files) LANDED the fork while `dispatch_import`
(HARD undeclared-scope gate) REFUSED it — an asymmetry that costs a full
review+consult+selector-correction+recommit+re-import cycle at the funnel, after
the worker already succeeded. Two design-affecting observations: (1) the two
scope gates (worker_commit vs classify_import) should agree, or the worker should
learn the hard boundary before it commits, not after; (2) a design that names
concrete file paths (§5.2 "main.rs") seeds a stale selector that only surfaces at
import — a layout-agnostic selector (`src/commands/**`) or a design that defers
file placement to the worker would avoid it. Net: ~1 extra orchestrator turn +
worker's correct-but-blocked delta held pending human adjudication.

[route→revision-apply; 7a1771cd REV-032 apply]
- `revision apply` surfaces `introduce` rows "for manual handling" but gives no
  copy-paste command — the operator must discover that (a) the requirement's full
  normative statement is stored as the requirement `title`/.md-H1 (not a
  `--statement` flag; `spec req add` takes only a positional TITLE), and (b) `add`
  does not set `pending`, so a follow-up `spec req status … --to pending` is
  needed per row. Learned only by inspecting an existing pending req (REQ-335).
  A `revision apply --emit-commands` (or apply auto-running `spec req add` for
  introduce rows) would remove a multi-probe archaeology loop (~several calls).
- Path-limited `git commit $PATHS` with a shell variable holding space-separated
  globs did NOT word-split under the hook-wrapped shell — git received the whole
  string as one pathspec and failed. Direct inline globs on `git add`/`git commit`
  worked. Cost one wasted round-trip. (Not doctrine's fault, but it interacts with
  the mandatory path-limit-the-commit rule — the safe idiom is inline pathspecs,
  never a variable.)

[dispatch pi-arm; fable-sl226-a]
- PHASE-01 funnel clean: fork → worker (single-file graph.rs delta, agent_end) →
  `worktree import --from-worktree --slice 226` (scope belt passed, post-import
  prove green in-process) → `check regression diff` green → 1 commit → record-delta
  → gc. No round-trips. Pre-distilled single-file-scope constraint + hand-construct
  fixtures held (worker touched ONLY graph.rs; import belt would've caught a stray).
- FOOTGUN: `git commit -- <pathspec> -F -` fails — everything after `--` is a
  pathspec, so `-F -` is swallowed. Options MUST precede `--`:
  `git commit -F - -- <pathspec>`. AGENTS.md's path-limit-commit rule shows
  `git commit <paths> -F -` which is itself the wrong order; worth correcting.
- `.pi-session/` lands untracked-and-unignored in the worker fork; `import
  --from-worktree` would stage it. Had to `rm -rf .pi-session` pre-import. A
  `.gitignore` entry (or import-side skip of `.pi-session/`) would remove the
  hand-step every pi phase.

[dispatch pi-arm; fable-sl226-a]
- PHASE-03 (new module) two-actor split worked cleanly: worker did source
  (dot.rs new + mod.rs reg + concept_map.rs dot_escape lift), orchestrator did
  the worker-forbidden `.doctrine/adr/001/layering.toml` MixedUmbrella entry
  (catalog::dot=engine) post-import. import --from-worktree staged the NEW
  untracked dot.rs correctly (regular file, not symlink).
- The high-trust memory mem.pattern.lint.module-split-needs-layering-entry
  fired via the Read/Edit hook exactly when editing layering.toml — corrected my
  earlier static-read belief that the umbrella-roll made the entry optional. It
  is gate-CRITICAL. Good example of the hook memory surfacing precisely at the
  decision point. (My static read of assertion-3's `sub_classified_modules`
  skip was wrong; empirically MixedUmbrella still reds without the row.)
- Sequenced as two coord commits: worker source (record-delta'd) + orchestrator
  governance (layering). Kept the phase source boundary clean of .doctrine/.
- DEVIATION worth a token-note: pi worker rendered the design's named-slice-const
  style tables (`NODE_STYLES: &[...]`) as match-lookup fns instead. Correct
  values, idiomatic, but the VT keyword literals then live only in comments.
  Prompts that inline a table SHAPE should say whether the shape is mandated or
  illustrative — the worker optimized structure and drifted from §5.3 wording.

[slice/research dogfood; db8e41f5]
SL-229 pre-design research round, first dogfood of the research stage
(pi-scout flash + pi-research pro, parallel, stdin->stdout).
- Cheap-model accuracy better than feared THIS round: 8/8 spot-checked
  path:line citations correct (gitignore:48, allowlist Tier::Research,
  doctor_checks:334, ADR-005:69, ADR-006:153, SPEC-010:55, SPEC-013:29,
  state.rs phases_dir/refresh_symlink). One wrong line number
  (contentset is_stale_against "99", actually 114-118) — harmless class.
- Both models emitted a preamble line despite explicit "no preamble".
- Verification asymmetry works: ~1 grep per load-bearing claim; unverified
  rows carry no ✓ in the artefact so design can't silently load-bear them.
  This is the cheap hallucination mitigation (vs running 2 of everything).
- Research round caught 2 things the (careful, expensive) shaping convo
  missed: pre-existing slice-folder research convention (SL-055 gitignore,
  SL-116 Tier::Research, doctor skip) that inverts the storage lean; and
  bare-NNN state-path naming (orchestrator had minted SL-229-named dir —
  exactly the class of error a mint verb prevents). Strong evidence for
  the IDE-044 thesis.
- Friction: none from doctrine tooling this round; scratchpad prompt files
  + background bash was smooth.

[dispatch-agent (conclude); SL-227-phase03-conclude-2026-07-25]
Conclude-cadence friction on the claude arm — four token sinks, all in the funnel tail, not the worker:

1. verify-vt FAIL from a stale plan.toml test_file pointer. PHASE-02's veneer landed
   in src/commands/library.rs (ADR-001 forbids the top-level src/library.rs design §5.2
   predicted); the selector was corrected mid-drive but the plan.toml VT test_file fields
   were NOT, so the conclude-time VT gate hard-FAILed (exit 1, halts handover) on 5 VTs.
   Cost: a full /consult round + diagnosis + a corrective commit. Root cause: the
   selector-correction and the plan-VT-correction are two surfaces for one fact; correcting
   one leaves the other to detonate at conclude. Candidate fix: when `slice selector` rewrites
   a design-target path, offer to re-point any VT test_file naming the old path (or have
   verify-vt fall back to selector-resolved paths).

2. verify-vt reports EVERY VT (all 3 phases, incl. prior-session PHASE-01/02) as
   UNATTRIBUTABLE ("keyword present but <file> not modified by this slice"). The source-delta
   attribution (IMP-228) mis-computes its base for a dispatch-concluded slice after an
   8-commit refresh-base merge moved the fork-point — so it credits NOTHING to the slice.
   Non-halting (exit 0), but it renders the whole gate signal useless at exactly the moment
   it matters, and cost tokens to distinguish "buggy attribution" from "real missing test".
   This is open ISS-226; adding a reproduction datum (universal, post-refresh-base).

3. Working-tree-free MCP funnel tools (conclude_phase, prepare-review) leave the coord tree's
   LIVE index stale: the staged boundaries.toml was the pre-conclude 2-phase registry, whose
   naive commit/inspection *regresses* PHASE-03's row. Cost: several commands to prove the
   committed graph (refs) was complete+correct while the working index was stale-and-wrong,
   before daring `git worktree remove --force`. A one-line "coord index is stale-by-design;
   truth is in refs; --force is expected at teardown" note in the conclude cadence would
   have saved the whole investigation.

4. slice lifecycle stuck at `ready` after a full dispatch drive (never advanced to `started`
   on either edge or dispatch/227). ready→audit is a [skip]; benign but had to verify across
   trees that `started` was genuinely never recorded (not a divergence) before transitioning.

[/audit; sess-audit-sl226]
Auditing a pi-arm dispatched slice from a fresh `git worktree add --detach review/226`
worktree: the build failed with `assets::Assets::get not found` (RustEmbed derive
emitted no `get`) because `web/map/dist/` — a gitignored embedded `#[folder]` root —
is absent from a fresh worktree (git doesn't carry gitignored artifacts). Cost: one
failed build + diagnosis before realizing it was environmental, not a slice defect.
Compounded by `cargo build 2>&1 | tail -N` masking cargo's nonzero exit behind tail's
0 — the "exit 0" was misleading. Mitigation that worked: `cp -r web/map/dist/.` from
the primary tree into the worktree, then rebuild. Latent doc gap: the audit/worktree
ritual for a slice touching (or coexisting with) an embed root should pre-seed
gitignored embed folders, or `doctrine worktree fork` should carry them.

[plan; db8e41f5]
Plan-time re-grep falsified an UNMARKED research row: scout claimed slice
parse tests live in src/main.rs (parse_coverage_show pattern); they live in
src/slice.rs's own test module (:4043). Zero cost incurred — the row was
never load-borne (✓ discipline held); caught by /plan's re-grep step in one
grep. Also: two VT-mandate keyword choices needed tightening at critical
review (variant-name keyword satisfiable without any test; prose-heading
keyword fragile to drift) — keyword selection for prose/test mandates is a
skill worth a line in /plan or the research skill's conventions.

[audit; SL-227-audit-2026-07-25]
Incidental complexity / token sinks during /audit of a dispatch-concluded slice:

1. `slice conformance` ran against the PRIMARY tree's (edge) slice-227.toml, which
   is STALE — the coord's authored selector corrections (adding src/commands/{cli,
   library}.rs as design-target; the PHASE-02 VT test_file fixes) live on review/227
   / dispatch/227, NOT edge (they land at close via `dispatch sync --integrate`).
   So conformance falsely reported src/commands/{cli,library}.rs as `undeclared`
   and src/library.rs as `undelivered`. An auditor MUST assess conformance against
   the projected authored state (review/227 selectors via `git show review/227:
   .doctrine/slice/NNN/slice-NNN.toml`), not the primary tree — cost: an extra
   git-show round to disambiguate real gaps from the pre-integration lag. The
   conformance verb has no flag to point it at the review bundle's registry.

2. Independent green-verification needed a review/227 worktree, but a fresh
   `git worktree add` LACKS the gitignored derived `web/map/dist/` RustEmbed root
   (built by `just web-build`). `cargo test --bin doctrine` failed to compile
   (map_server/assets.rs: `Assets::get` not found) — an environmental miss wholly
   unrelated to the slice. Cost: one wasted compile cycle + diagnosis + a manual
   `cp -r` of web/map/dist from the primary tree. A fresh audit worktree of any
   slice touching nothing map-related still can't build until the derived embed is
   sourced. (`doctrine worktree fork` provisions; a raw `git worktree add` for an
   ad-hoc audit checkout does not.)

3. On that worktree, `test_support::tests::doctrine_bin_returns_existing_executable`
   false-reds: it asserts ./target/debug/doctrine exists on disk, but during a
   from-scratch `cargo test` build the bin isn't placed when the unit test runs
   (1 fail / 3812 pass). Not an SL-227 defect (test_support.rs is untouched by the
   slice) — but an independent audit re-run in a clean worktree always eats this
   false-red and must recognise+discount it.

[phase-plan+execute; SL-229-PHASE-01-a1f]
New engine `mod research;` tripped the ADR-001 architecture-layering gate
(`tests/architecture_layering.rs` → `Unclassified("research")` ×4) — the module
must be added to `.doctrine/adr/001/layering.toml [tiers]` or `doctrine check
gate` fails. This coupling (new module ⇒ layering.toml edit) was NOT in the
handover terrain / code-impact table; cost one gate round-trip + a file read to
locate the classification source (it's a TOML the test loads, not an in-test
table). Cheap once known; a one-line "new module → layering.toml tier" note in
the design's code-impact summary would have pre-empted it.

[close; sess-sl226-close] SL-226 close (pi-arm): the handover's literal close
recipe `dispatch sync --slice 226 --integrate --trunk refs/heads/main` FAILED
with `no phase/226-NN code units to integrate`. Root cause: the pi-arm bundled
all phase code into a single `review/226` "impl bundle" commit with NO
`phase/226-NN` refs (they were never created / had been reaped), so `--integrate`
found no code units to project. Recovery required the candidate close_target
workflow (close skill §3a, undocumented as the fallback-when-phase-refs-absent):
`candidate create --role close_target --payload code --base main --source
review/226` → `candidate admit --review RV-301` → `--integrate --trunk`. Cost:
~6 investigative tool calls to discover the topology (journal has only a
review-surface row; phase refs absent; SL-204 precedent showed the trunk row is
a candidate merge). The handover author's mental model assumed phase refs would
exist. Second incidental cost: `check gate` on the primary edge tree was red from
a *different* live agent's uncommitted `src/research.rs` WIP — a concurrent-edit
false-red unrelated to the slice; had to isolate the SL-226 signal via audit
evidence (byte-identical src tree) + a corpus-only `doctrine doctor` before edge
went green/static.

[walkthrough; SL-228-extraction-b4c81d24]
Pre-design extraction of the dispatch funnel state machine. Low friction — the
surface was well-signposted by the handover (exact symbols + file anchors),
REV-032 rationale pre-quoted the crux claims, and the code carried dense doc
comments so the as-built graph read directly off ReceiptStatus/NextGuidance/the
verb bodies. No corpus-archaeology needed. One genuine cost: the "seven-state"
framing in REV-032/handover conflates two altitudes (per-phase sub-funnel vs
slice-lifecycle guidance) — reconciling that took a careful second read of
select_guidance to confirm the sub-funnel has NO node there today. Worth flagging
as a latent design ambiguity, not a token sink. ~1 read-round of redundancy.

[execute; SL-229-PHASE-02-b7c]
verify-vt `UNATTRIBUTABLE` verdict reason reads "keyword present but `<file>`
not modified by this slice" even when the keywords are NOT present — the
attribution check (vtgate.rs step 4) fires before keyword matching, so the
fixed reason string asserts a keyword state it never tested. During the
red-phase reading this sent me on a grep round to disprove "keyword present"
(keywords were absent in all three files). Minor token bleed + confusion;
phrasing like "attribution unknown (file not in slice delta); keywords not
evaluated" would be truthful.

[execute; SL-229-PHASE-02-b7c]
The sweep command embedded in mem.pattern.skills.yaml-frontmatter-colons
false-positives on every file: the awk prints `FILENAME": "` and the grep for
`: ` matches that separator itself, not the description value. Cost one
re-check round to falsify. The memory needs its recipe corrected (print the
value alone, or grep before prefixing).

[design; SL-228-design-b4c81d24]
Design session for SL-228 rode the extraction artifact cleanly — zero re-derivation
of the as-built graph; Q&A loop settled 7 decision forks in ~7 turns. Friction
worth logging: (1) two CLI shape misses (`spec req show` absent — requirement
bodies unreachable via a show verb, had to read raw entity files against the
reading-rule; `memory retrieve` positional query rejected — needs `--query`).
A `req show`/`requirement show` verb would close a real gap: requirement kind
has no CLI read surface. (2) `prompt resolve` requires `--role` — the boot
snapshot's floor directive omits it, so the model-band ritual fails as written;
boot text should carry the full invocation. Both are token-cheap here but
recur every session that hits them.

[inquisition; inq-228-0725a]
- Boot spine lists `explore  search inspect relation concept-map map onboard` as a
  command group, but the CLI has no `explore` subcommand — `doctrine explore inspect
  REQ-384` fails; the real shape is top-level `doctrine inspect`. Cost: one failed
  6-iteration loop + a --help round trip. Spine table grouping ≠ CLI grammar.
- Requirement statement text is hard to reach by CLI: `doctrine inspect REQ-NNN`
  shows relations only; `spec req list` shows `prose —` even when the requirement
  TOML carries a full `title` statement. Had to raw-read
  `.doctrine/requirement/NNN/requirement-NNN.toml` (storage-rule friction: no
  `req show`-equivalent surfaces the statement). ~3 probe commands wasted.

[inquisition; d757bead-rv304]
Round-2 external inquisition (RV-304). Friction was low — the handover packet
pre-paid the CLI gotchas — but two small token sinks: (1) shared-tree dirt
required a `git diff .doctrine/doctrine.toml` detour to prove `review new` had
NOT touched it before path-limiting the commit (the entity counter's home is
non-obvious; a `review new` result naming every file it wrote would remove the
check); (2) `review show --view full` returns brief + all finding details in
one large JSON blob — fine once, but there is no per-finding fetch for
re-reading a single detail during adjudication.

[reconcile+close; sl227-close-4edfdbfa]
- `dispatch candidate status` DRIFT flag advises "supersede with a fresh
  candidate" whenever the live tip ≠ recorded merge_oid — but a fix-now commit
  ON TOP of the recorded merge is a legit first-class flow that `admit` accepts
  (I3 ancestor-check, dispatch.rs:2099). Superseding would re-run the 3-way merge
  and DROP the on-top commit. Cost: ~4 calls + reading dispatch.rs:2016-2135 to
  trust admit over the status hint. The "next" hint should distinguish
  drift-off-lineage (supersede) from drift-forward-on-lineage (admit handles it).
- Handover prescribed reopening verified RV findings via contest→dispose→verify,
  but `verified` is a terminal sink (review.rs:703-728 — no verb leaves it). Only
  discovered by attempting `contest` (rejected "out of turn... verified !=
  answered"). A post-verify decision reversal (defer→fix-now) has no CLI path;
  it must be a prose amendment on the RV .md. An agent following the handover
  literally burns cycles before finding this.
- Reconcile-target ambiguity for `.doctrine/slice/NNN`: edge-authored (rides
  edge→main) vs carried by the candidate/integration. `dispatch sync --integrate`
  --help says "project the audited code units", but `--allow-corpus-clobber`
  reveals it ALSO projects `.doctrine/**` (FF+CAS, clobber-guarded). Resolving
  took an SL-226 git-log precedent (MISLEADING — SL-226 had no dispatch-time
  .doctrine corrections, led to a wrong intermediate "code-only" conclusion) +
  the notes.md auditor note + re-reading sync help. ~6 calls + one wrong turn.
  The integrate help's "code units" phrasing undersells the corpus projection.

[preflight; SL-229-P03-preflight-c4e]
- `doctrine slice notes <id>` is a **mint** verb that errors with "Refusing to
  overwrite existing …/notes.md" — but it reads like a sibling of `slice show`
  / `slice paths` (read verbs). Cost one wasted round-trip before falling back
  to `Read`. The spine line `slice → design plan phases notes phase status …`
  gives no read/write signal. Cheap fix: name the read path, or have the error
  say "notes.md exists — read it with `doctrine slice show`/Read".
- `plan.toml` SL-229 PHASE-03 VA-1 names `.doctrine/skills/` as an installed-
  copy location; that dir does not exist in this repo. The real derived mirror
  is `.agents/skills/` (gitignored, `.gitignore:5`). Cost ~3 probes to
  establish which path VA-1 actually means. Authored criteria naming a
  derived path that the repo doesn't materialise is a re-verification tax on
  every agent that picks the phase up.
- Third mirror discovered: `~/.claude/plugins/cache/doctrine/doctrine/0.30.0/
  skills/` (marketplace cache — the source of the running `/doctrine:*`
  skills). `doctrine install` does not refresh it, so a newly authored skill
  is not slash-invocable in the authoring session. Not in any phase criteria;
  bears on SL-229's closure evidence ("one real slice driven with the round").

[close; SL-227-close-integrate]
The close skill §3a verify gate (a) is `git diff --quiet HEAD` — the ISS-030
phantom-reverse-diff detector, deliberately whole-tree and not path-limited.
In a **shared coordination tree with a concurrent agent** it is unusable
verbatim: three unrelated uncommitted files (another agent's `doctrine.toml`,
`flake.lock`, plus this very case-notes file) make it exit nonzero regardless of
what integrate did, so the literal recipe reads as STOP on a clean integration.
Cost: a pre-integrate `git status --porcelain` snapshot, a post-integrate diff of
the two, then a second content-level pass (`git diff --name-only HEAD` minus the
known-foreign paths) because porcelain compares *status*, not bytes — ~3 extra
tool calls and the reasoning to notice the trap before tripping it.
The skill states the invariant (tracked tree matches HEAD) only as a command that
presumes a clean tree. Worth stating the invariant separately from the command,
with the shared-tree adaptation named: baseline the divergence set before
integrate, assert it is unchanged after, and assert no path outside it moved.
[preflight; sl228-rv-advice-0725] `doctrine review status <SL-id>` refuses — status takes only RV refs and
there is no `--slice` filter, so surveying a slice's review history costs a full `review list` scan + manual
grep. A `review list --entity SL-228` (or `status --slice`) filter would save the scan tokens.

[preflight; SL-229-P03-sl227-delta-9af]
SL-227 (minimal projection, ADR-019) landed *between* SL-229's plan authoring
and PHASE-03, invalidating that phase's EX-2/VA-1 verification surface:
- `install --dry-run` in this repo now emits **no local skill-file rows** at
  all. Claude = "register marketplace + install plugin"; codex/pi/universal =
  "delegates to npx". `.agents/` survives only as an *auto-detection probe*
  (`src/install.rs:1570`), no longer a write target. `.doctrine/skills/` never
  existed.
- So EX-2 ("embed ritual run; installed copies … match plugins/ masters") and
  VA-1 ("installed copies exist and match masters") name a mechanism that no
  longer exists. The real claude-side mirror is the marketplace cache
  `~/.claude/plugins/cache/doctrine/doctrine/<version>/skills/`, populated by
  `claude plugin install/update` — i.e. by a **release cut + plugin update**,
  not by `doctrine install -s <skill>`.
- Generalised friction: **phase verification criteria that name a projection
  path are hostage to any concurrent slice that changes projection policy.**
  Authored criteria should name the *contract* ("the harness-visible copy
  matches the master") and let the phase resolve the path, or the plan silently
  rots. Cost here: ~10 probes to re-derive the install topology, on top of the
  3 already spent on the phantom `.doctrine/skills/` path.
- `plugins/` is still a RustEmbed root (`src/install.rs:21`), so the
  `touch src/install.rs && cargo build` embed ritual is not obsolete — but it
  is no longer *sufficient* for EX-2, and the design's "Post-authoring ritual"
  reads as if it were.
- Live gate discovered, not in the handover: `dedup_skills_route_not_restate`
  (`src/install.rs:~2694`) asserts named skills carry no flag-syntax fragments
  and do point at a tier-1/2 reference. Of PHASE-03's four targets only
  `phase-plan` is in the named set — but hook edits there must satisfy it.

[execute; SL-229-PHASE-03-e1d]
- `record-delta` before `verify-vt` confirmed again: PHASE-03's VT-1..4 read
  `UNATTRIBUTABLE — keyword present but <file> not modified by this slice`
  *before* the delta was recorded, and PASS immediately after, with no file
  change between. The reason string's "keyword present" clause is still
  unearned (keywords aren't evaluated, `vtgate.rs:124`). Two phases in a row
  have paid attention-cost to this; the fix is one string.
- Renumbering a markdown ordered list to insert a step cost 3-4 lines of
  otherwise-untouched diff per skill (`/design`, `/phase-plan`). Cheap, but on
  shared live masters it widens the surface a concurrent agent can conflict on.
  Semantically-ordered hook insertion has no cheaper form; noting the cost, not
  proposing prose-only lists.
- `grep -c` on a binary prints nothing (suppressed as binary) rather than 0 —
  needed `grep -a -c` to prove the re-embed took. The silent-empty result reads
  as "not embedded" and invited a wrong conclusion.
- Verification-surface rot (see the sl227-delta note above) cost this phase a
  documented reinterpretation (D-a) plus a selector removal. The generalisable
  lesson for authored criteria: name the *contract*, not the projection path.
- Self-inflicted, but cheap to avoid: a stray `cat >> file` with no stdin in a
  compound command hung for the full 120s tool timeout and created an empty
  file. Append-via-heredoc is the instrumentation duty's own recipe; a typo in
  it silently costs a timeout window.

[record-memory; SL-229-memfix-c7b]
- `memory verify` refuses on a dirty tree ("a dirty tree cannot be attested"),
  but **verify itself writes the anchor into `memory.toml`** — so it dirties the
  tree it just required to be clean. Verifying N corrected memories costs N-1
  interleaved commits, each a one-line `chore(memory): re-attest …`. Correct as
  an attestation invariant; the cost is that a batch correction becomes a
  commit-per-item ratchet. A `--batch`/`--allow-own-writes` affordance, or
  deferring the anchor write to an explicit `attest`, would collapse it.
- The refusal is also whole-tree, not scoped-paths. An unrelated dirty file
  (here: the RFC-011 case-notes file this instrumentation duty tells agents to
  append to) blocks all attestation. Two duties in the same repo pull against
  each other. Note: the prior handover asserted case-notes "stays uncommitted";
  `git log` shows it is routinely committed (`doc(RFC-011): case note — …`), so
  that convention was local to one agent and is what created the deadlock.
- `doctrine memory validate` walks the corpus with per-memory git checks and
  exceeded a 120s tool timeout on this corpus; running it twice in one compound
  command (count, then grep) doubled that into a backgrounded read. Cheap
  mitigation for agents: one pass, filter the saved output.

[dispatch; SL-228/orchestrator-sessA]
Orchestrator-side token-inefficiency notes, PHASE-01 setup:
- `doctrine memory retrieve` flag mismatch: the /retrieve-memory skill prose says
  "scoped to the concrete files"; I guessed `--scope` (natural reading) → hard
  error → had to `--help`. Correct flag is `--path-scope`. One wasted call +
  a help round-trip. The skill could name the flag, or the CLI could accept
  `--scope` as an alias.
- Slice-lifecycle-flip tree placement under dispatch is under-documented: the
  /dispatch skill says "authored writes from session root" but doesn't state
  that the ready→started (and later audit) slice-status flip commits directly on
  edge/main, NOT via dispatch/<slice>. Had to infer from SL-229's commit history
  that lifecycle commits land on the primary line. Cost extra reasoning to
  convince myself the three-way merge at integrate wouldn't conflict. A one-line
  note in the dispatch skill ("slice lifecycle status commits on the session
  line, not the coord branch") would remove this.

## [audit; SL-229-RV306-a4d]

Friction during the SL-229 post-implementation audit (solo slice, parent tree).

- **`slice show <id>` has no plan/phases face.** The boot guardrail says "read
  entities via `doctrine <kind> show <ID>`, not raw files", but `slice show`
  takes only `<REFERENCE>` — `--plan` is rejected. To read the plan I had to
  `Read .doctrine/slice/229/plan.toml` directly, i.e. violate the guardrail the
  same snapshot mandates. Same for `notes.md`. Cost: one failed invocation plus
  the doubt about whether I was reading an authoritative surface.
- **`slice phase` has no read-only query form.** `doctrine slice phase 229`
  errors demanding `--status` and `<PHASE_ID>` — the verb is write-only. The
  read I wanted was `slice status 229`. One wasted round trip; the noun/verb
  collision (`phase` = mutate, `status` = read) is not guessable.
- **`backlog new` takes a positional title, not `--title`.** Every sibling mint
  verb I had used that session took flags. One wasted round trip.
- **Conformance's undeclared cell needed hand-adjudication.** 19 rows, 18 of
  them foreign commits swept in by shared-`edge` boundary ranges (IMP-175) and
  `.doctrine/**` authored metadata that should be classified out (IMP-292 #1).
  Establishing the real touch-set cost three `git show --stat` calls plus a
  `git log` per boundary range. The `/audit` skill tells you to "read the
  algebra"; on a busy shared branch the algebra has to be reconstructed before
  it can be read. Second datapoint after SL-138/SL-219, and the first on a
  *solo* slice — the defect is not funnel-specific.
- **Distribution is invisible from the authored tree.** Verifying that the
  slice's own deliverable reached a consumer took five probes across four
  surfaces (`git merge-base --is-ancestor` vs the tag, `git branch --contains`,
  `.claude-plugin/marketplace.json`, the plugin cache dir, `library show`).
  There is no single verb answering "is this authored asset live in a harness?".
  For any slice whose product IS shipped prose, that question is the audit.

[dispatch; SL-228/orchestrator-sessA]
PHASE-01 funnel (import→verify→conclude→reap) token-inefficiency notes:
- DOCTRINE_BIN-at-launch friction (cost the most tokens this batch): the MCP
  server launched under ${DOCTRINE_BIN:-doctrine} with DOCTRINE_BIN unset, so it
  ran the stale PATH binary (0.29.0) while the coord build is 0.31.0. `export`
  doesn't persist across Bash calls and the server was already up, so the
  boot.md rule ("set DOCTRINE_BIN to the coord build") could not be satisfied
  post-hoc. I burned a chunk of budget proving 0.29.0 still carried the needed
  import mechanics (advanced-tip merge_tree compose + SelectorIntent::DesignTarget
  — both present since ≤0.29.0, so it was safe). FIX: `dispatch setup` should
  emit a one-line advisory comparing the live MCP server's doctrine version to
  the coord build and telling the operator to set DOCTRINE_BIN + restart if they
  diverge — instead of leaving each orchestrator to discover the skew and
  re-derive its severity.
- Working-tree-free MCP writes leave a STALE coord worktree. Both dispatch_import
  and dispatch_conclude_phase mutate the dispatch/<slice> ref object-db-only and
  never touch the coord working tree, so after each the coord tree is behind HEAD
  (import: worker files show as reverted `M`; conclude: the new boundaries.toml
  shows as staged `D`). The verify beat (`check regression diff`) needs the
  worktree AT S, so a `git reset --hard HEAD` sync is required after import AND
  after conclude. The /dispatch-agent funnel steps don't mention this sync — I
  had to discover it from `git status` and reason it out (and reset --hard is
  exactly the op the AGENTS.md git-footgun rules make one hesitate over). FIX:
  add an explicit "sync coord worktree to HEAD after each MCP funnel write
  (git reset --hard HEAD — the writes are object-db-only)" beat to the skill.

[reconcile; SL-229/RV-306 reconcile pass]
`doctrine review show RV-306 --json` omits the finding array from its top-level
keys (`body`, `kind`, `review`) — findings are nested under `review.finding`,
which is not discoverable from the table view or `--help`. Cost: three probe
round-trips (a python filter over the wrong shape, a raw head dump, then a key
dump) to answer "are all findings terminal?", which is step 1 of every reconcile
pass. A `doctrine review status <RV>` that printed the per-finding disposition
table — or a documented `--json` shape — would collapse that to one call.
Secondary: the PATH binary (~/.cargo/bin) rejected `doctrine reports findings`
while boot.md's command map lists `reports  status next blockers survey explain
findings`; the map is written against a newer surface than the installed binary,
so an agent trusting it burns a call discovering the skew.

[dispatch; SL-228/orchestrator-sessA]
PHASE-02 funnel — scope-declaration friction:
- Design/plan design-target selectors named `.agents/skills/dispatch*/**`, but
  that tree is the UNTRACKED installed projection — the tracked skill source is
  `plugins/doctrine/skills/`. The worker discovered this mid-phase (had to notice
  .agents/ wasn't tracked and redirect to plugins/), and the orchestrator had to
  adjudicate + declare the plugins path at import. A design-time selector authored
  against the wrong tree costs every consuming phase (PHASE-06 too). FIX: author
  skill selectors against the tracked source path, and/or a `slice selector doctor`
  that flags a selector matching zero tracked files at plan time.
- Recurring pattern: design §10's per-file change map is non-exhaustive on
  "forced consequence" files (MCP registry wiring, a doctor Category enum, a
  publication/manifest reachability entry, a registry-count test twin). Each
  surfaces as an `undeclared-scope` import refusal the orchestrator must adjudicate
  and hand-declare. Not wrong per se (these ARE in-objective), but a design-time
  "forced-consequence" checklist (adding an MCP tool ⇒ tools.rs + manifest + count
  test; adding a doctor check ⇒ finding.rs Category) would let selectors be
  pre-declared and avoid the per-phase import round-trip.

[preflight; CHR-048/sessA]
Card-accuracy vs environment: CHR-048's Steps preface asserts the blocker as
"must run on the host — `just release-check` includes a hermetic nix flake build
and nix is absent in the jail." That's the wrong blocker, and it cost two extra
probes to establish. `just nix-build` self-skips gracefully when nix is off PATH
(justfile:141-146 — stderr note, exit 0), so `just release <bump>` would in fact
*succeed* in the jail with the hermetic build silently elided. The real hard
blocker is unmentioned: the jail points `GIT_SSH` at a nonexistent
`git-ssh-disabled` shim, so `git push` / `git ls-remote` cannot run at all.
Cheap general lesson: when a card names a host-only precondition, name the
*failing* mechanism, not a mechanism that degrades — an agent that trusts a
soft-skip gate as a hard gate will cut a release that skipped it.

[preflight; CHR-046/sessA]
Three instrumentation findings, all "the CLI's own read surface misled the
orienting agent" — the expensive class, because each one nearly caused a wrong
readiness verdict rather than just costing tokens.

1. `spec req list` prose column is a readiness trap. It renders a `prose │ —`
   column meaning "the requirement's `.md` tier is empty". Every one of PRD-016's
   and SPEC-025's 11 members shows `—`, which reads as "requirements are stubs,
   nothing authored". In fact all 11 carry a full structured tier — title,
   description, and 3-4 acceptance criteria each, visible only via `spec show
   <SPEC>`'s synthesized Requirements section. I was one step from reporting
   CHR-046's requirement work as unstarted when it is essentially complete. This
   is the two-tier "never judge an entity from one tier" guardrail biting through
   a *listing* rather than a `show` — the guardrail warns about reading raw files,
   not about a listing column that silently reports on one tier only. FIX: either
   label the column for the tier it describes (`md │ —`), or make it reflect
   whether the requirement has *any* content (structured or prose). Cost here:
   ~2 extra probes; cost in the bad branch: a wrong "not yet authored" verdict
   driving redundant re-authoring of 11 requirements.

2. No CLI verb owns the `draft → active` spec transition. `spec edit` covers
   "descent/parent scalar fields" only; there is no `spec status`. Establishing
   that activation is a legitimate hand-edit of `status =` in the TOML took a
   git-archaeology detour (`git log -S 'status = "active"'` → abd843922
   `req(ISS-023): activate 20 draft specs + 161 requirements`). This collides
   head-on with two standing guardrails — "use the CLI, don't guess command
   shapes" and "read entities via `show`, not raw files" — so an agent that obeys
   the guardrails cannot discover the transition at all, and an agent that finds
   it is uncertain whether hand-editing is sanctioned. FIX: either mint a `spec
   status` verb, or state explicitly in the spec skills that spec activation is a
   sanctioned edit-preserving hand-edit, with ISS-023 as the precedent.
   CORRECTION (same session): this note first claimed the gap extended to
   requirement `pending → active` as well. Wrong — `spec req status` exists
   ("Transition a requirement's authored status (free any→any, edit-preserving)").
   The asymmetry is itself the finding: the *requirement* axis has a verb, the
   *spec* axis does not, so an agent generalising from one to the other guesses
   wrong in whichever direction it starts. I generalised from spec→requirement and
   was wrong; a pi-research thread generalised the other way and asserted `spec req
   status --to <status>` for the spec too. Two agents, two directions, same
   asymmetry. FIX as above, plus: `spec --help`'s verb list should make the
   lifecycle affordance (or its absence) legible at the group level.

3. Second instance of the boot command-map skew already noted at e700350d6.
   The boot snapshot's Commands block lists `explore  search inspect relation
   concept-map graph map onboard`, formatted identically to real command groups
   (`change  slice revision rfc ...`). But `explore` is not a command — it is an
   editorial grouping heading, and `search`/`inspect`/`graph`/`map` are top-level
   verbs. `doctrine explore relation --help` fails with "unrecognized subcommand".
   Cost is small per occurrence (one failed probe) but it recurs for every agent
   that trusts the map, and it teaches distrust of the whole block. FIX: visually
   distinguish grouping headings from dispatchable parents, or drop the grouping.

[preflight/research; CHR-046/sessA-delegated]
Two findings about the `pi-research` delegated-thread arm, both about *verification
cost of a plausible brief* rather than raw token count. The thread (deepseek-v4-pro,
one prompt, ~6 min) returned a well-structured 200-line coverage brief that was
right on structure and wrong on three checkable facts. Net value was clearly
positive, but only because every load-bearing claim got re-verified locally.

1. Finding 1 above (the `spec req list` prose-column trap) caught the research
   agent too — independent confirmation that it is a genuine trap, not my slip.
   Its summary asserted PRD-016's and SPEC-025's requirements are "identity-only
   placeholders" with "empty prose bodies" that "need their statement/acceptance
   criteria written", and §4 recommended writing them. All 11 in fact carry
   complete structured title/description/acceptance-criteria tiers. Two agents
   reading the same corpus by different routes reached the same wrong conclusion
   from the same column. That elevates it from friction to a defect: a listing
   column that reports on one storage tier while reading as a verdict on the
   entity. If the brief had been trusted, CHR-046 would have re-authored 11
   already-complete requirements.

2. **A fabricated quote inverted a boundary verdict** — the expensive failure mode
   for delegated governance research. The brief's boundary table attributes to
   "PRD-011 §2 Out of scope" the claim "**any graph rendering or visualization**
   (this is the key seam against PRD-016)", and builds on it a "Seam A: CLEAN. No
   overlap and no gap" verdict. PRD-011 §2's out-of-scope list contains no such
   clause (verified verbatim: authored rank/band scalar, scheduling engine,
   time-pressure semantics, urgency score, persisting derived truth, replacing
   human priority, external policy engine — no rendering disclaim anywhere). What
   PRD-011 §2 *does* claim IN scope is "Registry-backed survey / next-work /
   inspect / explain / blockers surfaces". So the seam is the opposite of clean on
   the CLI-rendering axis — which is precisely what PRD-016's own OQ-003 flags as
   an open boundary question. A trusted brief would have closed a live open
   question as settled, in the wrong direction, with a citation that looks
   authoritative. Also wrong in the same brief: SPEC-025's `interactions.toml` was
   reported as missing `uses SPEC-001` / `uses SPEC-018` edges (both are present —
   I had read the file), and `src/concept_map/` reported as a directory (it is a
   single 143 KB `src/concept_map.rs`; the cheaper `pi-scout` thread got this right).
   Pattern: the research arm is reliable on *structure* (which specs exist, how
   they nest, where the seams are) and unreliable on *verbatim attribution* —
   exactly inverting where an agent's instinct to trust a citation is strongest.
   Practice worth codifying in `/research`: treat a delegated brief's quoted
   out-of-scope clauses and declared-relation claims as unverified until re-read
   at source, and prefer the cheaper scout for anything file-shaped. Cost here:
   ~5 verification probes, all of which paid for themselves.

[preflight; CHR-048/sessA — outcome]
Release landed as v0.31.1 (host). Verified in the live cache: all five skills at
`~/.claude/plugins/cache/doctrine/doctrine/0.31.1/skills/` are byte-identical to
their `plugins/doctrine/skills/` masters — `slice`/`plan`/`phase-plan` 1 hook
each, `design` 2, `research` 2. SL-229's mechanism is now live for the first
time; PHASE-03 EX-2 is satisfied under notes.md D-a's contract reading, and
RV-306 F-1/F-2 are discharged.

Two mechanism facts worth carrying:
- `just sync-plugin-versions` (run inside `just release`) derives all five plugin
  manifests from the Cargo version, so CHR-045's "plugin.json drifts from the
  skill set" failure is structurally fixed, not merely fixed-this-time: 0.31.0 →
  0.31.1 propagated with no manual step.
- The cache key is the *version directory*, and the marketplace sources
  `ref: main` — so a bump is load-bearing for invalidation, and pushing main
  without one leaves agents on the stale dir. Corollary: the plugin cache tracks
  main's tip, not the tag (two commits landed past v0.31.1 and the 0.31.1 cache
  still matches current masters).

Residual: SL-229's design VH — one further real slice driven through the round,
observations recorded here — is now *possible* but not *done*. R1 ("advisory
hooks may under-deliver without enforcement") becomes testable from the next
slice that reaches `/slice`; until one does, the harder-gating question stays
open. That is the eval this release unblocks.

## [preflight; IMP-221-preflight-2026-07-25]

- `doctrine spec req` has no `show` verb, and `req list` takes a **positional**
  `<SPEC_REF>` (not `--spec`). Cost 3 wasted probes to find the shape. Worse:
  `req list` renders `prose │ —` for every REQ, so the governed requirement text
  is **not reachable through the req surface at all** — I had to fall back to
  `grep` on `spec-007.toml`, which is exactly the raw-file read the guardrails
  tell agents not to do. Either `req list` should carry a prose column that works,
  or there should be a `req show <REQ-NNN>`.
- Backlog items carry no staleness signal. IMP-221's sub-item C asserts
  "`memory verify` currently refuses with a dirty working tree" — but
  `--allow-dirty` shipped 2026-06-18/21, ~10 days **before** the item was filed
  2026-07-01. Confirming the premise was stale cost a `--help` read plus a
  `git log -S` archaeology pass. A backlog item whose premise names a CLI
  behaviour has no cheap way to be re-checked against the current binary.

## [dispatch-agent / phase-plan; SL-228-P03-drive]

**1. `worker_commit` refusal payload embeds the whole test log.** The worker's
first commit attempt returned `Refused: commit-gate-red` with the *entire*
~400 KB `check commit` output inline. The actionable content was one line (an
`e2e_no_baked_paths` ban on `env!("CARGO_MANIFEST_DIR")`). Extracting it cost a
full re-read cycle inside the worker's context. The refusal should carry the
failing assertion(s) — or a path to the log — not the log.

**2. A worker's own `doctrine check gate` is not a usable signal.**
`commands/guard.rs:461` resolves the project root from **CWD**, ignoring the
test's `-p <temp root>`, so the worktree's `.doctrine/state/dispatch/worker`
marker makes ~8 authored-write e2e targets fail with `worker fork (signal:
marker): refusing authored write`. The same targets pass inside the
`worker_commit` gate's own run — so it is a worker-shell artifact, but it means
the worker cannot self-verify before committing and burns a commit-gate round
trip to find out. **ISS-028** (the CWD-vs-`-p` root cause is new; the item
previously read this as inherent collateral damage with a workaround only).
Item 1 above is **ISS-219**, open since SL-206 — a second sighting, not a new
defect. The two compound: a worker that cannot self-verify *provokes* the
refusal that then floods its context.

**3. `just validate` is a no-op in a worker fork, so the commit gate carries no
test signal** (`justfile:36-40`). Combined with (2), the worker has no reliable
local green signal at all: its own gate run is polluted, and the recipe the
commit gate runs short-circuits. Cost: the real verification only happens at the
orchestrator's `check regression diff` beat, one funnel step too late to be cheap
to fix.

**4. `.doctrine/` hard wall + a symmetric layering gate forces orchestrator
pre-authoring.** `classify_import` returns `doctrine-touch` before the selector
leg, so no `design-target` selector can admit a `.doctrine/` path. Meanwhile
`tests/architecture_layering.rs` flags BOTH a `[tiers]` row without an on-disk
module (`StaleEntry`) and a module without a row (`Unclassified`). A phase that
adds a new module therefore has no green ordering unless the orchestrator lands
the layering row, an empty module stub, and the module declaration together as
the phase base. This is discoverable only by reading two unrelated source files;
nothing in the design or the skills says it. Cost here: ~6 tool calls of
source-reading plus a non-obvious base-authoring beat.

**5. A selector that a worker can never satisfy reads as permission.** The
design declares `.doctrine/spec/tech/021/**` design-target, which is inert for
import purposes (see 4) and governs slice conformance only. It reads as if a
worker may write there. Same class as the `.agents/skills/**` mis-targeting
already logged for PHASE-06.

**6. Orchestrator-authored base-guard seams went stale/wrong.** The spawn
prompt's check-4 greps named `read_boundaries_at` in `src/dispatch.rs` (it is in
`src/mcp_server/dispatch.rs:663`) and `commit_on_behalf` in `src/git.rs` (it is
in `src/dispatch.rs:5113`). The worker correctly judged intent-satisfied and
proceeded rather than aborting. Root cause: the orchestrator wrote seam
assertions from the design's file map rather than from a grep. Cheap fix — grep
the seams while composing the prompt; a wrong guard either aborts a good phase
or trains workers to ignore the guard.

## [research; SL-230-research-2026-07-25]

- Both research threads (`pi-research`, `pi-scout`) emitted a preamble line
  ("I now have all the evidence needed. Let me compile...") despite an explicit
  "no preamble, start at `## Thread N`" instruction in the prompt. Small per-run
  tax, but it is 100% reproducible across both scripts and both models, so the
  assembler must always strip. Cheaper fix than repeating the instruction:
  have the wrapper scripts trim everything before the first `##`.
- **Escalation of the `spec req` note above — it now has a correctness cost, not
  just a token cost.** Because `doctrine spec req list` renders empty prose and
  the roster lives in `members.toml`, the governance thread concluded outright
  that *"SPEC-007 has no REQ-144 through REQ-156"* — they exist and the CLI lists
  them. A research agent told to read entities via the CLI found nothing and
  inferred absence. The verdict happened to survive (the binding text really is
  in the spec body), but the same failure on a spec whose requirements *do* carry
  prose would silently drop binding constraints from a research round. Requirement
  prose needs to be reachable from the req surface, or `req list` should at least
  signal "prose lives elsewhere" rather than rendering an unqualified `—`.

## [spec-tech; CHR-046-spec027-2026-07-26]

**1. The boot snapshot's command tree implies a subcommand that does not exist.**
The Commands block renders `explore` as a group heading with
`search inspect relation concept-map graph map onboard` indented beneath it —
the same visual shape as `slice`, `review`, `memory`, which *are* real parent
verbs. But `explore` is a documentation grouping only; `doctrine explore inspect
IDE-015` errors with `unrecognized subcommand 'explore'`. Cost: one failed
six-ref loop (every ref reported MISSING — a *plausible* result, which is the
dangerous part), then a debug round to discover the shape. The same trap exists
for `governance`, `relations`, `facets`, `reports`, `knowledge`, `infra`. Either
mark the pure groupings visually distinct from real parent verbs, or make them
real no-op parents.

**2. Requirement acceptance criteria have no write surface at all.** `spec req
add` reserves id + title + kind and nothing else; `description` and
`acceptance_criteria` are hand-edited into each `requirement-NNN.toml`. For a
7-requirement spec that is 7 Read+Edit pairs, or (what I did) writing a
throwaway Python script with a scaffold assertion. Both are pure ceremony
against a closed, known schema. This is the third distinct `spec req` friction
note in this file — the first two were about *reading* requirement prose
(`req list` renders `—`); this is the write half of the same gap.

**3. A handover asserting "settled, do not reopen" mixed a real decision with a
derived fact.** The packet fixed SPEC-027's anchor set at two files as part of
the settled boundary. The boundary *was* settled and correct; the anchor set was
not — `src/catalog/dot.rs` (719 loc, the pure DOT emitter SL-226 PHASE-03
delivered) sits squarely inside it, is governed by nothing else, and is not
covered by any module-root anchor. Honouring the instruction literally would
have shipped a new spec with a fresh coverage hole, in the very session
chartered to close coverage holes. Handover packets should separate *decisions*
(which a receiving agent must not relitigate) from *derived facts* (which it
must re-verify) — an undifferentiated "settled" list invites one or the other
error.

**4. `git commit <path>` cannot commit an untracked file.** The house rule is
path-limit the commit itself, not just the add — correct, but for new entities
it still needs `git add <paths>` first, or the commit fails with `pathspec did
not match any file(s) known to git`. Cost one round per new-entity commit (twice
this session). Worth stating explicitly in AGENTS.md beside the existing rule,
since entity-creating verbs always produce untracked files plus a symlink.
