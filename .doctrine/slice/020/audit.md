# Audit — SL-020 backlog-entity-v1

Conformance audit (post-implementation). Mode: reconcile the 6 implemented
phases against `design.md`, PRD-009, ADR-004/005, and the phase `VT-` criteria.
Hand-authored (no `slice audit` scaffold yet). Date: 2026-06-08.

Gate at audit time (HEAD + the two PHASE-audit reconciles below):
`just check` EXIT=0 — `cargo test` 521 pass, clippy zero (bins/lib), `boot
--check` clean.

---

## 1. Requirement coverage — PRD-009 REQ-049..059 (11 total)

v1 reach is fixed by `design.md` §1: `REQ-049..053`, `REQ-057..059` MET;
`REQ-054/055/056` designed-but-deferred. Reconciled item by item.

| REQ | What | Disposition | Evidence |
|---|---|---|---|
| 049 | `backlog new <kind>` capture | **aligned** | `run_new`; `backlog_new_reserves_monotonic_per_kind`, `..._counters_isolated_across_kinds` |
| 050 | `backlog list` survey + filters | **aligned** | `run_list`/`select`; `backlog_list_*` (visibility, AND-filters, order, total-fn reads) |
| 051 | `backlog show <ID>` inspect | **aligned** | `run_show`/`format_show`; `backlog_show_auto_detects_kind_from_prefix` |
| 052 | non-canon status/resolution rejected | **aligned** | clap `ValueEnum` boundary; `backlog_edit_rejects_noncanon_status_and_resolution` |
| 053 | risk facet (typed, descriptive) | **aligned** | `RiskFacet` typed axes; `risk_facet_levels_map_empty_to_none_and_parse_non_empty` |
| 057 | atomic edit-preserving transition | **aligned** | `set_backlog_status` (`toml_edit`); `backlog_edit_is_edit_preserving`, `..._noop_writes_nothing` |
| 058 | typed closed enums, no untyped bag | **aligned** | `ItemKind`/`Status`/`Resolution`/`RiskLevel` closed enums; storage-rule clean |
| 059 | status ⟂ resolution ⟂ facet coupling | **aligned** | `validate_transition` both-directions + D9; `validate_transition_couples_both_directions_and_d9_clears` |
| 054 | priority (authored seam) | **follow-up slice** | Non-Goals; PRD-011 OQ-001. Authored seam + outbound graph reserved, not built |
| 055 | promote bridge (`--from-backlog`) | **follow-up slice** | Non-Goals; PRD-009 OQ-003 semantics resolved (`resolution=promoted` first-class), command deferred |
| 056 | relation writing/derivation (`link`) | **follow-up slice** | Non-Goals/ADR-004. Outbound STORAGE built (`Relationships{slices,specs,drift}`, rendered by `show`); only the write verb + reverse scan deferred |

The three deferrals are conscious, design-fixed, and forward-compatible: the
model encodes `promoted` as a first-class `resolution`, stores outbound edges
PRD-011 reads, and reserves no priority field it cannot type. No silent gaps.

---

## 2. Governance gates

- **R6 — engine untouched (behaviour-preservation).** *aligned.* No SL-020
  commit touches `src/entity.rs` or `src/meta.rs`
  (`git log --grep=SL-020 --name-only` → only `backlog.rs` + `main.rs` + wiring).
  Backlog rides the engine as five `Fresh` callers only. Existing
  slice/ADR/spec/memory suites green unchanged.
- **ADR-004 — outbound-only relations.** *aligned.* `format_show` renders only
  the item's own outbound axes; inbound reverse-refs never computed
  (`backlog_show_renders_outbound_only`). Promotion-origin edge is slice-side.
- **ADR-005 — knowledge tiering (PHASE-06 skill wiring).** *aligned.* Boundary
  text lives in the PULL reference; the six revised skills name the verbs +
  point at the tier-1/2 docs rather than restate (`skills.rs::dedup_skills_
  route_not_restate` guards the named set); boot regenerated from
  `routing-process.md`. (Note: the embed/ship guards in `boot.rs`/`install.rs`/
  `skills.rs` are SL-023's, co-resident in the tree — outside SL-020's pathspec.)
- **Authored-entity wiring trap.** *aligned.* `install/manifest.toml`
  `[dirs].create` + `!.doctrine/backlog/` negation present; a created item is
  `git add`-able (`created_backlog_item_is_git_addable`).

---

## 3. Code-review findings carried in (6) — dispositions

Findings from the pre-close `/code-review` of `src/backlog.rs` + `src/main.rs`,
each dispositioned. Two reconciled in-slice now; one routed to a follow-up; three
tolerated/aligned with rationale.

