# Review RV-404 — reconciliation of SL-271

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed.** SL-271 after the design back-edge and PHASE-04, head
`3be4df419` (the retirement) plus `7c5ef368c` (notes) on `edge`. This is a
*re-audit*: RV-403 closed with F-1 (blocker, design-wrong) escalated to the
design back-edge, and PHASE-04 is its remedy. The pass holds the remedy to the
ratified option A and re-runs the mechanical evidence.

Mode: **conformance** (post-implementation, tied to the slice). Facet
`reconciliation`, self-audit driving both roles; `review prime` hashed 129
selector paths from the primary tree.

**Lines of attack.**

1. **Is the probe actually gone, and the read actually the file?** Grep the
   retired symbols to exhaustion; confirm no codex subprocess remains on the
   install path, and that `codex_project_hooks` reads the same
   `.codex/config.toml` the MCP leg writes.
2. **Does the new branch cover the RV-403 F-1 case?** A project file with
   `[features] hooks = false` (or no key) must still print step 1; `= true` must
   omit it. Check the e2e fixture, not the prose.
3. **Re-run the mechanical gates.** `slice conformance`, `slice verify-vt`,
   `doctrine check gate`, the focused suites.
4. **Did the retirement leave a red with no disposition?** PHASE-03's VT
   mandates pin the retired symbols; the plan is immutable-append, so the record
   must live in design sec-9.
5. **Is the disclosure still honestly scoped?** The project file is the signal
   the notice names; a user-layer override is out of doctrine's sight (design
   sec-6).

**Evidence run** (primary tree): `slice conformance 271` (4 conformant, 2
undeclared, 0 undelivered), `slice verify-vt 271` (PHASE-01/02/04 PASS; PHASE-03
VT-1..VT-4 FAIL — retired symbols), `doctrine check gate` (exit 0), `cargo test
--test e2e_codex_install` (12 pass), `--bin doctrine codex` (21 pass), `--test
architecture_layering` (25 pass).

## Synthesis

**Overall: acceptable.** PHASE-04 does what option A ratified: the `codex
features list` subprocess and its capture seam are gone, and the activation
notice reads `[features] hooks` from the project `.codex/config.toml` the MCP
leg already owns. The behaviour the blocker was about is now correct at its
source: a project that disables hooks prints step 1, because doctrine reads the
file it is actually telling the operator to edit. `RV-402` F-5 is obsoleted —
no codex subprocess remains, so README's Host-dependencies sentence stays
`sh`-only.

Two findings are not clean, and neither is a defect in the remedy:

- **F-1 (major, design-wrong)** is the audit discovery that a *retirement*
  leaves its predecessor's VT mandates mechanically unsatisfiable. PHASE-03's
  VT-1..VT-4 name `parse_codex_features`, `codex_hooks_state`, `Unknown` and
  `HooksState`; PHASE-04 deletes them. The plan criteria are immutable-append, so
  the mandate cannot be re-pointed — the disposition belongs in design sec-9,
  which the reconcile pass writes. The retirement is correct; the record is the
  gap.
- **F-2 (minor, fix-now)** was a real test gap caught by the VT gate: PHASE-04's
  VT-3 mandate named `Ensure [features] hooks`, which the e2e case did not
  exercise (it covered `false` and `true`, not the absent-key fallback). Fixed
  in-audit; the gate now reads PHASE-04 4/4.

Standing risks, consciously carried: `verify-vt` will read PHASE-03 red until
close, and that is a recorded supersession, not drift. The project file's value
is disclosed rather than the effective value — a user-layer override is the
operator's and out of doctrine's sight (design sec-6, ratified with option A).
And `design.md` is diverged from the locked run `dr-01a0e17c` rev 44, the
expected out-of-band edit at the tail.

Credit: the retired seam had no other consumer (grep to exhaustion), the
`DocumentMut` read is the house pattern, and the four `wire` call sites plus the
two production call sites were all updated in one commit.

## Reconciliation Brief

### Per-slice (direct edit)

- **design.md sec-9 "Verification alignment"** — `RV-404` F-1. Record that
  PHASE-03's VT-1..VT-4 are satisfied at execution and superseded by PHASE-04's
  VT-1/VT-2, so `doctrine slice verify-vt 271`'s current-tree FAIL for PHASE-03
  is expected. The plan criteria are immutable-append; the record lives here.

### Governance/spec (REV)

- None. `REV-066` (the two-member SPEC-011 Revision) is unaffected by the
  retirement: it describes the registration leg, not the hooks disclosure.

### Carried from RV-403 (resolve and record at reconcile)

- `RV-403` F-1 (blocker, design-wrong) — resolved by the design back-edge and
  PHASE-04. Record the resolution in the RV-403 `## Reconciliation Outcome`.
- `RV-403` F-2/F-3/F-4 — resolved in the reconcile pass before the back-edge
  (REV-066; the sec-5.4/sec-3 corrections; the `install/**` selector drop).
  Record them in the same outcome.
- `RV-403` F-5/F-6 — aligned as audited; `RV-402` F-5 obsoleted by PHASE-04.

## Reconciliation Outcome

Consumed RV-404's brief at reconcile.

### Direct edits applied

- **design.md sec-9 "Verification alignment"** — `RV-404` F-1: recorded that
  PHASE-03's VT-1..VT-4 are satisfied at execution and superseded by PHASE-04's
  VT-1/VT-2, so `doctrine slice verify-vt 271`'s current-tree FAIL for PHASE-03
  is expected.

### Fixed in-audit

- `RV-404` F-2: the e2e `[features] hooks` case gained the absent-key branch; the
  `verify-vt` PHASE-04 gate reads 4/4.

### Aligned

- `RV-404` F-3/F-4: the two undeclared paths are slice-level plan/status
  bookkeeping; the README host-dependency section is accurate post-retirement.

### Governance/spec

- None. `REV-066` is unaffected by the retirement: it describes the registration
  leg, not the hooks disclosure.
