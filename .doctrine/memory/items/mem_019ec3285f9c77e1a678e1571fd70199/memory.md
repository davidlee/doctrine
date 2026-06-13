# Blanket module dead_code suppression masks sibling dead symbols — gate per-symbol

A module-level `#![cfg_attr(not(test), expect(dead_code, …))]` (e.g. a PHASE-01
"leaf stands up alone, consumers land later" staging suppression) does not
self-clear cleanly once consumers arrive: it blankets the WHOLE module, so it
keeps swallowing any *other* genuinely-dead symbol that appears later. The lint
exists to catch exactly that drift; a blanket suppression defeats it silently.

**Symptom:** code that is in fact only reachable from `#[cfg(test)] mod tests`
compiles clean in the production build with no warning, and so do unrelated truly
dead symbols, because one module-wide `expect` covers them all.

**Fix:** gate the genuinely test-only items with per-symbol `#[cfg(test)]`
(functions, associated consts, even the `use` import that only they need), and
delete the blanket module suppression. Then rebuild and triage every newly
surfaced warning: truly test-only → `#[cfg(test)]`; truly unused → delete;
consumed → it compiles clean. Observed in SL-059 (`src/knowledge.rs`): removing
the blanket expect unmasked four test-only symbols (`default_status` + three
facet-enum `KNOWN` drift-canary sets) on top of the render subtree it was meant
to cover.

Related: [[mem.pattern.lint.dead-code-self-clearing-leaf]],
[[mem.pattern.lint.dead-code-expect-vs-cfg-test]],
[[mem.pattern.lint.expect-not-allow]].
