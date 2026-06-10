# Audit SL-030: Policy entity kind (POL)

Conformance audit (post-implementation, all 4 phases `completed`). Hand-authored —
no `slice audit` scaffold yet. Reconciles the frozen implementation against
`design.md`, `plan.toml` EX/VT criteria, and the relevant ADRs. Sibling of
`design.md`; durable, tracked.

## Mode & gate

- **Mode:** conformance. Implementation frozen; this is evidence + reconciliation,
  not new code.
- **Gate:** `just check` exit 0 — fmt + plain `cargo clippy` (zero warnings) +
  full suite (747 tests) + build. Re-run during this audit, green.
- **Behaviour-preservation proof (R1):** `tests/e2e_adr_cli_golden.rs` last touched
  at `8607e12` (PHASE-01, authored *before* the extraction); byte-untouched by the
  PHASE-02 spine move. The 10 black-box goldens for `adr show/status/list` are the
  executable proof the ADR CLI surface is byte-identical pre/post extraction.

## Per-phase verification

### PHASE-01 — ADR CLI golden net
- **EX-1/EX-2/EX-4 — ALIGNED.** `e2e_adr_cli_golden.rs` pins `adr show` (Table+Json),
  `adr status` (transition/no-op/malformed-refuse), `adr list` (byte-exact stdout +
  JSON rows, hide-set + ordering + prefix + header) over a hand-seeded fixed-date
  fixture. Committed `8607e12` as the migration baseline.
- **EX-3 — ALIGNED.** `src/` untouched in `8607e12` (tests-only phase).
- **VT-1/2/3 — ALIGNED.** Mutate-to-red proven (notes P1): em-dash→hyphen in
  `format_show` and `"id"`→`"ID"` header edit redden the goldens; reverted.

### PHASE-02 — governance.rs spine; ADR migrated
- **EX-1 — ALIGNED.** `src/governance.rs` at command tier; depends downward on
  `entity`/`meta`/`listing`, sideways on `root`/`clock`. No engine/leaf depends on
  it (ADR-001 layering preserved, no cycle). Production edge stays `adr→governance`
  one-way; the `crate::adr::ADR_KIND` import in relocated tests is cfg(test)-only.
- **EX-2/EX-3 — ALIGNED.** `adr.rs` reduced to thin kind (descriptor + `AdrStatus`
  + known-set + `is_hidden` + scaffold + 7 forwarders). `GovKind` is the 4-field
  struct {kind, stem, statuses, hidden}; `json_label` dropped per MINOR-7.
- **EX-4 — ALIGNED.** `boot.rs` rebinds `adr::list_rows` → `governance::list_rows
  (&ADR_KIND, …)`; `regenerate_projects_accepted_adrs` green (2nd preservation
  signal).
- **VT-1 — ALIGNED (the proof):** PHASE-01 goldens pass UNCHANGED through the move.
  **VT-2/3 — ALIGNED:** relocated unit tests green via ADR descriptor; gate green.

### PHASE-03 — policy.rs thin kind + 3 install surfaces
- **EX-1 — ALIGNED.** `doctrine policy new|list|show|status` ride the spine; ids
  `POL-NNN`; vocab `draft/required/deprecated/retired`, `deprecated|retired` hidden.
- **EX-2 — ALIGNED.** `install/templates/policy.{toml,md}` rust-embedded; toml
  seeds `draft`; md = supekku Statement/Rationale/Scope/Verification/References,
  no YAML frontmatter (storage rule).
- **EX-3 — ALIGNED.** Three surfaces wired: manifest `[dirs].create`, `.gitignore`
  `!.doctrine/policy/`, parity. `e2e_policy_install_commit.rs` (VT-2) proves a
  scaffolded `policy-001.toml` is committable AND — negative control — ignored
  without the negation.
- **EX-4 — ALIGNED.** `policy_known_set_matches_variants` drift canary pins
  `POLICY_STATUSES` ↔ `PolicyStatus`.
- **VT-1/2/3 — ALIGNED.** 8 unit + 4 e2e green; mutate-to-red (`draft`→`drafx`)
  reddens round-trip; reverted. **R3 (stem≠prefix) discharged:** POL is the first
  kind to break ADR's `stem==prefix.to_lowercase()` coincidence — files
  `policy-NNN`, ids `POL-NNN`, JSON key `policy` all independent.

### PHASE-04 — project required policies into boot
- **EX-1 — ALIGNED.** `Active Policies` section renders after `Accepted ADRs`,
  before `Memory` (ExecPath last); projects `governance::list_rows(&POLICY_KIND, …,
  status:["required"])`. Pinned by `boot_sequence_orders_active_policies_after_
  accepted_adrs` (`boot.rs:869`) — section == ADRs+1 and < Memory.
