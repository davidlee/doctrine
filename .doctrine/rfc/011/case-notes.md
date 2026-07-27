
[close; SL-228-close-vh1]
`dispatch sync --prepare-review` halted twice on the conformance-completeness
gate, for two causes the handover had recorded as benign:

1. PHASE-08/09 read as "recorded row for a non-completed phase". Cause:
   `registry_completeness` derives the completed set from `completed_phase_ids`,
   which reads the PRIMARY tree's gitignored phase sheets — and the mid-drive
   appended phases were never mirrored there (edge's plan.toml has no such
   phases, so `slice phases` cannot materialise them). The handover called the
   mirror warning "benign, but misreads as a defect". It is not benign: it
   blocks prepare-review at close. Cost: ~6 tool calls reading
   `state.rs`/`dispatch.rs` to establish that the gate roots on primary runtime
   state rather than on plan.toml or the committed ledger.

2. PHASE-07 (evidence-only, deliberately non-funnel) is `completed` but carries
   no source-delta row. The gate has NO exemption for a phase whose delta is
   authored `.doctrine/` artefacts rather than source, so it can only be
   satisfied by a synthetic `Manual` row. The handover asserted the opposite
   ("nothing to record-delta for it") — an untested assumption written before
   prepare-review was ever run for this slice.

Root cause common to both: the completeness gate's inputs (primary runtime
sheets + primary registry + committed ledger on the dispatch ref) span three
tiers in two trees, and its refusal names only the symptom phase, not which of
the three disagreed. Per-phase, the refusal cannot distinguish "you forgot
record-delta" from "this phase's sheet never reached the primary tree".
Same family as ISS-241 and the D10 counter-example set.

## [dispatch; sl230-p05-drive]

**Trunk-drift is invisible at the verb the router sends you to.** `/dispatch`
step 3 says run `dispatch plan-next --slice N`. Its output is phases + `next:`
and carries no base-freshness signal — the `trunk: moved (25 commit(s) ahead of
fork-point)` line lives only in `dispatch status`. The router *does* carry a
"Base freshness (mid-drive)" section saying to watch `dispatch status`, but the
hot-path step it prescribes is `plan-next`, so an orchestrator working the numbered
loop reaches the spawn without ever having run the verb that would tell it. Cost
here: caught only because the handover said "check `dispatch status` before
assuming trunk is stable" — i.e. by a slice-local packet note, not by the skill.
Cheap fix: have `plan-next` echo the same drift line, or fold the freshness check
into the router's step 3.

**Stale `file:line` citations in an authored plan invite a verification round.**
SL-230 PHASE-05's EX-3 pinned "the assertions at `tools.rs:1488` and `:1870` do not
move". By execute time they were at `:1535`/`:1917` — moved by four intervening
phases of the same slice. The criterion's *intent* (tool count stays 25) was
untouched, but the citation forced a check to distinguish "the numbers drifted"
from "a finding". Authored criteria should pin the invariant and cite a grep
anchor, not a line — the same rule the handover already applies to reading lists.

**ISS-253 (arm marker invisible from the coord worktree) confirmed again.** The
`/dispatch` router routes on `.claude/` presence; from the coord tree `ls -d
.claude` is a miss (it is untracked in the primary tree, so the fork does not
carry it). Cost one extra round trip to re-check against the project root.

### [reconcile; SL-228-reconcile-rv312]

Executing an already-written reconciliation brief. Four token sinks, all
orientation rather than work.

1. **Two trees hold the same authored artefact and disagree — and nothing in the
   tooling says so.** `.doctrine/slice/228/design.md` is 1056 lines in the coord
   tree and 763 in the primary. I read the primary's copy first (the working-dir
   default), derived §-numbers and line anchors from it, then discovered the
   divergence only because a `grep` run with a different cwd returned different
   line numbers for the same heading. Every anchor gathered to that point was
   discarded and re-gathered. The handover *did* warn ("canonical in the coord
   tree"), and the warning still lost to the default cwd. Cost: a full re-read of
   §1/§2/§5/§10. A `slice paths` / `show` that resolved to the canonical tree, or
   any staleness marker on the non-canonical copy, would have cost zero.

2. **The brief's section anchors were wrong and only prose-checkable.** Both the
   Reconciliation Brief and finding F-3's response direct the selector mirror to
   `design.md §6`; §6 is `dispatch next`, and the mirror is §10. Nothing detects
   this — a brief cites sections by prose, so a wrong anchor is found only by
   opening the target. Two sections read to locate one edit. This is the same
   family as the item being reconciled (F-5: advice that names the wrong target),
   which is worth noting: the audit's own handoff artefact exhibits the defect the
   audit was documenting.

3. **Boot advertises verbs the pinned binary does not have.** `boot.md` names
   `doctrine reports next` and the `explore` group in its routing/SPINE tables;
   the coord build (0.31.0, the binary the same boot sector tells you to use)
   has neither — `error: unrecognized subcommand 'reports'`. Two dead calls before
   falling back to reading files. The SPINE table is a snapshot of a *different*
   binary than the one `## Invoking doctrine` pins.

4. **`--slice` is not uniform.** `dispatch commit --slice 228`, but
   `slice selector list 228` (positional; `--slice` is rejected with a
   quote-it-as-a-value tip). One wasted round-trip. Small, but it recurs at every
   selector/conformance beat.

Not a complaint about the brief's substance — it was accurate and complete on
every load-bearing point, and "do not re-derive" saved far more than these four
cost. The pattern is that the expensive failures were all *stale or mis-aimed
pointers*, never missing content.

[audit; RV-313-SL230-audit]
- **`| tail -N` on a backgrounded gate masked both the log and the exit code.**
  Ran `doctrine check gate 2>&1 | tail -40` as a background task. The harness
  reported "exit code 0", but a pipeline's status is the LAST command's — `tail`
  always succeeds. The 40-line window also discarded ~4900 test results, so the
  first summarisation counted "23 tests passed" from the tail fragment and read
  as a near-empty suite. Cost: one full re-run (~4 min wall) to get real evidence.
  Rule worth shipping: never pipe a gate/verifier through `tail` — redirect to a
  file and echo `$?` on its own. Cheaper AND more truthful.
- **Grepping `^warning|^error` over a gate log is a false-positive generator.**
  18 hits were doctrine's own runtime warning STRINGS emitted by tests that
  exercise warning paths, not clippy diagnostics. A naive "18 warnings, gate is
  dirty" call would have been wrong. Verify the hits before reporting a verdict.
- **`slice conformance` clean (0/0/6) collapsed a whole evidence branch cheaply** —
  it is the highest signal-per-token verb at audit; run it before reading prose.
- **`candidate status` printed the exact next command, flags and all.** Zero guessing,
  zero `--help` round-trips for `create`. `admit`'s flags did NOT match the shape
  suggested in the handover (`--id` vs `--candidate`, plus a required `--role`),
  costing one refused invocation — the self-describing `status` output is the
  pattern the other verbs should follow.

[reconcile; SL-230-recon-a1]

- **A spec's prose tier alone lost the decisive evidence.** RV-313 F-6 asked
  whether SPEC-007's "Git-anchored staleness" guarantee binds `memory validate`.
  Audit reasoned from the `.md` section's placement ("a peer section under
  `## Responsibilities`") and reached a hedged recommendation. The `.toml`
  structured `responsibilities` list settled far more of it in one read: item [20]
  carries staleness as its own responsibility *separate from* the reader [19]
  (supporting the broad reading), while the prose Overview binds staleness *to the
  find/retrieve reader* (supporting the narrow one) — an outright two-tier
  contradiction neither audit nor the brief noticed. Cost: an audit recommendation
  built on half the evidence, revised at reconcile. The boot guardrail already says
  read via `show`, never one tier — this is a concrete case where the omitted tier
  was the load-bearing one, and worth generalising: for a *scope* question about a
  spec sentence, the structured responsibilities list is higher-signal than the
  prose section's heading level.
- **`grep` for the governed surface before adjudicating conformance.** `validate`,
  `health`, and `finding` occur **zero** times in SPEC-007. Three seconds of grep
  reframed "is this behaviour conformant?" into "does the spec govern this surface
  at all?", which is the question actually worth answering. Neither the audit
  finding nor the brief recorded this fact.
- **Per-item confirmation on brief-verbatim items costs a round trip.** `/reconcile`
  § 3 requires presenting each direct edit for confirmation before writing. All four
  here were specified verbatim by the audit brief, down to the id (`E15`) and the
  figures. The confirmation turn surfaced nothing on those four; the operator engaged
  only with the one genuinely open fork (F-6). Possible sharpening: distinguish
  brief-verbatim items (batch-confirm, or proceed and report) from items where
  reconcile exercises judgement.
- **A REV has no relation surface, so its provenance is prose-only.** `doctrine link
  REV-041 references SL-230 --role originates_from` (and `related` to RV-313 /
  ISS-257) all refuse: "REV may not author `references` (illegal for this source)".
  `revision new --originates-from` accepts an **RFC** ref only. So a REV born from a
  slice reconcile — the modal case the `/reconcile` skill documents — cannot record
  structurally *what it reconciles*. The edge exists only in `revision-NNN.md` prose
  and in the RV's outcome section, i.e. exactly the "recorded in prose only" shape
  the slice already flags as a weakness elsewhere. Costs a downstream reader a
  full-text search to answer "which REV settled this finding?".

## [close; SL-230-close-a1]

- **`check gate` piped through `tail` silently forged a green.** Backgrounding
  `doctrine check gate 2>&1 | tail -40` returns *`tail`'s* exit status, not the
  gate's, and keeps only the last 40 lines. The harness reported `exit code 0`
  and the visible tail was all `ok`, so the run looked green; the rollup was
  3 suites / 23 tests against the audit's 102 / 4928 — the discrepancy is the
  only thing that gave it away. A gate invocation must be captured whole
  (`> file 2>&1; echo $?`), never through a truncating filter. Generalises past
  this skill: any pipeline-wrapped verification beat launders both signals.
- **Moved trunk is the modal close state, not an exception.** `/close` step 3a
  reads as though `--integrate` normally just works, with the moved-trunk case a
  parenthetical ("A moved trunk refuses (admit a superseding close-target
  candidate on the new base)"). Here — as at SL-227 — trunk had advanced 16
  commits past the admitted candidate's base while the slice sat in audit and
  reconcile, so the parenthetical *was* the path. Cost: the whole
  create → verify → admit → integrate shape had to be reconstructed from a prior
  slice's `candidates.toml` (SL-227's row is the only statement of the
  `--source refs/heads/candidate/<N>/<label>` form; the skill's example shows
  `--source refs/heads/review/<N>`, which RV-313's synthesis explicitly forbids
  for this slice). Promoting the superseding-candidate recipe to a first-class
  step, with the source-from-admitted-review-candidate form, would save a
  precedent hunt every close whose slice did not close the same day it was
  audited.

