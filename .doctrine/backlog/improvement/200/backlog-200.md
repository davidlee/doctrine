# IMP-200: Per-kind golden regen on shared-renderer change

**Source:** SL-169 PIR S2 (golden fragility, HIGH). **Home:** RFC-005.

Shared renderers (`governance.rs` → adr/policy/standard/rfc; `listing.rs`) drive
multiple kinds' goldens, but goldens are per-kind. A conditional gate
(`any_tagged → splice tags`) makes the regenerating set *data-dependent*: only the
kind whose fixture trips fails visibly; siblings pass silently. SL-169 shipped a
`standard` regression masked because only standard's fixture was tagged — had no
fixture been tagged, the whole-class regression would have shipped undetected.

**Fix direction:** when a change touches a shared renderer / `COLUMNS` const,
regenerate ALL goldens routing through it; the design Code-impact table enumerates
the full set.

Related: RFC-005.
