# Review RV-057 — design of SL-088

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This Inquisition presses the SL-088 consolidated-installer design against seven
sanctioned invariants (D1–D7), with particular attention to five named attack
surfaces the User demands examined:

1. **y/N/a prompt flow** — Is it correct, clear, and non-confusing? Does `a`
   unambiguously mean "yes to all remaining" and not "abort"?
2. **Hard removal of `claude install` (no alias)** — Are there dangling
   references in the README, e2e tests, and worker-guard goldens?
3. **Agent-def generalization** — Flat canonical for Claude
   (`.doctrine/agents/dispatch-worker.md`) vs. namespaced for pi
   (`.doctrine/agents/pi/dispatch-worker.md`): correct, consistent with
   embed paths?
4. **Non-fatal agent resolution** — When no `.claude/` and no `--agent`,
   does the design truly skip skills without error (contra the current
   `resolve_agents` bail)?
5. **Forward-step ordering** — Memory sync → boot → skills: is this order
   justified, or arbitrary? Any hidden interdependencies?
6. **SL-084 embed path** — `install/agents/pi/dispatch-worker.md`: verify
   it actually lands at the claimed canonical + link paths.

The Inquisitor holds the accused to the seven invariants enumerated in the
primed domain_map. Let no heresy escape unnoticed.

## Synthesis

**Judgement:** The design is substantially sound but bears the taint of omission —
two blocking gaps and four lesser blemishes. The consolidated-installer concept is
clean. The prompt mechanism is correct. The module composition is well-targeted.
But the design's silence on its own public documentation and test migration is a
sin of omission that, if unconfessed, would ship falsehood to the faithful.

**Penance — ordered:**

1. **(F-1, blocker)** Add `README.md` to the Affected files table in design §1.
   Specify replacement wording for all five `claude install` references.
   Gate: README must not reference a removed command.

2. **(F-2, blocker)** Add `tests/e2e_claude_install.rs` to the removal-tests
   subsection (design §5). State whether it is rewritten as consolidated-install
   e2e or deleted. Gate: the test suite must still prove the SubagentStart hook
   and agent-def assertions under the new install surface.

3. **(F-3, major)** Update `tests/e2e_worker_guard.rs` goldens from `claude
   install` to `install` label. Rename the test function. Execute during
   implementation — no design change needed.

4. **(F-4, major)** Document the canonical-path asymmetry rationale in design §4.
   The Inquisitor accepts: Claude is grandfathered flat (it shipped first); new
   agents use namespaced canonical paths under `.doctrine/agents/<name>/`.

5. **(F-5, major)** Specify the degenerate case in design §2-§3: when no harness
   directories exist, the boot prompt must either use neutral language or skip.
   Also note that `install.rs` uses a non-bailing agent resolver distinct from
   `skills::resolve_agents`.

6. **(F-6, minor)** Justify the forward-step ordering in design §2 stage 3:
   least-invasive-first (additive → config-modifying → agent-path-mutating).

7. **(F-7, minor)** Neutralize the boot prompt text to avoid dangling `--agent`
   confusion: say "detected harnesses" rather than enumerating claude/codex.

8. **(F-9, nit)** Remove the stale `.doctrine/agents/claude/` subdirectory during
   implementation.

**Standing risks:**

- **RSK-3:** A user with only `.codex/` but no `.claude/` running `doctrine install`
  will see skills skipped (no `--agent`) and boot prompt may be empty — verify
  the UX for this case is intentional.
- **RSK-4:** SL-084's embed file (`install/agents/pi/dispatch-worker.md`) does not
  yet exist — the install path for pi cannot be tested until SL-084 ships.

**Tolerated:** The `prompt_step` function is correct (F-8 — `to_lowercase()`
normalizes `A` to `a`). The prompt label `[y/N/a]` follows convention; a one-word
inline hint (`a=all`) is cosmetic.

**Verdict:** The design shall proceed to plan — AFTER the two blockers (F-1, F-2)
are reconciled in the design document. The Inquisitor will not stand between the
faithful and their consolidated install, provided the word of the README is kept
true and the e2e tests are not abandoned to heresy.

> **HERESIS URITOR; DOCTRINA MANET**
