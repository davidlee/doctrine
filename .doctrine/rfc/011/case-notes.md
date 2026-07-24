
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
