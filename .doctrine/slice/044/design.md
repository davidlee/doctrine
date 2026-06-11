# Design SL-044: Reconcile writer + closure gate (SPEC-002 B)

> **STATUS: COMPLETE — pending adversarial pass + lock.** All four OPEN questions
> resolved (D-B5..D-B8, §6/§7); §5 fully drafted incl. the §5.4 sequence. Remaining
> before lock: external adversarial pass (`/inquisition` or codex), reconcile
> `slice-044.md` B·P1 wording, then `/plan`. Reference forms: padded entity ids
> (`SL-044`, `REQ-112`, `ADR-004`); doc-local refs bare (`D1`, `OPEN-1`).

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
  agent supplies the move and the explicit target status. NF-001 holds **type-level**
  — `coverage` exposes no `ReqStatus`, so the necessary `use coverage` (to read
  `drift`) still cannot derive a status (§5.6, D-B7).
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
prose hand-edit). **D-B6 (LOCKED):** the setter is a **free any→any** edit-preserving
transition (mirrors `governance::set_status`, the adr precedent — *not* the ordered
`set_slice_status` FSM). `ReqStatus` enforces no lifecycle order today, and reconcile
actively needs free movement: `revise` corrects a mis-claim and must move **any**
direction (e.g. `active→pending` when evidence refutes a claimed satisfaction) — a
forward-only FSM would refuse exactly the corrections `revise` exists to make. No
terminal guard in v1 (a mis-`retired` req must be un-retirable by `revise`); terminal
discipline, if later wanted, is additive.

### 5.3 B·P2 — the reconcile writer (sole author)

The only code that writes reconciled authored requirement status in the loop. Per
divergence it applies one move (§5.1) and emits **exactly one REC**:
- **accept** → call the B·P1 setter with the human's explicit target; record the
  delta + `evidence_ref`s (the coverage 4-tuples backing it).
- **revise** → B·P1 setter (corrected status); record delta; prose hand-edited.
- **redesign** → `slice status <id> design` (the ADR-009 back-edge); REC with empty
  `status_delta` (F7); **no** instance write.

**D-B8 (LOCKED) — one REC per requirement, composed atomically.** Granularity is
**forced**: `RecMeta.move` is a single `String` (`rec.rs:105`), so one REC carries
exactly one move — a session reconciling reqs under *different* moves (redesign X,
accept Y) physically cannot share a REC. One act = one requirement's move = one REC
(one `status_delta` row, its `evidence_ref`s). Population = the writer **composes the
full `RecDoc` and writes it atomically** — *not* `rec new` (empty) + append. Append
contradicts REC immutability (SL-042 D-Q3: REC is status-less/immutable) and no
append verb exists; the template's "appended later" comment is superseded. (A single
CLI invocation MAY still reconcile several reqs — it just emits one atomic REC each.)

### 5.4 Data flow / sequence

**Reconcile (B·P2), per requirement R the operator chose to reconcile:**

1. Shell resolves inputs: `scan_coverage(R)` → `composite`; read R's authored
   `ReqStatus`; `drift(authored, &composite)` → `Verdict` (read-only — *prompts*
   only, never feeds the write; D-B7).
