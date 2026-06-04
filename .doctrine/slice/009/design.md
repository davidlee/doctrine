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
   — fold a per-stem status slice into the full bucket set. No IO, no clock.
   Each stem carries either its `.toml` status string or a *missing-toml* marker
   (a `.md`-only crash-partial), so the fold never undercounts `total` (R-F4).
2. **IO reader (`state.rs`).** `fn phase_rollup(project_root, slice_id)
   -> anyhow::Result<Option<PhaseRollup>>` — discover the phase set via the
   **existing** `existing_phase_stems` (which counts a stem from *either* half of
   the pair — the same notion `init_phases` uses), classify each stem as
   `.toml`-present (tolerant-parse its `status`) or `.md`-only (`missing_toml`),
   then fold. `None` only when no stem exists at all (dir absent or empty) — the
   *not tracked* signal. (R-F4: never re-derive the phase set from `*.toml` alone.)
3. **Neutral table renderer (`meta.rs`).** Extract the one layout concern
   `meta::format_list` embeds — measure each column to its max cell width,
   left-align, two-space gap, last column unpadded, empty→`""`, trailing newline —
   into `render_table(rows: &[Vec<String>]) -> String`. `format_list` is
   reimplemented over it (output byte-unchanged); `slice.rs` calls the same
   renderer with its own cells, including the middle `phases` column and the
   header row. This is the single layout authority — no width/gap/newline logic is
   duplicated (R-F1) — and it is **not** a column framework: no per-column specs,
   alignment enums, or header config, just "render a grid of strings". `meta`
   learns nothing about phases; the `⚠`/`!N`/`?N` markers are baked into the cell
   strings by `slice.rs` before rendering, so `meta` stays neutral.
   *(Refines the round-1 `measure_meta_columns`/`format_meta_row(status_suffix)`
   sketch: a whole-row meta formatter can't host slice's middle `phases` column,
   and a `status_suffix` arg would be dead in `meta` — a cell-grid renderer
   composes for any column set and carries no phase concept.)*
4. **Slice-local presentation (`slice.rs`).** `run_list` reads metas (shared),
   sorts (shared), pairs each `Meta` with its `phase_rollup`, and renders via a
   slice-local formatter built on the `meta` helpers plus the `phases` column.

```text
run_list:
  metas      = meta::read_metas(slice_root, "slice")        # shared, unchanged
  ordered    = meta::sort_and_filter(metas, status_filter)  # shared, unchanged
  rows       = ordered.map(m => (m, state::phase_rollup(root, m.id)))
  stdout     = format_slice_rows(rows)                      # slice-local, new
```

### 5.2 Interfaces & Contracts

```rust
// state.rs — all status buckets carried; total is their sum (R-F3, R-F5)
pub(crate) struct PhaseRollup {
    pub planned: u32,
    pub in_progress: u32,
    pub completed: u32,
    pub blocked: u32,        // stall signal
    pub unknown: u32,        // status string outside the known enum (corruption)
    pub missing_toml: u32,   // .md-only stem: phase exists, status unreadable
}
impl PhaseRollup {
    pub fn total(&self) -> u32;          // sum of every bucket — never undercounts
    pub fn anomalies(&self) -> u32;      // unknown + missing_toml
}
// each stem reduces to one of these before the fold
enum StemStatus<'a> { Toml(&'a str), MissingToml }
pub(crate) fn fold_rollup(stems: &[StemStatus]) -> PhaseRollup;        // pure
pub(crate) fn phase_rollup(root: &Path, slice_id: u32)
    -> anyhow::Result<Option<PhaseRollup>>;                            // IO seam
// the terminal-status set — the ONE place "done" is named; the future
// lifecycle-transition verb reuses this, never re-hardcodes (R-F2).
pub(crate) fn is_terminal_status(authored: &str) -> bool;             // pure

// meta.rs — the single layout authority for every *list surface
pub(crate) fn render_table(rows: &[Vec<String>]) -> String;            // pure

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
    `!N` when `blocked > 0`; trailing `?N` when `anomalies > 0` (`unknown +
    missing_toml`) — decision Q1, extended by R-F3/R-F4.
  - `<status>` gains a trailing `⚠` when authored status and rollup **diverge**
    (decision Q3 / D3) — see § 5.5. Divergence is **suppressed when anomalies > 0**
    (corrupt tracking is not a lifecycle mismatch — R-F3).
- **`slice list` output is human-only.** The header + `phases` column make it
  structurally distinct from `adr list`; machine consumers wait for the
  `--format=tsv` follow-up (R-F6). `adr list` output stays byte-identical.

```text
id   status     phases   slug                 title
001  done       4/6      memory-entity-v1     Memory entity v1
007  done ⚠     2/6 !1   memory-anchoring     Memory anchoring …
008  proposed   3/6 ?1   memory-retrieval     Memory retrieval …
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
- **Typo/unknown status** (hand edit) → counted in `total` *and* in `unknown`;
  surfaced as `?N`, never silently bucketed (R-F3). `slice list` never errors.
