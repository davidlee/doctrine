# IMP-123: SL-122 RFC: add dedicated minted-RFC tests for PHASE-01 VT-2/VT-4 and PHASE-04 VT-3 boot byte-unchanged

Surfaced during SL-122 audit (RV-110, finding F-3). Coverage is **adequate** today
— the behaviour is proven — so this is hardening, not a correctness fix.

Optional dedicated tests that would make the VT mapping explicit:
- **PHASE-01 VT-2** — a test that mints an RFC then exercises validate/reseat on
  the RFC id directly, rather than relying on the generic `kinds_table_covers_*`
  assertions.
- **PHASE-01 VT-4** — a corpus-scan test that mints `RFC-001` and runs a
  debug-build `scan_entities`/`outbound_for` over it (no panic), rather than
  code-inspection of the routed arm.
- **PHASE-04 VT-3** — an explicit assertion that the boot snapshot carries no RFC
  section and existing boot output is byte-unchanged, rather than leaning solely on
  the pre-existing boot suite staying green.