2. Operator picks the move (accept / revise / redesign) and, for accept/revise, the
   **explicit** target `ReqStatus` (the human's value, not the verdict).
3. Writer applies the move:
   - **accept/revise** → B·P1 setter writes R's status edit-preservingly (D-B6,
     free any→any); compose a `RecDoc` { one `status_delta {R, from, to}`,
     `evidence_ref` = the backing `CoverageKey`s, `rec.move`, `owning_slice = S }`.
   - **redesign** → `slice status <S> design` back-edge; `RecDoc` with **empty**
     `status_delta` (F7); no status write.
4. Write the REC atomically (D-B8 — one REC, fully composed). Commit (one act → one
   commit + one REC, NF-003).

Purity split: steps 1 & 4 are the impure shell (git/disk/clock); the move
classification and the `RecDoc` composition are pure over resolved inputs.

**Close gate (B·P3), on a `reconcile→done` crossing for slice S** — see §5.5: resolve
S's gate req set (`covered ∪ declared`, D-B5) → per R, `drift` over fresh
`scan_coverage` → residual drift discharged only by an `owning_slice == S` REC
affirming R at its current value → any undischarged → refuse.

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

### 5.6 NF-001 enforcement = type-level, not an import-edge ban (D-B7, LOCKED)

**Correction to the original framing.** A "no import edge `coverage → status-writer`"
ban is *wrong*: the reconcile writer (§5.3) **must** import `coverage` — it reads
`drift` to *prompt* the human. The import is legitimate and necessary; banning it
would forbid the read the writer exists to do.

The real, load-bearing wall is **type-level** and already shipped by SL-042: `drift`
returns `Verdict` and `composite` returns `Composite` — **neither exposes a
`ReqStatus`** (`coverage.rs:222`/`:226`). So even with `use coverage` in scope,
`status = f(coverage)` **cannot compile** — there is no `ReqStatus` to derive. The
B·P1 setter (§5.2) takes an **explicit caller-supplied `ReqStatus`**; the writer
passes the *human's* value, and the verdict only steers the prompt, never the
argument. NF-001 holds by construction.

**Enforcement mechanism (D-B7):** continue SL-042's type-level approach — a
compile-fail-style VT at the writer mirroring `coverage.rs:685` (a documented
"would-not-compile" assertion that no `coverage`-derived value reaches the status
argument) plus a positive test that the written status is independent of the drift
`Verdict`. **No grep arch-test** — doctrine has no arch-test framework, and a
`use coverage` ban would be brittle *and* wrong (it forbids the legitimate read).

## 6. Open Questions (a continuing agent MUST close before lock)

- ~~**OPEN-1**~~ — **RESOLVED → D-B5** (drift scope is a risk-calibrated authored
  declaration in `coverage.toml`, set & peer-reviewed at `/plan`; gate set =
  `covered ∪ declared`). See §5.5 step 1, §7 D-B5.
- ~~**OPEN-2**~~ — **RESOLVED → D-B8** (one REC per req, forced by the single `move`
  field; atomic full-`RecDoc` compose, no append — append contradicts REC
  immutability). See §5.3.
- ~~**OPEN-3**~~ — **RESOLVED → D-B7** (NF-001 is a type-level guard, not an
  import-edge ban — the writer *must* import `coverage`; the wall is that `coverage`
  exposes no `ReqStatus`). See §5.6.
- ~~**OPEN-4**~~ — **RESOLVED → D-B6** (free any→any setter; `revise` must move any
  direction to correct a mis-claim; no enforced `ReqStatus` order). See §5.2.
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
- **D-B6 — `spec req status` is a free any→any edit-preserving setter, not an FSM.**
  Mirrors `governance::set_status` (adr), not `set_slice_status`. `revise` must move
  any direction to correct a mis-claim; a forward-only FSM refuses exactly those
  corrections; `ReqStatus` enforces no order today. No v1 terminal guard. *Alt
  rejected:* ordered FSM (fights reconcile's correction use case; no lifecycle order
  to enforce anyway).
- **D-B7 — NF-001 is enforced type-level, not by an import-edge ban.** The writer
  *must* `use coverage` (to read `drift` for prompting); the wall is that `coverage`
  exposes no `ReqStatus`, so `status = f(coverage)` cannot compile. Compile-fail VT
  (mirrors `coverage.rs:685`) + positive independence test. *Alt rejected:* a grep
  `use coverage` ban — brittle, no arch-test framework, and *wrong* (it forbids the
  legitimate read). *Corrects* the original "no import edge" framing.
- **D-B8 — one REC per requirement, composed atomically.** Forced by the single
  `RecMeta.move` String (mixed-move sessions can't share a REC). Writer composes the
  full `RecDoc` and writes it once; no `rec new`+append. *Alt rejected:* one
  REC/invocation spanning many deltas (breaks on mixed moves); rec-new-then-append
  (contradicts REC immutability SL-042 D-Q3; no append verb exists).

## 8. Risks & Mitigations *(to expand)*

- **R-B1 — ledger population (resolved by D-B8).** `rec new` writes empty and no
  append verb exists; B·P2 composes the full `RecDoc` and writes atomically (append
  would contradict REC immutability anyway). *Residual:* B·P2 needs a compose-and-
  write path the `rec` module doesn't expose yet — a new internal writer (not a new
  public append verb). Surfaces at `/plan` as a B·P2 task.
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
- **REQ-114 / NF-001** — type-level (D-B7): a compile-fail VT mirroring
  `coverage.rs:685` proves no `coverage`-derived value reaches the status argument
  (the writer imports `coverage` for `drift` yet cannot derive a `ReqStatus` —
  `Verdict`/`Composite` carry none); a positive test pins the written status
  independent of the drift `Verdict`.
- **REQ-116 / NF-003** — a REC + commits reconstruct a requirement's current status
  with no runtime-state recourse; the on-demand scan resolves "last act" from
  authored REC ids alone.
- Lint/format gates per house rules (`cargo clippy` zero-warning bins/lib, `just
  check`).

## 10. Review Notes

- **/consult (2026-06-12)** — override representation. Resolved: D-B1 (affirm
  reframe), D-B2 (per-slice gate ownership), D-B3 (on-demand scan over stored link,
  ADR-004 anti-desync). Recorded in §5.1/§5.5/§7.
- **OPEN-1..4 closed (2026-06-12).** D-B5 (OPEN-1, user-decided: risk-calibrated
  authored gate scope in `coverage.toml`, home user-chosen). D-B6 (OPEN-4: free
  any→any setter). D-B7 (OPEN-3: type-level NF-001, corrects the import-edge
  framing). D-B8 (OPEN-2: one REC/req, atomic — forced by the single `move` field).
  D-B6/B7/B8 resolved by code-grounded reasoning, not preference; surfaced for the
  user + the adversarial pass to contest.
- External adversarial pass (`/inquisition` or codex) — **pending** (design now
  complete; this is the lock gate).