- **`.md` sheet without its `.toml`** (crash-partial) → the stem still counts
  toward `total` as `missing_toml` (R-F4); surfaced in `?N`. The phase set is the
  one `existing_phase_stems`/`init_phases` see, so `total` never silently shrinks.
- **Divergence rule** (decision Q3 / D3). A pure predicate over
  `(authored_status, Option<&PhaseRollup>)`, keyed on `is_terminal_status` — never
  the bare string `"done"` (R-F2):
  - `anomalies > 0` → **never** divergent (corruption ≠ lifecycle mismatch).
  - `None` rollup (untracked) → **never** divergent (nothing to compare).
  - `is_terminal_status(authored)` + `completed < total` → **divergent** (marked
    terminal, work outstanding).
  - `!is_terminal_status(authored)` + `total > 0` + `completed == total` →
    **divergent** (work complete, not marked terminal).
  - all other pairs → not divergent.
  `is_terminal_status` is the **single** place a terminal token is named (v1 set:
  `{"done"}`). Because both directions key on it, a future synonym (`completed`,
  `closed`) is added in one tested function and stops false-flagging everywhere at
  once — and the deferred lifecycle-transition verb *reuses* this set rather than
  re-deriving it. The set is provisional; its membership, not the predicate shape,
  is what may change.
- **Empty repo / no slices** → empty output, header suppressed (matches
  `format_list`'s empty-string contract).

## 6. Open Questions & Unknowns

Resolved in interview (2026-06-04):

1. **Rollup glyph** → `completed/total` + trailing `!N` for blocked (Q1).
2. **Header row** → add an `id status phases slug title` header (Q2);
   suppress it on zero rows (`format_list` empty-string contract).
3. **Divergence** → compute a coarse hint, not juxtapose-only (Q3) — § 5.5 rule.

Remaining:

4. **`in_progress` column** — carried in `PhaseRollup` but not rendered in v1;
   `completed/total` + `!N` + `?N` is the surfaced signal. The detail-view
   follow-up renders the full bucket set.
5. **Untracked-vs-empty distinction** — both fold to `None` → `—` in v1 (R-F5
   partial). If the `slice status <ID>` detail view needs to tell "no phases dir"
   from "dir, zero phases" apart, promote the reader return to a `PhaseTracking`
   enum then — deferred, not built unused now.

## 7. Decisions, Rationale & Alternatives

- **D1 — Reader lives in `state.rs`, not `meta.rs`.** Phase state is `state.rs`'s
  domain; `meta` is the kind-neutral authored-toml substrate. *Alt:* put rollup in
  `meta` — rejected: pollutes the ADR-shared surface with a phase concept.
- **D2 — Extract a neutral `render_table` in `meta.rs`; no duplicated layout
  (R-F1).** The rollup column is slice-only, but the layout (measure, left-align,
  two-space gap, last col unpadded, empty→`""`, trailing newline) is shared. Pull
  it out of `format_list` into `render_table(rows: &[Vec<String>])`, reimplement
  `format_list` over it (byte-unchanged), and let `slice.rs` call it with its own
  cells — including the middle `phases` column and the header row. *Alt A:* add a
  rollup field to `Meta` — rejected (ADR carries a dead field). *Alt B (drafted,
  overridden):* copy the alignment logic into `slice.rs` — rejected on re-review:
  forks the two list surfaces the moment spacing changes. *Alt C
  (round-1 sketch, overridden):* `measure_meta_columns` + `format_meta_row(status_
  suffix)` — rejected: a whole-row meta formatter can't host slice's *middle*
  `phases` column, and the suffix arg is dead in `meta`. *Alt D:* a full N-column
  framework (per-column align/width specs, header config) — rejected as
  premature; `render_table` renders a string grid and nothing more.
- **D3 — Compute a coarse divergence hint keyed on a terminal-set helper (Q3,
  R-F2).** `is_divergent` marks `⚠` on the two unambiguous mismatches, both keyed
  on `is_terminal_status` — never the bare string `"done"`. So the v1 set
  `{"done"}` extends in one tested place (synonyms `completed`/`closed` stop
  false-flagging at once) and the deferred lifecycle verb reuses it. Suppressed
  when anomalies present. *Alt (drafted, overridden):* hardcode `"done"` inline —
  rejected on re-review: a latent false-flag trap against the unlocked free-text
  vocabulary. *Alt:* juxtapose-only — rejected in interview (too passive).
  *Alt:* full status-mapping table — premature (no locked vocab).
- **D4 — `None` for untracked, not `0/0`.** Explicit absence; never reads as done.
- **D5 — Tolerant fold, but surface anomalies explicitly (R-F3).** A read-only
  display must not crash on a hand-edited typo — but it must not *hide* the
  corruption either. Unknown statuses and `.md`-only stems are counted in dedicated
  buckets (`unknown`, `missing_toml`), rendered `?N`, and suppress the divergence
  hint. *Alt (drafted, overridden):* count unknowns in `total` only — rejected:
  makes corrupt tracking look like authoritative progress. *Alt:* `FromStr`-error
  on unknown — rejected; crashes the list path on one bad keystroke.
- **D6 — Recompute, no cache/index.** Disposability over a stale-cache failure
  mode; trivial cost at scale.
- **D7 — `PhaseRollup` carries every bucket; `total()`/`anomalies()` derived
  (R-F5).** All six status buckets are stored so `total` is their sum (cannot
  undercount) and the detail-view / `--format=tsv` follow-ups need no reshape. The
  reader still returns `Option` (None = untracked); the richer `PhaseTracking` enum
  is deferred until a caller needs untracked-vs-empty (Open Q5) — carry the data,
  not an unused type.
- **D8 — Phase set comes from `existing_phase_stems`, not a fresh `*.toml` scan
  (R-F4).** Reuse the module's own definition of "what is a phase" (either half of
  the pair) so the rollup agrees with `init_phases`; a `.md`-only crash-partial is
  `missing_toml`, never a vanished phase. No parallel enumeration.

