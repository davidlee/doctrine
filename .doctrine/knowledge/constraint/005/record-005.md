# CON-005: Capsule threat model boundary

Scope honesty for the RFC-025 capsule model. Follows the precedent RT-1 already
set for verification ("capsules make verification observe the exact candidate,
confined; they do not make the verdict honest when the worker authors the tests
being run").

## The claim, stated narrowly

> **The capsule model bounds what a worker process can *do*. It does not bound
> what a worker can *say*, nor make its outputs trustworthy.**

Any claim broader than that is an overclaim and should be challenged.

## Bounded by construction

- Filesystem and network authority of the worker process (OS sandbox).
- Trusted-side *interpretation* of capsule content ([[interpretation-surface]]).
- Git-level ingestion hazards: fsck, ancestry from a contracted base,
  forbidden paths, undeclared scope.
- Atomicity of landing — one CAS advance, nothing partial
  ([[landed-state-append-only]]).

## Explicitly NOT bounded — named, not chased

- **A lying test suite.** A worker authoring its own tests passes them in any
  sandbox. RT-1 names this; RFC-022 (agent trust) and RFC-023 (adversarial TDD)
  are the governing artifacts.
- **Prompt injection into the control-plane agent.** The orchestrator is an LLM
  that reads refusal messages, commit messages, file names, diffs, and artifact
  text — all capsule-authorable. Capsules bound the worker's *authority*, not
  its *rhetoric*.
- **The orchestrator's own authority** — RT-8, residual and named.
- **Semantically malicious code that passes review and lands.** Capsules do not
  make code good.
- **Supply chain inside the capsule.** The worker may pull dependencies; that is
  within its own blast radius by design.

## The cheap structural mitigation we DO take

Since the control-plane agent reads capsule-adjacent text, refusals report
**structured tokens computed trusted-side** — the stage, the refusal token, and
paths derived from git — never capsule-authored prose. This extends RT-4's rule
("artifact content is never authoritative for anything admission depends on")
with: artifact content is also never *relayed verbatim* into the orchestrator's
context unmarked.

That is a by-construction win available for free from the design already
arranged. It is a mitigation, not a solution.

## Review posture

Operator ruling, 2026-08-01: address what is achievable by construction,
especially where it falls out of design elements already being arranged. Do
**not** pursue watertightness against a fully compromised in-capsule agent, and
do not spend review rounds attempting to prove it. Reviewers should read the
narrow claim above as the thing under review.

## Related

- [[interpretation-surface]] · [[landed-state-append-only]]
- RFC-025 `red-team.md` RT-1 (scope honesty), RT-4 (artifact hygiene),
  RT-8 (orchestrator authority).
- RFC-022, RFC-023 — the governing artifacts for the unbounded half.
