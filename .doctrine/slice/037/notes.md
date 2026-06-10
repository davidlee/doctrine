# SL-037 — implementation notes

Durable, committed. Harvested from disposable phase sheets at each phase end.

## PHASE-01 (commit `4e56756`, branch `sl-037-phase-01`)

- **R1 canary clean.** Governance migrated with zero per-kind config on
  `Column<R>`; extractors stayed trivial non-capturing `fn(&GovRow)->String`.
  Weak evidence only (gentlest kind) — slice markers (P2) and spec subtype ids
  (P3) are the real over-config tests.
- **A4 held.** `GovRow` feeds both table and JSON from one materialisation
  (`gov_rows`); no display-row split needed for governance.
- **Golden repins (R2, intended).** `e2e_adr_cli_golden` + `e2e_standard_cli_golden`
  table goldens (default + `--all`) repinned slug-free per D4. Their JSON goldens
  passed UNCHANGED — the live D2 proof. Expect the same shape per kind in P2/P3.
- **Memory guard pinned.** `tests/e2e_list_conformance.rs` black-box test asserts
  `memory list --columns` fails with "--columns is not supported for `memory list`"
  and that the rejection is the guard, not a clap parse error (D9/R4, VT-3).
  IMP-017 removes the guard when memory adopts the model.
- **Column<R> derive posture.** No `derive(Copy)` (design) and no `derive(Debug)`
  either — both add spurious `R:` bounds; tests assert errors via
  `.err().map(|e| e.to_string())` instead of `unwrap_err`.
- **dead_code arc.** Leaf-before-consumer trips `-D unused` mid-TDD-chain; when
  the consumer lands in the SAME phase, finish the chain before running the gate —
  no `cfg_attr(not(test), expect(dead_code))` needed (that pattern is for
  cross-phase gaps; see mem `dead-code-expect-vs-cfg-test`).

## PHASE-02 (commit `11efae1`, branch `sl-037-phase-01`)

- **R1 STRONGLY refuted.** slice was R1's hardest test — the `?`/`⚠`
  drift+divergence markers AND the `completed/total` rollup both absorbed as plain
  `String` cell values via the existing `decorated_status`/`phases_cell` fns reused
  as non-capturing `fn(&SliceRowTuple)->String` extractors. `Column<R>` needed ZERO
  per-kind config. The over-abstraction IMP-013 feared did not materialise on the
  marker-bearing kind. spec (subtype id, P3) is the last canary.
- **backlog `R = BacklogItem` directly — no display row.** `resolution` is
  JSON-only (nullable, never a table cell), so the table-cell projection over the
  live item is total; no GovRow/SpecListRow-style intermediate needed.
- **slice uses the existing tuple as `R` via a `type SliceRowTuple` alias.**
  `const [Column<(Meta, Option<PhaseRollup>)>; 5]` is unwieldy inline; the alias
  keeps the const + test helpers readable. No new struct (A4 — tuple is the row).
- **No P2 golden repins (contrast P1).** backlog/slice have no e2e *table* golden
  yet — the cross-verb byte-exact harness is P4 (IMP-014/D8). Existing e2e
  (`e2e_backlog_filter_alias`) is filter/alias behaviour, slug-agnostic, passed
  unchanged. JSON envelopes byte-identical (D2). Direct-`render_table` slice unit
  tests migrated to the column path via `render_default`/`render_cols` helpers.

## PHASE-04 — IMP-014 cross-verb golden harness (branch `sl-037-phase-01`)

The regression net for the column churn (D8). TESTS ONLY — no production edit.
New module `tests/e2e_list_columns_golden.rs`, 16 golden cases.

- **T0 settled as a GAP-FILL net, not a re-render.** IMP-014's literal "one place"
  reading would have DUPLICATED the adr/standard/memory assertions the slice grew
  since (parallel implementation, CLAUDE.md). The module OWNS the genuine gaps and
  CITES the rest by `path:line` in the header — duplicating nothing. T0 needed no
  `/consult`; the cite-vs-own reading held in practice.
  - **OWNED** (no prior test pinned these): backlog/slice/spec/policy DEFAULT
    tables; the `--columns` projection (selection+order+slug-reveal) across all four
    migrated verbs — `--columns` was invoked by NO test before this; the empty-list
    `""` path per verb; spec multi-block + omitted-empty-block at the CLI surface;
    governance `policy` breadth; the uniform unknown-column error; the D7
    `--columns`-under-`--json` no-op.
  - **CITED** (already green): memory `--columns` rejection
    (`e2e_list_conformance.rs:126`); adr/standard slug-free default + `--json`
    (`e2e_adr_cli_golden.rs:286-330` / `e2e_standard_cli_golden.rs:288-332`);
    `{kind, rows}` envelope shape (`e2e_list_conformance.rs:149`).
- **Memory rejection: CITED, not re-homed (A4).** Left physically in
  `e2e_list_conformance.rs`; the new module references it in the T8 header comment.
  Re-homing would have churned a green test for no coverage gain.
- **A6 paid off — fixtures baked from OBSERVED bytes, not guessed.** Probed the live
  binary against hand-seeded corpora before writing any `assert_eq!`; all 16 cases
  passed first compile. Concrete shapes learned for the un-seeded kinds:
  - **slice phases cell with NO state tree = `—`** (em-dash, untracked), JSON
    `phases: null`. The slice golden is byte-stable WITHOUT seeding a runtime state
    tree (A6's open question — resolved: no state needed).
  - **spec multi-block = `<label>\n<table>` per subtype, concatenated with NO blank
    separator** (`product\n…tech\n…`); an empty subtype block (label + grid) is
    omitted ENTIRELY (R3).
  - **spec `members.toml` member shape is `{requirement, label, order}`** — `list`
    only COUNTS members, so no REQ tree is dereferenced (cheap fixtures).
  - **backlog sorts by `(kind.ordinal, id)`**, not id — issue before improvement.
  - **empty list = literally empty stdout** (`""`, header suppressed) on every verb.
  - **list `--json` has NO trailing newline** (`write!`, not `writeln!`).
- **D7 confirmed at the surface.** `--columns X --json` is byte-identical to plain
  `--json` (projection taken before the JSON build) — pinned as an equality assert.
- **Gate green.** Full suite 861 passed / 0 failed (16 new); `cargo clippy`
  (bins/lib) zero warnings; `just check` exit 0; `cargo fmt --check` clean. The
  behaviour-preservation gate held: every pre-existing suite green UNCHANGED.
