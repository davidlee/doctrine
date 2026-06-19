# IMP-115: Derived-cache policy-version-stamped staleness detection (REQ-094 / SPEC-001 NF-001)

Tracks **REQ-094** (SPEC-001 NF-001): "Derived cache is policy-version-stamped
for staleness detection."

A `PRIORITY_POLICY_VERSION = "priority.v2"` const already exists in
`src/priority/render.rs` and is stamped into JSON envelopes. What's missing
is the staleness-detection mechanism: comparing a cached version against
the current policy version and flagging/refusing a stale cache.

Part of the SPEC-001 Priority Engine closure work.
