# Unified corpus health doctor verb

## Context

Corpus integrity checking is fragmented across at least four disjoint command
surfaces:

| Surface | Scope |
|---|---|
| `doctrine validate` | id-integrity (basename==toml id, no duplicates, alias equality) **+ relation-graph integrity** (danglers / `IllegalRows` / supersession drift) |
| `doctrine spec validate` | FK integrity, dangling members/interactions, orphan requirements |
| `doctrine memory validate` | dangling relations, stale verification, draft expiry |
| `doctrine inspect <ID>` | per-entity dangling outbound refs |

Each is real and good; the gap is that there is **no aggregating surface**.
A team or CI job that wants "is the corpus sound?" must know and run four+
separate commands, union their exit codes, and reconcile four output shapes.

**Source:** IMP-121 (backlog improvement, open). Proposals 0011, 0026, 0029
(loop/proposals-2026-06-20), 2026-06-20. Raw-label edge detection folded in
from IMP-141 (validate relation visibility).

## Scope & Objectives

One `doctrine doctor` command that runs every integrity check across the whole
corpus graph and returns one go/no-go plus a unified, actionable report.

**In scope:**

1. **Aggregate existing checks** — run id-integrity **and relation-graph
   integrity** (both halves of `validate`), spec-FK (`spec validate`), and
   memory-health (`memory validate`) checks, collecting findings under a unified
   output model. (Relation-graph integrity is the eighth check — design D8: the
   original scope under-counted it, but without it `doctor` would not subsume
   `validate`.)
2. **Done-but-open detector** (proposal 0026) — advisory flag for open backlog
   items whose linked slices are all terminal. Pure graph query over item→slice
   edges. Advisory only, never auto-close.
3. **Prose citation integrity** (proposal 0029) — extract every `KIND-NNN`
   citation from authored `.md` bodies, report unresolved ones. Reuses the
   existing prose scan primitive `scan_danglers` (`integrity.rs:581`). Advisory
   only;
   precision-exclude code spans, sentinels, and doc-local refs.
4. **Raw-label edge detection** (from IMP-141) — scan the relation graph for
   edges carrying `Raw()` labels (pre-migration forms authored before the
   PHASE-04 migration to the validated `RELATION_RULES` table). These resolve
   identically but signal incomplete migration. Advisory warning only.
5. **Corpus TOML parse integrity** (absorbing IMP-176) — validate every TOML
   file the catalog touches (entity metadata, facets, members, edges) plus
   authored non-entity TOML (e.g. `plan.toml` under each slice). Rides the
   catalog's existing `scan_entities` diagnostic channel and its per-facet
   malformed-isolation pattern — the doctor turns silent degradation into
   surfaced findings. Advisory only: a parse failure that the tolerant read
   skips past is a finding, but the doctor never mutates.
6. **Unified exit code** — non-zero on any `error`-severity finding (the hard
   integrity checks); `warning`-only (the four advisory checks) → exit 0
   (design D4). No `--check`/`--strict` flag in v1 (design D10, YAGNI).
7. **Unified report format** — all findings under one structured output
   (table + JSON via `listing::json_envelope`), grouped by check category.

**Out of scope:**

- Auto-fixing any finding.
- Adding new validation checks beyond the eight listed above (two `validate`
  halves + spec-FK + memory-health + three proposals + raw-label detection +
  TOML parse integrity).
- Replacing or removing the existing per-surface commands (`validate`,
  `spec validate`, `memory validate`). They remain as targeted tools. **Note:**
  `doctor` is designed as a strict superset of `validate` (D8), so a future
  `validate` removal becomes clean — deferred until `doctor`'s speed is proven,
  captured as a follow-on backlog item (D9). Not done in this slice.
- Done-but-open auto-closure — detection only.
- Transitive/graph health checks beyond item→slice done-but-open (e.g.
  cycle detection, reachability). Those are future work.
- `doctrine check` hook integration (linters, pre-commit). The doctor is
  a manual/CI gate, not a hook.

## Affected Surface

- `src/finding.rs` — **new leaf**: `Finding`/`Category`/`Severity`, the
  `from_lines` adapter, table render + JSON rows (ADR-001: a leaf so the check
  modules import *down* into it, never up into the orchestrator)
- `src/commands/doctor.rs` — **new command module**: `run_doctor` orchestrator
  (resolve root once, run 8 checks, render, exit) — the validate precedent
- `src/commands/cli.rs` — `Command::Doctor` variant + dispatch arm
- `src/commands/mod.rs` — `pub(crate) mod doctor;`
- `src/integrity.rs` — id-integrity native `Finding` (newtype already structured)
- `src/memory.rs` — extract memory-health into a reusable fn (currently
  CLI-tangled in `run_validate`)
- `src/registry.rs` / `src/spec.rs` — spec-FK source (`BuildFinding`); adapter in
  v1, native-upgradeable
- `src/relation_graph.rs` — `validate_relations` (relation-integrity, adapter)
  + raw-label edge scan
- `src/backlog.rs` — item→slice edge query for done-but-open detector
- `src/catalog/scan.rs` — consume the existing `CatalogDiagnostic` channel
  (read-only; `plan.toml` handled by a doctor-local probe, **not** a catalog
  extension — design D6)
- `tests/` — black-box golden for `doctor` output; superset-of-validate test;
  unit tests per new check

## Risks & Assumptions

- **Assumption:** The existing per-surface checks can be extracted into
  pure, reusable functions without destabilising their existing CLIs.
- **Risk:** The prose citation scan primitive may need refinement to
  exclude code spans and sentinels precisely — false positives on
  `KIND-NNN` in backtick-fenced blocks or already-resolved refs.
- **Risk:** The done-but-open detector requires a graph query that doesn't
  yet exist as a reusable primitive; may need new edge-traversal helpers.
- **Resolved (D6):** `plan.toml` parse integrity is handled by a **doctor-local
  probe** (walk `.doctrine/slice/*/plan.toml`, skip slug symlinks), **not** by
  extending `KindRef`/the catalog walk — keeps `catalog/scan.rs` behaviour
  preserved. Entity/facet TOML parse failures come free from the existing
  `CatalogDiagnostic` channel.
- **Assumption:** Severity is coarse per-category (hard checks = error, the four
  advisory checks = warning); no per-finding configurable threshold in v1.
- **Resolved (D10):** always run all checks; no `--check` flag in v1 (YAGNI).
- **Resolved (D4):** exit non-zero on any `error`-severity finding; warnings → 0.

## Verification / Closure Intent

- `doctrine doctor` in a clean corpus → exit 0, empty findings report.
- `doctrine doctor` in a corpus with known **error**-severity violations → exit
  non-zero, findings listed under the relevant check category; advisory
  (warning) findings are reported but do not flip the exit code.
- `doctrine doctor` id+relation findings == `doctrine validate` on the same
  corpus (the superset invariant, D8).
- `doctrine doctor --json` → structured JSON envelope with categorised
  findings.
- Existing `doctrine validate`, `doctrine spec validate`, `doctrine memory
  validate` continue to work unchanged.
- Black-box golden test pins the output shape for a seeded dirty corpus.
- `just gate` passes (clippy zero warnings, fmt).