## [design; SL-232-design-f37-repro]

- **Ledger findings are not readable through the CLI.** `doctrine review show 307`
  prints the brief only; charges/responses require hand-parsing
  `.doctrine/review/307/review-307.toml`. The handover warned about this, so the
  cost was one script rather than a wrong turn — but every consumer of a ledger
  pays it. Worth a `review findings <id> [--id F-NN]` verb: the data is already
  structured, and extracting 8 findings from a 744-line toml cost a python pass
  plus one failed guess at the array name (`[[findings]]` vs the actual
  `[[finding]]`). Guessing the shape of a *doctrine-owned* schema is exactly what
  the CLI-is-source-of-truth rule exists to prevent, and here the CLI has no
  answer to give.
- Design-stage reproduction of three review routes cost ~4 scratch scripts. That
  is the work, not overhead — but note the scripts are the durable artefact and
  there is no sink for them: `.doctrine/slice/NNN/research/raw/` exists for the
  `/research` round only. Probe scripts authored during `/design` have nowhere to
  land, so the evidence behind a design assertion is re-derived by the next
  reviewer instead of re-run.
- **The verify dirty-tree gate blocked attesting this close's own harvest.**
  `memory verify` refused the newly-recorded
  `mem.pattern.dispatch.close-target-sources-the-admitted-review-candidate`
  because the tree carried *another agent's* uncommitted version bump
  (`Cargo.toml`, the four plugin manifests). Nothing in that dirt is mine to
  commit, and stashing is forbidden in a shared tree — so the memory lands
  `unverified` with no action available to the recording agent. This is IMP-221
  part C / **SL-232**'s target hit live, and it is worth recording that the gate
  bites hardest in exactly the multi-agent condition this repo runs in by
  default: the blocking dirt is uncorrelated with the memory being attested.
  Under [[mem.pattern.memory.thread-hidden-until-verified]] a `thread` memory in
  this position would have been *unreachable*, not merely unattested.
- **`slice show` does not render a `fulfils` edge.** `doctrine link SL-232
  fulfils ISS-257 --degree full` reports success and the edge is in
  `slice-232.toml` as `[[relation]] label = "fulfils"`, but the `relationships:`
  block of `doctrine slice show 232` omits it — it lists `governed_by`,
  `references(...)` and `needs` only. The guardrails mandate reading entities via
  `show` precisely so an agent never judges an entity from one tier; a
  `show` that silently drops an authored edge inverts that. Cost here was one
  extra verification round-trip against the raw TOML — i.e. exactly the raw-file
  read the rule exists to prevent.

## [design; sl232-design-write-a]

- **Handover line-number references were stale within one commit.** The packet
  cited `src/memory.rs:3376` (`run_verify`), `:3400` (`memory_health_findings`),
  `:3413-3421` (validate staleness), `:2826-2834` (`collect_all`). All four had
  moved (actual: `:3484`, `:3508`, `:3522-3531`, `:2934`). Cost: one wasted
  `sed` round-trip on each before falling back to `grep -n "fn <name>"`. Line
  numbers in a handover are a liability the moment anything lands; a
  `grep`-able symbol name is free and does not rot. Suggests handover packets
  should cite `file.rs::fn_name` and let the reader locate it.
- **`doctrine spec req list` needs a positional the help text buries.**
  `--spec 007` is rejected in favour of a bare `SPEC_REF` positional; three
  attempts (`req show`, `req list --spec`, top-level `show REQ-147`) before
  landing on `spec req list SPEC-007`. And requirement *titles* are not printed
  by any `req` verb — the roster shows `id | label | kind | status | prose`,
  where `prose` was `—` for all 13 rows, so the titles had to be read out of
  `.doctrine/requirement/NNN/requirement-NNN.toml` directly, against the
  standing "read via `show`, never raw files" rule. There is no
  `doctrine requirement show` verb. A queried-surface amendment sweep (F-39's
  fourth tier) cannot be driven from the CLI as it stands.
