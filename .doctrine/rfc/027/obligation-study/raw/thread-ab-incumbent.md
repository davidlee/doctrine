I now have a comprehensive picture. Let me compile the research brief.

---

# Research Brief: Incumbent obligation analogues and actionability semantics in Doctrine

## A — Incumbent Concepts

### A.1 — What corresponds to an "obligation" today

There is no entity called "obligation" in Doctrine, but three structures map directly to the proposed concept, all inside the per-phase `PlanPhase` model (`src/plan.rs`):

1. **Exit criteria (`EX-N`)** — the strongest analogue. Each `Criterion { id, text }` states a required state of affairs that must be true at the end of a phase. These are authored in `plan.toml` under `exit_criteria`, parsed into `Vec<Criterion>`, and are **normative authored data** in the TOML tier. Example from SL-170 PHASE-01 (`/workspace/doctrine/.doctrine/slice/170/plan.toml:43-45`):

   ```toml
   { id = "EX-1", text = "PlanPhase parses entrance_criteria: Vec<Criterion>, exit_criteria: Vec<Criterion>, verification: Vec<VerificationCriterion>; Criterion = { id, #[serde(default)] text }. Every added field is #[serde(default)]." }
   ```

   EX criteria are **obligations** — statements of "what must be true." They are NOT proofs; they are the things proofs verify.

2. **Verification criteria (`VT/VA/VH-N`)** — the proof layer. Each `VerificationCriterion` maps an obligation to a specific check. VT rows carry a structured mandate (`test_file`, `keywords`, `patterns`) that the vtgate (`src/vtgate.rs`) reads to mechanically verify compliance. VA (agent) and VH (human) rows carry free-text `expects` for non-automated checks. Example from SL-170 PHASE-03 (`/workspace/doctrine/.doctrine/slice/170/plan.toml:100-101`):

   ```toml
   { id = "VT-1", expects = "check_vt four-verdict unit (src/vtgate.rs): Pass; Fail-absent-file; ...", test_file = "src/vtgate.rs", keywords = ["VtVerdict", "Uncheckable", "Waived", "check_vt"] }
   ```

   The relationship is: EX states the obligation; VT proves it was met.

3. **Entrance criteria (`EN-N`)** — preconditions, not obligations. They gate whether a phase CAN begin. Example from SL-170 PHASE-01 (`/workspace/doctrine/.doctrine/slice/170/plan.toml:41-42`):
   ```toml
   { id = "EN-1", text = "SL-170 design.md is locked (2 adversarial passes integrated); §5.2 names the PlanPhase / Criterion / VerificationCriterion shape." }
   ```

4. **Phase `objective`** — a free-text `String` field on `PlanPhase`, stored in the TOML tier. Normative authored data (`src/plan.rs:63`): `pub objective: String`. It describes what the phase achieves. Example from SL-170 PHASE-01 (`/workspace/doctrine/.doctrine/slice/170/plan.toml:37-40`): a 4-line prose description of the lift. It is NOT structured, NOT queryable, and NOT consumed by any gate — it is for human reading only.

### A.2 — Where objectives live and their status

Phase objectives are **normative authored data in the TOML tier**. The field `objective: String` on `PlanPhase` (`src/plan.rs:63`) is populated from `plan.toml`'s `[[phase]]` rows. The template at `install/templates/plan.toml:56` shows `objective = ""`. The companion `phase.md` template (`install/templates/phase.md:9`) renders it as `## Objective\n\n{{objective}}` — but this is a **disposable** phase sheet under `.doctrine/state/`, not the authoritative source.

Every `#[serde(default)]` on `objective` means a legacy plan without it parses to `""`. The objective is NOT consumed by any gate or engine — it seeds the phase sheet for human context.

### A.3 — How criteria relate to objectives

**EN criteria** = preconditions (what must be true BEFORE the phase). They state required states of affairs about external context (design locked, previous phase merged, host available). Not obligations — gates.

**EX criteria** = obligations (what must be true AT THE END). They state the deliverables and invariants the phase must produce. These are the closest analogue to "obligation" in the proposed chain. Example from SL-183 PHASE-02 (`/workspace/doctrine/.doctrine/slice/183/plan.toml:83-84`):

```toml
{ id = "EX-1", text = "seatbelt_profile emits the PHASE-01 profile verbatim (ordering invariant F-A held) for a representative resolved policy." }
```

