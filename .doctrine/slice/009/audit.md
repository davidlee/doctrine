# SL-009 audit — slice status rollup

Hand-authored close-out (no `slice audit` scaffold yet — known CLI gap). Verifies
the shipped read-only rollup against `design.md` and the locked decisions D1–D8 /
R-F1–R-F6, and records the durable risks, divergences, and follow-ups harvested
from the three phase sheets.

- **Status:** all 3 phases completed; `just check` = 361 unit + 4 e2e green, clippy
  zero (bin). Commits `b8ef29e` (scope) · `cde8280`/`7614119` (design) · `f3db4bd`
  (plan) · `2bd527e` (design refine) · `c9d7653` (PHASE-01 `render_table`) ·
  `0c2a1ab` (PHASE-02+03 rollup core + `slice list` wiring) · `62e4a59` (notes).
- **Verdict:** ships. No defect. Two plan-vs-shipped deviations (D-A1 placement,
  D-A2 phase-collapse) are already harvested in `notes.md` and need no further
  action; one coverage gap (C-1, no spawned-binary e2e for `run_list`) is a
  tolerated v1 edge, not a closure blocker. The slice dogfoods its own `⚠` (009 is
  authored `proposed` at 3/3 → divergent) — closure flips it.

## Coverage vs design

| Area | Design | Shipped | Note |
|------|--------|---------|------|
| Neutral table renderer | §5.1.3, D2/R-F1 | `meta::render_table(rows: &[Vec<String>])` | single layout authority; `format_list` reimplemented over it ✓ |
| `meta` stays phase-blind | §3, EX-3, D2 | `Meta` unchanged; markers baked into cells by `slice.rs` | no phase/rollup field in `meta` ✓ |
| Pure fold | §5.1.1/§5.2, D7 | `fold_rollup(&[StemStatus]) -> PhaseRollup` | known set = `PhaseStatus` value names (one vocab source) ✓ |
| Bucket carriage | §5.2, D7/R-F5 | 6 buckets; `total()`=sum, `anomalies()`=unknown+missing | never undercounts ✓ |
| IO reader | §5.1.2/§5.2, D8/R-F4 | `phase_rollup` over `existing_phase_stems` | `.md`-only → `missing_toml`; `None` iff no stem ✓ |
| Terminal set | §5.5, D3/R-F2 | `slice::is_terminal_status` (`{"done"}`) | single source; moved to `slice.rs` (notes deviation) ✓ |
| Divergence predicate | §5.5, D3 | `is_divergent(authored, Option<&PhaseRollup>)` | anomalies/None suppressed; keyed on terminal helper ✓ |
| Rollup cell | §5.2 | `phases_cell`: `c/t` `!N` `?N` `—` | `?N` part of denominator (`total()` sums anomalies) ✓ |
| Slice-local formatter | §5.1.4/§5.2 | `format_slice_rows` over `render_table` | header row; suppressed on empty ✓ |
| `run_list` wiring | §5.1, EX-3 | pairs sorted `Meta` with `phase_rollup`; no new arm | `--status` still filters authored status ✓ |
| `adr list` byte-stable | §5.2, R-F1/R-F6 | `adr::list_rows == meta::format_list` test | proven via `format_list`'s own regression lock ✓ |

## Correctness

- **`render_table` layout authority — VERIFIED.** `meta.rs:54` is the one place
  width/gap/newline live: per-column max over `chars().count()`, two-space `COL_GAP`,
  last cell of each row unpadded (`c != last`), empty rows → `""`, trailing `\n`.
  `format_list` (`:91`) and `format_slice_rows` (`slice.rs:437`) both render through
  it — no duplicated alignment logic (R-F1 satisfied). The `render_table_aligns_a_
  middle_column_the_slice_case` unit proves the 5-cell middle-`phases` grid aligns,
  which the round-1 `format_meta_row(status_suffix)` sketch could not host (D2).
- **Tolerant fold, anomalies surfaced — VERIFIED.** `fold_rollup` (`state.rs:103`)
  reuses `PhaseStatus::from_str` as the known-set oracle (no parallel string list);
  `Err(_)` → `unknown`, `MissingToml` → `missing_toml`. `read_phase_status`
  (`:271`) maps absent/unparseable/`status`-less `.toml` to `None` ⇒ `MissingToml`,
  and only a non-`NotFound` IO error propagates — `slice list` never crashes on a
  hand-edit (D5/R-F3). Unit + tempdir tests cover empty, mixed, blocked, typo, and
  `.md`-only-counted-not-dropped (R-F4).
