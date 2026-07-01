# SL-184 Plan: Rename memory find + shared listing spine

## Sequencing Rationale

### Why two phases?

The work splits cleanly at a compile-and-test boundary:

**PHASE-01** is purely mechanical: every `Find`/`find` reference in the codebase
is renamed to `Search`/`search`. The `Find` variant is replaced by
`Search` with a hidden `alias = "find"`. MCP tool, internal functions, test fns.
No behavioural change — test output is byte-identical.

**PHASE-02** is where the output surface changes: the hand-rolled
`format_search_table` (renamed in PHASE-01) is replaced by `listing::render_columns`
with `search_columns()` and `--columns`. This changes table alignment, adds
headers and colours, and requires updating golden assertions and the three
`format_find_*` tests that asserted on the old output shape.

The boundary means:
- PHASE-01 is trivially reviewable as a rename diff
- If there's a regression after PHASE-02, the rename is already proven
- A reviewer can see the old `format_search_table` get deleted and the new
  `search_columns()` + `render_columns` appear in one phase

### Phase order dependencies

- PHASE-02 depends on PHASE-01 because `run_search`, `format_search_table`, etc
  must exist before they can be replaced
- No back-edges (PHASE-02 doesn't add symbols PHASE-01 needs)

## Phase Details

### PHASE-01: Rename find → search

**Scope:** All surfaces — CLI, internal, MCP, tests.

The rename is grep-driven: every occurrence of `Find` (as a variant/struct name)
and `find` (as a function/method name) in the find→search axis is renamed.
`memory_find` becomes `memory_search` in MCP. The `FindRetrieveArgs` struct name
is kept — renaming it is work-surface noise (only 4 refs).

The `color: bool` parameter is threaded into `run_search`'s signature even though
PHASE-01 doesn't use it yet — this avoids a second signature change in PHASE-02.

Expected diff: ~40-50 lines changed across 6 files, all mechanical.

### PHASE-02: Shared listing spine

**Scope:** Column definitions, `--columns` flag, render replacement.

The 15 columns are defined in `search_columns()` (function, not const, per
adversarial finding #1). The default 8-column set exactly matches the current
`format_find_table` order so default output is semantically equivalent
(different layout — comfy-table with headers and colours).

`run_search` switches from:
```
format_find_table(&visible)  // Vec<&Candidate>
```
to:
```
let visible = &ranked[offset..end];  // &[Candidate]
listing::render_columns(visible, &sel, RenderOpts { color, term_width: None })
```

The three `format_find_*` tests that asserted on the old hand-rolled layout are
rewritten as column-definition + render-columns tests.

Risk: low. The `listing::render_columns` code path is well-exercised by REC and
REVIEW list surfaces. The `Candidate<'_>` lifetime requires the slice-not-refs
pattern, which the adversarial review already caught.

## Risk Assessment

| Risk | Likelihood | Mitigation |
|---|---|---|
| `Candidate<'a>` lifetime makes column consts impossible | Caught in design | Use `fn search_columns()` instead |
| comfy-table output breaks golden tests | Certain | Update goldens in PHASE-02 (explicitly expected) |
| `--columns` bleeds into `retrieve --help` | Certain | Documented in design; no behavioural impact |
| `FindRetrieveArgs.columns` conflicts with clap flatten | Low | Single field, no naming conflict |
