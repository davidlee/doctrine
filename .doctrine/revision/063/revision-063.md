# REV REV-063 — reconcile SL-265

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

SL-265 (`RV-387` `F-1` path and design sec-5) lands the kind-blind `doctrine
show` router as a `SPEC-013` requirement: `REQ-482`, membership label `FR-006`,
introduced by `REV-062` (done). `spec req add` mints a requirement `pending` —
the frozen label and the statement are stageable, but no evidence exists yet,
and an `active` requirement with no verified evidence is residual drift
(`REQ-113`).

The evidence now exists, so the flip is owed: `PHASE-04` recorded a check-bound
coverage cell for `REQ-482`
(`.doctrine/slice/265/coverage.toml`), `doctrine coverage verify 265` re-derives
it to `verified`, and the requirement's scope is witnessed by the admitted
candidate `cand-265-close-001` (`330b247f0`) — `slice verify-vt 265` is 14/14
PASS, and the byte-equivalence suite (`tests/e2e_show_equivalence.rs`) is the
cell's bound check.

One row, `status`, which `revision apply` auto-lands: `REQ-482` `pending →
active`. No spec prose changes here — the `SPEC-013` grammar and responsibilities
prose, and the `spec-013.toml` `responsibilities` entry, landed in `PHASE-04`
under `REV-062`.

## Reconcile narrative (SL-265)

- [`RV-387` `F-1`]: the new requirement subtree had no design-target selector, so
  it reported `undeclared` in `slice conformance`. Fixed by the selector registry
  (`slice selector add 265 '.doctrine/requirement/**' --intent design-target`),
  not by prose — conformance reads `slice-265.toml`.
- [`RV-387` `F-3`]: design sec-5 predicted the REV/spec deliverable would report
  undeclared; it is in fact glob-covered and conformant. `design.md` edited
  directly at reconcile (out of band on the locked run) to the real reading.
- [`RV-387` `F-4`]: `check gate`'s lone red was the ambient jail
  `DOCTRINE_RESERVATION_FALLBACK=1`, not a slice defect (`reserve::tests` is 19/19
  green without it; the operator removed the export from `flake.nix`). Captured
  as `ISS-483` — see that item, not this REV.
