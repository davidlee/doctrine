# Design SL-045: Requirement status visibility: spec req roster + standalone drift read

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

> **STATUS: DRAFT (2026-06-12).** All three scope OQs resolved (OQ-1/2/3, §6→§7);
> §5 drafted. Awaiting internal adversarial pass + external challenge before lock.
> Descends **SPEC-002** / **PRD-013**; completes the user-facing half of **REQ-110**
> / **REQ-111**. Reference forms: padded entity ids (`SL-045`, `REQ-110`, `ADR-001`);
> doc-local refs bare (`D1`, `OQ-1`).

## 1. Design Problem

SPEC-002 (Requirement Reconciliation Engine) shipped its compute (`coverage::composite`
/ `coverage::drift`, SL-042) and its write paths (reconcile writer + closure gate,
SL-044). Nothing **surfaces** the derived reads to a user: `drift()`'s only callers
are write paths, `spec req` exposes no `list`, and `spec show` buries authored status
in per-requirement prose. Two questions go unanswered — "which of this spec's
requirements exist, and at what authored status?" and "where does authored status
diverge from observed coverage?"

This slice adds **two read-only surfaces** over the existing seams — no new engine, no
new store. It discharges the *report* half of **REQ-111** ("surface drift as a derived
read … reported as drift"; the fold half shipped in SL-042) and the user-facing half
of **REQ-110**. Pure surfacing.

## 2. Current State — the substrate this slice consumes (all shipped)

Code shapes verified against the tree:

- **`composite(entries: &[(CoverageEntry, IsStale)]) -> Composite`**
  (`src/coverage.rs:205`) — pure deterministic fold. `Composite` exposes four public
  predicates: `is_empty`, `any_fresh_verified`, `any_failed_or_blocked`, `only_forward`
  (`coverage.rs:215..238`); its `cells` field is private.
- **`drift(authored: ReqStatus, composite: &Composite) -> Verdict`**
  (`coverage.rs:272`). `Verdict ∈ {Coherent, Divergent(DivergentReason), Indeterminate}`;
  `DivergentReason ∈ {ObservedContradiction, EvidenceOutrunsAuthored}`. Read-only —
  returns **no** `ReqStatus`.
- **`scan_coverage(root, req) -> Vec<(CoverageEntry, IsStale)>`** (`coverage_scan.rs:37`)
  — the impure shell (the **only** git/disk seam): walks **every** `.doctrine/slice/*/`
  `coverage.toml`, filters by one `req`, resolves `HEAD` **once**, staleness per matched
  entry via `git::commits_touching`. Skips the **ISS-006 slug-alias symlink** to avoid
  the double-count. Per-requirement ⇒ one full corpus walk per call.
- **`spec` composition** (`src/spec.rs`): `read_members(members_path) -> Vec<Member>`
  (`spec.rs:458`); `Member { requirement: String /*REQ FK*/, label: String /*FR-/NF-*/,
  order }`; `resolve_spec_ref` parses `PRD-NNN`/`SPEC-NNN` → subtype+id. The listing
  column model is `SPEC_COLUMNS` const + `SpecListRow` + `select_columns`/`render_columns`
  (`spec.rs:1072..1170`), the SL-037 pattern (`mem.pattern.listing.column-model-extension`).
- **`requirement`** (`src/requirement.rs`): `canonicalize_fk` / `id_from_fk` /
  `load(root, "REQ-NNN") -> Requirement` (`requirement.rs:238/251/272`); `Requirement`
  carries `kind: ReqKind`, `status: ReqStatus`.
- **No `coverage` CLI surface exists.** `coverage`/`coverage_scan` are lib-internal —
  sole callers are `reconcile::run` and the `slice status` closure gate.

## 3. Forces & Constraints

- **F1 — two-tier wall (NF-001/REQ-114).** Authored status and observed coverage must
  never touch through a function. A read that co-displays both must not *imply*
  `authored = f(coverage)`. The SL-044 import-edge proof is load-bearing and must stay
  green unchanged.
- **F2 — pure/imperative split (slices-spec, CLAUDE.md).** No git/disk/clock/rng in the
  pure layer. The corpus walk + staleness live in the shell (`coverage_scan`); the view
  is pure over `(ReqStatus, Composite, Verdict)`.
- **F3 — no parallel implementation (CLAUDE.md).** Ride existing seams; one compute
  path. The drift verdict is computed in exactly one place (`drift`); the read reuses it.
- **F4 — uniform list/show contract (SL-025) + column model (SL-037).** New list-like
  surfaces inherit `--columns` / `--json` and the pre-materialised-row column model.
- **F5 — RSK-006 scan cost.** A spec-wide read fans `scan_coverage` across N members; N
  full corpus walks unless batched.
- **F6 — behaviour-preservation gate.** Existing `coverage` / `coverage_scan` /
  `reconcile` suites stay green unchanged.

## 4. Guiding Principles

- **Surface, don't compute.** The engine already decides; the slice only renders.
- **Tier split by command, not by flag.** The cheap authored survey and the derived
  coverage read are *different commands* — the wall is hardest to blur when the tiers
  don't share a code path or an output table by default.
- **One walk per invocation.** A spec fan is one corpus walk, not one-per-member.
- **Read co-displays both tiers; it never derives one from the other.**

## 5. Proposed Design

### 5.1 System Model

Two read surfaces, split by tier:

| Command | Tier | Cost | Answers |
|---|---|---|---|
| `doctrine spec req list <SPEC>` | authored | no corpus walk | "which reqs, at what authored status?" |
| `doctrine coverage <ref>` | derived | one batched corpus walk | "where does authored diverge from observed?" |

**`doctrine coverage <ref>`** — `<ref>` polymorphic, prefix-dispatched:
- `REQ-NNN` → single row (one requirement's coverage/drift).
- `PRD-NNN` / `SPEC-NNN` → fan over the spec's members (one row per member, in member
  `order`).

`coverage` is the **sole** derived coverage/drift read (OQ-1/OQ-3, §7-D1/D3): `spec req
list` stays authored-only. The `coverage` view needs the authored status *anyway* (to
compute drift), so it can render authored + observed + verdict in one table — it **is**
the joined view, with no `--coverage` flag on the roster.

**Module homes** (ADR-001 layering, no cycles):
- `src/coverage_view.rs` *(new leaf)* — the read's pure compute + render: ref dispatch,
  `CoverageRow` materialisation, the observed-state classifier, the column model, JSON
  rows. Serves both a REQ ref (no spec) and a spec fan, so it is **not** spec-owned —
  a leaf module is the correct cohesion (a requirement-scoped read must not import
  `spec`'s membership concern except through the thin fan resolver).
- `src/coverage_scan.rs` — **+1 additive fn** `scan_coverage_batch` (§5.2). `scan_coverage`
  unchanged in behaviour.
- `src/coverage.rs` — **+** terse `label()` on `Verdict` / `DivergentReason` (display
  helper; the pure fold is untouched).
- `src/spec.rs` — `spec req list` (mirrors `spec list`'s column-model shape).
- `src/main.rs` — wire the top-level `Coverage { reference, columns, format/json, path }`
  leaf + `SpecReqCommand::List`.

### 5.2 Interfaces & Contracts

**Batched scanner** (the RSK-006 fix — additive, `scan_coverage` untouched):
```rust
// coverage_scan.rs — ONE corpus walk: parse each coverage.toml ONCE, keep entries
// whose key.requirement ∈ wanted, resolve HEAD once, stale each matched cell.
// Dense result: every wanted req present (empty Vec if uncovered). Same ISS-006
// slug-symlink skip, same single-HEAD-resolve, same skip-malformed tolerance.
pub(crate) fn scan_coverage_batch(
    root: &Path,
    wanted: &BTreeSet<String>,
) -> BTreeMap<String, Vec<(CoverageEntry, IsStale)>>;
```
The single-REQ path goes through `scan_coverage_batch(root, &{req})` too — one shared
walker. `scan_coverage` either delegates to it (iff byte-identical for the
single-req case, its existing tests being the equivalence proof) or stays as-is; the
walk internals (`collect_matching_entries`, the symlink skip, the HEAD-once staleness
map) are **lifted, not reinvented** (F3).

**View compute** (pure over resolved inputs):
```rust
struct CoverageRow {
    id: String,             // REQ-NNN (authored)
    label: Option<String>,  // FR-/NF- — Some in a spec fan, None for a bare REQ ref
    kind: ReqKind,          // authored
    status: ReqStatus,      // authored — loaded independently of coverage (F1)
    observed: ObservedState,// derived classifier (below)
    verdict: Verdict,       // drift(status, &composite) — derived
}

// Total fold over Composite's four public predicates — no new accessor.
fn observed_state(c: &Composite) -> ObservedState {
    if c.is_empty()              { ObservedState::None }
    else if c.any_failed_or_blocked() { ObservedState::Contradicted }
    else if c.any_fresh_verified()    { ObservedState::Verified }
    else if c.only_forward()          { ObservedState::Forward }
    else                              { ObservedState::Stale }
}
```
`observed` is a lossy *hint*; `verdict` is the authoritative reading. The classifier's
five outputs map 1:1 onto the §5.2 drift-tree branch points (SL-042) — it adds no new
decision logic, it labels the same states.

**Verdict display** — terse `Verdict::label()` / `DivergentReason::label()` added in
`coverage.rs`, the **single source** for verdict cell text (`Coherent`,
`Indeterminate`, `Divergent: evidence-outruns-authored`). reconcile.rs's prose
`build_prompt` is **deliberately not merged** (D5): its register is a sentence for an
operator prompt, the cell wants a token; merging would distort one to fit the other and
would touch reconcile (behaviour-preservation gate). The shared invariant — the
`Verdict`/`DivergentReason` *variants* — already lives in one place (the enum).

**CLI / SL-025 contract:** both commands honour `--columns` and `--json`.
- `coverage` table default columns: `id, status, observed, verdict` (+ `label` auto in
  a spec fan). `kind`, `label` available via `--columns`. Unknown column → the SL-037
  declaration-order error.
- `coverage --json` → `{kind:"coverage", rows:[{requirement, label?, kind,
  status, observed, verdict, divergent_reason?}]}` — field names carry the tier
  (`status` authored vs `observed`/`verdict` derived).
- `spec req list` table default: `id, label, kind, status` (authored only — **no**
  observed/verdict column exists on this surface).

### 5.3 Data, State & Ownership

- **No writes.** Both commands are pure reads. No new store, no authored-file mutation,
  no coverage-recording surface (that is SL-042's, untouched).
- **Authored tier** owned by `requirement` (status/kind via `load`) and `spec`
  (membership label/order via `read_members`). **Derived tier** owned by the
  `coverage`/`coverage_scan` engine. `coverage_view` *joins* them for display and owns
  nothing persistent — it is regenerable on every invocation.
- The join is **display-only**: `status` is read from the requirement file; `observed`/
  `verdict` from the fold. No value flows from the derived tier into the authored one.

### 5.4 Lifecycle, Operations & Dynamics

`doctrine coverage <ref>` flow:
1. **Dispatch** `<ref>` by prefix → `Target::Req(REQ-NNN)` | `Target::Spec(SpecRef)`;
   unknown prefix → error (`expected REQ-/PRD-/SPEC-NNN`).
2. **Resolve req set** — single: `{canonicalize_fk(ref)}`; spec: `resolve_spec_ref` →
   `read_members` → ordered `[REQ-NNN]` (member `order`).
3. **Shell scan (once)** — `scan_coverage_batch(root, &wanted)` (the sole git/disk seam).
4. **Per req (pure)** — `requirement::load` → authored `status`/`kind`; `composite(cells)`;
   `drift(status, &composite)`; `observed_state(&composite)` → a `CoverageRow`.
5. **Render** — `Table` via the column model, or `Json` typed rows. Empty spec (no
   members) → `""` (the §5.5 empty contract). Single REQ that doesn't exist →
   `load` error.

`doctrine spec req list <SPEC>` flow: `resolve_spec_ref` → `read_members` → per member
`requirement::load` for kind/status → authored rows → column-model render. **No scan.**

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1 (wall, F1/NF-001).** No import/data edge from the coverage fold into the
  authored-status load. `status` is *always* sourced from `requirement::load`, never
  derived. Pinned by a test asserting `coverage_view` row `status` equals the requirement
  file's status regardless of coverage state.
- **INV-2 (one walk).** A spec fan triggers exactly **one** corpus walk
  (`scan_coverage_batch`), independent of member count. Pinned by the equivalence test +
  (where cheap) a walk-count assertion.
- **INV-3 (authored-only roster).** `spec req list` output carries no observed/verdict
  field and performs no scan. Pinned by an output-surface assertion.
- **ASM-1.** `composite`/`drift` are pure and reusable as-is — verified: signatures take
  in-memory cells, no shell. No engine change.
- **Edge cases.** Bare REQ ref → `label = None` (renders `—`); no spec context exists.
  Spec with zero members → empty table (`""`). Uncovered requirement → `observed = none`,
  `composite` empty, `drift` per the §5.2 tree (`Indeterminate` for most authored states).
  Unborn/non-repo HEAD → every cell `IsStale::Unknown` (inherited from the shell).
  Malformed `coverage.toml` → skipped, never fatal (inherited).

## 6. Open Questions & Unknowns

All three scope OQs are **resolved** (§7). No open blockers.

- **OQ-1 (placement).** RESOLVED → D1: top-level `doctrine coverage <ref>`.
- **OQ-2 (scope unit).** RESOLVED → D2: polymorphic ref, `REQ` single | spec fan.
- **OQ-3 (roster columns).** RESOLVED → D3: `spec req list` authored-only; the derived
  join lives solely in `coverage`.
- **Q1 (residual, non-blocking).** Should `scan_coverage` be *folded* into
  `scan_coverage_batch` (delegation) or left standalone? Decide at implementation by the
  behaviour-preservation gate — delegate **only** if byte-identical for the single-req
  case; else keep both over a shared private walker. Not a design blocker.

## 7. Decisions, Rationale & Alternatives

- **D1 (placement — OQ-1).** `doctrine coverage <ref>` as a top-level leaf, **not**
  `spec req drift` or a `coverage drift` subgroup. Rationale: the ref is polymorphic
  (REQ *or* spec) so a `spec req`-nested home reads wrong for a spec ref; "coverage" names
  the substrate honestly and avoids blurring with SL-009's slice-rollup `⚠` (a *different*
  drift). Noun-as-verb leaf, symmetric with `reconcile`. *Alt rejected:* `doctrine drift`
  — name collision with the rollup; `coverage drift` subgroup — a whole new noun group for
  one read, premature.
- **D2 (scope unit — OQ-2).** Polymorphic `<ref>`: `REQ-NNN` single row, spec ref fans
  over members. Rationale: answers both the point question and the survey question through
  one verb; the fan is the real user need ("where does this spec diverge?"). *Alt
  rejected:* per-req only (forces a manual loop for the survey).
- **D3 (roster columns — OQ-3).** `spec req list` stays authored-only; the observed/
  verdict join lives **only** in `coverage`. Rationale: tier split by command boundary is
  the strongest defence of the NF-001 wall (no shared output path to blur), keeps the
  roster cheap (no corpus walk) and single-responsibility, and avoids a `--coverage` flag
  that would duplicate the `coverage SPEC` fan. The `coverage` view already shows authored
  status beside observed, so nothing is lost. *Alt rejected:* `spec req list --coverage`
  joined mode — parallel implementation of the spec fan + scope-creep on a list verb +
  the blur risk.
- **D4 (batched scanner — RSK-006).** Add `scan_coverage_batch` (one walk, bucket by
  req); the spec fan and single-req both ride it. Rationale: collapses N walks to one;
  additive, so the gate holds. *Alt rejected:* N× `scan_coverage` (the RSK-006
  amplification); caching the walk (premature, stateful).
- **D5 (verdict display single-source).** Terse `Verdict::label()` in `coverage.rs`;
  reconcile's prose `build_prompt` left untouched. Rationale: one source for the *variants*
  (the enum); the two *renderings* are genuinely different registers and merging would
  touch reconcile (gate). Documented non-merge, not an accident.

## 8. Risks & Mitigations

- **R1 (perf, RSK-006).** Spec-wide read fans the corpus walk. *Mitigation:* D4's single
  batched walk bounds it to one walk per invocation; staleness stays one-HEAD-resolve.
  Acceptable for v1; no cliff (RSK-006 deferred). INV-2 pins it.
- **R2 (tier blur, NF-001).** Co-displaying authored + derived in one table risks
  implying derivation. *Mitigation:* D3 (separate command), INV-1 (independent status
  source + test), tier-marking column/field names; no fake separator column needed —
  the wall is semantic and read-only.
- **R3 (ISS-006 double-count).** The batched walk inherits the corpus-walk footgun.
  *Mitigation:* lift the slug-symlink skip verbatim; a no-double-count test
  (`mem.pattern.entity.corpus-walk-skip-slug-symlink`).
- **R4 (gate regression).** *Mitigation:* additive-only changes to `coverage_scan`/
  `coverage`; existing suites + the SL-044 NF-001 proof stay green unchanged (F6).

## 9. Quality Engineering & Validation

TDD red/green/refactor. New tests:
- **`scan_coverage_batch` equivalence** — `batch(wanted)[req] == scan_coverage(req)` for
  every `req` over a multi-slice fixture; determinism; empty/missing/malformed tolerance;
  **ISS-006 slug-symlink no-double-count** (R3).
- **`observed_state` classifier** — total over the five canonical composite states (reuse
  `coverage.rs`'s `composites()` fixture shape).
- **`coverage_view` row materialisation** — authored `status` independent of coverage
  (INV-1); spec fan preserves member order; single-REQ path; `label = None` for a bare REQ.
- **ref dispatch** — REQ / SPEC / PRD / garbage.
- **black-box CLI goldens** (`mem.pattern.testing.black-box-cli-golden` + assert *every*
  surface, not just the JSON envelope: `mem.pattern.testing.conformance-asserts-surface-not-just-envelope`)
  — `coverage REQ`, `coverage SPEC`, `--json`, `--columns`, unknown-column error;
  `spec req list` table/json/columns.
- **`spec req list` authored-only** — output carries no observed/verdict, no scan (INV-3).
- **Behaviour-preservation** — `coverage` / `coverage_scan` / `reconcile` suites + the
  SL-044 NF-001 import-edge proof green **unchanged** (F6).

Lint as you go (`cargo clippy` zero warnings; `just check` before each commit).

## 10. Review Notes

- Internal adversarial pass: pending.
- External challenge (codex gpt-5.5): pending — offered at lock.
