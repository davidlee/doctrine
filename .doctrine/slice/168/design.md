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
- **Raw labels (corrected, RV-183 F-7).** The carrier is
  `catalog::hydrate::CatalogEdgeLabel::Raw(String)` (`hydrate.rs:46`) — pre-`RELATION_RULES`
  / free-text forms in the **catalog graph** — **not** `RelationLabel::Raw` (no
  such variant exists, `relation.rs:45`). `relation_graph::outbound_for` /
  `tier1_edges` **drop** off-table rows (`relation_graph.rs:331`), and a Raw label
  on a *numbered* edge panics as catalog corruption (`relation_graph.rs:378`), so
  `outbound_for` is **not** the access path. IMP-141 reports ~173 catalog raw edges
  (156 `related`, 17 `descends_from`) that "resolve the same way" (valid, not
  illegal) — so check #6 mines `CatalogEdgeLabel::Raw` via the catalog graph and
  stays disjoint from #2's `IllegalRows` (R7).
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
  check #2 RelationIntegrity below), making a future `validate` removal a clean,
  non-lossy step.

## 5. Proposed Design

### 5.1 System Model

```
src/finding.rs            NEW leaf    Finding, Category, Severity, from_lines
                                      adapter, table render, JSON rows (envelope)
src/doctor_checks.rs      NEW command the three heavy NEW native checks:
                                      raw_label / toml_parse / prose_cite
                                      (reaches catalog::scan + integrity, so
                                      command-tier; classified in layering.toml)
src/commands/doctor.rs    NEW command run_doctor: resolve root once → run 8
                                      checks → collect Vec<Finding> → render
                                      (table | --json envelope) → exit
  imports down into:
    finding, integrity, registry, spec, memory, backlog, relation_graph,
    doctor_checks (and clock for today())
src/commands/cli.rs       EDIT        Command::Doctor variant + dispatch arm
src/commands/mod.rs       EDIT        `pub(crate) mod doctor;`
src/spec.rs               EDIT        spec_fk_findings composer (#3) lives here,
                                      NOT in the registry leaf (ADR-001; RV-185 F-1)
```

