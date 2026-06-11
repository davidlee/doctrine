# Design SL-044: Reconcile writer + closure gate (SPEC-002 B)

> **STATUS: IN PROGRESS (partial).** Foundational decisions locked through a
> `/consult` on the override representation; §6 lists the OPEN questions a
> continuing agent must close before this design locks → `/inquisition` → `/plan`.
> Reference forms: padded entity ids (`SL-044`, `REQ-112`, `ADR-004`); doc-local
> refs bare (`D1`, `OPEN-1`).

## 1. Design Problem

Build the **reconcile + close** half of SPEC-002 (the observe half shipped as
**SL-042**): author reconciled requirement/spec truth through **one** writer, and
gate slice closure on coherence. Three phases — **B·P1** authored-truth write seam ·
**B·P2** sole-author reconcile writer · **B·P3** closure gate. Realises **REQ-112**
(FR-005) and **REQ-113** (FR-006); strengthens cross-cutting **REQ-114** (NF-001,
proven *structurally at the write seam* here — its load-bearing import-edge guard
lands in B, the slice that first has a status-writer to wall off) and **REQ-116**
(NF-003). Descends **PRD-013**. Resolves backlog **IMP-030**.

The hard line stays: observed evidence and authored truth never touch through a
function (`REQ-105`/NF-001). B is where authored truth is finally *written* — the
writer authors an **explicit** value, never one computed from coverage.

## 2. Current State — the substrate B consumes (all shipped by SL-042)

Concrete code shapes (verified against the tree):

- **`drift(authored: ReqStatus, composite: &Composite) -> Verdict`**
  (`src/coverage.rs:249`). `Verdict ∈ {Coherent, Divergent(DivergentReason),
  Indeterminate}` (`coverage.rs:226`); `DivergentReason ∈ {ObservedContradiction,
  EvidenceOutrunsAuthored}`. Read-only — returns **no** `ReqStatus`. At the gate,
  `{Divergent, Indeterminate}` = *unreconciled*.
- **`composite(entries: &[(CoverageEntry, IsStale)]) -> Composite`**
  (`coverage.rs:182`) — pure deterministic fold; `scan_coverage` (impure shell,
  `coverage_scan.rs:50`) walks `.doctrine/slice/*/coverage.toml`, filters by
  requirement, resolves `HEAD→SHA` once, resolves per-entry staleness via
  `git::commits_touching` (`git.rs:901`).
- **REC** (`src/rec.rs`): `RecDoc { id, slug, title, rec: RecMeta, status_delta:
  Vec<StatusDelta>, evidence_ref: Vec<EvidenceRef> }` (`rec.rs:121`). `RecMeta {
  r#move: String, owning_slice: Option<String>, decision_ref: Option<String> }`
  (`rec.rs:105`) — `move` is a **free String**, validated to {accept,revise,redesign}.
  `StatusDelta { requirement, from, to }` (all `String`, `rec.rs:87`).
  `EvidenceRef = coverage::CoverageKey` (the stable 4-tuple, `rec.rs:99`). REC is
  **status-less/immutable** (SL-042 D-Q3). `rec new` (`rec.rs:223`) writes an
  **empty ledger**; the template comment says deltas/evidence are "appended later by
  the reconcile writer (Slice B)" — see OPEN-2.
- **`slice status` close-gate** (`src/slice.rs:run_status`, L353). Fires the gate
  **only** on a closure-seam crossing (`crosses_closure_seam`, L373→`L601`); today
  scans `review::unresolved_blockers_for` (the D-C9b RV-blocker gate). The gate
  lives in the **command shell**, not the FSM writer `set_slice_status` (L478) —
  one-way coupling `slice-shell → query` (ADR-001: the queried module never imports
  `slice`). `set_slice_status_is_the_sole_seam_crosser` pins that the gate can't be
  bypassed. **B·P3 adds a drift scan beside the blocker scan, same shape.**
- **`set_slice_status`** (`slice.rs:478`) — the edit-preserving authored-TOML
  transition: `toml_edit::DocumentMut` in-place mutation (L505) preserves comments /
  `[relationships]` / unknown keys; classifies the move and gates `FromTerminal` /
  `SeamBreach` (F12/F13). **The pattern B·P1's `spec req status` mirrors**
  (`mem.pattern.entity.edit-preserving-status-transition`).
- **`ReqStatus`** enum (`src/requirement.rs:91`): `{Pending, InProgress, Active,
  Deprecated, Retired, Superseded}` — authored, a field on `Requirement`. The
  `spec req` tree (`src/spec.rs`) has **`add` only** (`run_req_add`, L684) — **no
  status-transition verb exists yet** (B·P1 builds it).
