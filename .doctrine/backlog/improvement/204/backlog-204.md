# IMP-204: Selector under-declaration — design-time conformance check for compile-fallout

**Source:** SL-169 PIR S5; SL-165 PIR §2.3 (pattern). **Home:** RFC-004.

Compile-necessary fallout (CLI dispatch `cli.rs`, MCP tools, `reconcile.rs`,
sibling goldens) is correct but undeclared in `design-target` selectors; the
conformance algebra flags it at AUDIT — late, forcing a reconcile detour. Inverse
of IMP-162 (over-broad globs).

**Fix direction:** a design-time conformance dry-run — diff the design Code-impact
table against the planned/committed file set before lock; prompt for struct/field
change fallout (does this force edits in CLI/MCP/reconcile/sibling goldens?).
Natural fit for RFC-004's design-target accretion + actual-delta diff.

Related: RFC-004; IMP-162 (inverse over-declaration).
