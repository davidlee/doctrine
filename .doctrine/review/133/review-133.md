# Review RV-133 — reconciliation of SL-136

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Audit mode: conformance self-audit for SL-136 after dispatch completion.

Reviewed surface: `refs/heads/candidate/136/review-001` / worktree `.doctrine/state/dispatch/candidate/cand-136-review-001` at `898e6bd26da5fda745c6336e2e71478200ff3882`, created from `refs/heads/review/136` (`42c7691b0566`) onto `refs/heads/main` (`cda19d70a264`). Parent worktree dirt is intentionally excluded from evidence.

Lines of attack:

- Does the implementation follow SL-136 design D1/D2: one shared root-level tag write leaf, curated taggable set, and no write-only included kind?
- Does `doctrine tag` preserve backlog semantics, reject excluded/non-numbered kinds clearly, and keep memory tagging out of scope?
- Are all read surfaces wired for included kinds: `list --tag`, table `show`, and JSON output?
- Did the governance/RFC corpus and templates actually migrate away from `[relationships].tags` to root `tags`, including RFC fixtures and goldens?
- Does verification cover the phase VT criteria plus `just check`, and is the known D6 REV obligation for SPEC-005/SPEC-016/SPEC-018 explicit for `/reconcile`?

## Synthesis

Self-audit of the SL-136 candidate (cand-136-review-001, tip `94b664b2`) after dispatch completion. Three findings uncovered, all fixed on the candidate branch:

**F-1 (blocker) — standard golden fixture stale.** The `e2e_standard_cli_golden.rs` fixture (`std001_toml`) still carried `tags = ["style"]` under `[relationships]`, and the JSON byte-exact golden expected nested `relationships.tags`. The ADR golden had been migrated; the standard golden was missed because it lives in a separate test file from the ADR one. Fixed by moving tags to root in the fixture and updating both the table and JSON expected outputs.

**F-2 (major) — CHR-019 spike broken by corpus migration.** Three h2 tests (`h2_rfc002`, `h2_adr014`, `h2_pol001`) asserted pre-migration typed shapes on ADR-014, POL-001, and RFC-002. PHASE-04 migrated the corpus but did not update the spike. Design VT-2 explicitly states "the spike stays green." Rewrote the three tests to verify the post-migration shape (root tags present or absent) and exercise root-insert edits on the migrated files, proving the root-insert safety guarantee still holds.

**F-3 (blocker) — unit tests didn't compile.** 21 struct-literal sites across 7 files missing the new `tags` field (Meta, Doc, SliceDoc), plus one stale `relationships.tags` field. The dispatch e2e test runs bypass `#[cfg(test)]` code, so the gap was invisible. All Meta/Doc/SliceDoc literals now include `tags: vec![]`.

**Verification results:** All 36 e2e golden/migration tests green (adr, standard, catalog, relation_migration_storage, chr019 spike). Unit test suite compiles and passes. Clippy zero warnings. No residual typed `tags = [...]` data lines remain in any governance/RFC `[relationships]` block — only stale comments (benign). Template roots seeded. `governance_files()` extended to `rfc`. `relation_graph.rs` fixtures repointed root-ward. Read-surface parity confirmed per design §5.3: slice/spec/REQ/gov tags visible on all three surfaces (list --tag, show table, --json).

**Standing risk:** ADR/RFC `.toml` comments still say "tags stay typed" — stale but harmless; the REV at reconciliation can address them.

## Reconciliation Brief

### Governance/spec (REV)

- **D6 REV obligation — SPEC-005 D2, SPEC-016, SPEC-018:** The storage move to root-level tags contradicts all three specs which pin governance tags as typed in `[relationships]`. One Revision (REV) amending all three is required before `/close`. The corpus is intentionally non-canonical against these specs until the REV lands. VA-1 (PHASE-04) records this as a soft gate.

### Per-slice (direct edit)

- **design.md comment:** No code changes needed — the implementation matches design D1/D2/D3/D4/D5. The 3 audit findings were fixed on the candidate branch and require no further per-slice edits.

## Reconciliation Outcome

### REVs completed
- REV-006 (`reconcile-sl-136`): done — SPEC-005 D2, SPEC-016, and SPEC-018 amended to root-level tags (covers RV-133 D6 REV obligation). Spec prose updated: SPEC-005 D2 (`tags remain in typed [relationships]` → `tags moved to root-level`), SPEC-016 responsibility text and scaffold prose, SPEC-018 §relations (`tags stays typed` → `tags moved to root-level uniform storage`). Rationale in revision-006.md.

### Direct edits applied
- None needed — all changes are spec-level, addressed by REV-006.

### Audit findings fixed on candidate
- RV-133 F-1 (blocker): standard golden fixture stale — fixed on candidate/136/review-001 (c7552b44)
- RV-133 F-2 (major): CHR-019 spike broken by migration — fixed (93692812)
- RV-133 F-3 (blocker): unit tests didn't compile — fixed (ffe9e91c)
