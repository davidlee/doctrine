# IMP-321: Verify the advice surface: refusals and prescriptions against the machine

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The claim that is not verified

SL-228 design D10 / FR-009 states that **a refusal's text IS the recovery
procedure**. Everything else in doctrine that load-bears is verified — VT criteria
pin tests, the D7 golden pins the rendered funnel guidance — but the *imperative
content* of what the tooling tells an operator has no gate at all. Three
independent observations now contradict the claim, all found by running the
thing rather than by reading it.

## The evidence

**From the PHASE-07 memory-blind benchmark** (`.doctrine/slice/228/benchmark.md`,
raw transcripts in the evidence archive; the situations ledger derived from them
lists every friction event the blind subject met):

- `worker_commit` → `{"Refused":{"reason":"stale-record","detail":""}}` — an
  **empty** detail. The subject then read `dispatch_record.rs`, `worker_commit.rs`
  and `fork.rs` to work out what it meant (ledger s4b:69→70,80,81).
- ISS-250: a refusal named the verb that produced it, and the remedy the operator
  drew from it destroyed the unimported commit the guard had just protected. This
  is the severe case — advice that is not merely unhelpful but destructive.
- `dispatch commit` → "the coord pre-commit hook refused, or git errored — See the
  hook's message above", in the ISS-249 case where the hook printed **nothing**.
  Advice pointing at absent output.

**From the SL-228 PHASE-09 drive** (2026-07-27):

- `worker_commit` → `unprovable-fork`, detail = the branch name only. Names
  neither the cause (the spawn was armed `--slice` without `--phase`, so the fork
  carried no binding) nor a fixing verb — and no re-bind verb exists to find. A
  full worker run was spent before it surfaced. See
  `mem.pattern.dispatch.half-arm-unprovable-fork`.
- `slice phase --status completed` → "mirroring completion into the primary tree
  **failed**" for an appended phase, where the primary tree structurally cannot
  carry that sheet (its `plan.toml` predates the append). A structurally expected
  condition rendered as a failure.
- `dispatch next` prescribes `spawn — PHASE-07` for a phase deliberately not
  funnel-driven: the oracle cannot distinguish "not a dispatch phase" from "not
  started", and answers the only question it can.
- ISS-251: `selector doctor` reports green for a path named only in objective/EX
  text — an advisory that says "fine" when it is not.

**From the code** (the mechanical root of the first failure mode):

- **24 sites construct a refusal with a structurally empty detail** —
  `refused(refusal.token(), String::new())` and friends. **14 of them are in
  `src/mcp_server/dispatch.rs`**, the funnel tools every dispatch drive runs
  through; the rest in `src/dispatch.rs` and `src/mcp_server/worker_commit.rs`.
  The token carries the diagnosis; the remedy field is left empty by construction.

Rate, with its caveat: of roughly a dozen advice events actually encountered in
anger across the benchmark and this drive, about five misled or under-served. That
is a rate over advice the operator *hit*, not over all advice sites — the ledger
is a friction filter and over-samples trouble by design. It is still the number
that matters to an operator.

## The taxonomy (what an audit would be looking for)

- **A — contentless detail.** Reason token present, remedy field empty. Greppable.
- **B — self-referential or destructive remedy.** The text names the verb that
  produced it, or walks the operator around the guard (ISS-250).
- **C — advice pointing at output that does not exist** ("see the message above").
- **D — prescription from an incomplete model.** `next` conflating
  not-funnel-driven with unstarted; `selector doctor`'s blind spot (ISS-251).
- **E — expected condition rendered as failure.** The appended-phase mirror warning.

## Recommended shape — a gate first, an audit second

**A and B are a lint, not an audit.** They are mechanically checkable and, once
checked, cannot regrow:

- no refusal variant may construct an empty `detail`;
- no `detail` may name the verb that produced it (this VT is *already drafted* in
  SL-228's reconcile queue against ISS-250).

That converts a 24-site sweep into an invariant and is the cheapest large win.
C is a narrower version of the same idea (do not promise output you did not emit).

**D and E are the part that genuinely needs judgement** and is worth a real audit:
they are cases where the advice is faithful to a model that is itself incomplete.
Fixing them means either widening the model (give the oracle a way to know a phase
is not funnel-driven) or making the advice honest about its own limits.

## Sequencing

**After SL-228 closes.** Same cluster, and SL-228's reconcile queue already carries
the D10 revision this would feed — folding it in now is scope creep on a slice at
its terminal gate. The D10 repair should probably land the caller-relative arm
(single-source the reason *token*, not the *remedy*, because the correct remedy
differs between the CLI and MCP callers) and this item then verifies it.

Related: ISS-249, ISS-250, ISS-251, ISS-253, SL-228 D10/FR-009.
