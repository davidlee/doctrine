# Design SL-168: Unified corpus health doctor verb

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Corpus integrity checking is fragmented across four disjoint command surfaces
(`validate`, `spec validate`, `memory validate`, per-entity `inspect`), each
with its own scope, output shape, and exit code. There is no aggregating
"is the corpus sound?" gate. A team or CI job must know and run four+ commands,
union their exit codes, and reconcile four output formats.

Ship one `doctrine doctor` verb that runs every integrity check across the whole
corpus graph and returns a single go/no-go plus a unified, actionable report.

## 2. Current State

| Surface | Scope | Shape |
|---|---|---|
| `doctrine validate` | id-integrity **+ relation-graph integrity** (danglers / `IllegalRows` / supersession drift) | `Vec<String>`, any → non-zero |
| `doctrine spec validate` | FK integrity, dangling members/interactions, orphan requirements | `Vec<String>` from structured `BuildFinding` |
| `doctrine memory validate` | dangling relations, stale verification, draft expiry | inline `writeln!` to a buffer, `warning_count` |
| `doctrine inspect <ID>` | per-entity dangling outbound refs | per-entity |

Key code facts (verified, notes.md source-map):

- `integrity::id_integrity_findings(root) -> Vec<String>` (`integrity.rs:360`)
  composes `check_kind` (returns `Vec<Finding(String)>`, a newtype) plus
  parse-error diagnostics. **Already semi-structured.**
- `validate`'s command body (`commands/validate.rs::run_validate`) calls exactly
  `id_integrity_findings` **+** `relation_graph::validate_relations`. It is a
  thin 2-check composition over those fns — **not** a parallel implementation.
- `registry::Registry::validate` (`registry.rs:317`) returns `Vec<String>` built
  from `BuildFinding { spec: String, message: String }` (`registry.rs:63`) —
  **already carries the subject entity.**
- `memory::run_validate` (`memory.rs:3288`) is CLI-tangled: writes findings
  straight to a buffer, tracks `warning_count`. No reusable findings fn exists.
- `integrity::scan_danglers(root, needle)` (`integrity.rs:581`) finds every cite
  of **one** ref; `line_cites` (`:623`) is whole-token `KIND-NNN` detection;
  `is_disposable_prose` (`:609`) excludes handover/phase notes. Only `reseat`
  uses it today.
- `relation_graph` carries `RelationLabel::Raw(String)` — pre-`RELATION_RULES`
  migration forms; `outbound_for` surfaces edges. (IMP-141: ~173 raw edges.)
- `catalog/scan.rs::scan_entities` (`:177`) already emits `CatalogDiagnostic`
  (`Severity::Error`, `entity_key`, `file`, `field`, `message`) on malformed
  entity/facet TOML — a diagnostic channel the doctor can consume. It does **not**
  read sibling `plan.toml`.

## 3. Forces & Constraints

- **ADR-001 (module layering, leaf ← engine ← command, no cycles).** The shared
  `Finding` type must live in a leaf so `integrity`/`registry`/`memory` import
  *down* into it; it cannot live in the command-layer orchestrator (that would
  invert the dependency). The orchestrator is the command-layer composer — the
  validate precedent: "the one layer allowed to depend on both the id-scan and
  the relation walk" (`commands/validate.rs` doc-comment).
- **Behaviour-preservation gate (AGENTS.md).** `validate` / `spec validate` /
  `memory validate` are shipping commands; their existing goldens must stay green
  unchanged. An *adapter'd* source keeps its `Vec<String>` internals untouched
  (zero golden risk); only a source we actively *upgrade* to native `Finding`
  re-points its render and must reproduce byte-exact output.
- **STD-001 (no magic strings).** Category names, severity labels, summary-line
  templates → single-source named consts.
- **SPEC-013 (CLI surface).** `doctor` is a corpus-wide verb, **not** a
  `<kind> <verb>` entity — a top-level `Command::Doctor`, like `Command::Validate`.
  It does not ride the `list` spine (it is a findings report, not an entity
  roster) but reuses `listing::json_envelope` for the `--json` shape.
- **No parallel implementation (CLAUDE.md).** Each check is computed by exactly
  one fn; both the legacy per-surface command and the doctor consume that fn.
- **Pure/imperative split.** `finding` leaf is pure (no clock/rng/git/disk/clap);
  impurity (root resolution, dir walks, stdout) lives in the command shell.

