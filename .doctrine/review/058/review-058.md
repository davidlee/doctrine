# Review RV-058 — design of SL-087

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Lines of attack** — what the Inquisition presses this design against:

1. **ADR-005 compactness.** Does the proposed section shape (reference line +
   key listing) honour the PUSH-tier compactness invariant, or does it keep
   metadata that belongs in `/retrieve-memory`?
2. **Design precision.** Are sort order, key optionality, error handling, and
   the new API contract stated with enough precision to survive implementation
   without guesswork? Underspecification is heresy.
3. **Consistency with code surface.** Does the design's proposed code patch
   match the codebase it claims to modify — same types, same call shapes, same
   error-handling discipline?
4. **Scope coherence.** Post-IMP-095 drop: does the scope tell the same story
   as the design, or does residue linger?
5. **Test coverage.** Does the test plan cover the edge cases the change
   introduces (keyless memories, empty corpus, error corpus)?

## Synthesis

### Judgement

The design is **sound in its core intent** — trimming the Memory section from a
~50-line metadata table to a ~22-line reference + key listing is the correct
reading of ADR-005's PUSH-tier compactness. No architectural heresy. No
doctrinal violation.

But the design **confessed to underspecification under cross-examination.** Five
charges, all accepted without contest: key optionality unaddressed (F-1), code
example/prose inconsistency (F-2), sort-order contradiction between sections
(F-3), boot_keys() contract too vague (F-4), and a nit on pointer context
(F-5). None block the design's intent; all are fix-now precision defects.

### Ordered penance (verified remedies)

1. **F-1 (major):** State that keyless memories render their uid as the key
   line. `Memory.key` is `Option<String>`; the uid is always present and
   unambiguous. Verifier: grep design.md for "uid" near "keyless" or
   "boot_keys contract".
2. **F-2 (minor):** Align the code example to route through
   `section_or_marker`, matching the prose promise and every other producer
   arm. Verifier: the code example no longer contains `unwrap_or_default`.
3. **F-3 (minor):** Fix the test section's stale "sort_default" reference —
   replace with "key ascending," matching the Design Decisions table.
   Verifier: grep design.md test section for "sort_default".
4. **F-4 (minor):** State boot_keys() contract in one place: `pub(crate) fn
   boot_keys(root: &Path) -> Result<Vec<String>>`, internal filtering to
   active+signpost, key-ascending sort, uid fallback for keyless.
   Verifier: the contract paragraph is self-contained.
5. **F-5 (nit):** Reference line stands as-is. ADR-005 pull pointer pattern.
   Aligned — no change.

### Standing risks

- None. All findings are terminal and the remedies are mechanical precision
  fixes to the design document, not to the design's architecture.

### Tradeoffs consciously accepted

- Key-only listing preserves ~20 lines of discoverability that a pure
  reference line would drop. This was the User's explicit direction ("both")
  and is an acceptable compactness tradeoff — 22 lines vs 50 is still a ~55%
  improvement.
- Uid-as-fallback for keyless memories preserves the invariant that every
  active signpost gets a line; the one keyless memory in this corpus
  (`mem_019ecf85…`) has a uid that carries signal (the SL-076 origin is
  clear from context).