### F1 🟠 — TOML title-injection via raw `.replace` splice → **follow-up slice**

`render_backlog_toml` splices `title` into `title = "{{title}}"` with
`.replace`; `input::resolve_title` only trims, never escapes. A title carrying
`"`, `\`, or a newline writes a syntactically broken `backlog-NNN.toml` (`new`
succeeds; every later `show`/`list` over that tree fails to parse). Backlog
titles are freeform prose, so quotes are *likely* — exposure exceeds adr/slice.

`memory.rs::toml_string` (audit note A-1: "user-influenced value escaped …
never spliced raw") is the project's existing escaper. **Why follow-up, not
fix-now:** the identical raw-splice is shared across `adr.rs`/`spec.rs`/
`slice.rs`/`requirement.rs` — the correct fix extracts `toml_string` into a
shared escaping seam corpus-wide. A backlog-local patch would either fork the
escaper (violates *no parallel implementation*) or pull the shared extraction
(5 modules + the shared render seam, behaviour-preservation gate) into this
slice's reconcile — too broad. Captured as a follow-up; memory recorded so it is
not rediscovered (`mem.pattern.lint.string-build-no-push-format` neighbour).

### F2 🟠 — stale module-wide `#![expect(dead_code)]` → **fix now (done)**

The PHASE-01 module-level `#![expect(dead_code)]` reason ("no CLI this phase")
was false at HEAD — PHASE-06 wired all four verbs. The only dead item left is the
inert `KIND_PRECEDENCE`; a module-scope suppression blanket-hid any future dead
code across ~1900 lines and defeated the lint that would catch it.
**Reconciled:** removed the module attribute, scoped a `#[expect(dead_code,
reason = "inert until the PRD-011 multi-kind resolver consumes it")]` to the one
const, and refreshed the stale PHASE-01 module doc paragraph. Gate green.

### F3 🟡 — `validate()` does not check the status⟂resolution coupling on read → **tolerated drift**

`validate` parses enum tokens but does not re-check REQ-059 on the read path; a
hand-corrupted `status="open", resolution="fixed"` passes and `format_show`
would render ` · fixed` on a non-terminal item. **By design** (§5.5/D9): the
coupling is enforced at the *transition* (`validate_transition`), and the read
path trusts the on-disk file — hand-corruption is out of scope (the same
ungated-edit posture slices/ADRs/specs ship with). Tolerated; the only residue
is that the fn name `validate` slightly oversells. No follow-up warranted.

### F4 🟡 — `from_prefix` duplicated the `ItemKind::ALL` array literal → **fix now (done)**

Two hand-maintained copies of the five-variant list. **Reconciled:**
`from_prefix` now iterates `ItemKind::ALL` (single declaration).
`item_kind_from_prefix_round_trips_each_kind` still green.

### F5 🔵 — test-fixture TOML-builder sprawl → **tolerated drift**

`write_item`/`write_assessed_risk`/`write_related` hand-assemble overlapping
toml with slightly different field sets. Test-only, readable, low value; a
parameterised builder would help but does not gate closure. Left for a future
test-helper pass.

### F6 🔵 — `select` tag match case-sensitive while substr is case-insensitive → **aligned**

Intended: tags are canonical tokens (exact match correct); the title substring
is human prose (case-folded). The asymmetry is correct, not a defect. Noted here
in lieu of a code comment.

---

## 4. Prior review passes (reference — not re-litigated)

Design-time internal adversarial pass (R1–R7) and external inquisition (C1–C6,
C4 recanted) are dispositioned in `design.md` §10 / `inquisition.md`. Re-checked
as still-valid; not re-opened here.

---

## 5. Harvest

- **Recorded this slice:** `mem.pattern.entity.edit-preserving-status-transition`
  (the `toml_edit` + I5 no-op + F-1 refuse seam; medium trust, unattested — a
  dirty tree blocked `verify`). Boundary canon:
  `mem.concept.backlog.work-intake-membership`.
- **To record (F1 follow-up):** the raw-`.replace` TOML title-splice is a
  corpus-wide latent injection; `memory.rs::toml_string` is the existing fix to
  extract. Capture before close so the follow-up slice is findable.

## 6. Closure readiness

`audit.md`, `design.md`, and the deferral/follow-up refs tell a coherent story.
All 11 requirements dispositioned (8 met, 3 conscious deferrals); all 4 gates
aligned; all 6 review findings dispositioned (2 reconciled, 1 follow-up, 3
tolerated/aligned); gate green. **Audit-ready for `/close`.**
