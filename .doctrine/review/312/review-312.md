# Review RV-312 — reconciliation of SL-228

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode:** conformance. **Surface reviewed:** the `dispatch/228` coordination branch
in the coord worktree `/workspace/doctrine/.dispatch/SL-228` (tip `b69444c8` at open),
plus the prepared `review/228` + 8 `phase/228-NN` evidence refs. No candidate
interaction branch was published for this slice (`dispatch status` → `candidates: 0`),
so the coordination branch is the reviewed surface and is recorded as such per F-2 of
the ledger protocol.

**Lines of attack.**

1. **Does the funnel hold its own invariants under concurrency?** SL-228's central
   claim is claim→bind→act: a live fork implies its binding, by construction. That is
   a concurrency property, so probe it with the suite's own concurrency tests run
   repeatedly rather than once — a single green run proves nothing about a race.
2. **Does the mechanical conformance algebra agree with the narrative?** Run
   `slice conformance` and account for EVERY cell. An unexplained undeclared or
   undelivered entry is a lead, never a verdict; the goal is to reduce the report to
   findings whose cause is named.
3. **Does the close ritual itself execute?** `dispatch sync --prepare-review` had
   never been run for this slice. A ritual documented but unexercised is untested,
   so treat its first execution as evidence-gathering, not a formality.
4. **Do the drive's accumulated reconcile-queue claims survive checking?** The
   handover carries assertions written mid-drive. Verify each against the artefacts
   rather than importing them; a handover is testimony, not evidence.
5. **Does D10 / FR-009 survive its own benchmark?** VH-1 accepted benchmark.md's
   verdict, which already concedes the refusal half. Hold the design to that accepted
   record rather than re-litigating it.

**Invariants pinned:** claim→bind→act (design §3); `land_funnel_transition` as the
sole funnel writer with `splice_record` module-private (I1); `position=None`
byte-identical in `derive_receipt_status` (R1); ADR-001 layering, no `worktree →
dispatch` back-cycle; the `.doctrine/` hard wall as a wall, not a declarable scope.

## Synthesis

SL-228 set out to make a dispatch drive survivable without rescue: a pure state
machine, a sole writer, gates on every write verb, an oracle that names the next
command, and a memory-blind benchmark to find out whether any of it works for an
agent who knows nothing. Nine phases landed. The gate is green — 4011 tests, clippy
clean — `verify-vt` is 25/25, and the benchmark's own subject drove a full funnel and
a crash recovery with zero rescue and zero memory access. The slice does what it
said it would do.

The audit's substantive result is that it found the one thing the drive could not
see about itself.

**The blocker (F-1) was a hole in the slice's flagship invariant, and it hid inside a
passing suite.** PHASE-04's claim→bind→act is the design's strongest promise: a live
fork implies its binding. The implementation consumed the arming *before* acquiring
the claim lock, six lines above a comment asserting the lock covered the whole
window. Under a same-name race, one spawn takes the binding and refuses while the
other forks unbound — and an unbound fork is not a cosmetic defect, it is
`unprovable-fork`, discovered only after a full worker run is spent, with no re-bind
verb. This session had already hit that dead end once and attributed it entirely to
operator error (a spawn armed `--slice` without `--phase`). It was not only operator
error; there was a silent second route.

What makes this worth stating rather than just fixing: the test that proves it was
**already in the suite**, written by the phase that shipped the bug, and it failed
2 runs in 20. A single `check gate` returned exit 0; the immediate re-run returned
101. `slice verify-vt 228` had reported PHASE-04 VT-5 as PASS, and that PASS was
luck. A 10% flake is not a nuisance to be re-run until green — here it was the
invariant reporting its own violation, and the only reason it was caught is that the
gate happened to be run twice. Fixed in `707236c0` by moving both consumes under the
claim; 40/40 after, suite 4011/0.

**The conformance report was mostly instrument error, and saying so precisely is the
point.** Of 8 undelivered cells, 3 were delivered in PHASE-03's base commit and are
structurally invisible because a `[start, end]` range excludes its own start (F-2);
3 name `.agents/skills/**`, the untracked install projection, while the real source
under `plugins/` is already conformant (F-3); only 2 are genuine (F-4). All 6
undeclared cells are `.doctrine/**` authored metadata — zero source scope-creep.
Reduced to its residue, SL-228's conformance debt is two stale selector
declarations. That number is only trustworthy because every other cell has a named
cause; an auditor who reported "8 undelivered" would have been accurate and
useless.

**The close ritual was itself untested, and failed twice on first contact (F-6).**
`prepare-review` had never been run for this slice. It refused once for PHASE-08/09
(their sheets can never reach the primary tree — a warning the drive had recorded as
benign, which hard-blocks close) and once for PHASE-07 (evidence-only, and the gate
has no exemption for a phase whose deliverables are documents). Both refusals
prescribed `record-delta`; for the first, that advice was simply wrong. A ritual
written down but never executed is an untested code path, and this one had two
defects in nine lines of prescription.

**Standing risks, consciously accepted.**

- *The claude arm remains unexercised by the benchmark* (stated limit 3, accepted at
  VH-1). The 15 claude-arm memories in `oq6-retirement.md` Tier B2 stay held, and
  `oq6-retirement.md` remains a draft list — VH-1 accepted a benchmark, not a
  retirement. Nothing has been retired.
