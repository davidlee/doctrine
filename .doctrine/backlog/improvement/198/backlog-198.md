# IMP-198: Harden architecture_layering_gate to always-green (no pre-existing-red blind spot)

**Source:** SL-168 postmortem F-1, §5d.2 (the single highest-leverage fix). **Home:** RFC-005.

`architecture_layering_gate` was pre-existing RED (unrelated reasons), so its
failure was dismissed as noise and a new registry→spec upward edge (F-1) shipped.
A gate that starts red cannot detect new violations.

**Fix direction:** make the layering gate hardened / always-green so it never
starts red and blocks CI. Postmortem's verdict: "if the gate can't drift, workers
can't silently violate it." Pairs with IMP-194 (diff-aware funnel).

Related: RFC-005; IMP-194; governed_by ADR-001.

## RV-411 F-4 (2026-09-29): the gate is green while missing violations

The literal "pre-existing red" state is historical: the gate is green today.
The deeper form of this item's concern holds, though. RV-411 F-4
(`doctrine show RV-411`) found that the gate misses real upward edges:
- `extract_edges` ignores `crate::` paths inside macro bodies,
  struct-literal/pattern paths, `impl … for`, trait bounds and cross-module
  `super::`. Live miss: `src/coverage_verify.rs:257` (engine → command).
- Files that don't parse are silently skipped, which breaks STD-003 (no silent
  skip).
- `module::sub` tier rows only exempt their umbrella module, and are never
  enforced. Live: `priority::graph` (engine) imports command-tier modules.
- The tangle baseline (76 against 70 measured) never tightens.

Fixing the extractor will probably surface existing violations. Expect to
ratchet them in as accepted violations first.
