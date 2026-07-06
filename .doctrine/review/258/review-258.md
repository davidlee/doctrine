# Review RV-258 — design of SL-206

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Target.** `design.md` §5 (the unjail model), post two topology corrections +
internal adversarial pass (`b975e471`) + three refinements (`1cb9de68`); scope
reconciled (`9a0fa540`). Single facet: **design** aspect. Posture: **inquisitor**.

**Sanctioned doctrine held over the accused.** ADR-006 (orchestrator sole writer),
ADR-008 (jail build isolation + worker confinement — the amendment under trial),
ADR-011 (harness-agnostic spawn), ADR-012 (dispatch topology), POL-002 (platform
independence), STD-001 (no magic strings). Evidence corpus: `unjail-direction.md`
§5/§6 (P0/P1/P3/P4), `mem.fact.dispatch.unjail-nomination-round-trip`,
`mem.fact.claude.pretooluse-agent-carries-spawner-id`, CHR-039 findings.

**Lines of interrogation.**
1. **§5.6 security — is escalation truly closed?** I1 claims "one list, EVERY
   spawn seam" and names a *second* seam (the `Workflow` tool). Is the coverage
   complete, or does a third seam (`/fork`, MCP-initiated spawn, a future harness
   surface) escape? Is the I1 doctor check (nomination ⊆ gate) actually sufficient,
   or does it check the wrong direction? I2 (config outside every jail) — is the
   FIXED out-of-jail path genuinely unreachable by a jailed worker on BOTH arms?
   Fail-safe — does *every* failure mode truly land on more confinement, or is
   there a partial-nomination / hot-reload race that grants PassThrough?
2. **§5.1/§5.4 topology — does the two-job HopReceipt loop actually terminate and
   never lose work?** `fixup` XOR `prep` oneOf — does the loop handle the
   neither-set (halted) case on every branch? Is `PREP_INCOMPLETE` (A1) a real belt
   or a paper one? Can a moved `dispatch_tip` between dispose and prep corrupt the
   next base_B?
3. **C1 revive — is the fork-durable claim sound?** Mechanism 2 (worker-side
   `git reset --hard <fork_tip>`) — does the R-5 belt + O-supplied target really
   contain a worker touching refs? Can a revive on the wrong base silently rebase
   the delta?
4. **(B) self-commit P5 residual** — is authoring (B) as *target* with (A) as
   fallback honest, or does an unproven P5 make (B) load-bearing on a guess?
5. **ADR-008 amendment ledger** — is the escape-hatch + seam-symmetry framing a
   real reversibility guarantee, or rationalization? Does the amendment actually
   belong in *this* slice's scope?
6. **§9 verification** — do the named tests actually pin the invariants they claim?
   Is the phase-0 e2e a genuine de-risk or a demo? Selector accuracy (design-target
   vs scope-relevant) post the new touch-set.

The interrogation is delegated to an external adversary (GPT-5.5, codex) writable
against this ledger; the Inquisitor enters and disposes the charges.

## Synthesis

**Verdict: the design STANDS, its overclaims BURNED.** The accused was put to the
question by an external adversary and confessed five taints — one mortal
(`blocker`), four grievous (`major`). Not one struck the architecture: the unjail
topology, the two-job hop loop, the nomination+spawn-gate — all held under
cross-examination. Every charge was an **overclaim or an unguarded seam**, and
every one is now cauterised. Let the record show the penance, in the order it was
exacted (all in commit `e6b60ced`):

1. **F-1 (blocker) — the false gospel of "provably safe."** §5.6 preached blanket
   safety while a second spawn seam — the `Workflow` tool — stood **ungated and
   unproven** (OQ-4). A boundary declared closed on a seam never tried is heresy
   most foul. **Penance:** proof is now confessed **per-seam** — the `Agent` seam
   proven closed (P1/P3/P4); the `Workflow` seam sealed by a **fail-safe blanket
   subagent-deny** until OQ-4 earns it the finer identity-gate. The I1 doctor check
   now also demands **a matcher for every seam in the registry** — for
   nomination ⊆ gate is *necessary, not sufficient*, and a gate that does not cover
   a seam is no gate at all. The mortal taint is lifted; the close-gate is clear.

2. **F-5 (major) — the "guarantee" that was but a whispered hope.** The revive floor
   — a raw `git reset --hard` in the hands of a jailed worker — was crowned
   "guaranteed" while resting on nothing but the worker's obedience to a prompt. The
   adversary produced the receipts: `worker_commit.rs:214-271` holds the only *proven*
   base invariants, and this fallback bypassed them entirely, in a repo already
   scarred by a documented wrong-base trap. **Penance:** the revive now commits
   **through `worker_commit`** with its base **re-stamped to `fork_tip`** —
   wrong-base rejected by machinery, not manners — with a disposing-O parent-chain
   verify (`coord:revive-wrong-base`) on the import path, and a §9 test to pin it.

3. **F-2 (major) — the target path taken on faith.** Handoff mode (B) was authored as
   *the* claude-arm path while P5 sat unproven and §9 never demanded proof it ran.
   **Penance:** phase-0 must now **confess which mode ran** — a true `worker_commit`
   with non-null `fork_tip`, or an explicit recorded fall-to-(A). A silent degrade is
   a phase-0 **failure**.

4. **F-4 (major) — the central relic left outside the reliquary.** `/drive-slice` — a
   primary deliverable — had no committed home while the shipped POC still bore the
   **retired** confined model. **Penance:** OQ-1 **resolved** —
   `install/workflows/drive-slice.js`, a **design-target**, so conformance shall red
   the stale POC until the implementer overwrites it in the flesh.

5. **F-3 (major) — WITHDRAWN.** A shell-damaged citation, superseded verbatim by F-5.
   Raised in error, struck from the roll — not disposed, retracted.

**Standing risks, consciously borne.** OQ-4 (Workflow-seam identity carriage), OQ-6
(revive-base hook retargeting), and P5 (worker-leaf MCP retention) remain
**unproven** — but each is now closed by a **fail-safe default** (deny / worker-side
enforced base / recorded fall-to-A), so the design is safe *by construction* while
they are pinned in phase-0. This is deferral with a belt, not negligence.

**One clerical debt flagged, not charged here.** The slice is `ready` — a `plan.toml`
predates the §5 unjail rewrite. The plan must be reconciled against the current
design before execution; that is a `/plan` matter, out of this design tribunal's
aspect (one RV, one aspect).

The accused is scourged and made honest. Let it proceed to planning under watch.

> **HERESIS URITOR; DOCTRINA MANET**