- **`KINDS`** table (`src/integrity.rs:46`); `REC_KIND` row L120, `REQUIREMENT_KIND`
  L78.

Nothing today writes reconciled authored status, emits a populated REC, or gates
closure on drift.

## 3. Forces & Constraints

- **Governance.** SPEC-002 D7 (sole writer, one REC/act, accept/revise/redesign),
  D8 (closure gate default-refuse + recorded override), D9 (CLI shapes deferred to
  build). ADR-003 (explicit-authorship-not-derivation). ADR-009 (slice FSM, the
  `reconcile→design` back-edge, the F12 closure seam). **ADR-004** (relations
  outbound-only; reverse is *derived*, never stored — decisive for §5.5). ADR-001
  (leaf←engine←command layering; the gate's one-way shell→query coupling).
- **No parallel implementation.** Reuse `set_slice_status`'s edit-preserving pattern
  for B·P1; reuse the `run_status` close-gate shell for B·P3; reuse `drift` /
  `scan_coverage` verbatim; reuse the SL-040 close-gate-as-corpus-scan grain.
- **Pure/imperative split.** Gate predicate + any reconcile classification are pure
  over resolved inputs; git/disk in the shell (mirrors SL-042 F1, and
  `mem.pattern.safety.resolve-every-ref-before-pure-compare`).

## 4. Guiding Principles

- **Author, never derive.** The writer reads `drift` only to *prompt*; the human/
  agent supplies the move and the explicit target status. NF-001's import-edge guard
  (no edge `coverage → status-writer`) goes live here (§5.6).
- **One act, one REC, fully reconstructable.** Every reconciliation act commits + a
  REC; the REC plus commits reconstruct *why* a requirement holds its current status
  from the authored tier alone (NF-003) — no recourse to chat/runtime state.
- **Each gate owns its slice's drift.** A slice's closure gate discharges only the
  drift live at *its* close, via RECs *it* owns; later re-drift is re-examined at the
  later slice's own gate (no stale cross-slice override).

## 5. Proposed Design

### 5.1 The three moves — `accept` reframed as *affirm* (LOCKED, /consult)

The moves are defined by **intent**, not by whether they write — this dissolves the
"override overloads accept" objection:

| Move | Intent | Status write | REC `status_delta` |
|---|---|---|---|
| **accept** | *Affirm* authored status as correct against the evidence | moves `from→to` **iff** authored lagged; **no move** (`to==from`) if already correct | the `[req,from,to]` (possibly `from==to`) |
| **revise** | *Correct* the authored claim (spec over-/mis-claimed) | always moves (correcting = changing); prose hand-edited | the `[req,from,to]` |
| **redesign** | *Escalate* — needs design rework | none | **empty** (F7) |

Under "affirm," a `from==to` accept is **not** a second meaning — it is the same act
where affirming requires no change. Material spec/ADR *prose* edits are **out of
scope** — they go through a future **Revision** vehicle (IDE-003); `revise` here is a
**structural `ReqStatus` write only**.

### 5.2 B·P1 — the write seam (one setter)

`spec req status <REQ> --to <state> [--note]` — an edit-preserving transition on the
requirement's authored `status`, mirroring `set_slice_status`
(`toml_edit::DocumentMut`, preserve comments/unknown keys, stamp `updated`). **Both
accept and revise reuse this one setter** (the "spec-truth revise path" in the scope
doc collapses into it — revise differs only by REC move + direction + the human's
prose hand-edit). OPEN-4: whether requirement status is a free any→any setter (like
`adr::set_adr_status`) or an ordered FSM (like `slice status`) — the `ReqStatus`
lifecycle ordering is unconfirmed.

### 5.3 B·P2 — the reconcile writer (sole author)

The only code that writes reconciled authored requirement status in the loop. Per
divergence it applies one move (§5.1) and emits **exactly one REC**:
- **accept** → call the B·P1 setter with the human's explicit target; record the
  delta + `evidence_ref`s (the coverage 4-tuples backing it).
- **revise** → B·P1 setter (corrected status); record delta; prose hand-edited.
- **redesign** → `slice status <id> design` (the ADR-009 back-edge); REC with empty
  `status_delta` (F7); **no** instance write.

OPEN-2 (act granularity + REC population): one REC per invocation (possibly spanning
multiple `status_delta` rows for several reqs) vs one REC per requirement; and the
mechanism — does the writer compose the full `RecDoc` and write atomically, or does
`rec new` (empty) + append verbs populate the ledger? The template implies "append,"
but no append verb exists yet.

### 5.4 — *(reserved: data flow / sequence — to draft)*

### 5.5 B·P3 — closure gate (LOCKED shape)

A second predicate in `run_status`, beside the RV-blocker scan, firing on the same
`crosses_closure_seam` (specifically `reconcile→done`):

1. Determine the requirements the closing slice **S** is responsible for —
   **D-B5 (LOCKED):** the gate's req set is `covered ∪ declared`, where `covered` =
   the distinct reqs across S's `coverage.toml` entries (structural, always
   available) and `declared` = an **authored, additive extra-req list** in S's
   `coverage.toml` (a top-level `[gate] extra_reqs = [...]` table, `#[serde(default)]`
   so old/empty files still parse). The scope is **risk-calibrated, not a fixed
   policy** — the slice author casts it as wide as scope & risk warrant **at
   `/plan`**, where it is peer-reviewed *before* any REC exists. Additive by
   construction: you can never gate *less* than you covered. This sidesteps IMP-016
   (no prose spec→req relation needed — the author names the reqs).
