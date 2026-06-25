# SL-153: Implementation notes

## 2026-06-25 — Investigation & slice scaffolding

### What was done

1. **Audited all 23 `(label, source-set)` `RELATION_RULES` rows** in `src/relation.rs`
   against CLI verb coverage. Found exactly **3 edges with no CLI verb** that
   require hand-editing:
   - `descends_from` (SPEC→PRD) — scalar in spec TOML
   - `parent` (SPEC→SPEC) — scalar in spec TOML
   - `interactions` (SPEC→SPEC) — `interactions.toml` `[[edge]]` array

2. **Recorded a fact memory** `mem_019efe57896f` ("Relation edges still requiring
   hand-editing 2026-06-25") with the up-to-date inventory. Linked to
   `mem.signpost.doctrine.relating-entities` and
   `mem_019ec14b58a776429be79bd115d8917c` (relate-via-link, not hand-authored
   rows).

3. **Scaffolded SL-153** — scope document at `.doctrine/slice/153/slice-153.md`.
   The slice scope covers:
   - `doctrine spec edit --descends-from PRD-NNN [--parent SPEC-NNN]` — set/clear
     the two scalar fields
   - `doctrine spec interactions add SPEC-NNN --type <text> [--notes <text>]` —
     append an `[[edge]]` row
   - `doctrine spec interactions remove SPEC-NNN` — remove a matching row
   - **Update the shipped signpost memory** `mem.signpost.doctrine.relating-entities`
     (`memory/mem.signpost.doctrine.relating-entities/memory.md`) to replace the
     stale "What still requires hand-editing" section

4. **Linked SL-153** to ADR-010 (relation modelling) and PRD-002 (Specifications).

5. **Moved slice to `design` status.**

### Open design question

Should `descends_from` and `parent` share a single `spec edit` command (flags) or be
separate commands (`spec set-descends-from` / `spec set-parent`)? I recommended a
single `spec edit` with flags — cleaner dispatch, one file pass, extensible.

### Key files / code pointers

| File | Role |
|---|---|
| `src/relation.rs` | `RELATION_RULES` table — the legal-set vocabulary; `append_edge`/`remove_edge` for the edit-preserving `[[relation]]` write pattern |
| `src/spec.rs` | `Spec` struct (has `descends_from`, `parent` fields), `Interaction` struct, `InteractionsDoc` parser, `read_interactions`/`read_spec`/`relation_edges`, `append_member` (edit-preserving example for `members.toml`) |
| `src/commands/spec.rs` | CLI dispatch for `SpecCommand` — new subcommands land here |
| `src/requirement.rs` | `set_status` uses `dep_seq::set_authored_status` — the edit-preserving scalar-write pattern |
| `src/dep_seq.rs` | `set_authored_status` — shared scalar-field write seam via `DocumentMut` |
| `src/fsutil.rs` | `write_atomic` — the atomic write seam |
| `memory/mem.signpost.doctrine.relating-entities/memory.md` | The shipped signpost to update |
| `src/corpus.rs` | RustEmbed declaration for `memory/` folder — `touch` to re-embed |
| `.doctrine/slice/153/slice-153.md` | Scope document |
| `.doctrine/slice/153/slice-153.toml` | Metadata, relations |

### Shipped memory update flow

Per `mem.pattern.distribution.shipped-memory-authoring`:
1. Edit `memory/mem.signpost.doctrine.relating-entities/memory.md`
2. `touch src/corpus.rs && cargo build` (RustEmbed re-embed)
3. `doctrine memory sync` (materialise into `.doctrine/memory/shipped/`)
4. `doctrine claude install -y` (clients get it on next install)

### Relevant memories for the next agent

- `mem_019efe57896f` — current inventory of gaps (recorded today)
- `mem.signpost.doctrine.relating-entities` — the stale signpost to update
- `mem.pattern.distribution.shipped-memory-authoring` — shipped memory authoring flow
- `mem.pattern.entity.edit-preserving-status-transition` — edit-preserving TOML pattern
- `mem.pattern.distribution.skill-refresh-command` — re-embed + install pattern

### Next steps for design

1. Resolve the `spec edit` vs separate-commands question (ask user)
2. Decide whether clearing fields (set to `None`) is supported
3. Decide whether `interactions add` supports `--notes`
4. Write `design.md` following the `/design` skill process
5. Adversarial review → `/plan`

### Commit

```
a52bc872 slice(SL-153): scaffold CLI verbs for spec-internal edges (descends_from, parent, interactions)
```

## 2026-06-25 — External inquisition (codex / GPT-5.5)

Ran the external adversarial pass on design.md + source. 4 findings, all verified
against source, all **accepted** and integrated (design.md §10 E1–E4):

- **E1 (BLOCKER)** — `--parent` lacked a pre-write self/cycle gate; `registry.rs`
  treats those HARD-invalid (REQ-087). Added acyclicity gate (§5.4) + tests (§9).
  This was the only block; now resolved in-design.
- **E2 (MAJOR)** — removal pointed at `dep_seq::remove_after`, which is bound to
  `[relationships].after`/`to` and can't serve the `[[edge]]`/`target` AoT. New pure
  helper `spec.rs::remove_interaction_edges` specified instead.
- **E3 (MAJOR)** — add dup-check must canonicalize existing on-disk row targets
  (`target="SPEC-2"` vs `SPEC-002`) or it admits a duplicate. §5.3 sharpened + test.
- **E4 (MINOR)** — inline kind-validation re-encoded `RELATION_RULES`; reuse
  `lookup`/`check_target_kind` for declared rows + narrow product-`parent` branch.

Residual placement question (`apply_scalar` at the `dep_seq` leaf) was explicitly
pressure-tested by the reviewer and not flagged → closed (§10).

**Design ready to plan.** Next: `/plan`.

## 2026-06-26 — Dispatch drive (all 4 phases)

Dispatched via pi subprocess workers to dispatch/153 branch.

| Phase | Commit | Description |
|---|---|---|
| PHASE-01 | d3947526 | apply_scalar pure core |
| PHASE-02 | ab2c642f | SpecCommand::Edit (descends_from/parent) |
| PHASE-03 | 71466d0d | SpecCommand::Interactions (Add/Remove) |
| PHASE-04 | 0cc4800c | Shipped relating-entities memory refresh |

just check green; 111 spec tests + 24 dep_seq tests green.

## 2026-06-26 — Audit (RV-159)

Audit opened against candidate/153/review-001 (admitted at 9c045f5c). 3 findings:

- **F-1 (MAJOR → fix-now):** PHASE-02 spec edit integration tests missing. Design
  VT-1–VT-5 specified but only PHASE-03 (interactions) had test coverage. Applied:
  6 edit test functions (VT-1 through VT-5) + 1 canonical-storage test, all green.
- **F-2 (MINOR → fix-now):** Non-canonical `target` stored in `run_interaction_add`
  (design says "Target stored canonical"). Changed `toml_edit::value(target)` →
  `toml_edit::value(&canonical_target)`.
- **F-3 (NIT → tolerated):** Edit confirmation messages omit source spec ref.
  Cosmetic drift from design examples.

Both fix-now items applied on candidate branch (9c045f5c). Candidate admitted for
close. RV-159 done — all 3 findings terminal.