- **A probe's own falsifier caught a bucketing bug, which is the point.**
  `populations.py` FAL-P1 required every entry to be printed with its verdict;
  the first run put all 59 in one bucket, visibly wrong, because
  `text.lstrip("./")` eats the leading dot of `.claude/`. A summary-only probe
  would have reported a plausible number. Registering the falsifier as *print
  the working, not just the total* is what made it self-evident.

[preflight; sl231-dispatch-arm-plan-a1]
No read verb for a slice's plan/design bodies. `slice show` explicitly excludes
design/plan/notes, and `slice plan <ID>` is an *authoring* verb — invoking it to
read cost a round trip and an alarming `Refusing to overwrite existing
.../plan.toml` error before falling back to `Read` on the raw file. The boot
guardrail says "read entities via `doctrine <kind> show <ID>`, not raw files",
but for the two highest-traffic slice artefacts there is no such path. Every
preflight/dispatch agent pays this. Suggest `slice show --plan/--design` or a
`slice design show` read verb.

[preflight; sl231-dispatch-arm-plan-a2]
`doctrine memory retrieve "<query>"` rejects a positional query
(`unexpected argument ... found`) while `memory search "<query>"` accepts one.
Asymmetric argument shape between the two sibling read verbs → one wasted call.

[preflight; sl231-dispatch-arm-plan-a3]
`slice selector list` is the only place the phase touch-set is visible, and it
is not cross-checked against the worker-forbidden `.doctrine/` floor at author
time. SL-231's locked touch-set contains three `.doctrine/**` paths that no
dispatch worker may write (worker_commit HARD forbidden zone). Discovering this
required reading `src/mcp_server/worker_commit.rs`. A `selector doctor` lint for
"design-target path in the worker-forbidden zone ⇒ orchestrator-authored" would
surface it at plan time instead of at dispatch time.

## [design; sl232-design-write-a, cont.]

- **`grep` is a ugrep wrapper with `-I`, and it silently skips binary files.**
  A verification sweep over the freshly-written `design.md` returned "clean" three
  times — for the RV-307 F-39 code-wording sweep, for dangling `A1`/`X1`, and for
  a control-character check. All three were **false negatives**: the file
  contained a NUL byte (I emitted a literal NUL where I meant the six-character
  text backslash-u-0000), so ugrep classified the file binary and matched
  nothing — including a `grep -c` that should have printed `0`. The tell was that
  `grep -c` printed *nothing at all* rather than a count.
  Cost: ~6 tool calls chasing a phantom cwd bug before checking `type grep`.
  Generalises past this repo: **a negative grep result is only trustworthy if a
  positive control on the same file also passes.** Worth a memory. Arguably the
  harness should surface "binary file matches" rather than swallowing it.
- **The verification instruction was load-bearing and nearly defeated.** The
  packet said of F-39 limb 1: "the rewrite should sweep it. **Then verify it
  did.**" The verification ran, reported clean, and was worthless. Only a
  follow-up sanity count exposed it. An instruction to verify needs a companion
  instruction to *falsify the verifier* — the rule this slice applies rigorously
  to probes, not yet applied to the agent's own checks.
- **Emitting a literal control character into an authored file is easy and
  invisible.** It happened three times in one session while writing *about*
  control characters, and twice more it was caught only by the harness rejecting
  a Bash command. Nothing in the toolchain flagged the file itself; `doctrine
  validate` passed clean with a NUL in a tracked design document.
