# Verified RV findings are terminal — a post-verification decision change is a prose amendment, not a reopened disposition

ADR-007: no verb transitions a finding out of `verified` (`src/review.rs:703-728`).
Once a finding is verified, its structured disposition is the **immutable
audit-time record** of what was decided *then*.

So when the operator later changes their mind — a `follow-up`/deferred finding is
pulled to fix-now before close — you cannot reopen and re-dispose it. Reaching for
`review dispose` will fail, and hand-editing the TOML to force it would destroy the
audit trail the immutability exists to protect.

**How to apply.** Record the change as a prose section on the RV `.md`:

```markdown
## Post-verification amendment — F-6/F-7/F-8 pulled to fix-now (YYYY-MM-DD)
```

State what changed, why, where the fix landed (candidate ref + OID), and that the
verified `follow-up` dispositions stand as the audit-time record. Cross-reference
it from the `## Reconciliation Outcome` under a "Fixed-in-candidate (post-verification,
not a reconcile write)" heading so `/close` can see the finding is discharged
without a disposition change.

Observed at SL-227 / RV-302 (F-6/F-7/F-8).

Related: [[mem.pattern.dispatch.admit-fix-on-top-not-supersede]] — the ref-level
half of the same flow (how the fix-now commit reaches trunk).