2. For each such R: `drift(authored(R), composite(scan_coverage(R)))`. Any
   `{Divergent, Indeterminate}` = residual drift.
3. **Discharge predicate (LOCKED):** residual drift on R is excused iff ∃ a REC with
   `owning_slice == S` whose `status_delta` names R with `to == R's current authored
   status` (the act *affirmed R at the value it now holds*). `owning_slice == S` is
   load-bearing — a gate honours only its own slice's RECs.
4. Undischarged residual drift → **refuse** (bail, like the blocker gate). The
   `done`-only-from-`reconcile` F12 topology stays hard, independent of this check.

The override is therefore *unrepresentable as a flag* — it is a real REC
(`accept`/`from==to`, §5.1), so "closed with unreconciled drift" cannot exist (D8).

**Reverse lookup = on-demand scan, NOT a stored link (LOCKED, /consult).** The gate
+ NF-003 reconstruction find "R's last significant reconciliation act" by scanning
the REC corpus (max-id, `owning_slice`-scoped, naming R) — **not** a durable
`req→last_rec` field. Rationale: ADR-004 stores edges outbound-only and *derives*
the reverse; a stored "latest" pointer is a denormalization that can **desync** from
the corpus (the exact failure ADR-004 prevents), whereas a scan is always truthful.
REC ids are authored + monotonic, so "last" is reconstructable from the authored
tier alone (NF-003). Consistent with the two existing scan precedents (SL-040 D-C9b
close-gate; SL-042 D-Q2 coverage fan-in). **Perf escalation is RSK-006** — if the
scan cliffs below realistic scale, the reverse-index lands there, documented, rather
than denormalizing now.

### 5.6 NF-001 import-edge enforcement (B's load-bearing addition)

The structural guard SL-042 left vacuous (no writer to wall off): **no import edge
`coverage → status-writer`.** The reconcile writer (§5.3) is the *only* module that
imports both `coverage` (read `drift`) and the B·P1 status-writer — and it bridges
them by passing a **human-authored** value, never a computed one. OPEN-3: the
enforcement mechanism (a compile/grep assertion that the status-writer module has no
`use coverage`; doctrine has no arch-test framework, so SL-042 used grep/compile
assertions — `coverage.rs:222` already documents the `Verdict`-returns-no-`ReqStatus`
type-level half).

## 6. Open Questions (a continuing agent MUST close before lock)

- ~~**OPEN-1**~~ — **RESOLVED → D-B5** (drift scope is a risk-calibrated authored
  declaration in `coverage.toml`, set & peer-reviewed at `/plan`; gate set =
  `covered ∪ declared`). See §5.5 step 1, §7 D-B5.
- **OPEN-2** — reconcile **act granularity** (one REC/invocation vs one REC/req) +
  **REC population mechanism** (atomic `RecDoc` compose-and-write vs `rec new` +
  append verbs). No append verb exists today.
- **OPEN-3** — NF-001 **import-edge enforcement mechanism** (grep/compile assertion;
  where the seam boundary is drawn so the test is meaningful).
- **OPEN-4** — `spec req status` transition **discipline**: free any→any setter vs
  an ordered `ReqStatus` FSM. Depends on whether `ReqStatus` has a lifecycle order.
- **Carried (deferred, not blockers):** OQ-3 composite precedence (v1 surfaces all,
  `Indeterminate` = drift at the gate); IDE-003 Revision vehicle for prose; RSK-006
  scan perf revisit at this reader.

## 7. Decisions, Rationale & Alternatives (LOCKED so far)

- **D-B1 — accept = "affirm authored status against evidence."** Moves iff authored
  lagged; `from==to` if already correct. Dissolves the override overload; keeps 3
  moves; zero REC-schema change. *Alt rejected:* a 4th `override` move (user
  constraint: stay at 3); empty-delta accept (NF-003 weaker — affirmed value only
  implied; gate needs a second match field).