- **`total()` never undercounts — VERIFIED.** Phase set comes from
  `existing_phase_stems` (either half of the pair — the same notion `init_phases`
  uses, `:204`), not a fresh `*.toml` scan, so a crash-partial `.md` keeps its slot
  as `missing_toml` (D8). `total()` sums all six buckets; the `?N` count is inside
  the denominator (matches design example `3/6 ?1`).
- **Divergence keyed on the terminal helper — VERIFIED.** `is_divergent`
  (`slice.rs:404`): `None` → false, `anomalies()>0` → false (corruption ≠ lifecycle
  mismatch), then the two unambiguous mismatches, both via `is_terminal_status` —
  no bare `"done"` literal (R-F2). The unit table covers both true directions and
  all three suppression cases. Live: 009 (`proposed`, 3/3) and 011 (`in_progress`,
  6/6) both correctly flag `⚠`; 005–008/010 (terminal + complete) correctly do not.
- **Read-only — VERIFIED.** `run_list` (`:465`) only reads: `read_metas` (authored
  toml) + `phase_rollup` (gitignored state). No writer is reachable from the list
  path; the authored `slice-nnn.toml` is never touched. The safest slice shape (§4).
- **Path id-derived, symlink never followed — VERIFIED.** `phases_dir` (`:133`)
  derives from the id; the convenience `phases` symlink is not read (§5.3).

## Plan-vs-shipped deviations (already harvested in notes.md — no action)

- **D-A1 — `is_terminal_status` lives in `slice.rs`, not `state.rs`** (plan
  PHASE-02 EX-3 named `state.rs`). Moved for cohesion: it is slice-*authored*-status
  vocabulary, belongs beside `is_divergent` and the deferred lifecycle-transition
  verb, not in the phase-runtime-state module. Single definition confirmed (no
  `terminal` token anywhere in `state.rs`) — no parallel impl. The design intent
  (one terminal-set source the future verb reuses, R-F2) is fully honoured; only
  the file moved. Disposition: **aligned** (the design fixes the *contract*, not the
  module; notes records the move).
- **D-A2 — PHASE-02 and PHASE-03 landed in one commit** (`0c2a1ab`). The rollup
  core (PHASE-02) has no production consumer until the list wiring (PHASE-03), so it
  cannot pass `-D dead_code`/`unused` as a standalone commit. The logical phase
  boundary is preserved in `plan.toml`; execution collapsed it. Disposition:
  **tolerated drift** with rationale — a "pure core" phase whose sole consumer is
  the next phase structurally can't end green alone under the repo's lint denies
  (notes records the lesson for future plans). Both phases' EX/VT criteria are met
  in the combined commit.

## Accepted risks / known edges

