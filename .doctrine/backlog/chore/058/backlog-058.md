# CHR-058: Fill in SPEC-002 REQ-254 to REQ-257 statements and acceptance criteria

`REQ-254`, `REQ-255`, `REQ-256`, and `REQ-257` — the four SPEC-002 requirements
governing the executable verification seam — are **title-only**. No statement,
no acceptance criteria, in either `spec req show` or the woven
`spec show SPEC-002`.

Positive control: `REQ-113`, `REQ-114`, and `REQ-115` in the same spec carry
full statements and 3–6 acceptance criteria each. So this is an authoring gap,
not a spec-wide convention.

**Why it blocks other work.** RFC-027's `H6` and `H13` both say "reuse the
SPEC-002 seam" — which presently means reusing code governed by requirements
that assert nothing. ISS-325 and IMP-408 are schema changes to a persisted type
in that seam and are not conformance-checkable until this is repaired. RFC-027
Stage 4 sequences this **first**, before ISS-324, ISS-325, and IMP-408.

Evidence: `.doctrine/rfc/027/proof-binding-study/conclusion.md` § R4.
