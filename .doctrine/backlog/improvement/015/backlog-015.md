# IMP-015: boot --check flags empty governance sections (Active Policies/Standards) as unpopulated

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced as a pre-existing condition during SL-034 (design R4). `doctrine boot
--check` reports `unpopulated sections: Active Policies, Active Standards`. These
sections are empty because no policy/standard entities have been authored yet —
the `<!-- … not yet populated -->` markers are the intended empty-set state, not
corruption. But the sentry flags them, so a clean repo with no policies always
reads as "drift", which is noise.

Question for the fix: should an *unpopulated* governance section count as a
sentry failure at all? Likely the empty-set marker should be a clean terminal
state, distinct from a *stale* or *missing* section. Decide the intended
`doctrine boot --check` semantics, then either suppress the empty-set case or
document it as expected.

Not folded into SL-034 (kept the boot regen reviewable). Distinct from IMP-007
(memory-section trim).
