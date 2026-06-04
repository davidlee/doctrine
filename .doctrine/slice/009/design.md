# Design SL-009: Slice status rollup

## 1. Design Problem

Phase completion is invisible from the CLI. `doctrine slice list` renders only
the hand-edited `slice-nnn.toml` `status` — a free-text lifecycle label authored
by a human, **not derived** from the runtime phase tracking. The real progress
(`X/Y` phases complete) sits in `.doctrine/state/slice/nnn/phases/phase-NN.toml`
and is never aggregated. The slice must add a **read-only derived rollup** and
surface it beside the authored status, without writing anything, changing the
phase-tracking schema, or polluting the machinery `slice list` shares with
`adr list`.

## 2. Current State

- **`slice list` path** (`slice::run_list` → `meta::read_metas` →
  `meta::sort_and_filter` → `meta::format_list`). `meta` is the **shared**
  list substrate for slices *and* ADRs (design SL-006 D4): `Meta` carries exactly
  four fields (`id`, `slug`, `title`, `status`), the formatter is kind-blind. ADRs
  have **no phases** — anything phase-shaped in `meta` would be wrong for ADR.
- **Phase state** (`state.rs`). Owns the runtime tree under
  `.doctrine/state/slice/nnn/phases/`. Has `phases_dir(root, id)` (id-derived; the
  convenience symlink is never followed), `existing_phase_stems` (enumerate
  stems), and the *writers* `init_phases` / `set_phase_status`. **There is no
  status reader.** `phase-NN.toml` is `schema/version/phase/status/...` with
  `status ∈ {planned,in_progress,completed,blocked}` (the `PhaseStatus` enum,
  a `clap::ValueEnum`, `as_str` only — no parse-back).
- **State tree is gitignored, disposable, often absent.** Pre-phase-tracking
  slices (SL-004) and freshly-scoped slices have no `phases/` dir at all.

## 3. Forces & Constraints

- **Storage rule.** The rollup is *derived* — computed at query time, displayed,
  never written into the authored `slice-nnn.toml`. Derived never overwrites
  authored.
- **Keep `meta` neutral.** The shared formatter/struct must not learn about
  phases; the rollup column is slice-specific.
- **Pure/imperative split.** Reading the phase dir + tomls is IO (thin shell); the
  fold (status strings → counts) is pure and unit-tested off an in-memory list.
- **Behaviour-preservation gate.** `meta`, `entity`, the state writers, and
  `adr list` are untouched in contract; their suites stay green unchanged.
- **No new state, no schema change, no cache.** Recompute per invocation.
- **Honest absence.** No materialised phases must render explicitly — never
  `0/0`, which reads as "nothing done" rather than "nothing tracked".

## 4. Guiding Principles

- Read-only: the safest slice doctrine has. If it ever writes, the design is wrong.
- Derive, don't store; juxtapose, don't reconcile.
- Tolerant read: a hand-edited typo in a phase `status` degrades the rollup, never
  crashes `slice list`.
- Smallest neutral seam: the reader lands where phase state lives (`state.rs`); the
  column lands where slice presentation lives (`slice.rs`); `meta` stays put.

## 5. Proposed Design

### 5.1 System Model

Three pieces, each on the correct side of the IO seam:

1. **Pure fold (`state.rs`).** `fn fold_rollup(statuses: &[&str]) -> PhaseRollup`
   — count `total`, `completed`, `blocked` from a slice of status strings.
   Unknown/typo statuses count toward `total` only (tolerant). No IO, no clock.
2. **IO reader (`state.rs`).** `fn phase_rollup(project_root, slice_id)
   -> anyhow::Result<Option<PhaseRollup>>` — list `phase-*.toml` under the
   id-derived `phases_dir`, tolerant-parse each `status`, fold. `None` when the
   dir is absent or holds no phase tomls (the *not tracked* signal).
3. **Slice-local presentation (`slice.rs`).** `run_list` reads metas (shared),
   sorts (shared), then pairs each `Meta` with its `phase_rollup`, and renders via
   a **slice-local** formatter that adds the rollup column. `meta::format_list` is
   left for the no-rollup callers (and `adr list`) untouched.

```text
run_list:
  metas      = meta::read_metas(slice_root, "slice")        # shared, unchanged
  ordered    = meta::sort_and_filter(metas, status_filter)  # shared, unchanged
  rows       = ordered.map(m => (m, state::phase_rollup(root, m.id)))
  stdout     = format_slice_rows(rows)                      # slice-local, new
```

