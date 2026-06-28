# IMP-203: Gate-binary-not-on-edge — candidate-build path for self-modifying close

**Source:** SL-165 PIR §2.1.1, S4. **Home:** RFC-011.

When a slice modifies the close machinery itself, its implementation lives only in
a gc'd dispatch worktree, not on edge — so the closer must hand-rebuild a
gate-bearing binary from the candidate tip to dogfood `candidate create/admit`.
Manual and fragile.

**Fix direction:** `/close` step-3a detects the impl is not on the delivery trunk
and builds from the admitted candidate OID before dogfooding; or a
`dispatch candidate build` one-command path producing the gate binary from a ref.

Related: RFC-011; IMP-169 (close-gate manual/external integration).