> **As-built note (RV-185 reconcile).** The three new native checks #6–#8
> (RawLabel/TomlParse/ProseCite) live in a dedicated `src/doctor_checks.rs`
> module rather than inline in `commands/doctor.rs`, keeping `run_doctor` a thin
> orchestrator. The #3 SpecFk composer (`spec_fk_findings`) lives in the
> command-tier `spec.rs`, not the `registry` leaf, so `registry` stays pure
> (out=0) per ADR-001.

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
| 1 | `IdIntegrity` | E | `integrity::id_integrity_findings` (newtype already) | adapter → **native after D12** |
| 2 | `RelationIntegrity` | E | `relation_graph::validate_relations` | adapter |
| 3 | `SpecFk` | E | `registry::Registry::validate` (`BuildFinding`) | adapter (native-upgradeable) |
| 4 | `MemoryHealth` | E | extract pure fn from `memory::run_validate` | adapter render → **native after D12** (pure-fn extraction forced regardless) |
| 5 | `Lifecycle` | W | `backlog` item→slice edges + `is_transition_terminal` | native (new) |
| 6 | `RawLabel` | W | catalog graph `CatalogEdgeLabel::Raw` edges (NOT `outbound_for`, RV-183 F-7) | native (new) |
| 7 | `TomlParse` | W | facet TOML + `plan.toml` probe (NOT entity-toml — that is #1, RV-183 F-10) | native (new) |
| 8 | `ProseCite` | W | invert `scan_danglers` + precision excludes | native (new) |

The v1-native declaration (Guiding Principle §4) is satisfied by the four **new**
checks (#5–#8), which are native by construction. The legacy-backed Error sources
#1/#4 begin **adapter'd** and re-point to native only once their byte-exact
golden precondition lands (D12, RV-183 F-8) — so the phase sketch authors those
goldens *before* any native re-point. Native-vs-adapter for #2/#3 is a
**plan/execution call** (lower-risk-first), not pre-committed here. The contract
supports both via `entity: Option`.

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
- **`TomlParse` scope excludes entity-toml — no overlap with #1 (RV-183 F-10).**
  Malformed *numbered-entity* TOML is already an Error finding in #1 IdIntegrity
  (`scan_kind` pushes `"{PREFIX}-{id}: TOML parse failed"`, `integrity.rs:299`).
  `scan_entities` emits a `CatalogDiagnostic{Error}` on the same read
  (`scan.rs:191`). To avoid double-reporting one broken entity as both
  `IdIntegrity/Error` and `TomlParse/Warning`, #7 consumes **only** the
  diagnostics #1 does not cover: **facet** TOML (`estimate`/`value`/`risk`,
  `scan.rs:323`) and the doctor-local `plan.toml` probe. Entity-level catalog
  parse diagnostics are dropped/deduped against #1.

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
  findings += catalog_raw_label_findings(root)              -> native (new): CatalogEdgeLabel::Raw
  findings += toml_parse_findings(root)                     -> facet diag ∪ plan probe (NOT entity-toml; #1 owns that, F-10)
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
  byte-exact string output via the re-pointed render — but the **safety net does
  not yet exist** (RV-183 F-3): `validate`'s tests assert *substrings*
  (`tests/e2e_integrity.rs` `.contains("corpus clean")` etc.), not byte-exact
  output, and `memory validate` has **no output golden at all** (its only test
  reference is an MCP tool-registry presence check). The native re-point is
  therefore **gated on authoring true byte-exact goldens first, red** (D12); absent
  that, the source stays adapter'd.
- **ProseCite — candidate grammar, resolver gating, empirically-derived excludes
  (F1 pass 1; F-1..F-5 RV-183, verified by `grep` over `.doctrine/**/*.md`, not
  reasoned).** ProseCite inverts `scan_danglers`: extract every cite-shaped token
  from authored `.md`, classify, flag the unresolved. Precision is the whole game.

  **Candidate grammar (load-bearing — RV-183 F-1).** `line_cites`
  (`integrity.rs:623`) matches a *known needle* with alphanumeric-only boundaries
  — `-` is **not** a boundary — so it whole-token-matches `DEC-005` *inside*
  `DEC-005-C`. ProseCite needs a **new** scanner (not a reuse of that primitive)
  that recognises the **maximal** hyphenated token `[A-Z]{2,}-[0-9]+(-[A-Za-z0-9]+)*`
  *before* deciding 2-part vs 3-part — else "skip 3-part" never fires (a naive
  `KIND-NNN` regex extracts the 2-part head and re-introduces the bug). Fenced
  code blocks span lines, so the scanner carries fence state across lines (the
  line-by-line `scan_danglers` loop cannot); inline code spans are line-local.

  **Resolver gating (two bail-avoidance gates).**
  1. **3-part `KIND-NNN-XX`** → **skip**. Two sub-shapes, both excluded:
     external decision-log cites (`DEC-005-C` 519×, `DEC-010-06` 462× —
     mem.pattern.entity.dec-prefix-dual-namespaced, SPEC-019 D8; un-parseable —
     `parse_canonical_ref` rsplit → non-numeric *or* unknown-prefix bail) **and**,
     the empirically-dominant case, `KIND-NNN-<word>` compound adjectives whose
     2-part head *is* a real ref (`SL-048-style` 184×, `IMP-006-gated` 184×,
     `ADR-006-references` 92×, `RSK-003-primary` 92×, `REQ-082-AC3` 46×).
     **Accepted false-negative (RV-183 F-4):** a *dangling* 3-part head
     (`SL-999-style`) is invisible to ProseCite. Advisory check; consciously taken.
  2. **2-part, unknown prefix** (`FOO-1`, `PHASE-03`, `SHA-256`, `UTF-8`,
     `ISO-8601`) → **skip** via `kind_by_prefix(prefix) == None` *before*
     `ensure_ref_resolves` (which would `bail!`). This gate carries the bulk of
     the exclusion load — every non-KINDS prefix in the corpus lands here.

  **Classification per candidate, after lexical excludes:**
  - maximal token is 3-part → **skip** (gate 1);
  - strict 2-part, `kind_by_prefix == None` → **skip** (gate 2);
  - strict 2-part, `Some` + entity dir missing → **`ProseCite` finding** (dangling
    — covers `DEC-NNN` and the rest of `KINDS`);
  - strict 2-part, `Some` + dir present → resolved, no finding.

  **Lexical excludes (before classification):** fenced + inline code spans,
  `*-SENTINEL` tokens (`BOOT-SENTINEL`), bare doc-local refs (`OQ-1`, `D1`, `R1` —
  doctrine's doc-local decisions use the bare `D1` form, never `DEC-`).

  **Illustrative-example false positives — the largest empirical FP class
  (RV-183 F-2, unmodeled by pass 1).** Committed, non-disposable prose carries
  placeholder/example 2-part ids that resolve to nothing: `POL-123` in the shipped
  reference doc `.doctrine/glossary.md`; `SL-999` in 9 files incl. 6 slice
  `design.md`; `STD-002/003` in `slice/033/audit.md`; `SPEC-110/200` across
  `slice/015` docs + a committed memory body; `REQ-999`, `CM-999`, `CHR-003`. As
  pass 1 specified it (scan `.doctrine/**/*.md`, skip only handover+state),
  ProseCite would flag ~20+ legitimate examples every run, drowning the signal.
  **Scope decision (D11):** ProseCite's scan scope **diverges from `reseat`'s** —
  it additionally skips the process-exhaust / historical tier (`audit.md`,
  `inquisition.md`, `notes.md`, `research/**`, **`.doctrine/review/**`** — RV-183
  F-9: review ledgers are adversarial exhaust dense with placeholder ids, e.g.
  this very review cites `POL-123`/`SL-999`) where hypothetical and reseated-away
  ids cluster. Residual example noise in durable bodies
  (slice/adr/spec/memory `.md`, e.g. the `glossary.md` `POL-123`) is an **accepted
  v1 limitation** (R8); a precise example-detection heuristic is deferred (YAGNI).
  `is_disposable_prose` stays as-is for `reseat`; ProseCite composes it with the
  extra skip-set (RV-183 F-5).

  **Stale-memory note:** mem.pattern.entity.free-text-ref-not-forward-validated
  (SL-042) still says "`DEC` is not a kind" — predates SPEC-019; only the 3-part
  form is free-text now.
- **done-but-open terminal def + non-vacuous guard (F2, adversarial pass).**
  `is_transition_terminal` (done **or** abandoned). An item is **"open"** iff its
  *own* status is non-terminal (`!is_transition_terminal(item.status)`) — a
  closed/abandoned backlog item is never flagged (RV-183 F-6). Flag an open item
  **iff it has ≥1 linked slice** (`targets_for(item, Slices)` non-empty) **and**
  every linked slice is terminal → one `Lifecycle` finding worded "all slices
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
  field. **Facet** TOML parse failures come free from the existing
  `CatalogDiagnostic` channel; **entity** TOML parse failures are excluded from
  #7 because #1 IdIntegrity already reports them (RV-183 F-10) — no double-report.
- **D7 — raw-label is its own `RawLabel` category** (warning), separate from
  `RelationIntegrity` danglers (error) — different severity and remediation.
- **D8 — add check #2 `RelationIntegrity`** so doctor subsumes everything
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
- **D11 — ProseCite scan scope diverges from `reseat`'s** (RV-183 F-2/F-5). Beyond
  `is_disposable_prose` (handover + `.doctrine/state/`), ProseCite additionally
  skips the process-exhaust / historical tier — `audit.md`, `inquisition.md`,
  `notes.md`, `research/**`, and `.doctrine/review/**` (RV-183 F-9) — where
  hypothetical and reseated-away ids cluster and generate false positives. `reseat` keeps its narrower skip-set (it *wants* to
  nag about real inbound cites); ProseCite composes the extra skip on top. Residual
  example noise in durable bodies is an accepted limitation (R8). Alternative
  (one shared skip predicate) rejected — the two callers have genuinely different
  precision needs.
- **D12 — native re-point is gated on byte-exact goldens authored first, red**
  (RV-183 F-3). The pass-1 design assumed "existing goldens are the proof" of
  byte-exact output for #1/#4; empirically they are not (`validate` substring-only,
  `memory validate` golden-less). So a native re-point of #1 or #4 is admissible
  **only after** a true byte-exact golden over that command's current output is
  authored and green; otherwise the source ships adapter'd. Makes the
  behaviour-preservation gate real instead of assumed.

## 8. Risks & Mitigations

- **R1 — native re-point drifts a shipping command's golden** (#1 id-integrity,
  #4 memory). *Mitigation:* re-point through a render that reproduces byte-exact
  output — but the existing goldens are **not** that proof (RV-183 F-3):
  `validate` is substring-asserted (`tests/e2e_integrity.rs`), `memory validate`
  has no output golden. So byte-exact goldens are **authored first, red** (D12) as
  a precondition of each native re-point; fall back to adapter if drift is
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
- **R8 — ProseCite illustrative-example noise** (RV-183 F-2). Placeholder/example
  ids in committed durable prose (`POL-123` in `glossary.md`, `SL-999` across
  slice designs) resolve to nothing and read as dangling. D11 narrows the scan
  (skip the `audit.md`/`inquisition.md`/`notes.md`/`research/**` process-exhaust
  tier) to kill the bulk; residual example noise in durable bodies is an
  **accepted v1 limitation** — advisory severity bounds it, and a precise
  example-detection heuristic is deferred (YAGNI). Re-evaluate if the noise floor
  proves intolerable in practice.

## 9. Quality Engineering & Validation

- **Black-box goldens** (built binary, `tests/`): seeded dirty corpus →
  pinned grouped-table output **and** `--json` envelope; clean corpus → exit 0 +
  `corpus clean`.
- **Superset invariant test:** doctor's id+relation findings == `validate`'s on
  the same corpus (pins D8/§5.5).
- **Per-new-check unit tests:** ProseCite (one per exclusion class), done-but-open
  (done-only / abandoned-only / mixed / live-slice negative), `plan.toml` probe
  (malformed + symlink-skip), raw-label scan.
- **Behaviour preservation:** `spec validate` (#3, adapter'd) runs its legacy fn
  untouched, so its existing tests stay green unchanged. For the **native**
  re-points there is no existing byte-exact golden to lean on (RV-183 F-3) —
  `validate` (#1) is substring-asserted, `memory validate` (#4) has none — so this
  slice **authors byte-exact goldens for #1 and #4 first, red** (D12), then
  re-points onto them. Until those land, #1/#4 stay adapter'd.
- **`just gate`** clean (clippy zero warnings, fmt).

### Phase sketch (pencil — authoritative plan is `/plan`)

Non-binding. Likely ~4–5 phases: (P01) `finding` leaf + adapter + render + exit;
(P02) make the 3 existing sources callable as adapters (extract the memory pure
fn) — **no native re-point yet**; (P03) the 4 new native checks (ProseCite may
split out — heaviest); (P04) `Command::Doctor` wiring + black-box goldens +
superset test; (P05) **author byte-exact goldens over current `validate` /
`memory validate` output (red), THEN re-point #1/#4 native** — D12 ordering
(RV-183 F-8): goldens precede the native re-point, never the reverse. Granularity
is `/plan`'s call.

## 10. Review Notes

### Adversarial pass 1 (self, 2026-06-27) — integrated

Verified three load-bearing assumptions against source; found two correctness
bugs and five precision/honesty gaps.

- **F1 (bug, fixed §5.5; corrected by user re DEC).** ProseCite must classify a
  cite token before resolving. `integrity::KINDS` **includes** record kinds, so
  `DEC-001` (2-part) is a real entity cite to validate — not skip. The genuine
  crash triggers are (a) **3-part `DEC-005-C`** decision-log cites (un-parseable →
  `parse_canonical_ref` bails) and (b) truly-unknown prefixes (`ensure_ref_resolves`
  bails). Fix: exclude 3-part tokens; gate 2-part on `kind_by_prefix`; dir-probe
  the rest. Added the 3-part exclusion class (dual-namespace,
  mem.pattern.entity.dec-prefix-dual-namespaced) that pass 1 missed.
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

### Adversarial pass 2 (internal hostile, RV-183, 2026-06-27) — integrated

Inquisition (`design` facet, raiser `inquisitor`) on the dense surfaces the
handover named. Findings verified **empirically** against the live corpus prose
(`grep` over `.doctrine/**/*.md`) and the test suite, not reasoned. Six charges,
all resolved on the ledger; design body corrected.

- **F-1 (major → fixed §5.5).** ProseCite candidate-token grammar was undefined,
  and the only existing tokenizer (`line_cites`, alphanumeric boundaries) would
  whole-token-match `DEC-005` *inside* `DEC-005-C`, silently defeating the 3-part
  exclusion. Specified a new scanner that recognises the **maximal** hyphenated
  token before 2-part/3-part classification, plus cross-line fence state.
- **F-2 (major → fixed §5.5/§8 R8/§7 D11).** The largest empirical false-positive
  class — illustrative/placeholder ids in committed prose (`POL-123` in the
  shipped `glossary.md`, `SL-999` in 9 files, `STD-002/003`, `SPEC-110/200`, …) —
  was unmodeled; ProseCite would flag ~20+ legitimate examples per run. Narrowed
  the scan scope (D11), recorded residual noise as an accepted limit (R8).
- **F-3 (major → fixed §5.5/§8 R1/§9/§7 D12).** R1's "existing goldens are the
  proof" of byte-exact native re-point is false: `validate` asserts substrings
  (`.contains`), `memory validate` has **no** output golden. Gated the #1/#4
  native re-point on authoring byte-exact goldens first, red (D12).
- **F-4 (minor → fixed §5.5).** The 3-part rationale ("external decision-log
  cites") was empirically wrong — the corpus is dominated by `KIND-NNN-<word>`
  compound adjectives over real refs. Corrected the rationale; owned the dangling
  3-part false-negative.
- **F-5 (minor → fixed §5.3-via-D11/§5.5).** `is_disposable_prose` (built for
  `reseat`) is too narrow for ProseCite's purpose; ProseCite now composes an extra
  skip-set (D11) rather than re-using reseat's scope verbatim.
- **F-6 (nit → fixed §5.5).** Defined the done-but-open "open item" predicate
  (`!is_transition_terminal(item.status)`) so a closed/abandoned item is never
  flagged. Underlying primitives (`is_transition_terminal`, `targets_for(Slices)`)
  confirmed present.

### Adversarial pass 3 (external, codex / GPT-5.5, RV-183 F-7..F-11, 2026-06-27) — integrated

External hostile review over the pass-2 design. Five findings, **all verified
against source before integration** (codex's strong claims checked, not trusted —
two of them re-specified load-bearing seams). All resolved on RV-183; body fixed.

- **F-7 (major → fixed §2/§5.2/§5.4).** Check #6's seam was fiction: `RelationLabel`
  has no `Raw` variant (`relation.rs:45`), and `outbound_for` **drops** off-table
  rows (`relation_graph.rs:331`) — a Raw label on a numbered edge panics. The real
  carrier is `CatalogEdgeLabel::Raw` via the catalog graph (`hydrate.rs:46`);
  IMP-141's 173 raw edges live there and "resolve the same way" (so R7 disjointness
  holds). Re-specified #6 to mine the catalog graph. **This answered the pass-1/2
  P05 open question — negatively for `outbound_for`.**
- **F-8 (major → fixed §5.2/§9 phase sketch).** D12 (added in pass 2) contradicted
  §5.2's "native" v1-path for #1/#4 and a phase sketch that wired native before the
  goldens. Made #1/#4 adapter-first, native only after the D12 golden precondition;
  reordered the sketch (P05 authors goldens, then re-points).
- **F-9 (major → fixed §5.5/§7 D11).** D11's process-exhaust skip forgot
  `.doctrine/review/**` — review ledgers carry placeholder ids (this very review
  cites `POL-123`/`SL-999`), re-importing the noise D11 set out to kill. Added
  `review/**` to the skip-set.
- **F-10 (major → fixed §5.2/§5.3/§7 D6).** #7 TomlParse double-reported malformed
  *entity* TOML already flagged by #1 IdIntegrity (Error), as a conflicting
  Warning. Scoped #7 to **facet** TOML + `plan.toml` only; entity-toml is #1's.
- **F-11 (minor → fixed §4/§7 D8).** D8 (the superset pin) cited "check #8
  RelationIntegrity"; RelationIntegrity is #2. Corrected.

### Still open for the User / `/plan`

- Whether D11's blunt scan-scope cut is the right instrument vs. a sharper
  example-detection heuristic (R8 residual noise) — a v1 tradeoff.
- The adapter/native boundary for #2 RelationIntegrity / #3 SpecFk — a
  lower-risk-first plan call (OQ-A), not pre-committed.
- Whether D12's golden-first gate is design canon or pure plan sequencing.