### 5.2 Interfaces & Contracts

```rust
// state.rs
pub(crate) struct PhaseRollup {
    pub total: u32,
    pub completed: u32,
    pub blocked: u32,        // surfaced: a stall signal, not just incomplete
}
pub(crate) fn fold_rollup(statuses: &[&str]) -> PhaseRollup;            // pure
pub(crate) fn phase_rollup(root: &Path, slice_id: u32)
    -> anyhow::Result<Option<PhaseRollup>>;                            // IO seam

// slice.rs — pure divergence predicate + slice-local formatter
fn is_divergent(authored: &str, rollup: Option<&PhaseRollup>) -> bool;  // pure
fn format_slice_rows(rows: &[(Meta, Option<PhaseRollup>)]) -> String;   // pure
```

- CLI surface unchanged: `doctrine slice list [--status S] [-p ROOT]`. `--status`
  still filters the **authored** status (free-text), unchanged.
- **Header row** (decision Q2): a leading `id  status  phases  slug  title` line
  precedes the rows (the only output-shape change to `slice list`).
- Rendered row: `NNN  <status>  <rollup>  <slug>  <title>`:
  - `<rollup>` = `completed/total` (e.g. `4/6`); `—` when not tracked; trailing
    `!N` when `blocked > 0` (e.g. `4/6 !1`) — decision Q1.
  - `<status>` gains a trailing `⚠` when the authored status and the rollup
    **diverge** (decision Q3 / D3) — see § 5.5 for the rule.

```text
id   status     phases   slug                 title
001  done       4/6      memory-entity-v1     Memory entity v1
007  done ⚠     2/6 !1   memory-anchoring     Memory anchoring …
009  proposed   —        slice-status-rollup  Slice status rollup
```

### 5.3 Data, State & Ownership

- **Reads, never writes.** Source of truth for the rollup is the gitignored phase
  tree; the authored `slice-nnn.toml` is read (via `meta`) for its `status` only.
- **`PhaseRollup` is ephemeral** — a per-invocation value, never persisted, never
  cached. Owned by the call stack of `run_list`.
- The convenience `phases` symlink is never followed; `phases_dir` derives the
  path from the id (existing `state.rs` invariant).

### 5.4 Lifecycle, Operations & Dynamics

- `slice list` invoked → for every numeric slice dir, read authored meta + derive
  rollup → ordered by id → printed. Cost: N slices × (1 meta read + M phase-toml
  reads). Negligible at v1 scale; no index warranted.
- No mutation path. No new `main.rs` arm — the existing `slice list` arm is
  enriched.

### 5.5 Invariants, Assumptions & Edge Cases

- **No `phases/` dir / empty dir** → `None` → `—`. Never `0/0`.
- **Typo/unknown status** (hand edit) → counted in `total`, excluded from
  `completed`/`blocked`. `slice list` never errors on a malformed phase status.
- **`.md` sheet without its `.toml`** (crash-partial) → not counted (enumerate
  `phase-*.toml` only; the `.toml` is the status authority).
- **Divergence rule** (decision Q3 / D3). A pure predicate over
  `(authored_status, Option<PhaseRollup>)`:
  - `done` + rollup present + `completed < total` → **divergent** (marked done,
    work outstanding).
  - rollup present + `total > 0` + `completed == total` + `authored != "done"`
    → **divergent** (work complete, not marked done).
  - `None` rollup (untracked) → **never** divergent (nothing to compare).
  - all other pairs → not divergent.
  `"done"` is the single hardcoded terminal token (the only terminal authored
  status in use). It is **provisional** — the slice status vocabulary is not
  locked and there is no lifecycle-transition verb yet; the constant moves to a
  shared place when that verb lands (the deferred follow-up).