- **C-1 — no spawned-binary e2e for `run_list`.** VT-3 ("a slice with materialised
  phases shows `X/Y`; without → `—`; `--status` filters authored status") is proven
  at the *pure-seam* level — `format_slice_rows`, `phases_cell`, `is_divergent`,
  `phase_rollup`, and `read_metas` each have direct tests — plus the recorded manual
  dogfood (`notes.md`, re-confirmed live this audit: 005→`6/6`, 009→`3/3`,
  001–004→`—`). There is no `tests/e2e_*` that spawns the binary on `slice list`,
  unlike the memory slices. Disposition: **tolerated drift** — every behaviour in the
  pipeline is unit-covered and the wiring (`run_list`, `:465`) is three reused calls
  with no new logic; the dogfood closes the integration loop. A spawned e2e is a
  cheap follow-up if `slice list` grows logic, not a v1 blocker.
- **A-1 — `render_table` measures `chars().count()`, old `format_list` measured
  `.len()` (bytes).** For the ASCII authored vocabulary these are identical, so the
  `format_list` regression lock and the `adr list` byte-stability test pass
  byte-for-byte. For a hypothetical multi-byte status/slug the new code aligns by
  display-char (the *correct* choice — it is what lets `⚠`/`—` align in the slice
  column). Disposition: **aligned** — a behaviour improvement on a path the current
  data can't exercise, required by the rollup column; the byte-identical gate holds
  for real data. Worth a line in §5.1 if the design is ever revisited.
- **A-2 — `⚠`/`!N`/`?N` alignment is by char-count, not terminal display-width.**
  `render_table` counts `⚠` (and `—`) as one column each; on terminals that render
  them double-width, padding is off by one. Acceptable: `slice list` is documented
  human-only output (R-F6), the markers sit mid-row (not the unpadded last column),
  and the machine path waits for `--format=tsv`. Recorded, not fixed.

## Doctrine adherence

- **Pure/impure split honoured.** The pure core (`fold_rollup`, `is_terminal_status`,
  `is_divergent`, `phases_cell`, `format_slice_rows`, `render_table`) reads no clock,
  git, or disk and is unit-tested off in-memory values. IO is confined to the thin
  shell (`phase_rollup`/`read_phase_status`/`read_metas`/`run_list`) — the date/uid
  pattern, inverted to a reader (§3).
- **No parallel implementation.** `render_table` is the single layout authority
  (R-F1, D2); the known-status set is `PhaseStatus`'s own values (one vocab source);
  the phase set is `existing_phase_stems`, not a fresh scan (D8); `is_terminal_status`
  has exactly one definition (verified absent from `state.rs`).
- **Behaviour-preservation gate.** `meta`/`entity`/state-writer suites green
  unchanged; `format_list` reimplemented byte-identically over `render_table` (its
  pre-existing tests pass verbatim — the regression lock); `adr::list_rows ==
  meta::format_list` asserts `adr list` is byte-stable (R-F1/R-F6). `Meta` carries
  no phase field — the ADR-shared surface stays neutral (EX-3).
- **Storage rule.** The rollup is derived, computed per invocation from the
  gitignored state tree, displayed, never written to the authored `slice-nnn.toml`
  (§3, D4/D6). No cache, no index, no schema change (matches the slice non-goals).
- **CLAUDE.md updated.** The "no slice status rollup" known-gap note is replaced by
  the "SHIPPED (SL-009)" entry documenting the rollup column and its markers
  (PHASE-03 EX-4) — confirmed in the live project instructions.

## second-pass: confirmed (independent /code-review, 2026-06-05)

Adversarial second pass over `git diff c9d7653~1..62e4a59 -- src/` (the whole
slice). Tried to REFUTE the read-only, no-undercount, and layout-authority claims;
all hold. Gate re-run green: **361 unit + 4 e2e, clippy zero (bin)**.

- **Gate flake diagnosed, not a regression.** First `just check` failed 4 e2e with
  `spawn doctrine: NotFound` — the recorded `mem.pattern.testing.stale-cargo-bin-exe`
  pattern: the test binary embeds a stale `env!("CARGO_BIN_EXE_doctrine")` path.
  `touch tests/*.rs` forced a recompile and all 4 e2e passed; the clean re-run is
  green. Not a SL-009 defect — the slice ships no e2e and touches no spawn path.
- **Read-only** — traced every branch of `run_list`/`phase_rollup`/`read_phase_status`:
  the only `fs::write`/`create_*` in `state.rs` sit in `init_phases`/`set_phase_status`,
  neither reachable from the list arm. Confirmed.
- **`total()` cannot undercount** — `phase_rollup` folds over `existing_phase_stems`
  (either half of the pair); a `.md`-only stem becomes `MissingToml` ⇒ counted. The
  `phase_rollup_md_only_stem_is_missing_toml_not_dropped` tempdir test proves it.
  Confirmed.
- **Divergence suppression order** — `is_divergent` checks `None` then `anomalies()>0`
  *before* the terminal comparison, so a corrupt slice never also reads as a lifecycle
  mismatch (R-F3); `is_terminal_status` is the sole keying — grepped, no stray `"done"`
  literal in the predicate. Confirmed.
- **`adr list` byte-stable** — `format_list` reduced to a `render_table` call with the
  same column order; old byte-`.len()` vs new char-count diverges only on multi-byte
  cells (A-1), absent from the ASCII authored vocabulary, so the regression + adr
  byte-stability tests pass byte-for-byte. Confirmed.

Independent checks beyond the four probes: `phases_cell`'s `?N`/`!N` markers are
fixed glyphs (no free text — no injection surface); the `⚠` is appended in the
mid-row status cell, never the unpadded last column, so it can't corrupt alignment
of following columns; `run_list` adds no logic over three reused shared calls. The
two plan deviations are honestly harvested in `notes.md` rather than retrofitted into
the plan. No defect found — close-out stands.
