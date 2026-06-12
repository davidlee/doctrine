# Review RV-004 — reconciliation of SL-044

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-044 (reconcile writer + closure gate, SPEC-002 B). All
three phases `completed`; lifecycle `started`. Findings originate from an
external adversarial code review (Opus reviewer, verdict *revision-required*),
each independently re-verified against the committed source before being raised
here — citations point at the confirmed line, not the reviewer's claim.

**The invariant under test (NF-001 / REQ-114).** Observed coverage must never
reach authored status *through a function*. This slice is where authored truth is
finally written, so the audit holds it to the design's own layered wall (§5.6,
D-B7): (1) signature isolation of the status-select fn, (2) the drift `Verdict`
consumed only by the prompt builder, (3) a **behavioural** verdict-independence
test on the residual write call site. Lines of attack:

- **Does the proof touch the write path?** The wall is only as real as the test
  that exercises `run()`. A verdict-independence test that never calls `run()`
  proves a tautology, not the invariant. (→ F1)
- **Does the structural argument overclaim?** A signature that "the compiler
  proves" only constrains the function body, not the call site that feeds it.
  (→ F2)
- **Close-gate soundness.** The discharge predicate and drift gate consume the
  *whole authored REC corpus* (hand-editable), not just writer output — so any
  "the writer only emits one-delta RECs" assumption is latent, not enforced.
  (→ F3)
- **Atomicity vs NF-003.** "Exactly one atomic REC" must hold for the *act*
  (status write + ledger entry), not just the REC file in isolation. (→ F5)
- **Project doctrine.** No parallel implementation (CLAUDE.md); tests that assert
  nothing; stringly-typed seam logic where a typed classifier exists. (→ F4/F6/F7)

**Out of scope / not re-decided here:** the governing decisions (SPEC-002 D7–D9,
ADR-003, ADR-009 FSM + closure seam). Tensions route back through `/consult`,
not this ledger.

## Synthesis

Verdict: **reconciled — close-ready.** All nine findings terminal (verified);
the one blocker (F-1) is resolved. Fixes landed in commit `36156fd` plus the
F-5 write-ahead reorder. `just check` green (951 tests, clippy clean).

**The headline: the wall now has teeth (F-1, F-2).** The slice's reason to exist
is NF-001 — observed coverage must never reach authored status through a function.
The implementation's *structure* was sound, but its *proof* was theatre: VT-5
asserted `select_status(fixed, _) == fixed`, an identity tautology that never
called `run()`, so a future `select_status(status_from_verdict(verdict), prior)`
edit would have left the suite green. VT-5 now drives the real `run()` over four
on-disk coverage states (three distinct verdicts), holds `--to` fixed, and asserts
the authored status reconstructed from disk equals `--to`. The laundering surface
is finally exercised. In tandem (F-2), the `select_status` docstring stopped
overclaiming: signature isolation constrains the function body, not the call site;
the invariant is proven by the three layers *plus* VT-5, not by the compiler. The
honest framing matters because the next maintainer was being told Rust guaranteed
something it does not.

**The latent close-gate hole (F-3).** `rec_discharges` checked clause (b) —
"the REC affirms R at R's current status" — by scanning *all* deltas in the REC
for a coinciding `to`, without binding the delta to R. Because RECs are
hand-authored TOML that may carry multiple deltas, one requirement's `to` value
could discharge another's residual drift at the close gate. The reconcile writer
only emits one-delta RECs, so this was latent — but the gate consumes the whole
authored corpus, not writer output, so "latent" is not "safe." Now matched on
`d.requirement == req`, with a regression test that fails pre-fix.

**Atomicity, decided not papered over (F-5).** "Exactly one atomic REC" held for
the REC file in isolation, not for the *act*: accept/revise wrote authored status
then minted the REC — a torn window where a failed mint leaves status moved with
no REC, exactly the unreconstructable authored tier NF-003/REQ-116 forbids. Per
the user's decision, accept/revise now writes the REC first (write-ahead): a torn
write leaves REC-present / status-lagging — a detectable, re-runnable drift, never
an unexplained move. The redesign arm deliberately keeps transition-first: it
writes no requirement status (F7) so NF-003 does not bind, and its guarded
`reconcile→design` back-edge can legitimately refuse, where REC-first would orphan
ledger entries. The asymmetry is documented at the seam.

**Doctrine hygiene (F-4, F-6, F-7, F-8).** The self-confessed `distinct_keys`
twin — shipped against CLAUDE.md's "no parallel implementation" with a comment
admitting it — is now one `coverage::distinct_keys`. A `X || true` dead assertion
became a real one; a `String`-allocating stringly seam check became a typed
comparison; the REC corpus is read once per close, not twice.

**Standing risks / consciously accepted.** (1) The redesign torn-write window
(transition succeeds, REC mint fails) remains tolerated — no authored status
moves, so NF-003 is untouched. (2) F-9: `--note` carries three deliberate
semantics across the new surface (spec discards per "no invented field"; reconcile
accept/revise surfaces in the drift prompt; redesign forwards into the transition
record). Each is justified; a unifying pass is optional, not a defect. (3) The
single closure-seam classifier (F-7) was not extracted — two call sites do not
yet earn the abstraction. None of these block close.
