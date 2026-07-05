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
