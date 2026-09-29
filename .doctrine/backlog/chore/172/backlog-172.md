# CHR-172: Sample-based test quality review

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Intent

Audit test quality by depth, not breadth: choose a representative sample of
test modules across the codebase and review each one thoroughly.

## What to look for

- **Brittle** — asserts on incidental detail (exact prose, ordering, formatting,
  private structure) where the behaviour under test is narrower.
- **Vacuous** — passes regardless of the code under test: tautologies, asserts
  that can't fail, over-mocked seams, no assertion on the outcome.
- **Slow** — needless process spawns, real git/disk where a pure seam exists,
  repeated heavy fixtures.
- **Intent-opaque** — names and bodies that don't say what behaviour is pinned;
  setup noise drowning the assertion.
- **Duplication** — hand-rolled fixtures that an existing helper already covers.

## Sampling

Stratify by layer (ADR-001 leaf / engine / command), by test type (unit,
integration/CLI, golden), and by age/churn. State the sample and its rationale
in the review so coverage of the sample is auditable.

## Output

Findings on an RV (review) ledger via `/code-review`, severity-ranked. Follow-up
fixes become their own backlog items or a slice; systemic patterns become
test-helper improvements or a memory.

## Progress (2026-09-29)

- Review run as RV-411 (`doctrine show RV-411`). It raised 21 findings: 13
  major, 7 minor, 1 nit. The ledger is done, and every finding was routed as
  follow-up on user direction.
- Production defects surfaced and filed: ISS-500 (draft expiry sign), ISS-501
  (sha1-only ZERO_OID).
- ISS-008 appears stale (the test was renamed; 90 stressed runs were clean).
  Recommend closing it.

## Routing (fix work — tactical)

| home | RV-411 findings | what |
|---|---|---|
| SL-276 | F-2, F-17 | one hermetic shared git/test fixture layer |
| CHR-173 | F-1, F-7, F-9, F-10, F-11, F-12, F-15, F-19 | repair vacuous tests (after ISS-500) |
| ISS-500 / ISS-501 | F-1 / F-3 | production defects the tests hid |
| IMP-198 | F-4 | layering gate misses macro/sub-tier edges |
| IMP-502 | F-5 | `#[path]`-included suites run once per includer |
| IMP-503 | F-6, F-8, F-20 | slow e2e over the live corpus / network |
| IMP-504 | F-14, F-16 | brittle goldens, prose-coupled asserts |
| ISS-502 | F-13 | capsule conformance suite red and ungated |
| CHR-174 | F-18, F-21 | table-driven dedup, behaviour-stating names |

## Systemic analysis — input for the strategy session

The real point of CHR-172 is the next step: why these pathologies arise, and
how to stop producing them. Fixing the instances above is secondary. This
section is the durable evidence base for that session. **Facts** were measured
or verified in RV-411. **Hypotheses** are the reviewer's proposals (authority
tier 4/5, the reviewer's own judgement): test them, don't adopt them.

### Facts

- **Speed is not the problem.** 9,039 test executions, 2m04s wall (mostly
  compile), 166 s summed, 11 tests over 1 s. The slow tail is live-corpus or
  network e2e (IMP-503). Measurement recipe:
  `mem.pattern.testing.per-test-timing-stable`.
- **Vacuity is the dominant defect class.** 8 of 21 findings. Two of them hid
  live production bugs (ISS-500, ISS-501). Recurring shapes:
  1. A negative or zero assertion whose fixture drifted, so it can no longer
     fail. F-9: it searches slugs that are never rendered. F-10: it seeds a
     relation label SL-176 removed, so zero is always expected.
  2. A loop or comparison over a set that can be empty. F-7: `validate` is
     clean, so the loop never runs. F-6: `None == None`.
  3. A test named for function X that never calls X. F-1, and the `paths_*`
     tests in F-12, which copy X's body instead.
  4. An assertion on the test's own setup. F-12, F-15.
  5. A silent early return that still reports `ok`. F-11: 32 of 48 goldens
     under `DOCTRINE_WORKER`, on a dead premise. F-13: `eprintln` + `return`,
     hidden by libtest's output capture.