## 4. Guiding Principles

- **Contract first, adapt sources opportunistically.** The rich `Finding` is the
  target shape. Sources reach it natively where cheap or new; an adapter bridges
  the rest from `Vec<String>` with no behaviour risk. v1 ships ≥1 native source
  (all four new checks are native by construction) as a declaration of intent;
  the all-native end-state is a follow-on, not a v1 blocker.
- **Advisory means advisory.** The four new checks never break the build.
- **Doctor is the superset.** It subsumes everything `validate` does (hence
  check #8 below), making a future `validate` removal a clean, non-lossy step.

## 5. Proposed Design

### 5.1 System Model

```
src/finding.rs            NEW leaf    Finding, Category, Severity, from_lines
                                      adapter, table render, JSON rows
src/commands/doctor.rs    NEW command run_doctor: resolve root once → run 8
                                      checks → collect Vec<Finding> → render
                                      (table | --json) → exit
  imports down into:
    finding, integrity, registry, memory, backlog, relation_graph, catalog::scan
src/commands/cli.rs       EDIT        Command::Doctor variant + dispatch arm
src/commands/mod.rs       EDIT        `pub(crate) mod doctor;`
```

`finding` is a leaf (imports neither clap nor `entity`); `integrity`,
`registry`, `memory` return `finding::Finding` without a cycle. `doctor` is the
command-layer composer that depends on all check modules — the same role
`commands/validate.rs` plays for its two checks today.

### 5.2 Interfaces & Contracts

```rust
// src/finding.rs  — pure leaf
pub(crate) enum Severity { Error, Warning }

pub(crate) enum Category {
    IdIntegrity,        // Error  — basename==toml id, no dup, alias equality
    RelationIntegrity,  // Error  — danglers / IllegalRows / supersession drift
    SpecFk,             // Error  — FK integrity, orphan requirements
    MemoryHealth,       // Error  — dangling relations, stale verify, draft expiry
    Lifecycle,          // Warning — done-but-open (all slices terminal)
    RawLabel,           // Warning — pre-migration Raw() relation labels
    TomlParse,          // Warning — malformed catalog TOML + plan.toml probe
    ProseCite,          // Warning — unresolved KIND-NNN cites in authored .md
}

pub(crate) struct Finding {
    pub category: Category,
    pub entity: Option<String>,   // canonical id when known; None for adapter'd /
                                  // corpus-level findings
    pub message: String,
}

impl Category {
    /// Severity is a pure function of category — single source, no per-finding
    /// field to drift (F5, adversarial pass). Render/exit derive from this.
    pub fn severity(self) -> Severity;
}

impl Finding {
    /// The a-style adapter bridge: wrap a legacy `Vec<String>` source with no
    /// per-finding entity. One call per adapter'd source.
    pub fn from_lines(category: Category, lines: Vec<String>) -> Vec<Finding>;
}
```

Each check is exposed as a callable returning findings — the doctor never reads
stdout. Per-source v1 path:

| # | Category | Sev | Source seam | v1 path |
|---|---|---|---|---|
| 1 | `IdIntegrity` | E | `integrity::id_integrity_findings` (newtype already) | **native** |
| 2 | `RelationIntegrity` | E | `relation_graph::validate_relations` | adapter |
| 3 | `SpecFk` | E | `registry::Registry::validate` (`BuildFinding`) | adapter (native-upgradeable) |
| 4 | `MemoryHealth` | E | extract pure fn from `memory::run_validate` | **native** (extraction forced regardless) |
| 5 | `Lifecycle` | W | `backlog` item→slice edges + `is_transition_terminal` | native (new) |
| 6 | `RawLabel` | W | `relation_graph` `Raw` labels via `outbound_for` | native (new) |
| 7 | `TomlParse` | W | catalog `CatalogDiagnostic` ∪ doctor-local `plan.toml` probe | native (new) |
| 8 | `ProseCite` | W | invert `scan_danglers` + precision excludes | native (new) |

Native-vs-adapter for #2/#3 is a **plan/execution call** (lower-risk-first),
not pre-committed here. The contract supports both via `entity: Option`.

### 5.3 Data, State & Ownership

- **Read-only.** The doctor mutates nothing — the `reseat`/`validate` precedent.
  No auto-fix, no auto-close (slice scope, out-of-scope).
- **Root resolved once** in the command shell, threaded into every check (no
  per-check re-resolution).
- **`plan.toml` probe** walks `.doctrine/slice/*/plan.toml`, **skipping slug
  symlinks** (`if file_type.is_symlink() { continue }`) to avoid double-counting
  the `NNN-slug → NNN` alias — see mem.pattern.entity.corpus-walk-skip-slug-symlink.
  A parse failure is a `TomlParse` warning; the probe never mutates.
- **Catalog untouched.** The `plan.toml` gap is filled by the doctor's own probe,
  **not** by extending `KindRef`/the catalog walk — keeps the catalog's
  behaviour-preservation intact (no new file probe in `scan_entities`).

### 5.4 Lifecycle, Operations & Dynamics

```
run_doctor(path):
  root = root::find(path)
  findings: Vec<Finding> = []
  findings += integrity::id_integrity_findings(root)        -> native
  findings += Finding::from_lines(RelationIntegrity,
                relation_graph::validate_relations(root))   -> adapter
  findings += Finding::from_lines(SpecFk,
                registry::build+validate(root))              -> adapter
  findings += memory::memory_health_findings(root)          -> native (new fn)
  findings += backlog::done_but_open_findings(root)         -> native (new)
  findings += relation_graph::raw_label_findings(root)      -> native (new)
  findings += toml_parse_findings(root)                     -> catalog diag ∪ plan probe
  findings += prose_cite_findings(root)                     -> native (new)
  render(findings)                 // grouped table | --json envelope
  if findings.any(Error): bail!    // non-zero
  else: Ok(())                     // warnings-only → exit 0
```

- **Default output:** grouped table — a header per non-empty category, findings
  beneath, then a summary line. Clean corpus → `doctor: corpus clean`.
- **`--json`:** `listing::json_envelope<T: Serialize>(kind, rows)` (verified
  row-generic) with rows `{category, severity, entity, message}` (severity
  rendered from `category.severity()`). **Honesty caveat (F6):** `entity` is
  populated only for native sources (#1, #4–#8); adapter'd sources (#2
  RelationIntegrity, #3 SpecFk — both Error) carry the id inside `message` with
  `entity: null`. So machine-filtering by `entity` is partial in v1 until those
  sources go native.
- **Exit code:** any `Error`-severity finding → `anyhow::bail!` (non-zero, like
  `run_validate`); warnings-only → `Ok(())`.

### 5.5 Invariants, Assumptions & Edge Cases

- **Doctor ⊇ validate.** Checks #1+#2 are exactly what `validate` runs, so doctor
  is a strict superset. Invariant pinned by a test asserting the **set of rendered
  message strings** from doctor's `IdIntegrity`+`RelationIntegrity` findings
  equals `validate`'s finding lines on the same corpus (F4: compare rendered
  messages, not `Finding` structs — adapter'd vs native carry different
  `entity`).
- **Behaviour preservation.** Adapter'd sources (#2, #3) run their legacy fn
  unchanged. Native sources (#1, #4) must reproduce the legacy command's
  byte-exact string output via the re-pointed render — guarded by the existing
  goldens.
- **ProseCite precision + resolver gating (F1, adversarial pass).** Exclude:
  backtick-fenced code spans (inline `` `KIND-NNN` `` and fenced blocks),
  `*-SENTINEL` tokens (e.g. `BOOT-SENTINEL`), and doc-local bare refs (`OQ-1`,
  `D1`, `R1`). **Do NOT call `ensure_ref_resolves` blindly** — it `bail!`s on a
  prefix outside `KINDS` (e.g. `DEC-005`, a free-text decision ref), which would
  abort the whole doctor run (mem.pattern.entity.free-text-ref-not-forward-validated).
  Instead: for each candidate `KIND-NNN`, gate on `kind_by_prefix(prefix)` —
  - `None` (prefix ∉ `KINDS`, e.g. `DEC`) → **skip**, it is not a corpus ref;
  - `Some` + entity dir missing → **`ProseCite` finding** (dangling cite);
  - `Some` + dir present → resolved, no finding.
  This means ProseCite reuses the dir-probe but never the bailing wrapper. Reuse
  `is_disposable_prose` to skip runtime prose.
- **done-but-open terminal def + non-vacuous guard (F2, adversarial pass).**
  `is_transition_terminal` (done **or** abandoned). Flag an open item **iff it
  has ≥1 linked slice** (`targets_for(item, Slices)` non-empty) **and** every
  linked slice is terminal → one `Lifecycle` finding worded "all slices
  terminal". The `≥1` guard is load-bearing: `targets_for` returns `[]` for a
  slice-less item, so "all (zero) slices terminal" is vacuously true and would
  falsely flag every normal slice-less backlog item. An all-abandoned item reads
  as "stuck — re-slice or close", not "complete". Advisory; never auto-closes.
- **Empty corpus / no findings** → exit 0, clean summary, empty `--json` rows.

## 6. Open Questions & Unknowns

All design-level OQs from notes.md are resolved (→ §7). Remaining are
plan/execution-time:

- OQ-A — native-vs-adapter for SpecFk (#3) and MemoryHealth render: decided at
  the phase that touches each source, lower-risk-first. (MemoryHealth extraction
  is forced regardless; only its *return shape* is the open call.)
- OQ-B — exact grouped-table layout (column widths, category header text) — pin
  at golden-authoring time.

## 7. Decisions, Rationale & Alternatives

- **D1 — `doctor` is a top-level `Command`, not a `<kind> <verb>` entity.**
  It is corpus-wide, like `validate`. (SPEC-013: the grammar is for entity
  kinds; corpus verbs sit beside them.)
- **D2 — `Finding` lives in a new `src/finding.rs` leaf.** ADR-001 forbids the
  check leaves depending up into the command-layer orchestrator. Alternative
  (Finding in `doctor.rs`) rejected — creates a cycle.
- **D3 — rich `Finding` contract + universal `from_lines` adapter.** Chosen over
  pure-adapter (a-style, loses entity granularity) and full-refactor (b-style,
  invasive on three shipping commands). The adapter is the no-risk fallback;
  native is the upgrade path. ≥1 native source ships (all four new checks).
- **D4 — exit on Error only; warnings → exit 0.** Advisory checks must never
  break CI. `--strict`/`--fail-on` deferred to backlog (YAGNI). Alternative
  (any-finding → non-zero) rejected — punishes the advisory design.
- **D5 — done-but-open uses `is_transition_terminal`** (done+abandoned), worded
  "all slices terminal". Captures the stuck-behind-abandoned case that
  `is_terminal_status` (done-only) would miss.
- **D6 — `plan.toml` validated by a doctor-local probe, not a catalog
  extension.** Keeps `catalog/scan.rs` behaviour-preserved; no new `KindRef`
  field. Entity/facet TOML parse failures come free from the existing
  `CatalogDiagnostic` channel.
- **D7 — raw-label is its own `RawLabel` category** (warning), separate from
  `RelationIntegrity` danglers (error) — different severity and remediation.
- **D8 — add check #8 `RelationIntegrity`** so doctor subsumes everything
  `validate` does. Without it, doctor would silently drop the relation-danglers
  check and could not cleanly replace `validate`.
- **D9 — keep `validate` in v1; do not strip or quiet-alias.** It is already a
  thin non-duplicating composition over the shared fns, so removal buys ~no code
  and breaks CI/scripts; the slice scope lists removal as out-of-scope. Doctor
  becomes the documented primary; `validate` repositioned as "doctor's id+relation
  subset". **Removal deferred until doctor's speed is proven** — captured as a
  follow-on backlog item.
- **D10 — no `--check` subset flag in v1.** The unified report is the feature;
  checks are cheap. `--check`/`--verbose`/`--quiet` deferred (YAGNI).

## 8. Risks & Mitigations

- **R1 — native re-point drifts a shipping command's golden** (#1 id-integrity,
  #4 memory). *Mitigation:* re-point through a render that reproduces byte-exact
  output; existing goldens are the proof; fall back to adapter if drift is
  non-trivial.
- **R2 — ProseCite false positives** on code spans / sentinels / doc-local refs.
  *Mitigation:* explicit exclusion set (§5.5) + dedicated unit tests per
  exclusion class; advisory severity bounds the blast radius.
- **R3 — done-but-open requires an item→slice traversal not yet a primitive.**
  *Mitigation:* ride `backlog` `relation_edges()` / `targets_for(Slices)`
  (notes.md `:959`); read slice status via catalog `status_and_title_for`.
- **R4 — corpus-walk double-count** via slug symlinks in the `plan.toml` probe.
  *Mitigation:* skip symlinked dir entries (lstat semantics);
  mem.pattern.entity.corpus-walk-skip-slug-symlink.
- **R5 — KINDS membership is advisory** (mem.pattern.entity.numbered-kind-identity-table):
  a future kind missing from `KINDS` silently escapes id-integrity. Out of scope
  here (doctor inherits the existing guard), but noted so it is not assumed
  closed.
- **R6 — no shared corpus snapshot; each check re-walks** (adversarial pass).
  "Root resolved once" saves root-finding, not the 8 independent corpus walks
  (`build_registry`, catalog scan, `collect_all`, the dir probes). Acceptable v1
  (corpus is small), but this is exactly why D9 **defers** the `validate` removal
  until doctor's speed is measured. A shared-snapshot refactor is future work, not
  v1.
- **R7 — RawLabel vs RelationIntegrity double-report** (adversarial pass). A
  `Raw()`-labelled edge must not be reported by *both* `validate_relations`
  (as `IllegalRows`, Error) and the raw-label scan (Warning). IMP-141 holds that
  raw labels resolve identically (valid, not illegal), so the sets are disjoint —
  pin it with a test asserting no edge appears in both categories.

## 9. Quality Engineering & Validation

- **Black-box goldens** (built binary, `tests/`): seeded dirty corpus →
  pinned grouped-table output **and** `--json` envelope; clean corpus → exit 0 +
  `corpus clean`.
- **Superset invariant test:** doctor's id+relation findings == `validate`'s on
  the same corpus (pins D8/§5.5).
- **Per-new-check unit tests:** ProseCite (one per exclusion class), done-but-open
  (done-only / abandoned-only / mixed / live-slice negative), `plan.toml` probe
  (malformed + symlink-skip), raw-label scan.
- **Behaviour preservation:** existing `validate` / `spec validate` /
  `memory validate` goldens stay green unchanged.
- **`just gate`** clean (clippy zero warnings, fmt).

### Phase sketch (pencil — authoritative plan is `/plan`)

Non-binding. Likely ~4–5 phases: (P01) `finding` leaf + adapter + render + exit;
(P02) make the 3 existing sources callable (extract memory) + wire id-integrity
native; (P03) the 4 new native checks (ProseCite may split out — heaviest);
(P04) `Command::Doctor` wiring + black-box goldens + superset test. Granularity
is `/plan`'s call.

## 10. Review Notes

### Adversarial pass 1 (self, 2026-06-27) — integrated

Verified three load-bearing assumptions against source; found two correctness
bugs and five precision/honesty gaps.

- **F1 (bug, fixed §5.5).** ProseCite must **not** call `ensure_ref_resolves`
  blindly — it `bail!`s on a non-`KINDS` prefix (`DEC-005`), aborting the run.
  Gate on `kind_by_prefix` first; skip non-corpus prefixes, dir-probe the rest.
- **F2 (bug, fixed §5.5).** done-but-open is vacuously true for slice-less items
  (`targets_for → []`). Added the `≥1 linked slice` guard.
- **F3 (verified OK).** Item→slice edge is the outbound `RelationLabel::Slices`
  label; `targets_for(item.tier1, Slices)` is correct (`backlog.rs:1392`).
- **F4 (tightened §5.5).** Superset invariant compares **rendered message
  strings**, not `Finding` structs (adapter'd vs native differ on `entity`).
- **F5 (simplified §5.2).** Dropped the redundant `severity` struct field;
  `Category::severity()` is the single source.
- **F6 (honesty, §5.4).** JSON `entity` filtering is partial in v1 — adapter'd
  Error sources (#2, #3) carry the id in `message`, `entity: null`.
- **F7 (verified OK).** `json_envelope<T: Serialize>` is row-generic
  (`listing.rs:871`) — doctor reuses it; no bespoke envelope needed.
- **R6/R7 (risks added §8).** Per-check corpus re-walk (perf, → D9 defer);
  RawLabel/RelationIntegrity disjointness (pin with a test).

### Open for external pass

Attack surface for a second (external) reviewer: ProseCite exclusion
completeness (are there cite shapes beyond code-span/sentinel/doc-local?); the
adapter/native boundary for #2/#3; whether `outbound_for` exposes `Raw` labels
pre-resolution (P05 must verify the access path before relying on it).
