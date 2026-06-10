# SL-033 Audit — Standard (STD) governance kind

Hand-authored (no scaffold yet). Conformance mode — post-implementation, tied to
SL-033. Reconciles the implemented phases against `design.md`, `plan.toml`, and
governance (SL-030 spine, ADR-001 layering, the storage rule).

## Mode & scope

Conformance. Two phases: PHASE-01 (STD kind end-to-end — `40ee376`) and PHASE-02
(boot parameterization + Active Standards — `b6499e1`). PHASE-01 was a near-
mechanical mirror of `policy.rs`; PHASE-02 is the only non-mechanical change
(design §3.5). Audit weight is on PHASE-02.

## Evidence

- `cargo test --bin doctrine` — **751 passed, 0 failed**.
- `cargo test --test e2e_standard_cli_golden --test e2e_standard_install_commit`
  — **14 passed** (10 + 4).
- `just check` — green (fmt + clippy + full test suite incl. e2e worktree).
- `cargo clippy` (bins/lib, no `--all-targets`) — zero warnings.
- Diff for PHASE-02 confined to `src/boot.rs` (`git diff --stat`: 1 file).

## Findings

### F1 — EX-1 SourceKind collapse — **aligned**

Expected (design §3.5 / plan EX-1): drop `Adrs`/`Policies`, add one
`GovRows(&'static GovKind, &'static [&'static str])` with a single `produce` arm
building `ListArgs { status: set.iter().map(...).collect(), ..default }`.
Observed: `src/boot.rs` `SourceKind` now carries `GovRows` + `Memories` (+ the
untouched `ExecPath`/`Static`/`Governance`); the two near-verbatim arms are one.
Matches the design's named identifier (`GovRows`, NOT `Governance` — that binds
the disk reader). Aligned.

### F2 — EX-2 boot_sequence binding + Active Standards placement — **aligned**

Expected: ADR→`["accepted"]`, POL→`["required"]`, STD→`["default","required"]`;
Active Standards immediately after Active Policies, before Memory. Observed:
exactly as written in `boot_sequence()`. VT-2
(`boot_sequence_orders_active_standards_after_active_policies`) asserts
ADRs < Policies < Standards < Memory and that Standards sits at Policies+1. Green.

### F3 — EX-3 behaviour-preservation gate (byte-identity) — **aligned**

Expected: ADR + Policy section bytes unchanged; pre-existing boot ordering +
filtered-projection tests pass UNCHANGED. Observed: the single-element GovRows
set reproduces the old per-kind arms by construction (design §3.5 "byte-identity
by construction") — `list_rows` is called with identical inputs. VT-1 set
(`boot_sequence_orders_exec_path_last`,
`..orders_active_policies_after_accepted_adrs`,
`regenerate_projects_accepted_adrs_and_memory_pointers`,
`regenerate_projects_required_policies_filtered`) passes unedited. Aligned.

### F4 — EX-4 no spine / CLI overreach — **aligned**

Expected: no `governance.rs` change, no clap/CLI change; confined to `boot.rs` +
the `standard` module reference it binds. Observed: PHASE-02 diff is one file
(`src/boot.rs`); the only new coupling is `use crate::{… standard}` + the
`STANDARD_KIND` reference. Spine and CLI untouched. Aligned.

### F5 — VT-3 two-element in-force set — **aligned**

Expected: a test seeds standards across all five statuses and asserts Active
Standards projects ONLY `default` + `required` (STD- prefixed), excluding
draft/deprecated/retired — the only proof of the multi-status set (D3).
Observed: `regenerate_projects_in_force_standards_filtered` asserts `STD-002
default` + `STD-003 required` present, and `a-draft-rule`/`an-old-rule`/
`a-dead-rule` absent from the section body. Green. Aligned.

### F6 — VT-4 the gate bites — **aligned**

Expected: a one-char edit to the GovRows set reds a projection test. Observed:
empirically verified — dropping `default` from STD's set
(`&["default","required"]` → `&["required"]`) reds
`regenerate_projects_in_force_standards_filtered`; reverted. The gate is real,
not vacuous. No standalone test needed — VT-4 is a meta-property of VT-3, which
asserts both set members. Aligned.

### F7 — adjacent fixture update (`boot_check_reports_clean_when_populated…`) — **aligned**

Not in VT-1's named gate set. The new Active Standards section made this sentry
test red (its "fully populated" fixture seeded no standard → marker → not-clean).
Reconciled by seeding one `required` standard, preserving the test's intent
(no markers when every section is populated). This is a fixture extension for a
new section, not a behaviour change to ADR/POL projection. Aligned.

### F8 — Codex findings (design §8.1) — **aligned, PHASE-01 scope**

The six external-adversarial findings (install-conformance VT-5, worker-guard
VT-6, list-conformance VT-7, golden tightening, etc.) target the PHASE-01 kind
module + conformance suites, all landed in `40ee376`
(`e2e_standard_cli_golden`, `e2e_standard_install_commit`, worker-guard +
list-conformance extensions). Suites green this audit. Aligned — no PHASE-02
exposure.

## Lifecycle divergence (for /close)

`slice list` shows `SL-033 proposed ⚠ 2/2` — both phases complete, but the
hand-edited `slice-033.toml` status still reads `proposed`. This is the known
lifecycle-transition gap (no verb moves a slice proposed→done), NOT a defect.
`/close` reconciles the authored status against the 2/2 rollup.

## Disposition summary

All findings **aligned**. No fix-now, no design-was-wrong, no tolerated drift,
no follow-up slice. The inherited/shared gaps (boot error≡empty marker collapse,
supersession⇏status, inert `--tag`) remain consciously deferred per scope
Non-Goals / SL-030 §5.5 — STD inherits them unchanged, out of scope here.

## Harvest

No new durable memory — PHASE-02 is a clean application of the existing
numbered-kind / governance-spine patterns already captured in memory
(`mem.pattern.entity.numbered-kind-identity-table`,
`mem.pattern.authoring.reuse-tuned-prior-art-verbatim`). Nothing in `notes.md`
to add beyond design/plan. Audit-ready for `/close`.
