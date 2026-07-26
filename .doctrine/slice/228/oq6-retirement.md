# OQ-6 — the dispatch-memory retirement list (SL-228 PHASE-07 EX-3)

> **This is a DRAFT list, not an executed retirement.** EX-3 asks for the list;
> acting on it is a separate, reversible-in-principle act that should follow the
> holds recorded below. Nothing here has been retired.

## What was asked

Which dispatch memories do the verbs now obsolete? Each candidate judged against
**two** questions, not one:

1. **Q1 — does a verb now carry the fact?** A carrier must be *quoted* — a verb's
   output, its help text, or the shipped skill/mechanics prose.
2. **Q2 — did the memory-blind benchmark subject actually need it?** Answered from
   the PHASE-07 transcripts: `rediscovered` (hit it and had to work it out),
   `needed-and-handled` (hit it, the verbs carried it), or `never-arose`.

A memory whose fact the blind subject never needed *and* that a verb now carries
is a retire candidate. One it rediscovered the hard way is proof the fact is
load-bearing and uncarried — a keep, regardless of Q1.

## Method, and why it is auditable

- **Candidates:** 149, from `memory list | grep -iE
  "dispatch|funnel|worktree|worker|coord"`. 147 received a verdict (two items
  had no readable body).
- **Evidence base 1 — the situations ledger.** Derived from the raw stream-json
  in `evidence/phase-07-benchmark.tar.gz`: every action the blind subject took
  across all three rounds (176 dispatch-relevant), every friction event with the
  tool output verbatim, and its own reasoning passages. Steps are numbered per
  round, so every citation is checkable (`s0:NN`, `s4a:NN`, `s4b:NN`).
- **Evidence base 2 — the verb surface.** `dispatch next`'s help and live output,
  the D7 golden (`spec/tech/021/funnel-machine.md`), the `dispatch` and `worktree`
  CLI surfaces, and the four shipped prose artefacts PHASE-06 rewrote to the
  next-loop.
- **Triage:** delegated per-memory to `deepseek/deepseek-v4-pro` in 13 batches,
  each given both evidence bases plus its batch of full memory bodies, and
  required to answer Q1 and Q2 *separately with citations*, defaulting to KEEP
  when uncertain. **Adjudicated here** — every RETIRE was re-checked against the
  rule and the arm question below.
- **Raw output:** `verdicts.tsv` (uid, key, verdict, q1, q2, confidence).

Two delegation defects were caught and are worth carrying:

- **Batch 11 truncated silently** — 3 verdicts for 12 memories, no error. Worse,
  it *degraded before it died*: two of its pre-truncation verdicts were RETIREs
  whose own `q1` field said "no carrier found", contradicting the stated rule. On
  re-run with an explicit arity instruction, both came back correctly as KEEP.
  **Verify the arity of delegated output, not just its content.**
- One rule violation survives in the final set and is rejected below.

## The headline finding: `never-arose` is not evidence here

**28 of the 35 proposed retires rest on `never-arose`, and 15 of those 28 are
claude-arm / harness-specific facts.** The benchmark measured the **subprocess
arm** (the clone lacked untracked `.claude/`, so the router took the other path —
`benchmark.md` stated limit 3). For a claude-arm memory, "the situation never
arose" is therefore a *tautology of the harness*, not a finding about the memory.

This session supplies the direct counter-example. Driving PHASE-09 on the **claude
arm**, the orchestrator relied on exactly the facts several Tier-B entries carry —
the positional cwd discriminator, the explicit-base arming — and then hit a
provisioning trap the verbs did **not** carry: a spawn armed `--slice` without
`--phase` binds nothing, and `worker_commit` refused `unprovable-fork` only after
a full worker run (`mem.pattern.dispatch.half-arm-unprovable-fork`, ISS-321
evidence). A retirement pass that trusted `never-arose` would have thinned the
corpus for the arm that was never on trial, in the same week that arm produced a
new footgun.

## Tier A — retire (7): carrier quoted, and exercised in anger

Q1 `yes` **and** Q2 `needed-and-handled` — the blind subject hit the situation and
the verbs carried it through without friction. This is the only tier with
operational proof of obsolescence.

- `mem.pattern.dispatch.prepare-review-gate-couples-to-phase-completion`
- `mem.pattern.dispatch.pi-arm-worker-ops`
- `mem.pattern.pi.rpc-stdin-lifecycle`
- `mem.pattern.dispatch.landed-oracle-needs-import-receipt`
- `mem.pattern.dispatch.fork-rung3-base-not-session-head`
- `mem_019ee5f4900275339de3602badd7c5e9` (keyless — pi spawn fifo/keepalive)
- `mem_019ede16728471d39ec92b052b42d9a0` (keyless — subprocess tool allowlist)

Note the shape: all subprocess-arm or arm-neutral. That is what the benchmark
actually tested, and the tier is honest about it.

## Tier B1 — retire on carrier strength alone (13): arm-neutral, never arose

