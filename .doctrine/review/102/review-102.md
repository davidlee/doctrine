# Review RV-102 — design of SL-113

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->
## Brief

Hostile adversarial review of `slice/113/design.md` — "Shared entity mutation seam
over atomic write."

### Lines of attack

1. **Factual accuracy of the call-site table** — verify every line number, function
   name, and error-wrapper claim against the actual source at current HEAD.
2. **Completeness** — does the table enumerate every production `std::fs::write`
   call on authored entities? Are there missing sites?
3. **Code-impact precision** — does the design specify the *exact* mechanical
   transform for each site, or are there hand-wavy "similar" instructions?
4. **ADR compliance** — ADR-001 (layering), ADR-003 (change loop), ADR-004
   (outbound-only), ADR-010 (relation modelling). Does the design violate any?
5. **Doctrinal alignment** — pure/imperative split, storage tiers, write_atomic
   as the canonical seam.
6. **Verification coverage** — are VT criteria testable? Gaps?
7. **Hidden assumptions** — what does the design take for granted?

The reviewer has not been asked to praise; every finding below is an identified
weakness with specific evidence and a recommended fix.

---

## Synthesis

### Overall: revision-required

The design's core thesis — route every authored-entity write through
`write_atomic`, add an `AtomicU64` counter for intra-process collision
hardening, and install a clippy guard — is sound. The 18-site table is complete
(no production `std::fs::write` to authored entities is missed). The clippy
guard strategy (gated non-test, two noted exceptions) is correct. The
pure/imperative split is respected; the change is confined to the leaf tier.

But the design is **factually sloppy** in its implementation-facing detail: 6
of the 9 rows in the call-site table carry wrong function names. Three
functions are misnamed; two are fabricated entirely (`apply_with`,
`handle_edit_concept_map_edge`). An executor following the table as-written
would search for non-existent identifiers and waste cycles reconciling.

The VT-6 error-wrapper analysis is overly glib — `e.to_string()` does produce
valid output for both `io::Error` and `anyhow::Error`, but the content differs
in ways the design elides. No test asserts on error message format, so this is
cosmetic, not semantic — but the design should say so directly rather than
claim "works unchanged" for `map_server/routes.rs`.

ADR compliance is clean. The storage-tier distinction is correctly applied
(ledger.rs exception justified). No doctrinal violations found.

### Standing risks

- **Wrong function names survive into the plan.** If the plan copies the
  design's table without verification, the phase sheets will contain the same
  errors. The plan author must re-verify every row against source.
- **Error observability gap for map_server.** `anyhow::Error::to_string()` in
  `MapServerError::ConceptMapIoError` drops the OS-level cause. This is a
  pre-existing pattern in the MapServer error wrapping, not introduced by this
  change, but VT-6 should acknowledge the format shift.
- **VT-5 negative test is manual-only.** The "introduce a bare std::fs::write
  and confirm clippy fails" verification will never be re-run. Tolerated as
  disproportionate automation for a single lint guard.

### Tradeoffs consciously accepted

- Power-loss durability gap (no `fsync`) is accepted for authored files under
  git. The design states this; the synthesis recommends adding the mechanism
  explanation.
- The `AtomicU64` counter adds a `static mut`-adjacent global to the leaf tier.
  Collision-free per-process, zero-dependency, minimal change. Worth the cost.

### Haiku

*eighteen call sites named*
*six with names the code rejects*
*fix the table first*

