# Locked-design migration-fallout count goes stale on shared main; re-scan + F-G cutover re-eval at execution, and the stray-key-table disposition

A corpus-migration slice that enumerates its fallout at DESIGN time is stale by
EXECUTION time — concurrent authoring on shared `main` keeps minting new fallout
from the not-yet-fixed source. The execution-start re-scan + cutover re-evaluation
is load-bearing, not ceremonial.

**SL-058 PHASE-02 proof:** the design knew "10 backlog + SL-056, 1 populated key
(IMP-045)". The EN-2 re-scan found:
- a SECOND populated migrated key (IMP-052, identical `slices=["SL-056"]`) → the
  F-G cutover rule literally fired (>1 populated). Verdict was still hand
  link+strip — 2 trivial identical edges don't justify building a migrator; the
  threshold is a STOP-and-decide prompt, not an auto-switch.
- a whole unanticipated entity class: **SL-054 (done/terminal) hand-authored a
  stray-key `[relationships]` table** (`extends=[53]`, `adrs=[1]`) — NON-vocabulary
  keys, absent from `RELATION_RULES`, so not `link`-writable and invisible to every
  relation reader.

**Stray-key-table disposition (reusable):**
1. Convert each key that maps to a legal `RELATION_RULES` label to a `[[relation]]`
   edge via `doctrine link` (e.g. `adrs=[1]` → `link SL-054 governed_by ADR-001`).
2. Comment out / demote to prose the freestyle remainder with no legal label
   (e.g. `extends` — no slice→slice label exists).
3. Drop the typed `[relationships]` table entirely (slices are table-absent
   post-cut, F-E strict).

**Parser caveat:** the corpus oracle's `view()` in
`tests/e2e_relation_migration_storage.rs` only sees a `[relationships]` header
after the comment-strip hardening (F-C) — an inline-comment header
(`[relationships]   # outbound-only`) evades a bare `line == "[relationships]"`
match, so pre-hardening the fallout reads as a vacuous green. Re-scan with grep,
not the unhardened parser.

Related: [[mem.system.coordination.concurrent-design-shared-main-worktree]],
[[mem.pattern.dispatch.verify-governance-freshness-before-distilling-worker]],
[[mem.pattern.relation.authored-rows-tooling-half-wired]].
