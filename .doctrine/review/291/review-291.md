# Review RV-291 — design of SL-225

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This Inquisition arraigns **SL-225's design facet**, with DEC-003 as the
principal accused. It presses four lines of interrogation:

1. Does DEC-003 assign binary resolution and its preconditions to the correct
   project/platform layer under POL-002, without smuggling host-project cargo
   layout into shipped engine behaviour?
2. Does the proposed `DOCTRINE_BIN → ./target/debug/doctrine → PATH` order select
   a fork-consistent binary in every claimed host, jail, coordination-worktree,
   and non-Rust-phase case, or merely move the stale-binary ambiguity?
3. Does rejecting `current_exe()` publication preserve the worker gate's trust
   and confinement model, including inherited environment and the existing
   marker-clearing seam?
4. Can the stated verification distinguish elimination of ISS-218's false-red
   from a hidden, skipped, stale, or wrong-binary run, and does every invariant
   agree with DEC-003 rather than retain relics of the rejected engine design?

Sanctioned authorities: POL-002, ADR-008, ADR-012, SPEC-012, SPEC-021, RFC-016,
the SL-225 scope, DEC-003, ISS-218, CHR-044, and relevant attested dispatch
memories. Evidence is confined to design intent; implementation conformance is
not commingled into this tribunal.

## Synthesis

**Judgement.** DEC-003's *reversal* is orthodox: teaching the shipped engine
about this repository's cargo layout, or publishing `current_exe()` from
`worker_commit`, would violate POL-002 and can pin the very stale binary the
recipe must escape. Let that rejected mechanism be broken upon the wheel.
But the replacement decision is not yet fit for `/plan`. It moves correctness
into a `DOCTRINE_BIN` precondition without giving that precondition a possible
place in the dispatch chronology.

The principal heresy is **F-1 (blocker)**. The client resolves `DOCTRINE_BIN`
when it loads the MCP command, whereas the dispatch process creates the
coordination worktree—and therefore the named coord build—after the server is
already running. A later shell export cannot alter the server's inherited
environment. Fresh dispatch therefore needs an explicit build-and-rebind
sequence or a stable project-local indirection/distinct gate-binary channel;
mere governance prose cannot conjure an executable backward through time.

Two further taints attend it:

- **F-2 (major):** `design.md` still declares unconditional
  `current_exe()` publication, the corpse of the engine design DEC-003 rejected.
  Excise it and sweep every governed artifact for the same relic.
- **F-3 (major):** VT-1 records a stub's argv and proves only recipe precedence.
  It does not prove that a fresh dispatch can supply the coord binary, nor that
  stale PATH fails while the selected binary makes the actual `worker_commit`
  gate green. Restore the scope's discriminating end-to-end proof and retain
  the stub only as a narrow unit test.

**Sentence, in order.** First settle F-1 by naming an executable fresh-create and
resume lifecycle, including the refusal when its precondition is absent. Then
purge F-2's contradictory invariant. Finally strengthen VT-1 per F-3 so the
chosen lifecycle is tried against an intentionally stale PATH binary through
the real gate seam. Only then may DEC-003 move from `proposed` and SL-225 pass
from design to plan. Until that hour, let no architect smuggle this assumption
past the close gate beneath a monk's robe of “operator responsibility.”

No taint is tolerated. No follow-up exile is sanctioned: all three penances
belong in SL-225's present design unit.

> **HERESIS URITOR; DOCTRINA MANET**
