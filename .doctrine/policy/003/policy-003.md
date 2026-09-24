# POL-003: Harness independence from host-harness seams

## Statement

Harness-specific behaviour — anything doctrine wires for one particular agent
harness (Claude Code hooks, codex hooks, pi extensions) — ships as an **opt-in
supplement** whose correctness rests only on **doctrine-owned contracts**. It is
never baked into the neutral core, and never load-bearing on a host harness's
incidental seams. Three prohibitions follow:

1. **No harness vocabulary in the neutral core.** A harness's tool names, wire
   fields, matcher tokens, and environment markers belong in that harness's
   adapter or codec, never in the shared pipeline. The core sees only
   doctrine-owned concepts — a neutral request, a probe, an envelope — and the
   codec translates to and from a harness's vocabulary at the edge.
2. **No correctness resting on an incidental harness seam.** Doctrine may rest
   on a harness contract that is documented and stable — a hook schema, a
   published extension API. It may **not** rest on a seam that is incidental to
   the harness: an undocumented field order, a package-owned environment marker,
   an internal name a harness happens to canonicalise to, or a channel that
   merely works today. Where the only available seam is incidental, the
   behaviour is a documented delta or a follow-up, never a silent dependency.
3. **Opt-in, and disclosed.** A supplement is installed deliberately. The
   installer reports what it wired, names any manual activation step, and reports
   a skipped or failed install leg. It does not claim a hook is active when the
   harness still requires trust or approval. The neutral core functions with no
   supplement present; a harness that lacks the capability loses the supplement,
   not correctness.

## Rationale

Doctrine wires its behaviour into harnesses it does not own. Their hook
schemas, extension APIs, and tool vocabularies move on their own schedule, and a
mechanism coupled to one harness's incidentals silently stops working — or works
only under one spawner — the moment that harness changes. That coupling is
invisible until a second harness, or a second harness version, adopts doctrine:
the most expensive time to discover it.

This is the harness axis of the same discipline `POL-002` fixes for the host
*project* axis. `POL-002` forbids load-bearing on a host project's conventions
and transient state; it explicitly does not reach the host *harness*, and
stretching it there would let a real gap pass as already-governed. Strict-and-
owned beats lenient-and-coupled: harness glue behind a doctrine-owned contract
ports; harness glue baked into the core forks.

`ADR-011` decided the mechanism: moving behaviour into the binary makes it
identical under every harness by construction. This policy makes the *rule* that
decision implies enforceable — the recurring principle applied ad hoc in
`SL-205` (Claude `PreToolUse` ambient memory surfacing) and generalised across
the pi and codex ports (`SL-263`). Counting `ADR-011`'s surviving
mechanism-in-the-binary principle, those are four instances of the same rule.

## Scope

Applies to all shipped doctrine behaviour that targets a specific agent harness:
hook registries and their install legs, generated harness extensions, wire
codecs and their decoders, matcher sets, and any adapter that translates between
a harness and doctrine.

It does **not**:

- constrain a host project's own harness configuration or conventions (as with
  `POL-002`, those are client choices);
- forbid harness-specific *code* — it requires it to live in an adapter behind a
  doctrine-owned interface, not to be absent;
- reach a harness's own runtime trust policy (e.g. whether codex has trusted an
  installed hook). Doctrine discloses the manual step; it cannot guarantee a
  harness's trust state, and a documented runtime delta is the honest outcome.

## Verification

VH — by design review and audit. A reviewer challenges any harness-facing
mechanism with three questions:

1. Does this put a harness name, field, matcher, or tool token into the neutral
   core? If so, move it behind the harness's codec.
2. Does its correctness rest on a harness seam doctrine does not own? If so,
   re-ground it on a doctrine-owned contract (the envelope, the install
   registry, the ownership predicate), or record the behaviour as a documented
   delta with a follow-up.
3. Is it opt-in and disclosed? Does the installer distinguish wiring from
   activation, name any manual step, and report a skipped leg?

The conformance work is the port programme itself: `SL-205`'s ambient surfacing
and `SL-263`'s pi + codex ports are the worked examples the rule was induced
from.

## References

- `ADR-011` — harness-agnostic orchestrator spawn interface (mechanism in the
  binary; the decision this rule makes enforceable).
- `POL-002` — platform independence from host-project conventions and state
  (the sibling policy on the project axis).
- `STD-001` — no magic strings.
- `SL-205` — ambient memory surfacing (the second instance).
- `SL-263` — the pi + codex ports (the third and fourth instances).
- `IDE-034` — the backlog idea the policy was promoted from.
- `PRD-004` / `SPEC-011` — the memory and boot-snapshot capabilities the
  instances ride.