**VT criteria** = verification of obligations. Each VT row states how to prove a specific obligation. The mapping is many-to-many: one EX may have multiple VTs; one VT may cover multiple EXs. The chain is informal — there is no authored field linking `VT-1` to `EX-1`. The relationship exists only in the author's prose.

**VA / VH criteria** = agent/human verification. Same shape as VT but skipped by vtgate (`src/vtgate.rs:230` — `is_vt_mode` filters to `VT-` prefix only).

The proposed chain (requirement → design intent → obligation → verification criterion → proof) partially exists today as:
- **Requirement** → `REQ-NNN` entities in `.doctrine/requirement/`
- **Design intent** → no entity; implied by phase objectives and EX criteria
- **Obligation** → `EX-N` exit criteria
- **Verification criterion** → `VT/VA/VH-N` criteria
- **Proof** → the vtgate verdict (`Pass`/`Fail`/`Uncheckable`/`Unattributable`/`Waived`)

Missing: there is NO authored link between requirements and EX criteria or between EX criteria and VT criteria. The plan template has `specs = []` and `requirements = []` per-phase (`install/templates/plan.toml:49-52`), but these are empty in every existing slice's plan — they are scaffold-only.

### A.4 — Existing relation labels that express parts of the chain

From `src/relation.rs:45-187`, the `RelationLabel` vocabulary:

| Label | Chain role | Stored in |
|---|---|---|
| `References` with `Role::Implements` | slice → spec/PRD/REQ: the slice implements a requirement | slice TOML `[[relation]]` rows |
| `References` with `Role::OriginatesFrom` | slice → backlog: the slice originates from a backlog item | slice TOML `[[relation]]` |
| `Fulfils` | slice → backlog: the slice fulfils (completes) a backlog item, with per-edge `Degree` | slice TOML `[[relation]]` |
| `Members` | spec → requirement: the spec is woven from these requirements | spec TOML `[members]` table |
| `Shapes` | knowledge record → artifact: epistemic influence | knowledge record TOML `[[relation]]` |
| `Spawns` | knowledge record → backlog item: work creation | knowledge record TOML `[[relation]]` |
| `Interactions` | spec → spec: free-text edge type (`uses`, `calls`, etc.) | tech spec `interactions.toml` |
| `needs` (in `[relationships]`) | hard prerequisite — work item depends on another | slice/backlog/revision TOML `[relationships].needs` |
| `after` (in `[relationships]`) | soft sequence preference with per-edge `rank` | slice/backlog/revision TOML `[relationships].after` |

The `Members` relation (`install/templates/members.toml`) is the closest to the proposed "requirement → design intent" link: a spec declares `[[member]]` rows with `requirement = "REQ-NNN"` and a sticky `label = "FR-001"`. But it links spec→requirement, not requirement→design intent→obligation.

The `needs`/`after` relations (covered in detail in B.2) provide hard and soft dependency edges between work items, but at the **entity level**, not the phase or obligation level.

---

## B — Current Actionability Semantics

### B.1 — Where Doctrine decides something may run

There are **two independent actionability systems** operating at different altitudes:

**Cross-kind priority actionability** (entity-level, backlog/slice/revision):

| Component | File | What it does |
|---|---|---|
| `status_class()` | `src/priority/partition.rs:213` | Maps `(kind, status) → StatusClass::Workable/Gating/Terminal/Unrecognised` via a static `PARTITION` table |
| `eligible()` | `src/priority/channels.rs:58` | `class == Workable` — status-only gate |
| `blocked()` | `src/priority/channels.rs:86` | Has at least one non-terminal `needs` prereq |
| `blocked_by()` | `src/priority/channels.rs:73` | Direct non-terminal prereqs via `needs` edges |
| `actionable()` | `src/priority/channels.rs:91` | `eligible && !blocked` |
| `promoted()` | `src/priority/channels.rs:66` | Backlog-only: `resolution == Promoted` |

CLI verbs consuming this:
- `doctrine next` → `src/priority/mod.rs:103` → `surface::next()` → filters to actionable + not-promoted, then Kahn-sorts via surviving `after` edges + score
- `doctrine survey` → `src/priority/mod.rs:74` → `surface::survey()` → filters/ranks by actionability + score
- `doctrine blockers <ID>` → `src/priority/mod.rs:133` → `surface::blockers()` → shows what blocks a given entity
- `doctrine explain <ID>` → `src/priority/mod.rs:174` → `surface::explain()` → structured reasons
- `doctrine inspect <ID>` → `src/commands/inspect.rs:127` → `surface::actionability_block_from()` → per-entity actionability block

