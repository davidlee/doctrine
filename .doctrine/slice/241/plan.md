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