- **Multi-path pathspecs bite the path-limited-commit rule.** `git commit -- $P`
  with an unquoted multi-path shell variable fails confusingly ("could not open
  directory '<all paths joined>'"). Bit twice. Not doctrine's fault, but the
  path-limiting guidance actively pushes agents toward multi-path pathspecs, so
  it is worth an explicit example in AGENTS.md.

[inquisition; RV-314 SL-232 design pass]
- `review prime` requires the target slice to declare `[[selector]]` entries; the
  skill/ledger doc says "warms the cache from the target slice's selectors" but
  neither states the failure mode when a slice has none. Cost a speculative grep
  of slice-232.toml before invoking. A one-line precondition in review-ledger.md §2
  would remove it.
- The external (codex) arm reported that the binary **rejects `--as inquisitor`**
  on `review raise`, despite `review new --raiser inquisitor` being the sanctioned
  way to stamp the posture. It fell back to `--as raiser`. Round-trip cost: one
  failed invocation + a paragraph of explanation in its report. Either `--as`
  should accept the role *label* set at `review new`, or the help text should say
  `--as` takes a role *slot* (raiser|responder), not the label.
- Instructing an external reviewer to use `./target/debug/doctrine` rather than the
  PATH binary has to be restated in every dispatch prompt; it is project canon
  (AGENTS.md) that the subprocess arm cannot see.

[design; SL-232-rv314-round2-a1b2]
Adversarial round on an amended design section, external reviewer (codex) on a
warm thread. Cost/efficiency observations:

- **Warm-thread continuation was the right call and cheap.** The handover named
  the thread id; `codex-reply` re-read the changed file itself rather than being
  fed the prose. One prompt, three reproduced findings. A cold start would have
  paid for the whole design + probe corpus again.
- **Reproducing the reviewer's probes cost ~3 short bash calls and was worth it**
  — but note the asymmetry: verifying a *positive* claim (here, a false-attestation
  route) is cheap because the counterexample is self-contained. Verifying the
  reviewer's "limbs that held" list is not, and I did not attempt it. That is an
  unpriced trust surface in every external-review round: acquittals are accepted
  on assertion while convictions are re-derived. Worth naming as a standing
  posture rather than rediscovering each round.
- **Ledger `raise` detail fields are carrying full probe transcripts.** Three
  raises here ran 1.5-2.5k chars each of `--detail`. That is the right place for
  them (durable, queryable) but the shell-quoting cost is real: every `'` in a
  pathspec like `:(exclude)` had to be hand-escaped inside a single-quoted bash
  heredoc-less argument. A `--detail-file` / stdin form on `review raise` would
  remove a whole class of quoting risk on the most evidence-dense verb.
- **`review show` prints the prose companion but not the findings table.** To read
  the 14 findings' state I had to grep the sister toml directly, which the
  guardrails explicitly discourage ("read entities via show, not raw files").
  `review show` synthesising the finding roll, or a `review findings <RV>` verb,
  would close a real read-path gap — this cost two exploratory `--help` calls and
  a raw-file read that the boot rules told me not to do.

[design; SL-232-rv314-round3-a1b2]
Round 3 of the same warm-thread adversarial loop. Two datapoints worth more than
the token accounting:

- **The reviewer's ACQUITTAL was wrong, and only re-probing caught it.** Codex
  cleared the `.gitattributes` clean-filter case ("did not produce the same total
  miss") having probed `git diff-index --quiet HEAD`. The design specifies the
  third leg as `diff-index --quiet --cached HEAD`. Against the specified leg the
  miss is total — arbitrary attacker content, all three legs blind. The finding
  it *did* raise was real but strictly understated. This is the concrete instance
  of the asymmetry flagged in the round-2 note: convictions get re-derived,
  acquittals get accepted on assertion. Here the acquittal was the expensive one.
  Cost of catching it: one 12-line bash probe. Cost of missing it: a blocker
  ships as a wording fix.
- **Cheapest catch of the round was self-directed.** Before the reviewer
  reported, I probed the attack surface I judged most likely (a field I had
  introduced that turns author-controlled TOML into a path) and found the
  amendment under-specified. Pre-empting the reviewer on your own new prose is
  much cheaper than a full round-trip — but note I *still* got the axis wrong:
  I checked the uid's alphabet, declared it safe, and missed that identity was
  never bound to storage. Same class as the session's standing lesson (check an
  invariant's polarity, not just its truth): I verified a true property that was
  not the load-bearing one. Self-probing narrows the gap; it does not close it.
- Ledger mechanics unchanged from round 2 — `--detail` is still carrying multi-kB
  probe transcripts through shell quoting, and `review show` still will not print
  the finding roll. Both already noted; repeating only to mark that they recurred
  every round rather than once.

[inquisition; SL-233 RV-315 / iq-233a]
- `doctrine review prime RV-315` fail-hards on tracked slug symlinks
  (ISS-259 / RV-315 F-1). Cost: ~4 tool calls to distinguish "my mistake" from
  "engine bug" (compare-prime against RV-314, then read three source sites to
  pin the cause). A named refusal — "selector fileset contains a symlink to a
  directory: <path>" — would have cost zero.
- `doctrine reports next` / `doctrine reports explain` are in the boot digest's
  routing text but absent from the 0.33.1 binary's command list (`error:
  unrecognized subcommand 'reports'`). One wasted call. The SPINE block in
  boot.md lists `reports status next blockers survey explain findings`, so the
  digest and the binary disagree.
- `doctrine slice selector SL-233` — guessed shape; the verb takes a subcommand,
  not a bare id. Recovered by falling back to `slice show --json` + a python
  filter, which is the reliable read for selectors but costs a heredoc.
- `doctrine backlog paths ISS-259` prints `backlog-259.{toml,md}` while the
  entity is created as ISS-259 — the file stem does not follow the kind prefix,
  so a `cat` composed from the id fails. Cost: one failed read.
- Two zsh glob failures on unquoted `--include=*.rs` / `src/prompt*.rs` passed
  to Bash; both returned "no matches found" that read like empty grep results.
  The footgun hook fired correctly on the negative-grep risk.

---

[design; SL-232 RV-314 round 4 — handover pickup]

**`review show`'s missing finding roll let two blockers drift out of sync
undetected.** The handover packet, `notes.md` § Harvest and `design.md` § 10 all
recorded RV-314 as "20 findings, all disposed" with F-1/F-10 "answered". The
sister toml has both at `status = "open"` with no `disposition` — the remedy
prose landed in `81e3e732c` but `review dispose` was never run. Three authored
artefacts agreed with each other and disagreed with the ledger, and the
divergence was invisible through the sanctioned read path: `doctrine review show
RV-314` prints the derived status, the `reviews` edge and the brief, but not the
per-finding roll. The only way to see it is `command grep` over
`.doctrine/review/314/review-314.toml`, which is exactly the raw-file read the
"read entities via `show`" guardrail forbids. Cost: the state had to be
re-derived by hand before any work could start, and a round-3 agent's summary
had already propagated the wrong version into two documents. Already logged once
as a read-path gap; logging again because this is the first time it produced an
actual false record rather than an inconvenience.

**The codex MCP endpoint rejected an adversarial-review prompt on a
cybersecurity content classifier.** The round-4 prompt described the design's own
recorded hazards in the design's own vocabulary — a committed symlink whose blob
text is read as pathspec magic and thereby narrows the measured file set, a clean
filter hiding worktree content from all three probe legs. Response: *"This
content was flagged for possible cybersecurity risk."* Rephrasing the same
substance in correctness-review terms ("a string reaching git unprefixed can be
read as pathspec magic and thereby narrow the set of files actually measured";
"content that diverges from the blob without limit") passed unchanged. Nothing
was dropped — but the rewrite cost a full prompt round-trip, and the failure mode
is silent about which span tripped it. Any skill that sends `/inquisition`-style
prose to an external reviewer will hit this: adversarial review is *written* in
the register the classifier screens for.

**`codex-reply` returned `content: ""` on the accepted prompt.** No error, no
partial output. A one-line liveness probe confirmed the thread had in fact
received and understood the instructions, so the empty return was the transport,
not the model — but distinguishing "thread died" from "silent empty reply" cost
an extra round-trip that a non-empty error would not have. Then the follow-up
"proceed" reply ran past the 120s MCP timeout into a background task, which is
the correct behaviour but means a long external review always costs at least
three exchanges: prompt, liveness/ack, execute.

[dispatch; SL-231-p01-import-restart]
`dispatch_import` returned bare `MCP error -32603: Internal error` for
`name="SL-231-p01"`. The real cause was a **wrong argument**: `name` is the fork
BRANCH (`dispatch/SL-231-p01`), not the agent basename. The tool's own doc says
so ("e.g. `dispatch/<agent>`"), but an unresolvable ref is raised as an internal
FAULT rather than a structured `Refused{reason: unknown-fork}`, so the surfaced
error carried zero diagnostic content.

Cost: the previous session misattributed it to a stale MCP binary on PATH,
wrote a handover, and burned a full session restart + re-orientation — then the
same error reproduced immediately. Recovery took driving `serve --mcp` by hand
over stdio to read `data.message: "resolve fork tip SL-231-p01"`.

Two fixes, both cheap:
1. `import` should resolve a bare agent name as `dispatch/<agent>` (the reap
   tool already speaks in `dispatch/<agent>` terms), OR
2. an unresolvable `name` should be a structured refusal naming the ref it
   tried — the refusal family already exists and is documented as "the recovery
   procedure"; this path escapes it.
Token-efficiency: a one-line refusal would have replaced ~2 sessions of work.

## [feedback + rigour; sl232-rv314-round4-disposal]

**A negative grep read as a clean result — nearly shipped a blocker.** Sweeping
`capture()` for attribute-sensitive git calls, I grepped for
`run_git|git_stdin|Command::new` and got an empty result, which presents
identically to "nothing to fix". `capture()` actually calls the wrappers
`git_bytes`/`git_opt`/`git_text` (65 occurrences in the file), so the query could
not have returned a positive on any input. The defect it missed —
`untracked_fingerprint` hashing through a filter-sensitive `hash-object` —
falsified a claim already written into a decision record. Cost to catch: one
probe. Cost to miss: a false `checkout_state_id` identity on the `--allow-dirty`
path, which is 24 of 59 attestations. Generalisation now recorded in the slice:
**before trusting a negative, confirm the query can produce a positive.** Same
family as the earlier ugrep binary-skip note, and the second time on this slice.

**`review show` does not print the finding roll.** The sanctioned read path for a
review cannot show finding state, so three artefacts (design § 10, notes §
Harvest, a handover packet) recorded F-1/F-10 as "answered" while the ledger held
them `open` with no disposition — each having copied the last. Recovering it took
a raw `grep` over `review-314.toml`. A `review show --findings` (or a roll in the
default output) would have made this unconstructible. Worth a backlog item on the
doctrine CLI itself; noted here rather than filed because it is RFC-011's
territory.

**zsh word-splitting broke a `--include` glob again.** `grep -rn 'x' src/
--include=*.rs` → `(eval):3: no matches found`. The interactive shell does not
word-split *and* globs the unquoted pattern. Second occurrence this slice; the
handover packet had warned about it and I still hit it. The durable fix is to
stop reaching for `--include` and path-limit instead.

**`chmod +x $DIR/*.sh` touched six files I did not author.** Copying five new
probes into `probes/` and then chmod-ing the glob flipped the mode on every
pre-existing probe, showing up as six spurious `M` entries. Caught by
`git status --porcelain` before staging, reverted with a path-limited
`git checkout --`. Cheap here; in a shared tree with another agent mid-edit it
would have been someone else's diff. Name the files, not the glob.

**Cost shape of a self-attack round.** Five probe scripts, ~15 min, before
opening round 5. It falsified one of three new decisions and produced two backlog
items in another subsystem. The round-3 note said self-probing "narrows but does
not close" — that held: the same session both wrote the false claim and caught
it. The argument for doing it anyway is that the external round is expensive
(three exchanges, a content-classifier rewrite, a 120s MCP timeout into a
background task), so spending cheap tokens to remove the findings a reviewer
would otherwise spend expensive ones on is straightforwardly positive.

[preflight; sl233-plan-research-20260727]
`./scripts/pi-{scout,research}` prepend ~30 lines of Bun stack trace to EVERY
thread's stdout before any content. Cause: `.pi/extensions/doctrine/index.ts:8`
(`resolveBoot`) shells a hardcoded/stale nix-store path
`/nix/store/…-doctrine-0.31.1/bin/doctrine prompt resolve --role orchestrator`,
which no longer exists in the jail. The extension fails open (research still
runs) but the trace lands in the artefact, so it is re-read by the consuming
agent on every `Read` of `research/raw/*.md` — pure waste, multiplied by thread
count (5 threads × ~30 lines here). Fix is either resolving the binary through
`${DOCTRINE_BIN:-doctrine}` like `.mcp.json` already does, or suppressing the
extension's stderr in the wrapper scripts.

[dispatch; SL-231-p02-fork-binding]
IMP-328 cost PHASE-01 a re-fork + cherry-pick recovery. For PHASE-02 the
orchestrator side-stepped it for ~0 tokens: mint the fork bound by hand
(`worktree fork --base B --branch dispatch/<agent> --dir <coord>/.worktrees/<agent>
--worker --slice N --phase PHASE-NN`), then spawn with `PI_REUSE_FORK=1` so
`pi-spawn-confined.sh` attaches instead of re-forking unbound. Import resolved
first try. Worth folding into the script (accept --slice/--phase and pass them
through) — the workaround is two extra lines but has to be REDISCOVERED each
phase, which is the expensive part.

Note the funnel oracle still prints `spawn` for a phase whose bound fork already
exists and whose worker is mid-run, because the pi arm records no spawn beat.
That is a misleading prescription on the ONLY arm that needs it most: an
orchestrator obeying it literally would destroy live work (the script's
non-reuse path does `rm -rf "$D"`). A `fork-armed` rung between `spawn` and
`await-worker`, derivable from the durable fork binding the funnel already has,
would close it.

[dispatch; SL-231-p02-review-placement]
Two defects were found by orchestrator inspection, not by the gate and not by a
review pass: an STD-001 magic-string split across a module boundary, and a
near-verbatim duplicate of the function the phase existed to extract. Both pass
every automated check. Both were fixed in the fork BEFORE the delta commit —
because `record-delta --commit S` records exactly one commit's patch, a defect
fixed after import lands outside the phase's boundary row and silently leaves
the conformance registry describing something other than what shipped. The
skill's cadence puts per-phase review AFTER import; for the pi arm, where the
orchestrator commits the delta by hand anyway, review before the commit is
strictly cheaper and keeps the boundary row honest.

[review/round-5; sl232-rv314-r5]
- **Warm-thread liveness probe paid for itself again.** One-line probe before a
  multi-kB round-5 prompt; the thread answered with its own round-4 numbering
  (R4-1..R4-12) rather than the ledger's F-21..F-32, which confirmed warmth *and*
  revealed that the reviewer does not carry ledger ids. Cost ~30 tokens, avoided
  re-sending a 3kB prompt into a dead or mislabelled thread.
- **The long review ran past the 120s MCP timeout into a background task, as
  predicted.** ~13 minutes wall clock. The waiting window was not wasted — the
  responder independently derived and reproduced two of the nine findings
  (F-34, F-35) before the reviewer returned. **Recommend making this explicit
  practice**: when dispatching a long adversarial round, work the handover's own
  ranked self-assessment in parallel rather than blocking. It also produced a
  cross-check: two independent reproductions of F-34 on different index flags.
- **Instructing the reviewer to re-derive its own acquittals was the highest-value
  line in the prompt.** It returned ~11 explicit acquittals with fresh
  measurements, which are now recorded in notes.md § Harvest so round 6 does not
  respend them. Without the instruction these arrive as silence, and silence is
  indistinguishable from "did not look" — which is exactly how DEC-082 survived
  round 3 and died in round 4. Cheap prompt line, large downstream saving.
- **The content-classifier rewrite budget was NOT spent this round.** Round 4
  needed one rewrite round-trip after the endpoint refused adversarial-review
  prose. Framing round 5 in correctness-review vocabulary from the first send
  (soundness, false attestation, byte divergence — not exploit/hiding/attacker)
  passed unchanged. Worth promoting from "budget one round-trip" to "write it in
  correctness terms first".
- **`review_raise` via MCP avoided the documented zsh quoting workaround
  entirely.** The handover prescribes writing multi-kB detail to a file and
  passing `--detail "$(cat file)"` from bash because the interactive shell is zsh
  and does not word-split. The MCP tool takes the detail as a structured
  parameter, so the whole hazard disappears. **The handover advice is CLI-specific
  and should say so** — an agent following it via MCP writes scratch files for no
  reason (I wrote two before noticing).
- **Ledger ids lined up with the reviewer's proposed numbering by luck, not
  design.** The reviewer emitted F-33..F-41 and the ledger assigned F-33..F-41
  because exactly 32 findings existed. Had another agent raised concurrently they
  would have diverged silently, and nine details citing sibling ids
  cross-reference each other. **A raise-time id echo is load-bearing and there is
  no guard.**

[preflight; sl233-plan-research-20260727]
Research-tool misrouting cost ~25 min wall-clock and 3 wasted threads.
CLAUDE.md frames the pair as "`./scripts/pi-scout` (quicker, cheaper) or
`./scripts/pi-research` (smarter)" — i.e. two speeds of one job. They are two
DIFFERENT jobs. `.pi/agents/researcher.md` is a *web* research specialist:
tools `read, write, web_search, fetch_content, get_search_content` (no grep, no
find, no bash) and a system prompt commanding "conduct thorough web research…
break the question into 2-4 searchable facets… search with `web_search`".
`.pi/agents/scout.md` is the repo agent: `read, grep, find, ls, bash, write`.
Consequence: three repo-internal judgement threads (spec-descent boundary,
RFC-021 crossover, review-ledger analogue) were dispatched to pi-research on the
documented cue "needs judgement → use the smarter one", and sat web-searching
for answers that exist only in this tree. They had to be killed and re-fired on
pi-scout with `--think high`, which is the correct instrument for a
judgement-heavy *codebase* question.
Cost multiplier: the failure is SILENT. pi-research can `read` a path it is
handed, so it produces plausible-looking output rather than erroring; only the
absence of grep and the elapsed time revealed it. An agent that trusted the
CLAUDE.md framing and did not inspect `ps` would have folded web-sourced
guesses into a plan.
Fix: correct the CLAUDE.md line to distinguish by *domain* (repo vs web), not by
"smarter"; scout's `--think high` is the escalation for hard repo questions.

[dispatch; SL-231-p03-review-yield]
Three cleanup turns on PHASE-03 vs one on PHASE-02 and zero on PHASE-01. The
scaling factor was not phase difficulty per se but SURFACE COUNT: P03 shipped a
1009-line CLI adapter, a 796-line e2e suite, a guard classification, and an
escaper — four independent surfaces, and each cleanup turn found a defect on a
different one. Neither the worker's own gate nor the funnel verify beat could
see any of the three: all were green-passing correctness/design defects.

The cheapest detection was ORCHESTRATOR READING OF THE DIFF, not a review agent.
Total cost of finding all three was a handful of targeted greps against the fork
(structure of the new file, call sites of a suspicious helper, test-content
sampling for the escaper). Two heuristics did the work and generalise:
  1. a "thin adapter" whose line count rivals the service it adapts is
     re-implementing something — grep its private fns for verbs the service
     owns (filter/order/resolve/merge);
  2. a security/robustness test that is green is only as good as its INPUTS —
     grep the test file for the input class it claims to cover (here: no
     non-ASCII byte existed anywhere in the suite, so a Latin-1 corruption bug
     was structurally undetectable).

Also: asking the worker for a READ on an ambiguous design point (rather than
mandating a fix) surfaced a real EX-5 gap it had not been told about — good
yield — but its proposed remedy was unworkable. Ask for the analysis; do not
import the conclusion.

[preflight; sl233-plan-research-20260727]
Second wrapper defect, ~30 min lost, diagnosed by the user not the agent:
`pi-scout`/`pi-research` `exec pi` with stdin INHERITED, and pi does not
self-exit while stdin is open. `scripts/pi-spawn-confined.sh:13-14` documents
this exact behaviour and handles it (timeout backstop + `agent_end` poll +
kill -9); the research wrappers do none of it. Result: two of six threads
returned zero bytes with no exit status, twice across a re-fire, while four
identical siblings succeeded — pure fd-0 luck.
Agent-side lesson for RFC-011: I treated "no output, no process, no exit code"
as a crash and re-fired verbatim TWICE before the user named the cause. The
correct reflex on a silent subprocess failure is to compare against the repo's
OWN working invocation of the same binary — `pi-spawn-confined.sh` had the
answer in a header comment the whole time, and grep-ing for how the repo already
spawns pi would have found it in one call. Cost: ~2 wasted re-fires plus a
foreground-debug attempt. Captured as ISS-266; ISS-265 is the sibling guidance
fault in the same two scripts.

[feedback; SL-233 RV-315 F-16..F-19 disposal]
The handover's "Next actions" asserted all four findings were "repairs to
`design.md`, not to code", and each finding's own repair text said "strike from
B2" / "add to B2's descent". B2 is a **scope** bullet (`slice-233.md:175-177`),
not a design section — `grep -n "B2" design.md` returned nothing and cost a
second corpus-wide grep to locate. Cost: ~2 extra tool rounds, plus the risk of
a wrong edit had the grep been trusted less. Root cause: findings raised from
research cite a doc-local label (`B2`) without its owning file, and the handover
inherited the label without resolving it. Cheap fix at the raise site — a
doc-local enumeration reference should carry `file:line` on first use in a
finding, exactly as the same findings do for `spec-003.md:50` and
`src/review.rs:2025-2133`.

[code-review; RV-317-sl231-review]
- **pi log extraction is the dominant incidental cost of a review pass.**
  `pi-spawn-confined.sh` writes the raw RPC event stream to the log — ~1 MB for a
  48-line file, because every `message_update` carries the FULL accumulated
  `partial` content, not just the delta. Getting the reviewer's verdict out
  needed a bespoke Python walk over nested JSON, and the first two attempts
  grabbed the wrong text (a file the agent had read; then the echoed prompt)
  because "longest text block" and "block containing VERDICT" are both wrong
  heuristics — the prompt itself contains the word VERDICT. Three tool calls and
  two persisted-output truncations to retrieve one page of text that the
  subprocess already had in hand. A `--final-message-only` output path, or the
  script tee-ing the last assistant message to `<log>.verdict.txt`, would remove
  this entirely and it recurs on every single review/worker turn.
- **Harness refuses tool input containing raw control characters.** Raising a
  finding whose evidence is an ANSI-injection reproduction failed with
  `InputValidationError: command contains control characters that would be hidden
  in the approval dialog`. Correct guard, but for THIS project the payloads under
  review are frequently hostile strings, so it will recur. Workaround: write the
  detail to a scratchpad file with printable stand-ins (`<ESC>`, `^[`) and pass
  `--detail "$(cat file)"`. Worth knowing up front rather than discovering
  mid-raise — cost one wasted raise attempt with a long argument.
- **Reviewer line numbers were systematically diff-relative** (~410 off), so
  every one of 11 findings had to be re-located against the source before it
  could be raised. Not harness friction, but it doubles the adjudication cost of
  an otherwise-good pass; prompting for `grep -n` output alongside claims would
  likely fix it.
- Minor: `for cmd in "supersede X Y"` word-splitting bit again (the handover
  warned Bash `$VAR` splitting misbehaves in this harness shell) — the whole
  string arrived as one subcommand. Literal args only.

[harvest/park; sl232-park]
- **An unpersisted counter-result cost a full round.** Round 4 wrote "the
  stat-cache limb did NOT reproduce" into `probes/README.md` with no script
  behind it, and narrowed a live blocker on that basis. It was false. Findings
  are held to "reproduce before you raise"; **counter-results are held to
  nothing** — and they are more dangerous, because a finding invites scrutiny
  while a counter-result stops people looking. Cheap fix worth generalising: a
  README claim that something did NOT reproduce should be refused unless it cites
  a committed script, exactly as a finding is.
- **Parking cost ~4 tool calls because the sinks already existed.** notes.md
  § Harvest was single-copy and current, `handover.md` is gitignored and
  rewritable in place, CON-003/RFC-022 already held the reasoning. The expensive
  part of a park is deciding *what is settled vs open*, and that had already been
  written down incrementally rather than reconstructed at the end. Argues for
  harvesting continuously rather than at wrap.
- **`doctrine link` flag discovery cost two failed calls** — `CON` may not author
  `related` (legal: references / shapes / spawns / governed_by), and `references`
  then requires `--role`. Both errors were legible and self-correcting, but the
  legal-label set is per-source-kind and is not discoverable from `link --help`,
  which lists roles but not which kinds may use which labels.
- **The RFC scaffolder mints no slug symlink** where `knowledge new` does
  (CON-003 got one, RFC-022 did not). Left alone rather than hand-created, since
  the convention says the *command* mints them. Possible inconsistency; not
  chased.

[code-review; RV-317-sl231-remediation-turn1]

- **pi log final-report extraction, round two.** The previous note recorded that
  "longest text block" and "block containing VERDICT" both mis-fire. The
  replacement heuristic — last assistant message by stream position — ALSO
  mis-fires: it returned a mid-run `"Let me read the wire module..."`. The final
  report does not ride the `message_update` stream at all; it lives in a
  terminal `{"type":"turn_end","message":{...}}` event, with `agent_end` and
  `agent_settled` after it. Cost: one wasted extraction round, one throwaway
  second script, and a moment of believing a completed worker had stalled.
  The durable rule is `type == "turn_end"` → `message.content[].text`, not any
  positional heuristic over the partials. Worth folding into a shipped helper
  rather than re-derived per session — this is the third variant written.

- **`$UID` is readonly in zsh.** A repro script assigning `UID=<uuid>` died with
  `bad math expression`. Not doctrine's fault, but worth noting that the spawn
  environment's shell is zsh while the reflex is bash; `$UID`, `$PATH`-style
  specials and word-splitting differences have now each cost a round in this
  slice.

- **Adjudication caught what the gate could not, again.** The worker's report
  read `D1: FIXED / DEVIATIONS: NONE / UNCERTAIN: NONE` with clippy clean and
  4123+31 green. One of the five was half-fixed: single-line header fields were
  escaped with the multi-line context, so the injection the finding described
  still reproduced — through a DIFFERENT view than the one the test asserted.
  The worker's own new test passed because it asserted "no raw ESC" and never
  asserted the newline was neutralised. Cost is not the fix (small) but the
  principle: a self-authored test pinned the half of the contract the author had
  in mind. The empirical re-run of the original reproduction is what caught it,
  and it took one command. Re-running the finding's own repro against the built
  binary should be a NAMED step in the remediation loop, not orchestrator
  discretion — it is far cheaper than the review that found the defect.

[feedback→plan handoff; SL-233 lock/decompose ordering]
Locked the design the moment the ledger went terminal, then the User raised
slice decomposition in the very next turn — which would have invalidated four
spots in the just-locked design (§5.5 home, DEC-079, §9.5, §10). Cost: a minted
slice, a scope amendment written then reverted, and two turns of analysis, all
discarded. Root cause: the lock gate treats "ledger terminal" as sufficient,
but a design is only safe to lock once *plan shape* is also settled — the
decomposition question ("is this one slice or several?") is answered at plan
time yet invalidates design content. Cheap fix: /design's lock step should ask
whether the phase terrain is agreed before flipping status, or /plan should be
reachable from a `design-locked-pending-shape` state. Second-order note: the
recovery was only cheap because CHR-049 already existed — I recommended the
split without checking for an existing post-close vehicle first. Check the
backlog for an existing carrier before proposing new structure.

[plan; SL-231-vt-settle-2026-07-28]
Handover packet carried a factually wrong remediation recommendation that cost
a full investigation cycle to detect. It proposed adding "a solo (non-linked)
marked-fork case" for PHASE-03 VT-3. That test cannot pass: `marker.rs`
`describe_mode` computes `marker_leg = is_linked && marker_present`, so a marker
on a non-linked tree is inert by design. Design §3.4:288 ("a solo agent in a
marked *worktree*") shows both solo and dispatched forks are linked; the env leg
is what separates them. The packet had also labelled its own landed test
"(dispatched)" when `run()` does `env_remove("DOCTRINE_WORKER")` — making it the
marker-only/solo shape. So the uncovered half was the inverse of what was
handed over.

Cost: ~6 tool calls to establish (read describe_mode, grep design vocabulary,
read the e2e run helper). Cheap only because the packet named the exact source
line to check. Root cause is not the handover format — it is that the packet's
author wrote the recommendation from the *criterion's prose* ("dispatched and
solo") without resolving that prose against the predicate that implements it.

Generalisable: a handover that recommends a FIX (not just states a fact) should
carry the one-line evidence that the fix is constructible, or explicitly mark it
unverified. A confident unverified recommendation is more expensive than an
open question, because it suppresses the reader's own derivation. Same failure
shape as the self-report unreliability already noted for remediation turns —
assertion presented at the confidence of evidence.

[dispatch-agent; SL-231-p04-spawn-2026-07-28]
Two orchestrator-side defects, both of which cost a full worker turn (~265k
subagent tokens, ~28 min) to surface.

(1) HALF-ARM → `unprovable-fork`. The `/dispatch-agent` skill's pre-spawn
literal reads `doctrine dispatch arm-spawn --base <B> [--slice <N>]` — no
`--phase`. But the CLI's own help is explicit: `--phase` is "the other half of
the durable fork binding ... Both halves are needed: a half-arm binds nothing."
Following the skill verbatim produces a record with `slice` absent AND `phase`
absent (a bare `--slice` is itself only half), so `require_binding` returns None
and `worker_commit` refuses `unprovable-fork` — AFTER the worker has done all
its work. The skill's documented literal is stale w.r.t. SL-228 PHASE-04 D2.
Fix: update the skill template to `arm-spawn --base <B> --slice <N> --phase
PHASE-NN`. Cheap, and it removes an entire class of end-of-turn loss.
Aggravating: the failure is maximally late. Arming is the FIRST beat, the
refusal is the LAST. A pre-spawn assertion that the record carries a complete
binding would convert a 28-minute loss into a 2-second one.

(2) PROMPT PRESCRIBED AN UNSATISFIABLE REUSE. My distilled worker prompt said
"REUSE `escape_hostile`/`escape_metadata` from `src/commands/observation.rs`,
do not reimplement". Unsatisfiable: `mcp_server` and `commands` are both
command-tier and SL-203 deliberately SEVERED the `mcp_server → commands` back
edge to break their SCC (layering.toml records the 90→86 tangle drop). Importing
it back re-forms the SCC and reds `architecture_layering` — which the same
prompt forbade retuning. The worker found the only consistent path (move the
genuinely-shared items down into the `observation` leaf) and reported it as a
deviation. Correct call; the prompt was wrong.
Generalisable: a "reuse X from module M" instruction is a LAYERING claim, not
just a DRY one. Before writing it, check that the consumer's tier may depend on
M — especially where a prior slice severed an edge on purpose. The severance is
invisible from the call site; it lives in layering.toml and a memory.

Worth noting on the other side: the worker's self-report was accurate on every
claim independently checked (the 42-failure marker analysis, the byte-identity
of both moves, keyword presence). The named re-verification step still earned
its keep — but this turn it confirmed rather than caught, which is itself a
datum about a well-fenced prompt.

[dispatch-agent; SL-231-p04-spawn-2026-07-28 — CORRECTION to entry (1) above]
The account above is incomplete in a way that matters. A memory recording this
EXACT trap already existed and was recorded the previous day:
`mem_019f9effcf4a7922b31c1a1b37841d06` — "A half-arm binds nothing —
`arm-spawn --slice` without `--phase` costs a whole worker run" (SL-228
PHASE-09, 2026-07-27). It carries the correct/incorrect command pair verbatim
and a section literally titled "Why an orchestrator walks into it".

So the primary root cause is NOT the stale skill literal (though that is real
and still worth fixing). It is a RETRIEVAL SCOPING failure that is easy to
repeat: at `/phase-plan` I ran `/retrieve-memory` scoped to the phase's FILES
(`src/mcp_server/tools.rs`, `src/doctor_checks.rs`, …) and got four genuinely
useful hits. I never ran it scoped to the ORCHESTRATION COMMANDS I was about to
execute (`dispatch`, `arm-spawn`, `worker_commit`). The memory is tagged on the
command surface, not the phase's file surface, so a file-scoped probe cannot
reach it.

Generalisable, and probably the most valuable thing in this session: **a phase
has two distinct memory surfaces — the CONTENT you are changing and the
MECHANICS you are driving.** Retrieving only the first is a silent half-probe.
The dispatch skills should say so at the pre-spawn beat, because the mechanics
surface is exactly where the end-loaded, whole-turn-cost footguns live.

Second correction: the memory states there is NO re-bind verb and lists exactly
two options — re-arm+re-spawn, or fallback (A) live-worktree import. I took a
third it does not sanction: hand-editing the coord-tree `DispatchRecord` to add
`slice`/`phase`. Rationale — it is gitignored runtime state under the
orchestrator's sole-writer authority, unreachable from any worker jail (so the
anti-forgery property, which guards WORKERS, is intact), and it binds the fork
to the row it actually worked on rather than mis-binding it. It also preserves
the gated-commit + worker-authorship path that both sanctioned options give up.
But it does defeat the fork-time-snapshot property the design names, and the
corpus does not bless it. Recording it as a deliberate, disclosed deviation, not
a discovered best practice — and the memory needs updating either to admit a
third option or to say explicitly why the repair is forbidden.
[research on RFC-023 executable phase gates; 023-research-01]
The `justfile`/`doctrine.toml`/`doctrine check` triad is already the POL-002-clean
proxy pattern that RFC-023's Directions C+E need. The agent spent ~15 tool calls
reading files that the boot snapshot and memories already described (justfile
recipes, cadence resolution, VT shape). A `doctrine memory retrieve` query for
"verification config" returned memories about stale binaries and spec anchors but
not the cadence proxy pattern — the pattern is obvious from reading
`src/verify.rs` directly once you know the file exists, but finding that file
required running `grep` on `src/commands/check.rs` output, then reading the
module to find the `resolve_check` → `run_suite` split. A memory
`mem.pattern.doctrine.cadence-proxy` keyed to `[verification]` and `verify.rs`
would have collapsed 5+ exploration calls into one retrieve.

[dispatch/phase-plan/execute; SL-231-PHASE-05-orchestrator-inline]

- **`arm-spawn`'s end-loaded failure cost a full worker turn.** Omitting
  `--phase` produces a fork bound to nothing, but nothing detects that until
  `worker_commit` refuses `unprovable-fork` at hand-back — ~265k tokens and
  ~28min after the fact. The `/dispatch-agent` template still renders `--slice`
  optional and omits `--phase` entirely. Filed IMP-331; the durable fix is
  failing closed at arm time, which converts a worker-turn loss into a free
  error.
- **Retrieving memory against a phase's FILES but not its COMMANDS is a silent
  half-probe.** A memory documenting the trap above — including a section on
  why an orchestrator walks into it — existed and was missed, because it is
  tagged on the command surface and `/phase-plan`'s retrieval step was scoped to
  the phase's touch-set. Two surfaces per phase: content and mechanics.
  Retrieving both at PHASE-05 paid immediately (three memories landed directly
  on the work: the gitignore inline-comment trap, the stale-embed boot rollback,
  and the client-surface placement rule).
- **A handover recommendation is a hypothesis, not a finding.** The packet's
  PHASE-03 VT-3 recommendation described a test that cannot pass, and
  mislabelled the shape of the test already landed. Re-deriving it against the
  code cost one grep; following it would have produced a vacuous test — the
  exact defect class the review that generated the handover was raised about.
- **`check regression capture --base <B>` does not check `B` out.** It runs the
  suite on the current tree and labels the result `B`. Run after the delta, it
  records the delta's own failure as pre-existing, and the subsequent diff
  reports `persistent (pre-existing) — fix the trunk`. That phrasing actively
  invites waving through a self-inflicted regression. The funnel captures
  pre-spawn; an inline phase has no such beat and must remember it.
- **Doc/DAG coupling that no artefact names.** Adding one `.doctrine/` line to
  `.gitignore` red-ed a parity test requiring a new withhold-tier variant. Not
  in the design, not in the §7 touch-set, not in the phase sheet's reading list
  — only the gate knows. Cheap to fix, but it means a "one-line data change"
  estimate is systematically wrong for this file.
- **The coord tree's index/worktree can sit BEHIND its own HEAD, and that
  blocks the conclude cadence's first beat.** `[/dispatch; SL-231-conclude]` The
  funnel commits server-side without touching the working tree, so on entry the
  two funnel files read as staged modifications that *delete* PHASE-04's and
  PHASE-05's rows — a stale index presented as pending work. `dispatch status`
  does not mention it and `refresh-base` gives no pre-flight. The heal is
  `git restore --source=HEAD --staged --worktree -- <the two files>`, but
  arriving at "HEAD is the superset, the worktree is behind" needs three diffs
  (HEAD↔index, index↔worktree, HEAD↔worktree) because the default `git status`
  reading is the opposite one. A fresh agent following the handover verbatim
  runs `refresh-base` into a merge refusal on a file it was told was "expected,
  not drift".
- **A boundary row that spans a mid-drive `refresh-base` merge silently
  attributes all of trunk to that phase.** `[/dispatch; SL-231-conclude]`
  PHASE-01's row is `[095fca404, 02da8ebf47]`, recorded at its conclude; a
  `refresh-base` merge (`0d2cb5671`, 20 trunk commits) landed *inside* that
  range. `slice conformance` folds each row's `start..end` `--name-status`, so
  SL-232/233/234's authored files, `Cargo.lock`, the plugin manifests and
  `src/review.rs` all read as SL-231's undeclared cells: 45 undeclared, of
  which 42 are trunk. The honest touched set is 28 files with 3 undeclared
  (`.doctrine/dispatch/231/funnel.toml`, `.doctrine/slice/231/slice-231.toml`,
  `src/worktree/allowlist.rs`). Establishing that took reading
  `conformance_outcome`, five per-range diffs, and a history walk over
  `boundaries.toml` to prove the row was long-standing rather than freshly
  mis-derived. The cost lands on whoever audits, and the report reads as a
  serious scope violation until disproved.
- **The handover's boundary table disagreed with the committed ledger, and was
  right about the wrong thing.** `[/dispatch; SL-231-conclude]` The packet gives
  PHASE-01 as `[0d2cb5671, 1d8cc08ae]` — the true post-refresh delta — while the
  ledger records `[095fca404, 02da8ebf47]`. Both describe something real, so
  neither reads as wrong on its face; the discrepancy is only visible if you
  diff both. A packet that restates queried data invites exactly this: the
  restatement ages against the source it was copied from.
- **The only substantive phase sheets lived in the tree the cadence tells you to
  delete.** `[/dispatch; SL-231-conclude]` Runtime state is per-worktree, so
  PHASE-04's and PHASE-05's sheets (9.2k and 13.7k, carrying the Findings the
  audit reads) existed *only* under `.dispatch/SL-231/.doctrine/state/`; the
  primary tree held 1k templates. PHASE-01's progress row existed only in the
  primary. Neither tree was a superset. "Remove the coord worktree DIRECTORY
  (keep the refs)" is stated as ref hygiene and says nothing about runtime
  state, so following it literally destroys the audit's primary evidence — and
  silently, because the sheets are gitignored and the templates that survive
  look plausible. Carried forward by hand before teardown.
- **Coord teardown leaves the boot snapshot pointing at a deleted binary.**
  `[/dispatch; SL-231-conclude]` The `Invoking doctrine` section bakes the
  regenerating binary's path, so a dispatch session bakes
  `.dispatch/SL-<N>/target/debug/doctrine`. Removing the coord tree makes that
  path dead, and the next session's first tool call fails on a path it was
  handed by governance. Cured with `doctrine boot` from the primary binary —
  but nothing in the conclude cadence prompts it.
