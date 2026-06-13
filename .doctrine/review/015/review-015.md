# Review RV-015 — reconciliation of SL-059

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Closeout reconciliation for SL-059 (knowledge records). All four phases landed
green; a `/code-review` ran this session and produced findings C1–C6 plus an
agreed remediation plan. This RV is the durable home for that review's
disposition — it records what was fixed in-slice, what was deferred, and the
conformance probes against the locked design.

**Lines of attack / invariants held:**
- **Dead-code honesty.** No production symbol masked by a blanket suppression;
  the only non-production code is gated `#[cfg(test)]` per-fn (C1/C5).
- **Guard coverage.** Every silent-corruption guard ships with its golden — the
  `set_record_status` malformed-refuse parity with adr/standard (C2).
- **Drift containment.** Duplicated rules (the `list_rows` reveal vs
  `listing::retain`) are at minimum flagged; structural unification deferred (C3).
- **Tolerant-read boundary.** The hand-edited tier's accepted silence (typo'd
  facet keys C4, unvalidated read-status C6) is consciously deferred, not lost.
- **Design conformance (probes):** NF-001 behaviour-preservation (suites green
  UNCHANGED), F-A1/A7 totality, NF-003 partition + F-A5 dual assertion, D8
  `DecisionRef` stays `Unvalidated`, F-A2 seed-status anti-drift.

Committed surface is green/clippy-clean/fmt-clean before audit; these findings
are quality + a test gap, not breakage — no `blocker`.

## Synthesis

**Closure story.** SL-059 shipped all four phases green; this audit dispositions
the `/code-review` findings. Two `major` quality findings were fixed in-slice
(F-1 dead-render unmask, F-2 malformed-refuse golden), one `minor` was reconciled
with a drift comment + deferred structural fix (F-3), and the two tolerant-read
findings (F-4 facet-key typos, F-5 read-status validation) were consciously
deferred to IDE-009 as owned future work. No finding was a `blocker`; the
committed surface was green, clippy-clean, and fmt-clean before audit and remains
so after.

**What F-1 taught.** The blanket `cfg_attr(not(test), expect(dead_code))` was not
just stale for the render subtree — it was *masking four other genuinely
test-only symbols* (`default_status` + three facet-enum `KNOWN` drift-canary
sets). A blanket module suppression is strictly worse than per-symbol gating: it
hides the very drift the lint exists to catch. Per-fn `#[cfg(test)]` restores the
signal. This is the general lesson worth carrying (see notes / memory).

**Behaviour-preservation (NF-001).** Held. The only existing-test touch was an
*addition* (the F-2 golden); no existing assertion was edited to pass. The F-1
`opt_enum_line`→`opt_text_line` collapse is byte-identical (both splice through
`toml_string`), so VT-1's byte-stable round-trip stays green unchanged — verified
by the full `--workspace` gate.

**Conformance probes.** Suites green unchanged (NF-001); F-A1/A7 totality, NF-003
partition + F-A5 dual assertion, and F-A2 seed-status anti-drift all still pinned
by their existing tests (untouched, green). D8 `DecisionRef` remains `Unvalidated`
(unchanged by this remediation).

**Standing risk (tolerated as deferred, not unbounded).** The hand-edited
knowledge tier still accepts typo'd facet keys and out-of-vocab read-status
silently — the accepted cost of the R2 tolerant read. The mitigation is a
read-only `knowledge lint` surface (IDE-009), not tightening the read. C3's
duplicated reveal rule is flagged in-code and its structural unification (a
kind-aware `retain` closure) is folded into IDE-009.

Ledger resolved: 5/5 findings terminal (verified), `done · await=none`. Audit-ready
for `/close`.
