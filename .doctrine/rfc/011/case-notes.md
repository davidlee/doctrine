
[close; SL-228-close-vh1]
`dispatch sync --prepare-review` halted twice on the conformance-completeness
gate, for two causes the handover had recorded as benign:

1. PHASE-08/09 read as "recorded row for a non-completed phase". Cause:
   `registry_completeness` derives the completed set from `completed_phase_ids`,
   which reads the PRIMARY tree's gitignored phase sheets — and the mid-drive
   appended phases were never mirrored there (edge's plan.toml has no such
   phases, so `slice phases` cannot materialise them). The handover called the
   mirror warning "benign, but misreads as a defect". It is not benign: it
   blocks prepare-review at close. Cost: ~6 tool calls reading
   `state.rs`/`dispatch.rs` to establish that the gate roots on primary runtime
   state rather than on plan.toml or the committed ledger.

2. PHASE-07 (evidence-only, deliberately non-funnel) is `completed` but carries
   no source-delta row. The gate has NO exemption for a phase whose delta is
   authored `.doctrine/` artefacts rather than source, so it can only be
   satisfied by a synthetic `Manual` row. The handover asserted the opposite
   ("nothing to record-delta for it") — an untested assumption written before
   prepare-review was ever run for this slice.

Root cause common to both: the completeness gate's inputs (primary runtime
sheets + primary registry + committed ledger on the dispatch ref) span three
tiers in two trees, and its refusal names only the symptom phase, not which of
the three disagreed. Per-phase, the refusal cannot distinguish "you forgot
record-delta" from "this phase's sheet never reached the primary tree".
Same family as ISS-241 and the D10 counter-example set.