- **D-B2 — each gate discharges only its own slice's drift.** Override REC must be
  `owning_slice == closing slice`. Removes the stale-cross-slice-override case
  entirely (it's re-examined at the later slice's gate). *Alt rejected:* corpus-wide
  override honoured anywhere (a later re-drift would be silently excused).
- **D-B3 — reverse `req→rec` lookup is an on-demand scan, never a stored link.**
  ADR-004 outbound-only + anti-desync; reconstructable via monotonic REC ids; matches
  two precedents; perf escalation = RSK-006. *Alt rejected:* durable `req.last_rec`
  field (mutable authored state, `requirement→rec` coupling, denormalization desync).
- **D-B4 — B·P1 is one setter; both accept & revise reuse it.** revise = structural
  status only; material prose → IDE-003. *Alt rejected:* a separate spec-truth-revise
  verb (no distinct structural write exists; it would overlap IDE-003).
- **D-B5 — closure-gate drift scope is a risk-calibrated authored declaration, not a
  fixed policy.** The gate's req set = `covered ∪ declared`: `covered` derives
  structurally from S's `coverage.toml` entries; `declared` is an **additive**
  authored extra-req list (`[gate] extra_reqs`) in the same `coverage.toml`,
  decided **at `/plan`** (ahead of any REC), peer-reviewed, calibrated to the
  slice's scope & risk. The gate reads it at closure. *Rationale:* the right scope
  is not knowable in the framework — it is a per-slice risk judgement, and risk
  judgements belong in authored, reviewable artifacts (ADR-003 author-don't-derive
  ethos), not baked into gate code. Additive so a slice can never silently gate
  *less* than it covered. Home = `coverage.toml` (the gate already reads it for
  `covered`; one place, one read). *Alt rejected:* (a) a hardcoded
  `covered`-only-or-realised-specs policy — un-calibratable, and the realised-specs
  pole needs the IMP-016 prose→req relation that doesn't exist; (b) home in
  `plan.toml`/`slice-nnn.toml` — splits the gate's input across two files for no
  gain (user-chosen: `coverage.toml`).

## 8. Risks & Mitigations *(to expand)*

- **R-B1 — OPEN-2 ledger population.** If `rec new` writes empty and no append verb
  exists, B·P2 must either compose the full `RecDoc` atomically or add an append
  path; the choice affects the one-REC-per-act invariant. Resolve in OPEN-2.
- **R-B2 — gate scope under-check (mitigated by D-B5).** A pure coverage.toml-derived
  scope under-checks a req the slice realises but recorded no coverage for. **D-B5
  mitigates:** the author widens the gate via the authored `extra_reqs` list at
  `/plan`, peer-reviewed against scope & risk. *Residual risk:* the widening is a
  human judgement — an author can still under-declare. The peer review at `/plan` is
  the control; the additive default guarantees the floor (never < covered). The
  spec-wide automation that would remove the judgement entirely still needs the
  IMP-016 prose→req relation.

## 9. Quality Engineering & Validation *(sketch — to expand)*

- **REQ-112** — writer applies each move; **exactly one REC per act**; accept writes
  status via the B·P1 setter; redesign escalates with empty-delta REC + no instance
  write.
- **REQ-113** — gate refuses undischarged residual drift; an `owning_slice`-scoped
  `from==to` accept REC discharges it; F12 `reconcile→done` topology stays hard
  independent of the drift check. **D-B5 scope:** gate set = `covered ∪ declared`;
  an `extra_reqs`-declared req with residual drift blocks closure same as a covered
  one; an old/empty `coverage.toml` (no `[gate]` table) parses to `declared = ∅`
  and gates on `covered` alone (additive default, back-compat round-trip test).
- **REQ-114 / NF-001** — structural: no `coverage → status-writer` import edge
  (grep/compile, OPEN-3); the writer authors explicit values (the bridge passes a
  human value, not `f(coverage)`).
- **REQ-116 / NF-003** — a REC + commits reconstruct a requirement's current status
  with no runtime-state recourse; the on-demand scan resolves "last act" from
  authored REC ids alone.
- Lint/format gates per house rules (`cargo clippy` zero-warning bins/lib, `just
  check`).

## 10. Review Notes

- **/consult (2026-06-12)** — override representation. Resolved: D-B1 (affirm
  reframe), D-B2 (per-slice gate ownership), D-B3 (on-demand scan over stored link,
  ADR-004 anti-desync). Recorded in §5.1/§5.5/§7.
- Internal adversarial pass + external (`/inquisition` or codex) — **pending** (after
  OPEN-1..4 close).
