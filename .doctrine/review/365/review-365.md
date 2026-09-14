# Review RV-365 — design of SL-259

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Adversarial review of `SL-259`'s design (`.doctrine/slice/259/design.md`, nine
sections, revision 40 of design run `dr-01a08e32`). Reviewer: codex-cli
(`gpt-5.6-sol`). The design was drafted by the same agent that made the rulings
it defends, so the review's first duty is to test the defences, not the prose.

**The subject's own claim.** `apply` must tell the truth about whether it
changed state. One invariant, four legs: (1) error implies nothing landed,
(2) success implies truthful rows, (3) unknown/inert input implies a typed
refusal, (4) old snapshots stay readable.

**Lines of attack, in priority order.**

1. **`sec-6` states leg 1 conditionally.** The authored tier is unmoved
   *"except across the named late-check window"*. That is weaker than the
   slice's headline invariant. Probe whether the carve-out is genuinely forced
   by `SPEC-029`'s reserve-then-journal protocol, or whether `DEC-250`'s hoist
   could be pushed far enough to make the promise absolute. `DEC-100` is cited
   as precedent for stating a window rather than designing it away — test
   whether the analogy holds or is doing rhetorical work.
2. **`sec-2` defers the `Declaration` wire/stored split** on the ground that all
   16 live snapshots read `delegation = []`, so the type overlap is latent. Test
   whether "latent" is a reason to defer or only a reason it has not bitten yet.
   The design concedes the justification "expires the first time delegation is
   used in anger" — ask what happens to already-stored declarations at that
   moment, and whether the roster-plus-floor in `sec-2` actually covers it.
3. **`sec-3`'s refusal mechanism** drives strictness off `payload_contract`'s
   key inventory rather than serde. Probe the single-source claim: `sec-8` pin 1
   is said to prove the contract equal to the wire shape — verify that pin
   exists and proves what is claimed, at every nesting level including inside
   `#[serde(flatten)]` and the internally-tagged enums. A drift between contract
   and wire type is a silent hole in leg 3.
4. **`sec-5`'s history/state line.** `DEC-249` degrades tokens that *record*
   and refuses tokens that *constitute*. Attack the boundary: find a token that
   is plausibly both, or a path where a degraded row feeds something that treats
   it as state. `render/envelope.rs:1071` is named as the only production reader
   of `.rows` — verify that, because the encapsulation argument rests on it.
5. **`sec-4`'s emit-at-the-seam rule.** `DEC-248` rejects widening a derivation.
   Check that the two named sites (`CheckpointActGroup::record`, `declare_node`
   diffing against an empty prior) actually cover the class, and that diffing
   creation against an empty prior does not emit rows the update path suppresses.

**Invariants to hold it to.** `ADR-001` (leaf ← engine ← command, no cycles);
`STD-003` (no silent skip — a degraded read is disclosed); `SPEC-029`
(reserve-then-journal, no failure path deletes authored knowledge); the
behaviour-preservation gate in `AGENTS.md` (existing suites stay green
*unchanged* when shared machinery moves); and this project's refusal of parallel
implementation.

**Where the bodies are likely buried.** The measurements in `sec-9` R1 were
taken against a runtime tier that moves; the design says re-probe. Any argument
in the design that rests on a count (16 snapshots, `delegation = []`,
`CHANGE_LOG_REVISIONS` = 32, `SL-244` at revision 92 / floor 61) is a claim to
verify against the tree, not to accept.


## Synthesis

**Closure story.** Six findings, all raised by an external adversarial reviewer
(codex-cli, `gpt-5.6-sol`) against design run `dr-01a08e32` at revision 40; all
six verified independently against the source tree before disposition; all six
disposed `design-wrong` and verified terminal by the raiser against the amended
text at revision 48. No ruling was reversed. The design stands at revision 50.

Three findings were blockers and each named the same species of failure, which
is worth stating plainly because the design's own subject is truthfulness about
failure:

- `F-1` — `DEC-248`'s emit-at-the-seam rule was applied to one of two act
  stores. The design argues against instance-by-instance repair and then
  committed one, inside the section making the argument.
- `F-2` — `DEC-250` asserted a defect with no witness. `sec-6` refuses to close
  `ISS-361` on a plausible story; it had not held its own ruling to that
  standard. Resolved by *withdrawing the claim*, not by hunting for a witness:
  the hoist is restated as hygiene that makes the two-pass doc's claim true by
  construction rather than by the accident that today's predicates do not
  discriminate.
- `F-3` — `sec-2` claimed a degradation floor beneath stored declarations that
  `DEC-249` forbids it from having. Not an implementation shortfall; a claim
  incompatible with a neighbouring ruling.

`F-4` (no reason carried on `RawRow`, so `STD-003`'s *why* was unserviceable),
`F-5` (retention arithmetic off by 24 revisions) and `F-6` (a cited source
reader that does not exist) were the factual tail.

A seventh defect — `sec-5` calling four methods the type's "whole surface" when
`floor` is also read from outside (`envelope.rs:560`, `:1064`) — was introduced
*by the amendments* and caught on the verification pass. Corrected at revision
49 without reopening the ledger: it is a factual phrasing repair that touches no
ruling, and the encapsulation conclusion it supports is unaffected because the
field in question is a `u64` this design does not change.

**Standing risks.** `sec-9` `R0` now records the general case: three of this
design's factual claims failed external checking, none caught by the author.
Every file:line and every count in the design is a claim to re-verify at
implementation. `R1` (measurements taken against a moving runtime tier) is the
specific instance.

**Tradeoffs consciously accepted.**

1. *The `Declaration` wire/stored split stays deferred* (`IMP-446`). Delegation
   has zero live exposure — all 16 snapshots read `delegation = []` — and
   protecting an untravelled path does not earn a layering change in a slice
   already carrying four legs. The cost is honest and now stated in `sec-2`: on
   that one path the retired-member roster is the sole guard, protecting exactly
   the surface `sec-2` argues a roster cannot be trusted to protect alone. The
   deferral carries a trigger, not a note.
2. *Leg 1's promise stays conditional.* The carve-out survives on one mechanism
   rather than two — the late watermark check, which `SPEC-029` prescribes and
   no hoisting reaches.
3. *No pin is written for a pass-1-clears/pass-2-fails submission.* No reachable
   instance exists, and a pin over a manufactured predicate proves only that the
   manufacture worked.

**On the review itself.** The default reviewer changed mid-flight: the `codex`
MCP server is dead (codex-cli 0.154.0 removed the `mcp-server` subcommand), and
the review ran through the `codex-cli` wrapper instead. `CLAUDE.md` now records
that and the two knobs it needs.