- *n=1, on a two-phase markdown fixture.* The scenarios carry the difficulty, not the
  substrate. No statistical claim is made or implied.
- *F-2 and F-6's residues ship unfixed*, as IMP-292 defect 3, ISS-254, and the IDE-028
  refinement. These are conformance-engine and gate defects that every base-beat
  slice inherits identically; fixing them inside a reconciliation gate would smuggle
  a platform change through a slice close, which POL-002 exists to prevent.
- *D10 / FR-009 ships refuted in its strong form* (F-5) and is delegated to a REV with
  IMP-321 as the named verification vehicle. The counter-example set is now four —
  ISS-250's destructive remedy, the benchmark's empty-`detail` `stale-record`,
  PHASE-09's bare-branch-name `unprovable-fork`, and ISS-254's two
  same-text-different-cause completeness refusals. The mechanical root is located
  (24 empty-detail construction sites, 14 in `src/mcp_server/dispatch.rs`) and
  deliberately not fixed here.

**The tradeoff I want on the record.** This audit fixed exactly one thing and
deferred six. That asymmetry is deliberate: F-1 was a correctness hole in *this
slice's own* shipped invariant, provable by *this slice's own* test, closable in two
lines. Everything else is either a defect in shared machinery that SL-228 merely
revealed, or a truth-gap in prose that `/reconcile` is the sanctioned writer for.
The temptation at a close is to tidy everything within reach; the discipline is to
fix what this slice broke and route the rest to owners who can fix it properly.

**One methodological note for whoever audits the next dispatched slice.** Run the
gate more than once. Concurrency invariants do not fail on schedule, and a suite that
passes once has told you almost nothing about a design whose central claim is about
races.

## Reconciliation Brief

Every non-`aligned`, non-`tolerated` finding that touches design or governance,
grouped by the surface `/reconcile` will actually write. F-1 is absent by design: it
was dispositioned `fix-now` and is already landed in `707236c0`. F-2 and F-6 are
absent: dispositioned `follow-up`, owned by IMP-292 / ISS-254 / IDE-028, with no
reconcile write.

### Per-slice (direct edit)

- **`slice-228.toml` selector registry — F-3.** The load-bearing change; prose alone
  leaves conformance red. `doctrine slice selector rm` these four design-targets:
  `.agents/skills/dispatch/**`, `.agents/skills/dispatch-agent/**`,
  `.agents/skills/dispatch-subprocess/**` (all name the untracked install projection;
  the real source under `plugins/doctrine/skills/**` is delivered and already
  conformant under its own selectors), and `.doctrine/spec/tech/021/**` (unsatisfiable
  by construction — `classify_import`, `src/worktree/import.rs:146-153`, returns
  `doctrine-touch` before the selector leg, so a `.doctrine/` design-target reads as
  permission where the wall forbids writing).
- **`slice-228.toml` selector registry — F-4.** `doctrine slice selector rm`
  `flake.nix` and `src/worktree/shared.rs`: declared, never touched, not load-bearing
  for any exit criterion. Withdraw the declaration rather than manufacture work to
  satisfy it.
- **`design.md` §6 — F-3, F-4.** Mirror both selector removals in the prose listing.
  This is the human mirror of the registry change above, not a substitute for it.
- **`design.md` §2 — F-7 item 2.** Acknowledge the act/replay discriminator:
  `attempt` returns `Result<Position, _>` and cannot tell the sole writer act from
  replay, so PHASE-03 added `attempt_advance` as the real authority, keeping `attempt`
  verbatim as a thin projection with a test pinning that the two never disagree.
  Additive — the existing signature is incomplete, not wrong.
- **`design.md` §5 — F-7 item 3.** State the `paths_since_verify: None` case and that
  it fails closed as `conclude-verify-stale`.
- **`design.md` D7 — F-7 items 1 and 4.** (a) Soften the generate-then-commit
  implication and name its cause: the `.doctrine/` hard wall inverts the authoring
  direction, so the golden under `.doctrine/spec/tech/021/` is authored first and the
  renderer is pinned to it by a golden test — same wall as F-3, same cause as F-2's
  base beat. (b) Widen the `already-<position>` gloss, which is narrower than the
  code: it also covers backward attempts.

**Off-surface, deliberately excluded:** no `plan.toml` edit appears above. `EN-` /
`EX-` / `VT-` / `PHASE-NN` ids are immutable-append and are not a reconcile
direct-edit surface. Nothing in this audit requires changing a plan criterion.

### Governance/spec (REV)

- **D10 / FR-009 — F-5 → REV modify.** Split the claim rather than delete it. `next`
  retains the forcing-function claim for **positional** guidance — proven, a
  memory-blind orchestrator drove a full funnel and a crash recovery unaided. The
  clause "a refusal's text IS the recovery procedure" demotes from claimed property
  to stated goal with a named verification vehicle (**IMP-321**, already sequenced
  `after: SL-228` with `references(originates_from): SL-228`). Cite the four
  counter-examples; do not re-derive them, and do not re-open VH-1, which accepted
  this verdict on 2026-07-27.
- **`.doctrine/spec/tech/021/` — F-5.** Wherever the tech spec restates D10's strong
  form, apply the same split. The D7 golden itself is unaffected: it is pinned by a
  passing golden test and is correct as authored.

**Requirement status.** REQ-387 may remain `pending` at close — subprocess-arm full
gating was deferred by design, not missed. Do not mark it satisfied to tidy the
rollup.
