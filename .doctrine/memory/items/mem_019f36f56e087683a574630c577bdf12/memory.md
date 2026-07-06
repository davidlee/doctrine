# Dispatch confinement posture: unjail charges once, confined charges per-capability

Decision framing for the dispatch **orchestrator confinement posture** —
nominated-**unjailed** (SL-206, ADR-008 amendment) vs **confined** Mode-B (SL-199,
every coord write via a server-side MCP tunnel). Carry this to the ADR-008
amendment authoring and any broader governance/direction reconciliation of dispatch
confinement — it is the *why*, not just the *what*.

## The crux: where complexity charges rent

- **Confined charges per-capability, FOREVER.** Mode-B's real cost isn't the three
  funnel tools — it's the rule *"every coord write needs a server-side tunnel"*.
  Each new orchestrator behaviour (fixup verdicts, prep worktree ops, journal
  writes, whatever the drive loop grows next) = a new Rust MCP tool + grant + doctor
  row + prompt-teaching. Plus a per-run **agent-confusion tax**: the orchestrator
  lives in a "jailed except N tunnels" model, discovers the boundary by `EROFS`
  mid-flight, burns tokens reasoning about placement. The SL-206 PHASE-05/06/07 lore
  (coord-root parking, placement-not-permission, hand-stamping) *is* this tax made
  visible; RFC-011 case-notes are full of it.
- **Unjail charges ONCE.** Nomination + spawn-gate + `SubagentStop` hygiene is fixed
  machinery: built once, mechanically checked (doctor), grows with nothing. A new
  orchestrator capability = **zero marginal security work**, provided seam symmetry
  holds. The trust model collapses to the **two tiers everyone already holds from
  ADR-006** — orchestrator = main-thread trust, worker = jailed. Mode-B invented a
  **third tier** (confined-but-tunneling) and that tier generated most of the
  confusion this slice fought.

**Net:** marginal-cost-zero vs marginal-cost-per-capability. For a system whose
whole point is *growing orchestrator behaviours*, unjail wins on the **trajectory /
derivative**, not merely today's ledger. The appeal is not cognitive comfort — it's
that the simplicity compounds in the growth direction.

## The honest price — buy unjail with these clauses in the amendment

1. **Steered-orchestrator blast radius, named out loud.** Confinement's real defence
   was **defence-in-depth against a misbehaving ORCHESTRATOR**, not a misbehaving
   worker. The orchestrator's primary attacker-shaped input is the **worker's diff**
   (fork → O). Under Mode-B a steered O was bounded to the funnel ops; nominated-
   unjailed, it has **main-thread reach** (primary tree, history, `.claude/`). The
   P1/P3/P4 probes prove **jailed→O escalation is closed** (a worker can't *become*
   the orchestrator) — they prove **NOTHING** about **O-being-steered**. Residual ≈
   what the main thread already accepts running `/dispatch` by hand; mitigated by a
   trusted committed def, a narrow prompt, and **structured inputs (diffs + typed
   receipts, not free-form instructions)**. The amendment must say this sentence
   out loud, not let "proven safe P1/P3/P4" imply the escalation proof covers it.
2. **Seam-symmetry must be CONFORMANCE-CHECKED, not prose.** Every future harness
   spawn seam inherits gate duty (I1: `Agent`, `Workflow`, any future surface → one
   deny-set). The toll stays **one-time only while a doctor check enforces it**
   (nomination ⊆ gate **and** every registry seam has a matcher). As prose it is
   **re-paid per harness release** — and that is exactly when the simplicity claim
   would quietly break.

And: record the **confined model as the reversible fallback** — it loses nothing
functional under Workflow (import already routes via the shipped MCP), so if seam-
symmetry ever proves unmaintainable, revert with no functional loss.

See [[mem.fact.dispatch.unjail-nomination-round-trip]] (nomination leg proof),
[[mem.fact.claude.pretooluse-agent-carries-spawner-id]] (the gate leg), the
confined-viability counter-evidence (Mode-B *works*, it just costs per-capability),
and SL-206 `design.md` §5.6 (amendment ledger) / §7 D8.
