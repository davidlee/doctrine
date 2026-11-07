# IMP-526: Surface inquiry growth by relay, not graph-review staleness

Implements DEC-421 (partially replaces DEC-302's coverage clause). Small-item
route, agreed with the user 2026-11-08 instead of a slice.

## Scope

1. **Gate.** `graph-reviewed` (`initial-concerns-recorded`) compares node
   material over carried keys only — drop the full-set blocking-marks conjunct
   (`ContentCoverage::reviewed_graph_moved`, `src/design_run/attestation.rs`).
   A flipped judgement on a covered node still stales it.
2. **Relay names the added ids** and the consultation rule (don't disposition
   an inquiry you added without consulting the user).
3. **Sidecar mode owes a line too** — tell the user to look at the tree and note
   the new ids (`relay()`, `src/design_run/mod.rs`; `relay_line`,
   `src/design_run/render/tree.rs`).
4. **`/design` skill** states the same contract; regenerate shipped docs
   describing the gate.

Out of scope: per-actor provenance on dispositions (who answered an inquiry);
the consultation rule stays behavioural.

## Verification

- Flipped tests: adding a blocking node leaves the graph review current.
- Relay tests: added ids named; sidecar line emitted on map change; no line
  when nothing was added and the map is unchanged.
- `doctrine check gate` green.

## Notes

- **Scope change (agreed in principle by item 1):** rather than keep
  `Coverage::ReviewedGraph` as a second name for the same comparison, it is
  removed; `initial-concerns-recorded` binds `Coverage::InquiryMap`. The token
  `reviewed-graph` was never in a stored snapshot (checked
  `.doctrine/state/slice/*/design.toml`: 0 hits), only in the generated
  `install/design-run-stages.md`, now regenerated. Dead helpers removed with it:
  `reviewed_graph_moved`, `blocking_marks`, `NodeMaterial::effective_blocking`,
  `judged_blocking` (inlined into `InquiryNode::effective_blocking`), and the
  `legacy` parameter of `CoveredSet::moved`.
- **Relay:** `relay()` now emits in both modes and names ids present in `next`
  but not `prior`. On `design start --from-design` every imported node is
  "new", so the consultation clause also appears there — acceptable (the user's
  own OQs), noted rather than special-cased.
- **Guidance:** contract stated in `install/design-prompts/inquiry.md` (the
  fragment the run serves during inquiry), not the `/design` SKILL.md.
- **Tests:** unit — `adding_a_blocking_node_stales_neither_attested_row`,
  `relay_is_owed_in_both_modes_on_a_map_change`,
  `relay_names_added_inquiries_and_the_consultation_rule`; change-log agreement
  cases flipped. e2e — `map_growth_after_sufficiency_re_faces_no_human_gate`
  (renamed), `sidecar_mode_sends_the_user_to_the_tree` (was
  `sidecar_mode_prints_no_relay_line`), relay-line assertions updated.
- `doctrine check gate` green; clippy and fmt clean.
- **RV-430 (Opus code review):** 6 findings (4 minor, 2 nit), all fixed. Code
  change: the consultation clause now names only agent-proposed added ids, and
  sidecar asks the agent to note the new ids. The rest are records, comments
  and one redundant test act. The import special case noted above is gone
  (imported nodes carry no consultation clause).
