# Doctrine routing gate

**Route before you act.** At the start of ANY substantive work in a doctrine
repo — before inspecting files, running commands, or writing code — choose the
governing skill. This is a mandatory gate, not a suggestion. Skip it only when
the user has explicitly told you to.

The mechanism is `/route`: it reads the routing table and picks the skill that
governs the work (e.g. `/canon` for governance questions, `/preflight` for
unclear-but-substantive work, `/slice` when code intent has emerged with no
governing slice, `/design` → `/plan` → `/execute` down the lifecycle). The table
itself rides the boot snapshot at `.doctrine/state/boot.md` (`## Routing &
Process`), which is the authoritative copy — read it there, don't reconstruct it
from memory.

Two rules the gate enforces:

- **When unsure, route to the stricter skill.** If two paths fit, take the one
  that demands more rigour. Cheap to over-route, expensive to under-route.
- **No code without an approved plan.** The lifecycle exists so implementation
  follows a locked design and an approved phase plan. The plan is not higher
  authority than the design or `/canon`.

Mid-flight, an unanticipated obstacle, tradeoff, or emergent complexity routes to
`/consult` — don't improvise past it.

Point of truth: `/route` and the routing table in `.doctrine/state/boot.md`
(inlined into `CLAUDE.md` via an `@`-import). See
[[pattern.doctrine.core-loop]] for the full lifecycle the gate feeds into,
[[signpost.doctrine.lifecycle-start]] for where to begin,
[[concept.doctrine.memory-model]] for the retrieve-before-you-assume habit the
gate leans on, and [[concept.doctrine.reading-entities]] for the read-via-show
rule that keeps the gate honest.
