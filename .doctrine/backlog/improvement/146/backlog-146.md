# IMP-146: SL-116: distribute remaining lifecycle tests per D2

D2 requires each machine file to carry its own `#[cfg(test)] mod tests`.
Only allowlist.rs (10 tests), marker.rs (6), and shared.rs (2) got theirs.
The 7 lifecycle machine files (provision/import/land/gc/fork/coordinate/
subagent) have zero `#[cfg(test)]` blocks — ~32 tests remain in mod.rs's
monolithic test block.

Extract them per the D2 per-machine co-location design, keeping bodies
byte-identical. This is a post-close cohesion cleanup.

Source: RV-135 F-3 (SL-116 reconciliation audit).
