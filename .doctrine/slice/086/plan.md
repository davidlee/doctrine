# SL-086 Implementation Plan

## Phase ordering rationale

Three phases, in dependency order.

**PHASE-01 goes first** because it establishes the shared `FindRetrieveArgs` struct
that PHASE-02 extends. The args-struct ceiling (`too_many_arguments` at >7 params,
`fn_params_excessive_bools` at >3 bools) is ALREADY breached by the existing
`Find` (9 fields) and `Retrieve` (11 fields) variants. Adding pagination, format,
and query fields would push to 15-17 — well past the ceiling. The struct extraction
is the enabling refactor that all subsequent changes ride.

PHASE-01 also wires `--json` on `memory find` (IMP-092) because it's the simplest
of the three IMPs touching the memory command surface: a Format field, a JSON
serialization path, and an envelope call. This establishes the pattern (format
resolution, `--json` wins over `--format` precedence) that PHASE-02's truncation
notice respects (notice suppressed under JSON).

**PHASE-02 builds on PHASE-01's args struct**, adding positional query and
pagination fields. These share the same clap surface and rendering module
(`retrieve.rs`), so separating them into distinct phases would force rework.
IMP-090 (positional query) and IMP-091 (pagination) are bundled because:

- Both extend the `FindRetrieveArgs` struct
- Both touch the rendering pipeline (format_find_table, truncation notice)
- The truncation notice's page computation depends on offset/limit exactly as
  the pagination defines them
- They're small enough to review and test in one pass

**PHASE-03 is file-disjoint from PHASE-01/02.** New module `src/status.rs`, new
command variant in `src/main.rs` (non-overlapping with the `MemoryCommand::Find`
variant). Only shared touch: `main.rs` adds a new `Command::Status` variant (and
the required `CommandClass` arm + test helper), which is trivially mergeable.
PHASE-03 could run in parallel with PHASE-01/02 if dispatched, but serial is fine
given the small scope.

## Verification strategy

Each phase drives TDD red→green→refactor. The golden-test pattern is the primary
VT surface:

- **PHASE-01**: Black-box golden on `memory find --json` output shape. Existing
  table output byte-compared to pre-refactor output (behaviour-preservation gate).
- **PHASE-02**: Goldens for positional query equivalence, pagination page boundaries,
  truncation notice text, and JSON suppression of notice.
- **PHASE-03**: Golden on full `doctrine status` output against a known test corpus,
  plus `--json` parseability check.

VA-1 (PHASE-03) is an agent-verified criterion: blocked items respect hard `needs`
edges only. This cannot be a pure golden test because it requires validating the
semantics of the dep/seq graph traversal, which is easier for an agent to
reason about than to encode in a byte-exact output comparison.

## Risk

- **Args-struct clap flattening.** `#[command(flatten)]` on a borrowed struct with
  clap attributes needs care — the existing `CommonListArgs` pattern proves it
  works, but `FindRetrieveArgs` carries more field types (Vec, Option, usize, Format).
  Risk: low. Mitigation: follow `CommonListArgs` pattern exactly.
- **`run_find`/`run_retrieve` signature churn.** Adding params to these functions
  touches call sites in two places (dispatch + test helpers in main.rs). Risk: low.
  Mitigation: the test helper in main.rs uses `..Default::default()` for new fields.
- **`doctrine status` git dependency.** `git log` is impure and needs the cwd to be
  in the repo. Risk: low. Mitigation: `root::find` already resolves the repo root;
  `git log` runs in that directory. If git is unavailable, the section is silently
  suppressed.