## 8. Risks & Mitigations

- **`meta` pollution creep** — a later "just add it to `Meta`" shortcut. Mitigation:
  D2's neutral row helpers give the clean alternative + a test asserting `adr list`
  output is byte-unchanged.
- **Output-format churn** breaking anyone parsing `slice list`. Mitigation: fixed
  column position; `slice list` documented human-only; machine output deferred to
  the `--format=tsv` follow-up (R-F6).
- **Misread absence** — `—` mistaken for an error. Mitigation: `—` is the
  documented "not tracked" token; covered in tests and the CLAUDE.md note.
- **Anomaly markers misread as progress** — `?N` taken for a count. Mitigation:
  `?N` is documented as the corruption marker; divergence suppressed under it so a
  corrupt slice never also reads as a lifecycle mismatch (R-F3).
- **Terminal-set drift** — `is_terminal_status` and a future lifecycle verb
  disagree. Mitigation: one shared function, reused not re-derived (D3/R-F2);
  membership change is one tested edit.

## 9. Quality Engineering & Validation

- **Pure fold unit tests:** empty; all completed; mixed; blocked present; unknown
  status → `unknown` bucket; `missing_toml` stem counted in `total`; `total()` =
  sum of buckets; `anomalies()` correct.
- **IO reader tests:** absent dir → `None`; empty dir → `None`; populated → correct
  buckets; `.md`-only stem → `missing_toml` (counted, not dropped — R-F4);
  `.toml` typo status → `unknown`; symlink/non-phase entries ignored (mirror
  `read_metas` fixture style); phase set agrees with `existing_phase_stems`.
- **`is_terminal_status` (pure):** `"done"` true; `"proposed"`/unknown false —
  the single tested terminal-set gate (R-F2).
- **Divergence predicate (pure):** terminal + incomplete → true; non-terminal +
  complete → true; terminal + complete → false; `None` rollup → false; `anomalies
  > 0` → false (suppressed); both non-divergent directions.
- **`meta::render_table` (pure):** `format_list` reimplemented over it reproduces
  the old output byte-for-byte (regression lock); empty→`""`; ragged column
  widths align; last column unpadded; a multi-cell grid with a middle column
  aligns (the slice case).
- **Formatter (pure):** header present with rows, suppressed on empty; `—` for
  `None`; `!N` only when blocked; `?N` only on anomalies; `⚠` only when divergent
  and not suppressed; column alignment with mixed-width rollups.
- **`run_list` integration:** a slice with phases shows `X/Y`; without → `—`;
  `--status` still filters authored status.
- **Behaviour-preservation:** `meta`, `entity`, state-writer suites green
  unchanged; an explicit `adr list` **byte-identical** assertion (R-F1/R-F6).
- `just check` green (clippy zero-warnings, fmt) before commit.

## 10. Review Notes

**Adversarial review — codex (gpt-5), 2026-06-04.** 5 MAJOR + 1 MINOR, all
adjudicated and folded:
- **R-F1** (alignment fork) → accepted. D2 now extracts neutral `meta` row helpers
  instead of copying layout into `slice.rs`.
- **R-F2** (`"done"`-only divergence trap) → accepted. `is_terminal_status` helper;
  both predicate directions key on it; future verb reuses it (D3, § 5.5).
- **R-F3** (tolerant fold hides corruption) → accepted. `unknown` bucket, `?N`
  marker, divergence suppressed under anomalies (D5).
- **R-F4** (`*.toml`-only undercounts total) → accepted. Phase set via
  `existing_phase_stems`; `.md`-only stem → `missing_toml` (D8).
- **R-F5** (lossy interface) → partially accepted. `PhaseRollup` carries every
  bucket (D7); the 4-variant `PhaseTracking` enum deferred to its first caller
  (Open Q5) rather than built unused.
- **R-F6** (header churn) → kept the header (interview Q2) + `adr list`
  byte-stability test + `slice list` documented human-only; `--format=tsv`
  remains the follow-up.

Decisions locked. Ready for `slice plan`.