**Dispatch phase-level actionability** (phase funnel, within a single slice):

| Component | File | What it does |
|---|---|---|
| `next_core()` | `src/dispatch.rs:6823` | THE oracle — reads funnel record + phase sheets, delegates to `select_next()` |
| `select_next()` | `src/dispatch.rs:6600` | 4-rung purity ladder: triage → runnable verb → await worker → spawn |
| `compute_next_phases()` | `src/dispatch.rs:5214` | Readiness authority: scan in plan order, skip completed, first `in_progress` gates alone, first actionable `pending` + consecutive pendings run |
| `plan_next_rows()` | `src/dispatch.rs:5195` | Reads `plan.toml` phases + runtime sheet statuses into `(id, status, name)` rows |

CLI verbs consuming this:
- `doctrine dispatch next --slice N` → `dispatch.rs:6896` → `next_core()`
- `doctrine dispatch plan-next --slice N` → `dispatch.rs:4486` → `plan_next_rows()` + `compute_next_phases()`
- `doctrine dispatch status --slice N` → `dispatch.rs:4940` → full rollup including readiness

### B.2 — Where `needs` and `after` already exist

**Storage**: In entity TOML files under a `[relationships]` table, two axes:
- `needs = ["ISS-002", "RSK-001"]` — string array of hard-prerequisite canonical refs
- `after = [{ to = "ISS-003", rank = 2 }, { to = "ISS-004" }]` — array of `{to, rank}` inline tables for soft-sequence edges

**Leaf module**: `src/dep_seq.rs` — provides `read(toml_path) -> DepSeq`, `append(toml_path, RelEdit)`, `remove(toml_path, to, rank_ceiling)`. Pure `toml_edit` mutations — edit-preserving, idempotent, F-1 strict refuse on malformed entities.

**CLI**: `doctrine needs <SRC> <TGT>` and `doctrine after <SRC> <TGT> [--rank N]` → `src/commands/dep_seq.rs:133` and `:149`.

**Which entity kinds may carry them**:

*As source* (the `is_work_like()` predicate, `src/commands/dep_seq.rs:28`):
- `SL` (slice)
- `ISS`, `IMP`, `CHR`, `RSK`, `IDE` (five backlog kinds)
- `REV` (revision)

*As target* (the `is_admissible_dep_target()` predicate, `src/commands/dep_seq.rs:44`):
- All of the above (work-like) PLUS
- `ASM`, `DEC`, `QUE`, `CON`, `EVD`, `HYP`, `CPT` (knowledge records, SL-158 D2)

Governance docs (`SPEC`, `ADR`, `POL`, `STD`, `PRD`) are **excluded** from both gates — "depending on governance routes THROUGH a Revision, never the evergreen doc" (`src/commands/dep_seq.rs:33`).

**What consumes them**: The priority graph (`src/priority/graph.rs`) reads every entity's `dep_seq_for()` via `src/relation_graph.rs:69` → builds dep and seq cordage overlays → used by:
- `channels::blocked_by()` / `channels::blocking()` for direct blocker computation
- `channels::blocked_by_transitive()` / `channels::blocking_transitive()` for transitive walk
- `order::surviving_seq_predecessors()` / `order::frontier_order()` for `next` ordering
- `surface::next()`, `surface::survey()`, `surface::blockers()`, `surface::explain()`

**Limitation for phase/obligation altitude**: `needs`/`after` edges are between **entities** (SL-001 needs ISS-002), not between phases or between obligations within a phase. They cannot express "EX-1 depends on EX-2 having been verified" or "PHASE-03's VT-3 needs PHASE-02's EX-4 as a prereq." They are reusable in the sense that the `dep_seq` leaf is kind-agnostic — a new `Obligation` kind could carry `[relationships].needs`/`after` with zero engine changes — but the target kind gate (`is_admissible_dep_target`) would need widening.

### B.3 — What dispatch infers at runtime that is NOT in authored state

1. **Phase ordering is array-order from `plan.toml`** (`src/dispatch.rs:6830`: `record.rows.sort_by(|a, b| a.id.cmp(&b.id))` — `PHASE-NN` lexicographic = numerical sort). The plan file's `[[phase]]` array order is the sole sequencing authority. There is no authored `after` edge between phases.