- **The engine rewards existence, not strength.** `doctrine slice verify-vt`
  is by its own help text an "existence/shape gate". It matches a VT
  criterion's `test_file` / `keywords` / `patterns` in source. A test that
  exists and matches passes the gate whether or not it can fail. Test names
  like `vt1_…` and `phase07_tests` are fingerprints of this incentive.
- **Tests rot across slices.** F-10's fixtures were correct when written and
  went vacuous when SL-176 and SL-222 changed the production rules. Nothing
  re-checks older tests when later slices change the code they cover.
- **Fixtures are copied, not shared.** 16 `init_repo` copies and 30 `fn git`
  copies in `tests/`, plus about 8 in-binary copies. Shared `common::git` has
  2 users. The copies have already drifted: memory TOML fixtures spell a
  non-key, and e2e_design_state's local fixture lost the PATH prepend.
  Contributing structure: the crate has no lib boundary, so integration tests
  `#[path]`-include source modules (IMP-404, IMP-502).
- **Tests are non-hermetic by default.** Production shells out to git with the
  full environment, and fixtures pin only dates
  (`mem.fact.testing.git-fixtures-not-hermetic`). Several tests read the live
  repo corpus for convenience (F-7, F-8, F-16's plugin check).
- **Goldens are made by pasting output.** e2e_review_golden's header says
  expected values came from running the binary, with "a behaviour that looks
  like a bug … pinned as-is". A golden then proves "unchanged", not "correct".
- **Structural guards fail open.** The layering gate silently skips unparsable
  files and ignores macro paths (F-4). The capsule suite's silent skip passes
  its own `#[ignore`-only audit (F-13). STD-003 (no silent skip) is not applied
  to test code.
- **The good modules share traits.** The strongest tests (review refusals, the
  boot merge planners, memory ambient-surface, the capsule verdict algebra)
  consistently use:
  - typed-error assertions;
  - "bytes unchanged after refusal" checks;
  - positive controls and anti-vacuity floors (assert the premise first);
  - injected env/clock inputs;
  - child re-exec for process-wide state.

  These are the patterns to codify.

### Hypotheses (for the strategy session to test)

- **H1: the incentive gradient.** The VT gate checks existence, and phase-scoped
  authoring means nobody re-reads old tests. Together they select for tests
  that exist and pass, not tests that can fail. Candidate lever: a strength
  signal on VT criteria, such as mutation testing (`cargo-mutants`) scoped to
  the phase diff, or a mandated "fails under mutation M" note per VT.
- **H2: absence assertions need a paired positive control.** Candidate lever:
  a convention plus a cheap lint. Flag `!contains` / `== 0` / a `for` over a
  collection with no preceding non-empty assertion / `return;` inside
  `#[test]`.
- **H3: copying beats discovery because discovery costs context.** An agent in
  a phase finds it cheaper to write a local helper than to locate the shared
  one. Candidate levers:
  - a lib boundary (IMP-404) so there is one import path;
  - a signposted test-support catalogue (memory or skill);
  - a review lens for new fixture types.
- **H4: hermetic by construction, not by care.** A hostile-env CI leg (global
  gpgsign/hookspath, `DOCTRINE_*` set) catches every leak cheaply. SL-276 may
  be the vehicle.
- **H5: golden policy.** Every golden names the behaviour it pins; no
  "as-is" bugs; incidental text normalised or parsed. IMP-196's live-corpus
  lint is part of this.
- **H6: STD-003 extended to tests.** A skip must be visible, or it must fail.
- **Open: where do levers live?** Doctrine the product (verify-vt, the
  code-review skill lens, shipped guidance) or this repo's conventions? The
  answer decides spec/ADR/RFC vs local policy. POL-002 and POL-003 (platform
  and harness independence) apply to anything shipped.

### Sample and method (so the sample can be extended)

The strata and units are in RV-411's `## Brief`. Each unit was read in full by
one reviewer. The lead verified every candidate against the cited lines
before raising it. Unsampled strata worth a second pass: `dispatch.rs`
(141 tests), `concept_map.rs`, `backlog.rs`, `slice.rs`, `src/commands/*`, and
the `cordage` crate.