- **Empty repo / no slices** → header only (or empty — Q: suppress header on zero
  rows; lean: suppress, matching `format_list`'s empty-string contract).

## 6. Open Questions & Unknowns

Resolved in interview (2026-06-04):

1. **Rollup glyph** → `completed/total` + trailing `!N` for blocked (Q1).
2. **Header row** → add an `id status phases slug title` header (Q2);
   suppress it on zero rows (`format_list` empty-string contract).
3. **Divergence** → compute a coarse hint, not juxtapose-only (Q3) — § 5.5 rule.

Remaining:

4. **`in_progress` column** — not surfaced in v1; `completed/total` + blocked is
   the signal. Revisit if a per-phase detail view (follow-up) lands.
5. **Where the `"done"` terminal constant lives** — local to `slice.rs` for v1;
   moves to a shared home with the deferred lifecycle-transition verb.

## 7. Decisions, Rationale & Alternatives

- **D1 — Reader lives in `state.rs`, not `meta.rs`.** Phase state is `state.rs`'s
  domain; `meta` is the kind-neutral authored-toml substrate. *Alt:* put rollup in
  `meta` — rejected: pollutes the ADR-shared surface with a phase concept.
- **D2 — Slice-local formatter; `meta::format_list` untouched.** The rollup column
  is slice-only. *Alt A:* add `rollup: Option<…>` to `Meta` — rejected (shared
  struct, ADR carries a dead field). *Alt B:* generalise `format_list` to accept
  extra columns — rejected as premature; one extra caller doesn't justify a
  column abstraction (YAGNI). Accept small alignment duplication in `slice.rs`;
  extract a shared aligner only if a third caller appears.
- **D3 — Compute a coarse divergence hint (interview Q3).** A pure predicate over
  `(authored_status, Option<PhaseRollup>)` marks the status with `⚠` on the two
  unambiguous mismatches (done-but-incomplete; complete-but-not-done), anchored on
  a single provisional `"done"` terminal token (§ 5.5). The mapping is deliberately
  minimal — one constant, no per-status table — so it carries no commitment the
  deferred lifecycle-transition verb must honour; that verb will own the real
  vocabulary and relocate the constant. *Alt (drafted, overridden):* juxtapose-only,
  no computed flag — cleaner but leaves the reader to spot every mismatch by eye;
  rejected in interview as too passive for the gap this slice exists to close.
  *Alt:* a full status-mapping table — rejected as premature (no locked vocab).
- **D4 — `None` for untracked, not `0/0`.** Explicit absence; never reads as done.
- **D5 — Tolerant fold over a `PhaseStatus` parse-back.** A read-only display must
  not crash on a hand-edited typo. *Alt:* add `FromStr for PhaseStatus` and error
  on unknown — rejected for the list path; counting by string match is robust and
  needs no change to the `clap::ValueEnum`.
- **D6 — Recompute, no cache/index.** Disposability over a stale-cache failure
  mode; trivial cost at scale.

## 8. Risks & Mitigations

- **`meta` pollution creep** — a later "just add it to `Meta`" shortcut. Mitigation:
  D2 + a test asserting `adr list` output is byte-unchanged.
- **Output-format churn** breaking anyone parsing `slice list`. Mitigation: append
  the column in a fixed position; defer machine output to the `--format=tsv`
  follow-up rather than reshaping the human listing.
- **Misread absence** — `—` mistaken for an error. Mitigation: `—` is the
  documented "not tracked" token; covered in tests and the CLAUDE.md note.
- **Tolerant parse hides real corruption.** Accepted: a malformed phase toml is a
  state-tree problem `slice phases` surfaces; the list stays robust by design.

## 9. Quality Engineering & Validation

- **Pure fold unit tests:** empty → `0/0`; all completed; mixed; blocked present;
  unknown status counted in total only.
- **IO reader tests:** absent dir → `None`; empty dir → `None`; populated dir →
  correct counts; `.md`-only stem ignored; symlink/non-phase entries ignored
  (mirror `read_metas` fixture style).
- **Divergence predicate (pure):** `done` + incomplete → true; complete +
  non-`done` → true; `done` + complete → false; `None` rollup → false; both
  directions of the non-divergent cases.
- **Formatter (pure):** header present with rows, suppressed on empty; `—` for
  `None`; `!N` only when blocked; `⚠` only when divergent; column alignment.
- **`run_list` integration:** a slice with phases shows `X/Y`; a slice without
  shows `—`; `--status` filter still applies to authored status.
- **Behaviour-preservation:** `meta`, `entity`, state-writer, and `adr list`
  suites green unchanged; an explicit `adr list` byte-stability assertion.
- `just check` green (clippy zero-warnings, fmt) before commit.

## 10. Review Notes

Pending adversarial review (the slice-002/003/004 rhythm — second agent or codex
mcp). Focus areas for the reviewer: D2 (is the small alignment duplication the
right call vs a shared aligner?), D3 (is juxtaposition-only honest enough, or does
v1 owe at least a coarse divergence hint?), and Open Q1/Q2 (glyph + header).
