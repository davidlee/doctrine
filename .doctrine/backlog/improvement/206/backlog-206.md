# IMP-206: REV prose-fidelity lint — diff claimed invariants vs gate predicates

**Source:** SL-165 PIR §2.4, S5 (STRUCTURAL). **Home:** standalone (governance/review quality — no current RFC).

A REV `modify` row targeting a requirement's normative prose can under-describe the
landed gate. SL-165: codex found 4 under-described teeth (INV-3 Created-only, INV-5
count-exact, INV-1/F3 full-journaled-gate, kind=audit) — caught only via an
external review round-trip. The manual prose-vs-predicate fidelity check is
labor-intensive for what is a mechanical diff.

**Fix direction:** `doctrine revision check-fidelity REV-N` / a hook in
`revision approve` — diff the invariants the code enforces against the constraints
the proposed prose claims; warn when prose mentions fewer.

Note: governance/review tooling, not dispatch — does not belong in RFC-004/005/011.
Awaiting a governance-quality RFC or direct slice.