Q1 `yes`, Q2 `never-arose`, and the fact is **not** claude-arm-specific — mostly
close/integrate-time knowledge the benchmark stops short of (it ends at
`prepare-review`). The carriers are real; the operational proof is absent because
the benchmark never reached that phase, not because of the arm. Retire at lower
confidence, or defer to a close-time exercise.

- `mem.pattern.dispatch.confined-orchestrator-placement-not-permission`
- `mem.fact.dispatch.confined-orchestrator-driveloop-realizable`
- `mem.pattern.doctrine.dispatch-phase-status-per-tree-split-brain`
- `mem.pattern.dispatch.close-preff-trunk-absorbs-repair`
- `mem.pattern.dispatch.prepare-review-rerun-not-idempotent-until-gate`
- `mem.pattern.audit.dispatched-phase-green-but-incomplete`
- `mem.pattern.dispatch.integrate-clean-trunk-or-phantom`
- `mem.pattern.dispatch.close-integrate-shared-trunk-race`
- `mem.pattern.dispatch.split-lineage-close-conflict-direct-land`
- `mem.pattern.dispatch.glob-add-sweeps-foreign-untracked-on-shared-main`
- `mem.pattern.dispatch.gc-squash-indistinguishable-from-unlanded`
- `mem.pattern.dispatch.reanchor-base-on-disjoint-head-move`
- `mem.pattern.dispatch.authoring-entities-not-dispatchable`

⚠ `dispatch-phase-status-per-tree-split-brain` deserves a second look before
retirement: PHASE-09 hit a live instance of per-tree phase-status divergence (the
primary-tree mirror cannot carry a mid-drive appended phase). The carrier may be
narrower than the fact.

## Tier B2 — HOLD (15): claude-arm facts the benchmark never put on trial

Q1 `yes`, Q2 `never-arose` — but `never-arose` here means *the arm was not
measured*. **Do not retire on this evidence.** Re-judge after a claude-arm
benchmark round with `.claude/` present in the clone (the remedy `benchmark.md`
already names for stated limit 3, ~$10 / ~1 hour).

- `mem.fact.workflow.isolated-fork-reaches-doctrine-mcp`
- `mem.fact.workflow.confined-fork-mints-at-armed-base`
- `mem.fact.dispatch.claude-arm-spawn-needs-isolation-worktree-flag`
- `mem.fact.dispatch.claude-fork-path-persist-footer-proven`
- `mem.pattern.dispatch.claude-worktree-subagent-bwrap-confinement`
- `mem.fact.claude.worktree-remove-auto-teardown`
- `mem.fact.dispatch.single-slot-arming-rendezvous`
- `mem.fact.claude.subagentstop-awaited-tree-intact-capture-seam`
- `mem.fact.dispatch.worktreecreate-cwd-channel`
- `mem.pattern.dispatch.claude-arm-isolation-fallback`
- `mem.pattern.dispatch.worktreecreate-replace-base-control`
- `mem.signpost.doctrine.dispatch-claude-arm-wrong-base`
- `mem.pattern.dispatch.claude-agent-worktree-integrates-commit-onto-parent`
- `mem.pattern.dispatch.claude-isolation-worktree-forks-orchestrator-session-head`
- `mem.pattern.dispatch.agent-worktree-forks-bash-cwd-head`

## Tier C — retire REJECTED on adjudication (1)

- `mem.fact.claude.subagentstop-awaited-tree-intact-capture-seam` — the verdict was
  RETIRE while its own `q1` read **"no — no carrier found"**, violating the stated
  rule (no carrier, no retire). Its actual argument was *supersession* ("the
  SubagentStop capture seam is a superseded design"), which may well be true but is
  a different claim requiring different evidence. **Kept**, and also listed in B2
  above pending the claude-arm round. Re-judge as a supersession question, not a
  coverage question.

## Amend (2)

- `mem.fact.dispatch.prepare-review-reads-primary-phase-status`
- `mem.pattern.dispatch.claude-arm-coord-placement`

Both are partly carried; see their `why` fields in `verdicts.tsv` for the
cut/keep split.

## Keep (110)

Of which ~28 were marked *out-of-scope* — memories that merely mention dispatch
while belonging to another subsystem (close/integrate lineage, audit surface, git
plumbing, jail infrastructure, frontend). Those were never OQ-6 candidates and the
grep-based candidate selection is what swept them in; a future pass should scope
by `mem.*.dispatch.*` plus explicit tags rather than a body grep.

## Recommended disposition

1. **Retire Tier A now** (7). Operationally proven.
2. **Retire Tier B1** (13) if you accept carrier-strength without operational
   proof — minus the flagged `split-brain` entry, which PHASE-09 contradicted.
3. **Hold Tier B2** (15) until a claude-arm round exists. This is the whole
   value of insisting on two questions instead of one.
4. **Amend 2**, **reject 1**.

Net: **20 of 149 retirable today** (13%), not the 35 (23%) the raw triage
proposed. The gap is the arm.
