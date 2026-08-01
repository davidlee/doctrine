# Implementation Plan SL-241: Capsule spike rig

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Six phases in two halves. PHASE-01 through PHASE-03 **build** the rig;
PHASE-04 through PHASE-06 **run** it and bank the evidence.

The split is not stylistic. `probe-specs.md` § Order and gating rules that a
failed probe row is a **finding**, never a quiet rig edit — and that rule only
has force if building and running are separable activities. A phase that both
builds a mechanism and scores rows against it can always relieve the pressure
by adjusting the mechanism, and nothing in the artifact would show it happened.
So the build phases exit on "the mechanism exists and is correct", and the
evidence phases exit on "every row has a recorded outcome". Different exit
shapes, different phases.

The deliverable is evidence, not product (slice § Scope). Nothing in the rig
migrates into dispatch machinery, and no `src/` change is in scope by default.

## Sequencing & Rationale

### Why R2 is probed in PHASE-01, before anything depends on it

The conform stage's scope leg is `doctrine slice conformance <id> --against
B..S --strict` (design § 5.2). R2 is the residual risk that its `--strict`
semantics diverge from the import belt's at some edge. If that divergence is
real, it is a `/consult` and not an improvised `src/` change — and the moment
to discover it is before the pipeline is written on top of it, not during
PHASE-03 when the cost of learning it is a rewrite. PHASE-01 therefore probes
the verb against a fixture and answers R2 in writing, and the answer is an
entrance criterion of PHASE-03 (EN-2).

The verb was confirmed present at planning time, along with the `-p <root>`
flag the rig needs to point it at a fixture rather than at this repository.
What PHASE-01 settles is not existence but semantics.

### Why A2 is proved in PHASE-02 and not deferred to PHASE-06

A2 — that headless `claude -p` authenticates inside the capsule sandbox — is
the assumption whose failure is most expensive to learn late. If credentials do
not survive nested bwrap, the capsule model needs a credential-proxy design,
and that is a design-level consequence, not an implementation detail. R3
mitigates by testing it on day one as a near-free standalone smoke, decoupled
from P-C1b. So the smoke sits in PHASE-02 with the sandbox that carries it,
while the expensive agent run (P-C1b) stays in PHASE-06 where its cost is
justified by the token measurement.

The smoke is **two** assertions (design § 5.4, internal finding A8): network
reachability unauthenticated, then an authenticated print. Credential
availability and network egress are distinct failure modes, and one test that
conflates them tells you only that something is wrong.

### Why PHASE-03 ends on the rig's own red/green

R4 is that the rig lies: a hostile row "passing" because the rig is broken is
indistinguishable from a real kill. The happy-path self-test on the light
fixture is the answer, and it is a precondition rather than a nicety — until it
is green, every subsequent "refused" is uninterpretable. It is therefore the
last exit criterion of the last build phase and the entrance criterion of the
first evidence phase, which is the only placement that makes it load-bearing.

The same logic runs through the verification rows: the positive controls in
PHASE-02 (I4a), PHASE-04 (both audits) and PHASE-05 (per-cell `planted?`) all
exist because a negative result without a positive control proves only that the
check ran. That principle is applied at every altitude where a negative is
claimed, not just where § 9 spelled it out.

### Why step 0 leads PHASE-04

Step 0 — enumerating the TypeScript/npm ecosystem's interpretation triggers
*without consulting CPT-001*, then classifying the result against it — is the
only mechanism in the rig capable of **falsifying** an exhaustiveness claim.
Every other mechanism is confirmatory: the sixteen rows instantiate triggers
that are already classified, and the DQ-4 audit greps for tokens the
declaration already names. A trigger that no class describes is invisible to
both of them.

The ordering constraint is real and easy to lose: step 0 must run *before* any
light-fixture row is instantiated, or the classification can be back-fitted to
the rows it was supposed to test. Hence EX-3 states the ordering as a criterion
rather than leaving it to the phase's narrative order.

Its output amends a knowledge record now; the alternative is amending shipped
enforcement later.

### Why PHASE-05 is alone

The hostile matrix is sixteen rows across two mechanisms with per-cell positive
controls, a two-harness split on two of them, five guard probes, and a fixture
variant built solely to manufacture an exposure that the rig as drawn does not
have. It wants its own context, and it is the phase where the portability claim
is actually made — so it is also where the scoring discipline has to hold
under the most pressure.

Two scoring rules carry that weight, and both are exit criteria rather than
prose: `n/a` is a legal recorded outcome and a silent pass is not; and `n/a`
cells are *excluded* from the altitude computation rather than counted as
holds. Without the second rule a cell that was never attempted computes as
"holds under both" and stamps its row `model-level` — the strongest claim the
table can make — on a probe that never ran.

`matrix.tsv`'s source of truth is design § 5.6, not `probe-specs.md`. The
inherited expected-kill column was authored against a pipeline tail this design
removes, and five of its rows name machinery that is deleted or never
specified. Scoring against it would produce results that are properties of the
matrix bookkeeping rather than of the model under test. RFC-025 prose is not
edited to fix this — that is a slice non-goal and a separate cleanup pass.

### Why the go/no-go's scope is a verification target

PHASE-06's VT-1 greps the go/no-go artifact for its own qualifying language.
That is an unusual thing to assert mechanically, and it is deliberate: § 9
identifies over-claiming as the specific failure mode that would damage the
downstream REV, and the qualifiers are the only defence. A verdict that reads
"16/16" would be wrong in three separate ways at once — it would count two
incumbent regression legs as capsule-model coverage, imply ASM-007 was
discharged when it can only be strengthened, and bury the fact that
conflict/staleness *resolution* (QUE-202) is out of evidence entirely.

VH-1 asks the operator to accept the verdict **and** its scope together, for
the same reason. Accepting a verdict without its scope is precisely what the
REV would then over-claim from.

## Notes

### What the plan's critical pass changed

The first draft of this plan was stressed against the design and the actual
environment before the phase sheets were cut. Five things moved:

1. **`nix` and `direnv` are absent in the jail.** `/nix/store`, `bwrap`,
   `node`, `npm` and `claude` are present; the two provisioning binaries
   probe-specs § P-C1 step 2 names are not. So provisioning is a read-only bind
   of the store plus PATH — which is what the P-C2 profile already specifies —
   and P-C1a's "nix env ready" step is recorded `n/a` with its reason rather
   than quietly dropped from the step list (PHASE-02 EX-8, PHASE-04 EX-4). The
   same `n/a`-with-a-reason discipline the matrix uses, applied to a
   measurement step.
2. **The verify-stage tokens were in the wrong place.** The draft asserted
   `verify-timeout` / `sandbox-failed` inside `capsule/verify.sh`, which
   contradicts I5 — refusals report *trusted-side-computed* tokens, and I4 puts
   the verdict in the parent's reading of an exit status. The tokens moved to
   PHASE-03's pipeline mandate; PHASE-02 now proves the *bound fires*
   (PHASE-02 VA-4), which is the part that belongs to the capsule side.
3. **The conflict sub-probe needs a fixture the design's fixture rules do not
   describe.** Design § 5.3's "no plan and no phases" is scoped to the
   pipeline, and is true because prepare-review's phase-completion gate is out
   of it. The sub-probe runs prepare-review explicitly, so it meets that gate.
   PHASE-05 EX-15 provisions a variant carrying a plan and phases up front,
   because discovering it mid-phase costs a rebuild.
4. **`doctrine install` into a non-Rust project is unproven.** The light
   fixture is the first thing in this repo to test POL-002's independence claim
   outside Rust. PHASE-01 EX-8 makes proving it a criterion; a failure there is
   a POL-002 finding, and it is much cheaper in PHASE-01 than in PHASE-04.
5. **Two VT keyword sets named functions that do not exist yet.** A mandate
   keyed on an invented helper name (`positive_control`) reds for a naming
   choice rather than a missing mechanism. The audit mandates now key on
   design-given or externally-forced strings — `SubagentStart`,
   `WorktreeCreate`, `worker_mode`, `interpretation-surface` — and the
   positive-control *behaviour* is verified by the VA rows, where it belongs.

### On the VT mandates' brittleness

The mandates bind to the layout in design § 9.1. If execution refactors — say
conform moves out of `pipeline.sh` into its own file — the mandate moves with
the code; that is a mandate edit, not a criterion failure. What must not happen
is the reverse: keeping the file and weakening the keywords until the gate
passes. The keywords chosen are the ones that encode findings the review rounds
paid for (`core.quotePath=false`, `--no-renames`, `160000`, `cas-lost`), so a
keyword that stops matching is worth reading as a signal before it is worth
editing.

### The research advisory is about an artifact that does not exist

`doctrine slice research SL-241` reports a drifted baseline. The baseline
stamps `slice-241.md` only; there is no `research.md` — the pre-design research
round was never run for this slice, and the design was authored directly
against the RFC-025 groundwork. The advisory is therefore not reporting a stale
research artifact but the absence of one, and it is deliberately **not**
restamped: silencing the nag would assert a currency that nothing backs. The
selectors were drafted from design § 9.1's code-impact table instead, which is
the authoritative statement of what this slice touches.

### Fixture sequencing, and the contamination guard

The fixture set the plan implies is five things, not one, so PHASE-01 EX-10
authors `fixtures.md` as a build sheet naming each with its delta and its
consuming criterion. It is a build sheet rather than a dependent slice
deliberately: the fixtures are disposable scaffolding deleted at spike end, a
slice would imply a lifecycle for something with no durable existence, and
design § 5.3 already settled the design questions (DEC-107, D5). What was
missing was concrete literals, not a decision.

PHASE-01 builds only the two *base* fixtures (EX-11). The in-repo-declaration
variant and the plan+phases variant are instantiated by PHASE-05, the phase
that consumes them — building them three phases early means maintaining
artifacts nothing yet exercises. They cannot be forgotten, because PHASE-05
EX-11 and EX-15 gate on them independently of the sheet.

The sequencing that actually needs care is the light fixture's declaration.
Authoring an `interpret:` list means reasoning about what TypeScript
auto-loads, evaluates, and execs — which *is* a trigger enumeration. If that
reasoning happens in PHASE-01 and then step 0 runs in PHASE-04, step 0 is not
an independent pass; it is a re-run of thinking already done, and it returns an
empty residue by construction. An empty residue is precisely what would be read
as ASM-007 surviving. So the guard is two-sided: PHASE-01 EX-12 authors the
declaration from the fixture's own build needs only (what does this project
exec to build and test), and PHASE-04 EX-9 runs step 0 in a context that did
not author it. The declaration is amended from the residue afterwards, never
before.

Design § 5.4 already protects step 0 from CPT-001. This adds the other side,
which the design did not need to state because the design does not schedule the
work — the plan does, and the plan is where the two activities land in an order
that can contaminate.

### One placement decision beyond the design's tree

Design § 9.1 sketches the rig's layout but does not enumerate a file for the
happy-path self-test, which § 9 nonetheless mandates. The plan places it at
`control/selftest.sh` with an entry on the `rig` dispatcher (PHASE-03 VT-4).
This is a placement choice consistent with the trusted-side layout, not a
design change; if execution finds a better home the criterion moves with it.

### Carried forward from the design's review history

Two habits the two review rounds paid for, recorded here because the plan is
where they get lost:

- **Amending a knowledge record means both tiers.** PHASE-04 EX-2 and PHASE-05
  EX-13 both amend or link records; verify through `doctrine knowledge show`,
  never the `.md` alone. A structured tier left empty makes a prose amendment
  half-invisible to anything that queries.
- **Check a repair against the finding's evidence, not its direction.** This
  design twice fixed a finding by overshooting it (§ 10.4). The same failure is
  available to a phase that receives a `/consult` ruling and implements more
  than the ruling licensed.

### Forward compatibility

RFC-023 will substantially revise plan-gate machinery. Operator ruling
(2026-08-01): adopt the current machinery as-is for this slice. Nothing in the
four-stage pipeline depends on plan-gate mechanics, so those revisions should
land orthogonally (`notes.md` § Forward compatibility).