- **EX-2 — ALIGNED.** Empty/absent corpus → `not yet populated` marker
  (`regenerate_empty_policy_corpus_renders_marker`, `:1177`).
- **EX-3 — ALIGNED.** `boot --check` covers the section via the same recompute;
  markers informational (exit 0), only `stale` is the hard signal.
- **VT-1/2/3 — ALIGNED.** `regenerate_projects_required_policies_filtered`
  (`:1120`) — required shows; draft/deprecated/retired absent, scoped to the
  section body. Gate green; `boot --check` clean.

## Code-review findings (PHASE-04 diff, reviewed post-impl)

- **Action 1 (🟡 EX-1 order pin) — FIX NOW, DONE `f4048ae`.** Order was unverified
  (only ExecPath-last pinned). Added the order test above. *Confirmed resolved.*
- **Action 3 (🟡 section-scoped negatives) — FIX NOW, DONE `f4048ae`.** Absence
  loop scanned the whole snapshot; scoped to the section body via `split_once`.
  *Confirmed resolved.*
- **Action 2 (🔵 boot per-kind DRY collapse) — FOLLOW-UP SLICE (SL-033).** boot's
  `SourceKind` variant + match arm is a near-verbatim clone per kind. NOT a SL-030
  defect — scope was "mirror the ADR arm" (design §2, scope Non-Goals). Collapse to
  data-carrying `SourceKind::Governance(&GovKind, status_filter)` is **carried in
  SL-033 scope** (`slice-033.md:35-43`), confirmed. Correct route: the second rider
  (STD) is what justifies the abstraction (R3 ≥2-kinds rule).

## Inherited / shared gaps (design §5.5 — confirmed still out of scope)

Each is pre-existing shared boot behaviour, ADR-identical, design-acknowledged, and
inherited by SL-033 (`slice-033.md:51-52`). **Disposition: tolerated drift** —
consciously accepted, rationale in design §5.5/§6, follow-ups captured there.

- **error≡empty marker collapse (MAJOR-4):** `boot::section_or_marker` renders a
  producer `Err` and a genuinely-empty listing identically (`not yet populated`),
  hiding a malformed corpus. boot-wide concern; `boot --check` disk sentry is the
  backstop. §6 follow-up.
- **supersession⇏status double-show (MAJOR-5):** a `required` policy named in
  another's `supersedes` still projects into Active Policies — ADR-identical.
  Authored-discipline invariant (move to `retired`); no `policy supersede` verb in
  v1 (parity with ADR's unbuilt verb). §6 follow-up.
- **inert `--tag` axis (BLOCKER-2→MAJOR):** `key` returns empty tags; `meta` never
  reads them — POL inherits ADR's no-op `--tag`. Not claimed as supported; real tag
  reader is a §6 follow-up.

## Nit recorded (not a defect)

- `src/boot.rs:1319` — `// PHASE-05 — boot --check disk sentry` banner, but the plan
  has only PHASE-01..04; `boot --check` landed under PHASE-04 EX-3. **Stale label.**
  Cosmetic, test-only comment; no behaviour impact. Left as-is (an edit would be the
  audit touching frozen code for zero functional gain) — recorded here so the next
  reader knows it is mislabelled, not missing work.

## Durable risks harvested

- **R1 (extraction regresses ADR) — discharged.** The author-goldens-first pattern
  (pin the surface before the refactor, hold green through it) is the reusable
  safeguard; captured in `mem_019eae92…` (black-box CLI goldens) + notes P1/P2.
- **R3 (over-abstraction) — discharged.** Every `GovKind` field exercised by ≥2
  kinds (ADR+POL) from day one. The deferred Action 2 follows the same discipline:
  the abstraction lands with STD as the second boot rider, not speculatively now.
- **Untracked-file revert footgun** (notes P2): `git checkout <untracked>` is a
  silent no-op — a mutate-to-red probe on new `governance.rs` did not revert.
  Re-grep after manual reverts on new files. Candidate for memory if it recurs.

## Closure readiness

`audit.md`, `design.md`, `notes.md`, and SL-033 scope tell a coherent closure
story. Every EX/VT aligned; every code-review finding dispositioned; inherited gaps
consciously tolerated with captured follow-ups. The only open item is the lifecycle
⚠ (`slice-030.toml status = "proposed"` vs the 4/4 rollup) — reconciled by `/close`.
Audit-ready.
