# SL-022 audit — decomposition spine & integrity checks

Conformance audit (post-implementation). Reconciles the implemented four phases
against `design.md` (canonical), `plan.toml` (EX/VT authoritative), and the
resolved adversarial history (§6a, §10: codex F1–F6, inquisition A–H). No audit
scaffold yet — hand-authored sibling of `design.md`.

## Mode & evidence baseline

- **Mode:** conformance, tied to SL-022.
- **HEAD:** `7064d0b` (PHASE-04 closure). All four phases DONE; rollup 4/4.
- **Gate (re-run this audit, not trusted from handover):**
  - `cargo test --bin doctrine` → **546 passed, 0 failed**.
  - `cargo clippy --bin doctrine` → **zero warnings**.
  - `just check` → **green** (fmt + clippy + unit + e2e; install/skills e2e pass).
- **Code review:** completed this pass, verdict **solid**. No blocking findings.
  Findings dispositioned below.

## Criteria reconciliation (EX / VT → evidence)

| Criterion | Evidence | Disposition |
|---|---|---|
| P01 EX-1/VT-1 spine fields parse Some/None | `tech_spec_parses_descent_and_parent_when_present`, `product_spec_toml_defaults_tech_flat_fields`, `*_defaults` None asserts | aligned |
| P01 EX-2/VT-2 render kind-gated, ordered, no children | `render_emits_descent_and_parent_for_tech_in_order` (asserts c4<descends<parent<resp, no "children"), `render_omits_descent_and_parent_when_none_and_for_product` | aligned |
| P01 EX-3/VA-1 scaffold emits comments (re-embed) | live `doctrine spec new tech` → `# descends_from` / `# parent` present in compiled binary's scaffold | aligned (verified live) |
| P01 EX-4/VT-3 existing spec.rs green, only `None,None` constructor edits | deletion-grep: sole existing edits are mechanical constructor fills; suite green | aligned |
| P02 EX-1/VT-4 registry edges + product set, `on_product`, canonicalised | `build_registry_harvests_product_set_and_relational_edges` | aligned |
| P02 EX-2/VT-1 descent 4-way | `descent_clean…`, `descent_dangling…`, `descent_to_tech_target_is_invalid_kind`, `descent_on_product_subject_is_invalid_kind` | aligned |
| P02 EX-3/VT-2 parent 4-way + self excluded | `parent_clean…`, `parent_dangling…`, `parent_to_product_target_is_invalid_kind`, `parent_on_product_subject_is_invalid_kind`, `parent_self_case_is_excluded_owned_by_self_parent` | aligned |
| P02 EX-4/VT-3 interaction split (REQ-084) | `product_interaction_target_is_invalid_kind_not_dangling` (2 findings, distinct msgs) | aligned — **intended behaviour change**, see F-1 |
| P02 EX-5/VT-5 only `clean()` literal edit | `..Default::default()` (absorbs P03 `build_findings` too); no assertion value moved | aligned |
| P02 VT-4 Layer C(iv) malformed-toml surfaces | `build_registry_surfaces_a_malformed_spec_toml` (real seam) | aligned — Charge I widening, see F-2 |
| P03 EX-1/VT-1 self + cycle dedup, count==1, tail-fed | `self_parent_reports_a_to_a_once`, `parent_cycle_two_node_reports_once`, `…three_node…`, `…tail_feeding_a_ring_reports_the_ring_once`, `…clean_chain_to_root…`, `…scoped_to_a_member_node…` | aligned (count-asserting, not existence) |
| P03 EX-2/VT-2 second_parent classifier + carrier | `second_parent_duplicate_key_surfaces_end_to_end`, `…array_value…`, `second_parent_classifier_*` | aligned — R2 fragility, see F-3 |
| P03 EX-3/VT-3 commented `# parent` no false-hit | `scaffold_commented_parent_does_not_trip_second_parent` (real `fresh()` scaffold) | aligned |
| P03 VT-4 A→A one finding total | `self_loop_yields_exactly_one_finding_across_both_checks` | aligned |
| P04 EX-1/VT-1 CLI sweep non-zero per violation, zero clean | `assert_validate_flags` ×11 + `sweep_clean_corpus_exits_zero` | aligned — substring precision, see F-4 |
| P04 EX-2/VA-2 REQ-082 AC3 by construction | id-only `Option<String>`, no prose slot | aligned (VA, not a gate), see F-5 |
| P04 EX-3 clippy zero, storage rule (no derived persisted) | gate green; cycle inversion ephemeral in `parent_cycle` | aligned |

## Findings & dispositions

### F-1 — REQ-084 interaction contract move (intended behaviour change)
- **Expected (design §5.2, §3, PRD-012 §6):** an interaction target that is a
  product spec is *invalid kind*, not *dangling*.
- **Observed:** `dangling_interaction_targets` rewritten; SL-015 test
  `non_tech_interaction_target_is_flagged_tech_only` → rewritten as
  `product_interaction_target_is_invalid_kind_not_dangling`.
- **Evidence:** deletion-grep confirms this is the *only* existing assertion whose
  value changed; all other existing-test edits are mechanical (`None,None` /
  `..Default::default()`).
- **Disposition:** **aligned.** The behaviour-preservation gate guards *unrelated*
  machinery from *accidental* change; this is a deliberate, spec-mandated contract
  move, disclosed in commit + design §9/§10-E + notes. Not a gate breach.

