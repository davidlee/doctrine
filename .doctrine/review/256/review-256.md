# Review RV-256 — reconciliation of SL-205

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject surface reviewed (F-2).** Dispatched slice (claude arm). Reviewed the
candidate interaction branch `candidate/205/review-001` (`cand-205-review-001`,
tip `3e550ced`) = impl-bundle `review/205` (`b6692b90`) merged onto `refs/heads/main`.
Evidence refs `review/205` + `phase/205-01..04` are immutable (R2). Code landed
across four phases on `dispatch/205`: PHASE-01 `5afa59c0`, PHASE-02 `8c28f6b7`,
PHASE-03 `9e0afeef`, PHASE-04 `a8ac15c1`.

**Lines of attack — invariants the slice is held to:**
- **INV-1** — emit only `additionalContext` (never a block/allow verdict).
- **INV-2 (fail-open)** — every path exits 0; a missing root / IO error / write
  failure never blocks the tool call.
- **INV-3** — subagent fires run no retrieve and surface nothing.
- **INV-6 (dedup integrity)** — a uid enters the seen-set only after a successful
  non-empty emit (F-3), so dedup can never suppress an undelivered memory.
- **F-2** — absent/empty `session_id` disables dedup but the fire still surfaces.
- **Path conformance** — every touched path declared by a `design-target` selector.
- **VT shape** — every `VT` criterion's mandated `test_file` + keywords present.

**Where bodies are likely buried:** the seen-set / tuning-log ordering (F-1/F-3
swallow), and the design's self-consistency on *when the tuning log is written*
(the §"Tuning log" whenever-retrieve-ran clause vs the §"Emit ordering"
only-on-non-empty-emit rule — already flagged mid-dispatch as ISS-217).

## Synthesis

**Closure story.** SL-205 ships ambient memory surfacing as a fail-open
`memory surface` subcommand invoked from two PreToolUse hook entries
(`Read|Edit|Write` + `Bash`) in the plugin `hooks.json`. It surfaces relevant
high-severity memories as `additionalContext` on a main-thread tool call and
stays silent on subagent fires. Delivered across four layered phases, each landed
as one commit on `dispatch/205` through the claude-arm dispatch funnel:
PHASE-01 (`5afa59c0`) the read-model `SurfaceRow`/`retrieve_rows` seam;
PHASE-02 (`8c28f6b7`) the pure `admits`/`dedup_diff`/`cap`/`format_block` helpers +
constants; PHASE-03 (`9e0afeef`) the shell (`run_surface`/`emit_surface`), the
`MemoryCommand::Surface` dispatch arm, the guard classification, and the three
RV-254 fail-open penances (F-1 swallow, F-3 delivery-gated seen-set+log, F-2
sessionless dedup-off); PHASE-04 (`a8ac15c1`) the `hooks.json` wiring.

**Evidence.** Path conformance clean (0 undeclared, 0 undelivered, 5/5
conformant — the durable `root.rs` design-target selector closed the one
mid-dispatch scope question). `doctrine check gate` on the admitted candidate
`3e550ced` exits 0 (clippy `--workspace` zero-warning + fmt-check). VT
shape-gate 17/17 across the four phases. The one pre-existing test failure
(`test_support::doctrine_bin_returns_existing_executable`) asserts a `cargo build`
artifact absent under `cargo test` — environmental, unrelated to this delta.

**Standing risks / tradeoffs consciously accepted.**
- **F-1 (ISS-217).** Suppression/empty fires write no tuning-log line, so the
  §"Tuning log" retuning signal for the suppression cases is uncaptured this
  slice. Tolerated: the shipped F-3 contract preserves INV-2/INV-6 and is
  VT-gated; the split-gate fix is behavioural, out of scope for a green-phase
  reopen, tracked in ISS-217.
- **Fail-open by construction (INV-2).** A missing root, IO error, or write
  failure degrades the surface to silence, never a blocked tool call — the
  correct safety posture for an advisory surface. This is a deliberate
  non-defensive-against-silence stance: a broken surface is invisible, not loud.

**VA-1 is post-close, not skipped.** The live PreToolUse battery cannot be
observed until the code reaches trunk: the live plugin sources `hooks.json` from
a github clone (`davidlee/doctrine`), so the new entries + the `memory surface`
subcommand are only live after integration + reinstall/`--dev` + a session
restart. This is an ordering fact, not an audit gap; VA-1 runs as a post-`/close`
acceptance check.

**Cleanup owed at close.** The design-validation prototype probes
(`.doctrine/slice/205/prototype/{command,memory}-probe.sh` and the matching
`.claude/settings.local.json` entries) are scaffolding to tear down before the
VA-1 live run.

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md §"Tuning log" (L183-185)** — rewrite the "Logged **whenever
  retrieve ran** … including when the emit is nothing" clause to match shipped
  behaviour: the tuning-log line (and the seen-set) are appended **only after a
  successful, non-empty emit** (F-3), consistent with §"Emit ordering & the
  swallow" (L212-222) and the ASCII funnel (L203-204). Removes the internal
  self-contradiction so design.md tells the shipped truth. Cite finding F-1.
  Cross-reference ISS-217 as the tracked follow-up for the alternative
  (split the gates so the log fires on every retrieve while the seen-set stays
  delivery-gated) should the suppression retuning signal later be wanted.

### Governance/spec (REV)
- None. No ADR/spec/REQ divergence surfaced; no standard/policy touched.

### Off-surface (not reconcile items — noted for completeness)
- No `plan.toml` criterion edits (immutable-append; the design/code are
  self-consistent with the plan).
- No selector-registry change (`slice conformance` clean).

## Reconciliation Outcome

### Direct edits applied
- **design.md §"Tuning log" (L183-185)** — rewrote the "Logged **whenever
  retrieve ran** … including when the emit is nothing" clause to the shipped
  truth: the tuning-log line and the seen-set append share the delivered branch
  (**F-3**), so a fire that surfaces nothing writes no funnel line — consistent
  with §"Emit ordering & the swallow" and the funnel diagram. Removed the
  internal self-contradiction. Cross-references ISS-217 as the tracked follow-up
  for the split-gate alternative (drives finding **F-1**).

### REVs completed
- None. No governance/spec surface touched — no ADR/spec/REQ/standard/policy
  divergence surfaced.

### Withdrawn / tolerated
- **RV-256 F-1** — tolerated as-built (suppression/empty fires log nothing this
  slice); rationale in the finding disposition and §Synthesis, split-gate fix
  tracked in ISS-217.

Reconcile pass complete — every brief item resolved, no half-applied REV.
Handoff to /close.
