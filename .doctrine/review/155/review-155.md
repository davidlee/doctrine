# Review RV-155 — design of SL-151

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Lines of attack — this Inquisition probes:**

1. **Design coherence.** Are D1–D3 internally consistent? Does any decision
   contradict another? Does the design claim something that the affected
   surface cannot deliver?

2. **Scope alignment.** Does `slice-151.md` still reference the dropped
   structural scanner or remediation hint? Does the scope's "Non-Goals"
   section match what the design actually excludes?

3. **ADR-001 layering.** Does `parse_entity_toml` live at the right
   altitude (pure leaf, no IO)? Does `scan_kind`'s new parse belong at the
   engine layer? Is the `commands/validate.rs` boundary clean?

4. **Caller survey accuracy.** The design claims ~13 callers for
   `read_meta`/`read_id`. Are there missing callers in the survey table?
   Are the stated prefixes correct for each kind?

5. **Verification coverage.** Do VT-1 through VH-1 actually cover every
   claimed behaviour? Are there gaps — e.g. no test for the `read_id`
   wrapper path, or no test proving `parse_entity_toml` is a pure 1:1
   replacement for `toml::from_str` on valid TOML?

6. **Performance claims.** The design states "no catalog performance
   impact" because `scan_kind` is only called by `validate`. Is this
   actually true at the code level? Does any other code path call
   `scan_kind` or `id_integrity_findings`?

7. **Error quality.** With the remediation hint dropped (F-1), does the
   canonical-id context alone improve the error enough? Is the example
   error shape in the design achievable with `anyhow::Context`?

8. **Governance.** Does the design conflict with ADR-004 (outbound-only
   relations) or ADR-006 (tier merge-safety, detect-half)? Does it
   violate any project convention (POL-001 clankspeak, POL-002 platform
   independence)?

**Held to these invariants:**
- No string-matching on error text (fragile memory)
- No catalog performance regression
- No stale scope references to dropped features
- Every stated caller actually exists with the claimed prefix
- `parse_entity_toml` is a pure leaf (no IO, no config dependency)
