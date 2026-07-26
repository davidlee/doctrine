
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
