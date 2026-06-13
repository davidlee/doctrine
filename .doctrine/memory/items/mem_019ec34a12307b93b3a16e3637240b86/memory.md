# Migration oracle restates expected vocab; never derive from the SSoT under test

`tests/e2e_relation_migration_storage.rs` is the SL-048 migration ORACLE: it reads
the on-disk corpus directly and asserts the post-cut relation shape. Its legal-label
and migrated-axis allow-lists (e.g. slice `["specs","requirements","supersedes",
"governed_by"]`) are written as **literals on purpose** — they are NOT derived from
`RELATION_RULES` (`src/relation.rs`, ADR-010).

Why: `RELATION_RULES` is the production source of truth that DRIVES the migration. An
oracle that derived its expectations from the same table would agree with any
vocabulary bug the migration itself shipped — it would rubber-stamp the SUT instead of
pinning it. Independence is the whole point of a corpus oracle (render goldens already
launder row order through the BTreeMap regroup).

A `/code-review` pass flagged the literals as "transcription — the anti-pattern
`using-doctrine.md` forbids". That guidance targets entity AUTHORS choosing edge
labels, not oracle tests; conflating the two is a category error. Correct resolution:
keep the literals, add an oracle-independence note + `SSoT: RELATION_RULES` pins so a
vocab evolution stays greppable. The coupled (derive-from-table) view is already
covered by relation.rs's own unit tests (`sources_match_shipped_accessors`).

Corollary: the "kept-typed" axes (needs/after/triggers, tags, supersedes/superseded_by)
are not in `RELATION_RULES` at all — they are the typed-tier complement and can ONLY be
literals.

Related: [[mem.pattern.review.guard-test-asserts-property-not-proxy]],
[[mem.pattern.relation.relate-via-link-not-hand-authored-rows]].
