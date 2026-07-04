# Notes SL-202: Memory body wikilinks as catalog edges

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest (audit RV-246)

### Design decisions that stuck
- **Two-pass, TOML-then-body ordering (design §5.4).** `seen: BTreeSet<CatalogKey>`
  is populated by the TOML relation loop (insert-only, no check) then consumed by
  the body pass (check+insert). One-directional (INV-1): body defers to TOML;
  TOML-vs-TOML multiplicity is untouched. Body pass placed *after* the full TOML
  loop so `seen` reflects every TOML edge before any body dedup.
- **`Option<String>` body, not `String` (PHASE-01).** Absence is meaningful (a
  memory may legitimately have no `.md`); `.ok()` gives `None`, mirroring the
  `corpus.rs:370` seam. PHASE-02 skips the wikilink pass on `None`.
- **F-1 divergence is deliberate and scoped.** A body `mem.<key>` wikilink that
  fails to resolve warns (`UnvalidatedText` → 1 edge + 1 Warning, INV-2); the TOML
  path stays silent. The extractor's `mem.word.word` shape excludes prose, so a
  dangling body wikilink is near-certainly a real broken reference. `classify_target`
  and the TOML `:341` diagnostic are untouched — VT-4 proves TOML stays silent.
- **`UnresolvedRef` arm is unreachable** from the body pass (`extract_wikilinks`
  yields only `mem.*`/`mem_*`; `parse_canonical_ref` rejects → memory branch). No-op
  fallthrough for match exhaustiveness — do NOT warn on it (dead + misleading).

### Gotchas (see also RFC-011 case-notes)
- **Phase-split transiently-dead field.** `MemoryCatalogRecord.body` was dead until
  PHASE-02 consumed it → clippy `-D unused`. Bridged with
  `#[cfg_attr(not(test), expect(dead_code, reason=…))]` — the `not(test)` gate is
  REQUIRED because VT-1 reads the field in the test build (a bare `#[expect]` there
  is unfulfilled → error). Self-cleaning: the production read unfulfils it → clippy
  forces removal. Promoted to memory (see below).
- **`verify-vt` reads UNATTRIBUTABLE while a phase is `in_progress`** — attribution
  range is `code_start..code_end`, and `code_end` stamps only at completion. Working
  as designed; flip the phase to `completed` before trusting `verify-vt`. Distinct
  from IMP-228 (closed).
- **Diverged-base diff trap.** `git diff edge..fork` conflates edge's advance with
  the fork delta (spurious `check.rs` deletion). Audit the true delta:
  `git diff $(git merge-base edge fork)..fork`. Conformance (boundary-OID based) is
  immune.

### Verification evidence
- Conformance: 0 undeclared / 0 undelivered / 2 conformant. Gate exit 0. Suite
  3089/0. VTs: PHASE-01 VT-1 + PHASE-02 VT-1..4 all PASS.
- Fork `SL-202-exec`: `a0f567cf` (PHASE-01) → `a380aace` (PHASE-02). Base
  `4ddb5244`. Linear, non-merge (F-6 clean).
