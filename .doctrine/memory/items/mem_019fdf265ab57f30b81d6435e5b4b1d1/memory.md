Two rules for driving an external adversarial pass (codex MCP or another agent),
both learned the hard way on `RV-346`'s later rounds.

## Put the scope bar on the ledger, not in the driver's prompt

A late round should usually be told the bar is **blocker-or-regression**, and
told explicitly that *a round which raises nothing is the successful outcome* —
otherwise a reviewer paid in findings will manufacture marginal ones.

Write that binding into the **review brief on the ledger**, not into the prompt
you hand the spawner. Prompt text dies at handover; the brief survives it, and
the reviewer reads the brief directly. `RV-346` round 7 was bound this way,
raised nothing out of scope, and said so — the intended result.

## Do not self-rule on a bar you authored

When you find a defect **in your own remediation**, do not fix it unilaterally
and record it as handled, even when you are confident and even when the fix is
obvious. Put it to the external reviewer.

`RV-346`: a mutant-parity gap was found while remediating `F-30`. Self-assessed
it looked like a tidy-up. Put to codex, it was ruled a **blocker** and became
`F-38`. A self-issued finding on one's own work is worth little precisely
because the author's scope judgement is the thing under test; the external
ruling is the whole value, and the round is already open, so the marginal cost
of asking is one turn.

The generalisation: **authoring the bar disqualifies you from ruling on your own
compliance with it.**

Related: [[mem.pattern.doctrine.conventions]].