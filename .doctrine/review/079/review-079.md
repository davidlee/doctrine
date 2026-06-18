# Review RV-079 — reconciliation of SL-095

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed:** candidate `cand-095-review-001` at `a953947f` (merge of
`dispatch/095` onto `main` with conflict resolution), targeting RV-079.

**Lines of attack:**

1. **PHASE-01 — RELATION_RULES row** — is the slice/backlog `Related` row
   present with `AnyNumbered` target? Does `lookup(SLICE, Related)` return
   `Some`? Is the exact-coverage invariant updated?
2. **PHASE-02 — governance migration** — `Relationships` struct drops
   `supersedes`; `relation_edges` reads from `tier1_edges`;
   `supersession_pair` uses `read_block` (not `tier1_edges`); `format_show`
   accepts `supersedes: &[String]` from caller; templates + 14 corpus files
   have `supersedes = []` removed; `show` output byte-identical.
3. **PHASE-03 — POL/STD verb extension** — `PolicyStatus::Superseded` and
   `StandardStatus::Superseded` variants present with `as_str()`,
   `POLICY_STATUSES`/`STANDARD_STATUSES`, `is_hidden`, and drift canaries.
   `SupersedePolicy` has POL/STD arms with `StorageTarget::RelationRow`.
   `run_supersede` dispatches on `StorageTarget` — governance writes
   `[[relation]]` via `append_edge`, typed `superseded_by` carve-out
   unchanged. Idempotent re-run is no-op. `is_terminal` check preserved
   from SL-097 for record kinds.
4. **Partition** — `"superseded"` present in POL and STD terminal partitions.
5. **Gate** — `just gate` clean, clippy zero, fmt clean, 1687 bin tests pass.

**Bodies likely buried:** The candidate merge from `dispatch/095` onto `main`
required conflict resolution in `main.rs` (StorageTarget dispatch vs SL-097's
is_terminal), `knowledge.rs` (test assertions), `relation_graph.rs` (emitted
labels vs table labels), and `supersede.rs` (API evolution). The merged
`TypedArray` arm now correctly uses `old_policy.superseded_status` for
cross-kind record supersession and respects `is_terminal`.

## Synthesis

SL-095 delivered three phases cleanly. The implementation matches the design
closely; the single finding (F-1) is a design-doc accuracy issue, not a code
bug.

**PHASE-01** added the slice/backlog `Related` RELATION_RULES row with
`AnyNumbered` target — one row, pure addition. The exact-coverage invariant
(`reader_emitted_labels_equal_table_labels_per_source`) covers the new source
kinds automatically (it iterates every source in RELATION_RULES). No golden
churn beyond the expected table growth.

**PHASE-02** migrated governance `supersedes` from the typed
`[relationships]` table to `[[relation]]` rows. The `Relationships` struct
dropped the field cleanly; `relation_edges` now delegates both `supersedes`
and `related` to `tier1_edges`; `supersession_pair` correctly uses
`read_block` (not `tier1_edges`) so illegal rows surface to `validate`. The
3 templates and 14 corpus TOML files had `supersedes = []` removed
(cosmetic — serde ignores unknown keys). `doctrine show` output is
byte-identical pre/post migration for empty arrays. `doctrine validate`
reports clean.

**PHASE-03** added `Superseded` to `PolicyStatus` and `StandardStatus`
(with `as_str()`, known-set, hide-set, and drift canaries), extended
`SupersedePolicy` with POL/STD arms and a `StorageTarget` discriminant,
and refactored `run_supersede` to dispatch on `StorageTarget::RelationRow`
(governance → `append_edge`) vs `TypedArray` (records → existing
`dep_seq::apply_string_append`). The typed `superseded_by` carve-out on
OLD is unchanged for all kinds. Idempotent re-run reports `already recorded`.
The SL-097 `is_terminal` check is preserved in the `TypedArray` arm.

**Merged candidate** (`cand-095-review-001` at `a953947f`) passed 1687 bin
tests, clippy clean, fmt clean, `doctrine validate` clean. The merge
required conflict resolution in 4 files where `dispatch/095` and main had
diverged. The resolved `TypedArray` arm correctly uses
`old_policy.superseded_status` for cross-kind record supersession status
flips.

**Standing risks:** R1 (no POL/STD entities exist for live terminal-flip
testing) is mitigated by unit tests in `src/supersede.rs`. R2 (coexistence
with SL-097) resolved — `is_terminal` and `StorageTarget` cohabit the
merged `run_supersede`.

**Tradeoffs consciously accepted:** The JSON output preserves
`relationships.supersedes` (spliced from `[[relation]]` rows) for backward
compatibility, contrary to design D4's assertion that it would disappear.
Zero consumers exist; the pragmatic choice is correct.

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md §D4** (F-1): Update the JSON surface statement from
  "relationships.supersedes disappears from --json output" to "JSON surface
  unchanged — supersedes is spliced back from [[relation]] rows, preserving
  the byte-identical relationships object."

## Reconciliation Outcome

### Direct edits applied
- `slice/095/design.md` §D3: updated JSON surface claim → "JSON surface unchanged —
  `supersedes` is spliced back from `[[relation]]` rows in `show_json`, preserving
  the byte-identical `relationships` object." (RV-079 F-1)
- `slice/095/design.md` Verification alignment: `show --json` bullet updated to
  match (RV-079 F-1)

### REVs completed
- None — no governance/spec items in brief.

### Withdrawn / tolerated
- None — the single finding (F-1) was verified and remediated.
