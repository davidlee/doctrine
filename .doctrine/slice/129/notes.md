# SL-129 Notes — entity id→path helper (entity::id_path over KINDS)

## Audit outcome (RV-119)

- **Verdict:** clean conformance — zero findings.
- All 36 Kind initializers seeded with `stem:` field.
- 22 KINDS rows dropped `stem:` (now derived from `kind.stem`).
- 4 GovKind constructors dropped `stem:` (now on `kind.stem`).
- ~85 production format! sites replaced with `entity::id_path`/`entity::rel_path`.
- 20 files changed: 152 insertions(+), 170 deletions(-).
- 2117 tests pass, 0 failed, clippy zero warnings.
- Reconciliation brief empty — no spec/governance changes needed.
