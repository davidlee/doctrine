# REQ-288: Arm routing is deterministic: doctrine.toml [dispatch] claude-force-subprocess-dispatch forces /dispatch-subprocess; otherwise the env-marker (.claude/ presence) selects /dispatch-agent vs /dispatch-subprocess; a self-belief vs env-marker mismatch refuses naming the cause and never blind-spawns.

## Statement

> **AMENDED — FALSIFIED IN WHOLE (SL-254, 2026-08-14).** Every referent in the
> title line above is gone. There are no arms to route between, so there is no
> arm routing. RETIREMENT RECOMMENDED — the `status` field is deliberately
> untouched here; a status transition is landed by the orchestrator through the
> governing revision's typed `[[change]]` rows, not by an in-place prose edit.

SL-254 collapsed dispatch from two arms — an in-session "claude arm" driving
Claude's `Agent` tool with `isolation: worktree`, and a subprocess arm for
codex/pi — onto ONE subprocess arm. Every harness, claude included, is spawned as
a confined subprocess by `scripts/spawn-confined.sh <harness>`. Specifically:

- **`/dispatch-agent` and `/dispatch-subprocess` no longer exist.** Both skills
  were merged into the single `plugins/doctrine/skills/dispatch-spawn/SKILL.md`
  (PHASE-07). There is no pair to select between.
- **`doctrine.toml [dispatch] claude-force-subprocess-dispatch` is gone**,
  vestigial once there is only a subprocess arm to force onto.
- **`dispatch arm-spawn` and all per-harness arm routing are deleted.**
- **The env-marker (`.claude/` presence) selector is gone**, and with it the
  self-belief-vs-env-marker mismatch refusal it existed to produce. There is no
  routing decision left that a mismatch could corrupt, so there is nothing left
  to refuse and nothing left to blind-spawn.

What survives of this requirement's INTENT — that dispatch never guesses its
execution posture — is now discharged structurally rather than by a routing
predicate: there is one path, and a missing `bwrap` is a NAMED refusal that fails
closed at spawn on Linux (`jail.rs::REASON_NO_BWRAP`), with no unconfined
fallback. On macOS the naming does not hold — there is no `sandbox-exec`
presence probe (corrected SL-254) — so a missing backend there fails closed
unnamed, at exec.

## Rationale

Deterministic arm routing was the price of having two arms whose capability
altitudes differed. Once the difference is abolished — one spawn path, one
kernel-level confinement, one identity mechanism — the routing question does not
have a better answer; it stops being a question.
