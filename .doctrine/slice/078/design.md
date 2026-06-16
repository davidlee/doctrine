# Design — SL-078: Chore sweep (spec-010 rename + supersede recovery test)

## CHR-006 — SPEC-010 rename sweep

### Current vs target

SL-056 PHASE-11 renamed the CLI surface (`skills install` → `claude install`,
top-level `skills` → `claude`). `skills list` was dropped — no public `claude list`
verb exists. SPEC-010 and its descendant REQ-177 still reference the old names.

**Target**: All references to `skills install`, `skills list`, and `doctrine skills`
in SPEC-010's authored tier are corrected to match the current CLI. No code changes.

### Code impact

| File | Kind |
|---|---|
| `.doctrine/spec/tech/010/spec-010.md` | Body prose |
| `.doctrine/spec/tech/010/spec-010.toml` | Metadata responsibilities |
| `.doctrine/requirement/177/requirement-177.toml` | Title + slug |

Zero `.rs` impact.

### Detailed changes

**spec-010.md** (6 edits):

1. L18 — channel reference
   - Old: `the \`doctrine skills\` channel`
   - New: `the \`doctrine claude\` channel`

2. L26 — surface description
   - Old: `the \`skills list\`/\`skills install\` surface`
   - New: `the \`claude install\` surface`

3. L31-32 — Responsibilities mirror
   - Old: `surface the \`list\`/\`install\` commands`
   - New: `surface the \`install\` command`

4. L115-118 — Responsibilities prose
   - Old: `surface the \`list\`/\`install\` commands`
   - New: `surface the \`install\` command`

5. L130 — install container reference
   - Old: `the \`skills list\`/\`skills install\` surface. It rides`
   - New: `the \`claude install\` surface. It rides`

6. L137-142 — The command surface §
   - Old: `\`skills list\` enumerates the catalog grouped by domain with per-agent install status, read from symlink presence under \`.claude/skills/\` (a dangling-but-managed link counts as installed — status uses \`symlink_metadata\`, never \`exists\`, which would follow the link and hide it). \`skills install\` selects a subset by`
   - New: `Catalog enumeration is internal — there is no public list verb. \`claude install\` selects a subset by`

**spec-010.toml** (1 edit):

- `responsibilities[7]`:
  - Old: `Surface the catalog and routing through \`skills list\` (per-agent install status by symlink presence, including a dangling-but-managed link as installed) and \`skills install\` (subset by...`
  - New: `Surface the catalog and routing through \`claude install\` (subset by...`

**requirement-177.toml** (2 edits):

- `title`:
  - Old: `Surface skills list and skills install with subset, agent, global, dry-run, and yes selection`
  - New: `Surface claude install with subset, agent, global, dry-run, and yes selection`

- `slug`:
  - Old: `surface-skills-list-and-skills-install-with-subset-agent-global-dry-run-and-yes-selection`
  - New: `surface-claude-install-with-subset-agent-global-dry-run-and-yes-selection`

**requirement-177.md** (1 edit):

- H1:
  - Old: `# REQ-177: Surface skills list and skills install with subset, agent, global, dry-run, and yes selection`
  - New: `# REQ-177: Surface claude install with subset, agent, global, dry-run, and yes selection`

### Verification

- `doctrine spec show SPEC-010` — no `skills install`, `skills list`, or `doctrine skills` in output
- `doctrine requirement show REQ-177` — title reflects `claude install`

---

## CHR-008 — Supersede torn-state recovery e2e test

### Current behaviour

`run_supersede` (src/main.rs) writes NEW then OLD in a specific order so a crash
between writes leaves a detectable torn state: `NEW.supersedes ∋ OLD` without
`OLD.superseded_by ∋ NEW` and without `OLD.status = superseded`.

### Target behaviour (already implemented — adding the test)

Re-running `doctrine supersede NEW OLD` after a crash naturally completes the
recovery through the existing flow:

1. F-1 pre-flight passes (both docs have seeded keys)
2. F-D not-already-superseded guard: OLD.status ≠ "superseded" → skipped
3. `push_str_if_absent(NEW.supersedes, OLD)` → no-op (already present)
4. `push_str_if_absent(OLD.superseded_by, NEW)` → writes the missing entry
5. `OLD.status → superseded` → writes the missing status transition
6. Writes both files — both now correct

### Code impact

| File | Kind |
|---|---|
| `src/main.rs` | One test function in `#[cfg(test)] mod tests` |

Zero production code changes.

### Test design

```rust
#[test]
fn supersede_recovery_from_torn_new_only_state() {
    // Arrange: create ADR-001 (NEW) and ADR-002 (OLD) in a temp dir.
    // BOTH must have seeded `[relationships]` with `supersedes` and
    // `superseded_by` arrays (F-1 pre-flight requires both).
    //
    // Simulate torn state:
    //   ADR-001 supersedes = ["ADR-002"]  (NEW written)
    //   ADR-002: status=accepted, superseded_by=[] (OLD not yet written)
    //
    // Act: run_supersede(Some(tmp), "ADR-001", "ADR-002")
    //
    // Assert:
    //   1. ADR-002.status == "superseded"
    //   2. ADR-002.superseded_by contains "ADR-001"
    //   3. ADR-001.supersedes contains "ADR-002" (no duplicate)
}
```

**Fixture note**: The test writes fixtures inline using `catalog::test_helpers::tmp()`
and `write()`. Do NOT adapt `seed_adr` (blast radius). The F-1 pre-flight guard in
`run_supersede` requires BOTH `supersedes` and `superseded_by` arrays present on
both entities:

```rust
// NEW.fixture (seeded with empty arrays)
"id = 1\nslug = \"a1\"\ntitle = \"A1\"\nstatus = \"accepted\"\n\
 created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
 [relationships]\nsupersedes = []\nsuperseded_by = []\n"
```

### Key invariants

- `push_str_if_absent` is idempotent — no duplicate entries land in NEW's array
- The torn state is recoverable without a dedicated recovery code path
- The test catalogue fixture helpers from `catalog/test_helpers.rs` (`tmp()`, `write()`, `seed_adr()`) provide the setup infrastructure

### Verification

- New test passes under `cargo test -- supersede_recovery`
- `just check` clean (clippy zero warnings)

---

## Design decisions

- **D1 — No code changes.** Both items are test-only or spec-only. Zero `.rs` production code impact.
- **D2 — spec-010 L137 rewritten, not mechanically renamed.** `skills list` was dropped, not renamed to `claude list`. The paragraph is rewritten to reflect current state — catalog enumeration is internal, no public list verb.
- **D3 — REQ-177 title simplified.** The requirement originally covered both `list` and `install`; since only `install` survives, the title drops `list` and `skills` → `claude`.
- **D4 — Torn-state test uses existing helpers.** `catalog::test_helpers::tmp()` and `write()` provide the fixture infrastructure; the test is co-located with `run_supersede` in `src/main.rs`.

## Open questions

None. Both items have clear, bounded scope with no ambiguity.