2. **Phase readiness comes from disposable runtime sheets** (`src/state.rs:311`: `read_phase_status()` reads `.doctrine/state/slice/<N>/phase-nn.toml`). These carry a single `status` field (`pending`/`in_progress`/`completed`/`blocked`). This is a gitignored, disposable tier — NOT authored state.

3. **The `compute_next_phases()` algorithm** (`src/dispatch.rs:5214`) encodes manual sequencing judgement that is never authored:
   - Scan in plan order
   - Skip `completed`
   - First `in_progress` → gate alone (nothing else runs alongside an active phase)
   - First actionable `pending` + consecutive `pending` followers → batch together
   - Any `blocked` after an actionable phase → stop
   - This algorithm is inference, not authored rule — it is implicit in the code, not in a plan file

4. **Funnel machine transitions** (`src/funnel_machine.rs` — `expected_next()`) infer the next verb from a phase row's current `Position` + transition facts. This is a pure state machine, but the rules are encoded in Rust, not in authored TOML.

5. **The `specs` and `requirements` link tables** in `plan.toml` are scaffold-only — every existing slice leaves them empty (`specs = []`, `requirements = []`). The plan template carries them as commented-out future surface. No gate reads them.

### B.4 — Consumers that would read obligation-level actionability

| Consumer | What it uses today | Would it use obligation-level? |
|---|---|---|
| **Dispatch** (`dispatch next`, `dispatch plan-next`) | Phase-level runtime sheet status + plan array order + funnel machine | Yes — the readiness authority would need to know which obligations within a phase are unblocked |
| **Context construction** (boot snapshot, phase sheet rendering) | Phase `objective` string + phase sheet status | Yes — would surface which obligations remain in a phase |
| **VT gate** (`doctrine slice verify-vt`) | Reads `plan.toml` VT rows, runs file/keyword checks; per-phase, not per-obligation | Yes — could gate at obligation granularity (only run VTs for obligations whose prereqs are met) |
| **Audit** | Phase-level VT verdicts at conclude; per-phase not per-obligation | Yes — would track which obligations are proven vs. waived vs. unprovable |
| **Reconciliation** (`doctrine reconcile`) | Requirement coverage scan — REQ → spec → implementation evidence | Maybe — obligations could carry `fulfils REQ-NNN` edges, giving reconciliation finer-grained targets |
| **Progress display** (`dispatch status`) | Phase receipt status per phase (boundary-ledger-backed) | Yes — would show obligation completion within a phase rather than a single phase status |
| **`doctrine next` (priority)** | Entity-level actionability via `needs`/`after` edges and status class | No — this operates at entity altitude; obligation-level actionability is intra-slice |
| **`doctrine blockers`** | Entity-level direct/transitive blockers via `needs` edges | No — same altitude distinction |

---

## Unknowns / Low Confidence

1. **Whether `PlanPhase` will remain the envelope for obligations.** The plan model (`src/plan.rs`) currently skips `entrance_criteria`/`exit_criteria` on legacy plans via `#[serde(default)]` — adding an `obligations: Vec<Obligation>` field would need the same belt. I could not determine from code alone whether the design intent is to replace `exit_criteria` with obligations or to add obligations alongside them.

2. **The `specs` / `requirements` per-phase link tables.** They are `[]` in every existing slice. I could not find any code that reads them beyond the parse. The design intent for when/how they would be populated is not in the repository — it may be in a spec or RFC outside my search scope.

3. **Whether `is_admissible_dep_target` would need widening for obligation edges.** The predicate (`src/commands/dep_seq.rs:44`) already admits work-like + knowledge records. If obligations carry their own kind prefix, they'd need an entry in `kinds::KINDS` and membership in `ADMISSIBLE_DEP_TARGETS`. If they use a different storage pattern (e.g., inline in `plan.toml` rather than independent TOML files), they would need a different read path entirely.

4. **The exact shape of the `phase-NN.toml` runtime sheet.** I only read the `read_phase_status()` function which reads a single `status` field. The `render_tracking()` function (`src/state.rs:351`) produces a richer skeleton with a progress log, but I did not dereference it to confirm the full schema. The comment "Richer per-criterion/task rows graduate to TOML when a consumer lands (D5/Q2)" (`src/state.rs:348`) strongly suggests this was anticipated.

5. **The `derive_receipt_status` function's logic** — I only confirmed it exists and is called by `phase_projection()`, but did not read the folding logic that translates runtime sheet status + boundary ledger into the `ReceiptStatus` enum. This could be relevant if obligations need their own receipt status derivation.
