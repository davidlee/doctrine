# IMP-198: Harden architecture_layering_gate to always-green (no pre-existing-red blind spot)

**Source:** SL-168 postmortem F-1, §5d.2 (the single highest-leverage fix). **Home:** RFC-005.

`architecture_layering_gate` was pre-existing RED (unrelated reasons), so its
failure was dismissed as noise and a new registry→spec upward edge (F-1) shipped.
A gate that starts red cannot detect new violations.

**Fix direction:** make the layering gate hardened / always-green so it never
starts red and blocks CI. Postmortem's verdict: "if the gate can't drift, workers
can't silently violate it." Pairs with IMP-194 (diff-aware funnel).

Related: RFC-005; IMP-194; governed_by ADR-001.