### F-2 — Charge I error-surface widening (intended)
- **Expected (design §5.3):** `build_registry` now parses each `spec-NNN.toml`
  (none before); a malformed spec toml that `validate` never opened will now fail
  the build.
- **Observed:** new per-spec `read_to_string` + `toml::from_str::<Spec>`; malformed
  → `Failed to parse` context error.
- **Evidence:** `build_registry_surfaces_a_malformed_spec_toml` proves it through
  the real seam (not the hand-built-registry suites, which bypass it).
- **Disposition:** **aligned.** Owned, intended widening; proven, not assumed.

### F-3 — second_parent classifier is toml-version-fragile (R2)
- **Expected (design §5.2 `second_parent`, §6a):** classify the `toml::from_str`
  error (span enclosing-line key == `parent` AND message contains `duplicate key`
  / `invalid type: sequence`); degrade safely on miss (non-zero, never silent).
- **Observed:** `is_second_parent` (spec.rs) matches on toml 0.8.23 message text —
  no stable error-kind enum. A plain `parent = 5` intentionally falls through to
  `Failed to parse`; `slug = []` does not false-hit (span attribution required, as
  the array message omits the key).
- **Evidence:** `second_parent_classifier_matches_{duplicate,array}_parent`,
  `…ignores_unrelated_parse_errors` (the `parent = 5` / dup-other-key / `slug = []`
  trio). These are the canary if a toml bump shifts the shapes.
- **Disposition:** **tolerated drift.** Conscious tradeoff: no stable toml error
  taxonomy exists; the named diagnostic (literal REQ-087 AC1) is worth the
  fragility, the scalar field is defense-in-depth, and a classifier miss degrades
  to `Failed to parse` (non-zero, never a silent pass). Pinned by canary tests.
  **Sub-note (cascade noise):** on a second_parent hit, `build_registry`
  `continue`s *before* `tech_specs.insert` and the member/interaction harvest, so
  the malformed spec drops out of `tech_specs` — other specs naming it as
  parent/interaction target then read as *dangling*. Louder than the root cause,
  but only on an already-broken corpus the user must fix first. Accepted; logged
  for IMP backlog should classifier coverage ever expand.

### F-4 — sweep assertion substrings under-discriminate (test precision)
- **Expected (design §9, VT-1):** each crafted corpus proves its *intended* check
  fired, not merely that some error did.
- **Observed:** `sweep_parent_invalid_kind_product_target` and
  `sweep_interaction_invalid_kind_product_target` both assert the *shared* suffix
  `"is a product spec (must be tech)"`. Correct only because each corpus is
  single-violation; the assertion itself does not name the check.
- **Evidence:** message-text inspection — parent and interaction invalid-kind
  findings share the suffix; the prefixes (`invalid parent:` /
  `invalid interaction target:`) differ and would discriminate.
- **Disposition:** **tolerated drift** (test-only). The corpora are minimal and
  single-violation, so today the tests cannot pass for the wrong reason. Hardening
  (assert the discriminating prefix) is a cheap future improvement, not a
  correctness gap. No production impact.

### F-5 — REQ-082 AC3 satisfied by construction (VA, not VT)
- **Expected (design §9, F4):** "`descends_from` does not restate product intent"
  is authoring discipline — no machine gate.
- **Observed:** `Spec.descends_from` / `Spec.parent` are `Option<String>` (id only,
  no prose slot to restate).
- **Disposition:** **aligned.** Satisfied-by-construction; `VA` review check,
  correctly *not* a code gate (the slice's "every requirement → a test" is
  corrected to "every machine-checkable AC → a test").

### F-6 — VA-1 scaffold re-embed
- **Disposition:** **aligned.** Verified live this audit: the compiled binary's
  `doctrine spec new tech` scaffold emits the `# descends_from` / `# parent`
  comment lines (rust-embed footgun cleared).

## Edge-case observations (review, no action)
- Product spec carrying a *duplicate* `parent` key classifies as
  `second parent: PRD-NNN …` rather than invalid-kind-on-product — a hard finding
  either way (non-zero exit), but the message masks the deeper "parent on product
  at all". Cosmetic; product hierarchy is out of scope (§5.2 F5).
- The new `spec-NNN.toml` read has no `NotFound` tolerance, unlike its
  `members.toml`/`interactions.toml` siblings; a scanned spec dir missing its toml
  now hard-fails the build. Defensible (malformed corpus), inconsistent with the
  tolerant sibling seams. No action.

## Doctrinal alignment (re-checked)
- **ADR-001** (registry pure leaf): `build_findings` is inert data populated only
  by the impure `build_registry`; `registry.rs` runs no clock/rng/git/disk. Held.
- **ADR-004** (outbound-only, derived reciprocity): no children/reverse view
  stored or rendered; cycle inversion ephemeral. Held.
- **Storage rule:** no derived/queried data persisted. Held.

## Closure handoff
- All EX/VT criteria **aligned**; F-3 and F-4 recorded as **tolerated drift** with
  rationale (no follow-up slice warranted — F-3 is irreducible toml fragility with
  canary coverage; F-4 is a test-only precision nicety). No **fix-now** findings.
- **Lifecycle:** `slice-022.toml` `status` is still `proposed` → `slice list`
  shows `⚠` vs the 4/4 rollup. Reconciling that is **`/close`'s** job, not this
  audit's. Audit-ready for `/close`.
