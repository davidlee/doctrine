# Review RV-255 — design of SL-206

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

**Target.** `SL-206` design (`.doctrine/slice/206/design.md`, commit `fde4b025`)
— a workflow-templated slice-driver. Facet: **design**. Posture: **inquisitor**.

**Prior process (context, not exemption).** The design already survived a
3-round GPT architect panel pre-write; this inquisition is the formal hostile
pass on the *written* artifact. Prior survival is no acquittal — the ledger
presumes guilt.

**Doctrine held over the accused.** ADR-006 (orchestrator sole-writer),
ADR-011 (harness-agnostic spawn), ADR-012 (dispatch topology), ADR-007 (this
tribunal), STD-001 (no magic strings), STD-002 (naming), the AGENTS.md
behaviour-preservation gate, the CHR-039 empirical findings
(`.doctrine/rfc/011/chr-039-findings.md`), and the SL-199 funnel as-shipped
(`src/mcp_server/dispatch.rs`, `src/dispatch.rs`).

**Lines of interrogation.**
1. **`phase_projection` behaviour-preservation** — the design (D6, §9) claims a
   delegate-only extract from `run_status` leaves the existing dispatch suite
   green *unchanged*. Is that mechanically true against the real code, or does
   `run_status` carry per-phase coupling that resists a clean extract?
2. **Grant-gate safety claim** (D2, §5.4) — static allowlist + runtime
   negative-probe. Does this actually contain a background worker, given
   CHR-039's un-prompted-MCP-write finding? Any gap where a spawned agent reaches
   a write tool the design assumes it cannot see?
3. **IMP-174 posture** (§1a/§5.5) — coord-committed truth, no auto-land, a seeded
   raw divergence advisory. Does the driver anywhere silently cross the authored
   split-brain? Is `dispatch_tip`-vs-boundary status truth sound under a torn/
   stale coord?
4. **ReceiptStatus soundness** (§5.2) — does `Completed ⟺ boundary row exists`
   hold under conclude's documented half-landed self-heal, and are all
   `resolve_coord` refusals genuinely first-class?
5. **STD-001 / naming** — magic strings in the halt-reason vocabulary; the
   `dispatch_tip`/`code_end` role-naming; the in-script constants.
6. **Scope integrity** — does the design over-reach the slice scope
   (batching, auto-land) or under-deliver a declared target?

**Reviewer.** codex / GPT-5.5, full workspace-write (may raise on the ledger
directly). The Inquisitor synthesises the verdict.

## Synthesis

**Judgement.** The accused is *not* a heretic in its bones — the architecture of
SL-206 stands acquitted. But the written artifact confessed **seven taints** under
cross-examination (five major, two minor, no blocker), each proven against the
living code, and each has now been scourged from the design and its penance
sealed. Prior survival of the architect panel bought no indulgence; the ledger
found what loose prose had let pass.

**The taints confessed, and their penance (all verified terminal):**
1. **F-1 — the unshriven conformance gate.** The design would have granted three
   new tokens to the orchestrator while `allowed_mcp_tokens` (doctor check #9,
   `src/doctor_checks.rs`) still permitted but three — a false-red upon its own
   agent-def. Penance: the allowlist authority is now a declared touch-target,
   grown in lockstep, asserted by test.
2. **F-2 — the false witness of the source scan.** A static scan of authored
   templates was paraded as proof of the *live* grant, though ISS-216 (open) holds
   that `install` cannot reseat a changed def. Penance: the claim is bound to
   source-conformance + a verified install; phase-0 bears a clean-install
   precondition and probes the installed copy.
3. **F-3 — the fabricated tip.** The receipt demanded a `dispatch_tip` even on the
   coord-refusal path that can bear none. Penance: `PhaseReceiptResult =
   Resolved | CoordRefused` — truthful by construction.
4. **F-4 — the dropped states.** `Blocked` and the malformed `Unknown` were cast
   out of `ReceiptStatus` though the authorities distinguish them. Penance:
   restored, fail-loud, every branch tested.
5. **F-5 — the read-only lie** (the sharpest charge). The "read-only" probes were
   to wear the orchestrator's raw `Edit`/`Write`/`Bash`/`Agent`. Penance: a
   genuinely shorn `dispatch-probe` role, stripped to `Read`/`Grep`/`Glob` + the
   three reads.
6. **F-6 — the ungoverned tongue.** A freehand pile of halt-string literals drove
   protocol. Penance: a named `HALT` contract + the re-exported `funnel:`/`coord:`
   closed vocabs.
7. **F-7 — the lying scope.** "Two new surfaces" where the design bore three.
   Penance: the scope prose now counts honestly.

**Standing risks (consciously borne, not heresy).** ISS-216 (R6) remains outside
this slice — SL-206 leans on a manual reseat until it lands. SQ3 (R4) rests on
SL-199 prior art until phase-0's narrow demo. Batching (R2) is deferred; the
dumb-zone ceiling ships advisory-only. None gate the close.

**Sentence.** No blocker stands. The design, penance wrought into it (commit to
follow), is fit to pass to `/plan`. Let the wheel rest; the doctrine is kept.

> **HERESIS URITOR; DOCTRINA MANET**
