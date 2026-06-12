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
proven *behaviourally at the write seam* here — the load-bearing
verdict-independence guard lands in B, the slice that first has a status-writer; see
§5.6/D-B7) and **REQ-116** (NF-003). Descends **PRD-013**. Resolves backlog
**IMP-030**.

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
  agent supplies the move and the explicit `--to` status. NF-001 is an **info-flow
  invariant** (no coverage→authored-status path but through human judgment) — not
  type-expressible, so the wall is layered: a status-select fn whose params exclude
  coverage types (type-level), the verdict consumed by the prompt builder, and a
  verdict-independence test on the residual call site (§5.6, D-B7).
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
discipline, if later wanted, is additive. **Caveat (codex finding 8):**
`doc/spec-entity-spec.md:325` describes an *intended ordered* `ReqStatus` lifecycle —
not enforced in code today. The free setter is scoped to the **reconcile** writer
(its job is correction); a terminal exit (`→ retired/superseded`) should still carry
a REC/note, and once a real successor edge exists, setting `superseded` without one
becomes a validation target (deferred, additive — not v1).

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
   **D-B5 (LOCKED):** the gate's req set is `covered ∪ declared ∪ reconciled`:
   - **`covered`** = the distinct reqs S itself recorded coverage for. **Defined
     precisely** (codex finding 6): the entries *physically in S's own*
     `.doctrine/slice/044/coverage.toml`, **and** the reader **validates**
     `key.slice == S` for each, refusing a slice/key mismatch (a foreign `slice =`
     in S's own file is an integrity error, not silently in-or-out). Needs a new
     **slice-local coverage reader** returning distinct requirements — `scan_coverage`
     is per-req-across-all-slices and does not enumerate one slice's reqs.
   - **`declared`** = an **authored, additive extra-req list** in **`slice-044.toml`**
     (`[gate] extra_reqs = [...]` — authored slice-metadata tier, *not* the
     observed-evidence `coverage.toml`; codex finding 4). Risk-calibrated: the author
     casts it as wide as scope & risk warrant **at `/plan`**, peer-reviewed *before*
     any REC exists. Sidesteps IMP-016 (no prose spec→req relation — the author names
     the reqs).
   - **`reconciled`** = every req named in a `status_delta` of a REC with
     `owning_slice == S` (codex finding 3 — **closes the opt-in dodge**: you cannot
     reconcile a req via a REC and then escape its gate by not covering/declaring it).
   Additive by construction — the gate can never check *less* than the union of what
   S covered, declared, or reconciled.
2. For each such R: `drift(authored(R), composite(scan_coverage(R)))`. Any
   `{Divergent, Indeterminate}` = residual drift.
3. **Discharge predicate (LOCKED, strengthened — codex finding 2):** residual drift
   on R is excused iff the **latest** REC for `(owning_slice == S, R)` satisfies **all**:
   (a) `move == accept` (an *affirm* — not redesign/revise);
   (b) its `status_delta` names R with `to == R's current authored status` (affirmed
   *at the value R now holds* — guards a status edited away-and-back);
   (c) its `evidence_ref` set **covers the current residual drift's evidence keys**
   (the `CoverageKey`s feeding today's `composite` for R) — so *fresh contradictory
   evidence arriving after the REC re-opens drift* the stale REC can no longer excuse.
   `owning_slice == S` stays load-bearing — a gate honours only its own slice's RECs.
   *Without (a)+(c) a months-old affirm would discharge live drift it never saw.*
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

### 5.6 NF-001 enforcement = info-flow, walled at the function signature (D-B7, LOCKED)

**NF-001 is an information-flow invariant**, not a type property: *no causal path
carries observed coverage into authored `ReqStatus`, except through human judgment.*
Rust's type system cannot express information flow (no taint/effect tracking), so
**no choice of types makes the forbidden derivation un-writable** — any value in
scope can be hand-mapped to any enum. Three prior framings drew the wall at the wrong
granularity:

- ✗ **import boundary** ("no `use coverage` in the writer") — wrong: the writer
  *must* read `drift` to prompt.
- ✗ **return-type boundary** ("`coverage` exposes no `ReqStatus`") — wrong (codex
  finding 1): a `match` over the verdict launders one and still compiles —
  ```rust
  let to = match verdict {            // compiles; IS a coverage→status derivation
      Verdict::Divergent(EvidenceOutrunsAuthored) => ReqStatus::Active,
      Verdict::Divergent(ObservedContradiction)   => ReqStatus::Pending,
      _ => authored,
  };
  ```
- ✓ **function-parameter boundary** — the correct structural wall.

**The wall (D-B7), layered — no single mechanism suffices:**

1. **Signature isolation (type-level, the load-bearing part).** Status selection is a
   pure fn whose parameter list **excludes every coverage-derived type**:
   ```rust
   fn select_status(to: ReqStatus, prior: ReqStatus) -> ReqStatus  // no Verdict/Composite/coverage in scope
   ```
   Inside `select_status` the compiler *does* prove no derivation — you cannot use
   data you were never handed; the laundering `match` won't compile because `verdict`
   is out of scope. This shrinks the laundering surface from "anywhere in the writer"
   to **one call site**.
2. **Verdict consumed by the prompt builder.** `drift`'s `Verdict` flows *only* into
   `fn build_prompt(verdict) -> PromptText` and is consumed there — not live at the
   write — so it cannot be threaded into `select_status`'s `to` argument.
3. **Test on the one residual site.** `--to` is a clap-parsed independent CLI input;
   a **verdict-independence VT** holds `--to` fixed, varies **every coverage-derived
   input the handler can see** (the whole `Composite`/staleness, not just the
   `Verdict` discriminant), and asserts the written status never moves. This guards
   the only place left to launder — the handler wiring `--to` into `select_status`.

**Honest limit:** info-flow is *approximated, never proven* in Rust. (1) makes the
bulk type-impossible; (2) removes the verdict from the write scope; (3) covers the
residual call site. The combination is the guarantee — the test alone (an earlier
framing) understates it, the type system alone cannot reach it.

## 6. Open Questions (a continuing agent MUST close before lock)

- ~~**OPEN-1**~~ — **RESOLVED → D-B5** (drift scope is a risk-calibrated authored
  declaration in `slice-044.toml`, set & peer-reviewed at `/plan`; gate set =
  `covered ∪ declared ∪ reconciled`). See §5.5 step 1, §7 D-B5.
- ~~**OPEN-2**~~ — **RESOLVED → D-B8** (one REC per req, forced by the single `move`
  field; atomic full-`RecDoc` compose, no append — append contradicts REC
  immutability). See §5.3.
- ~~**OPEN-3**~~ — **RESOLVED → D-B7** (NF-001 is a *behavioural* no-derivation guard
  — `--to` is an independent CLI input, tested verdict-independent across every
  `Verdict`; neither an import-edge ban nor a type-level proof, both of which fail).
  See §5.6.
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
  fixed policy.** The gate's req set = `covered ∪ declared ∪ reconciled`: `covered` =
  the distinct reqs *physically in S's own* `coverage.toml`, each validated
  `key.slice == S`; `declared` = an **additive** authored extra-req list
  (`[gate] extra_reqs`) in **`slice-044.toml`** (authored slice-metadata tier);
  `reconciled` = every req a REC `owning_slice == S` names. `declared` is decided **at
  `/plan`** (ahead of any REC), peer-reviewed, calibrated to scope & risk.
  *Rationale:* the right scope is not knowable in the framework — it is a per-slice
  risk judgement, and risk judgements belong in authored, reviewable artifacts
  (ADR-003 author-don't-derive ethos), not in gate code. Additive so a slice can
  never silently gate *less* than it covered/declared/reconciled. *Home* = the
  authored **slice TOML, not `coverage.toml`** (codex finding 4: coverage is the
  *observed-evidence* tier — `coverage.rs:16` — and putting closure policy there
  mixes tiers; the original "gate already reads coverage" convenience evaporated once
  findings 5/6 showed a typed field + a new slice-local reader are needed regardless).
  The `reconciled` term (codex finding 3) **closes the opt-in dodge** — a hardcoded
  `covered`-only scope let a slice reconcile a req then escape its gate by not
  covering it. *Alt rejected:* (a) a hardcoded `covered`-only-or-realised-specs policy
  — un-calibratable, realised-specs needs the absent IMP-016 prose→req relation;
  (b) home in `coverage.toml` — tier breach (user re-decided to slice TOML after the
  adversarial pass); (c) `plan.toml` — per-phase-plan tier, but gate scope is a
  slice-level closure property outliving any single plan.
- **D-B6 — `spec req status` is a free any→any edit-preserving setter, not an FSM.**
  Mirrors `governance::set_status` (adr), not `set_slice_status`. `revise` must move
  any direction to correct a mis-claim; a forward-only FSM refuses exactly those
  corrections; `ReqStatus` enforces no order today. No v1 terminal guard. *Alt
  rejected:* ordered FSM (fights reconcile's correction use case; no lifecycle order
  to enforce anyway).
- **D-B7 — NF-001 is an info-flow invariant, walled at the function signature +
  tested at the residual site.** Info-flow is not type-expressible in Rust, so no
  type choice forbids the derivation outright. The wall is layered (§5.6):
  (1) status selection is a pure fn whose params **exclude** every coverage-derived
  type (`select_status(to, prior) -> ReqStatus`) — type-level, shrinks laundering to
  one call site; (2) the `Verdict` is consumed by the prompt builder, not live at the
  write; (3) a verdict-independence VT (vary *all* coverage-derived inputs, hold
  `--to`) guards the residual handler site. *Alt rejected:* (a) import-edge ban —
  wrong, writer must read `drift`; (b) "`coverage` exposes no `ReqStatus`, so it can't
  compile" — **false**, a `match` launders it (codex finding 1); (c) behavioural test
  *alone* — understates it, misses non-`Verdict` channels. *Corrects three* prior
  framings (import → return-type → test-only).
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
- **R-B2 — gate scope under-check (mitigated by D-B5's `reconciled` term).** A pure
  `covered`-derived scope under-checks a req the slice touched but recorded no
  coverage for. The **`reconciled`** term (any REC-named req, codex finding 3) closes
  the worst case — a req the slice *reconciled* is auto-gated regardless of coverage/
  declaration, so it cannot be dodged. *Residual:* a req the slice changed but neither
  covered, declared, **nor** reconciled still escapes — that requires a structural
  spec/slice→req relation (IMP-016) to fully automate; until then `declared` +
  `/plan` peer review is the control. Floor guaranteed: never < `covered ∪ reconciled`.
- **R-B3 — stale-REC discharge (mitigated, codex finding 2).** A `to==current` match
  alone let an old affirm excuse drift it never saw (status edited away-and-back, or
  fresh contradictory evidence). Discharge now requires `move==accept` + latest-REC +
  `evidence_ref ⊇ current residual evidence keys` (§5.5 step 3). *Residual:* the
  evidence-coverage check assumes residual drift's `CoverageKey`s are enumerable at
  the gate — they are (the `composite` cells feeding the verdict); pin with a VT where
  a post-REC evidence cell re-opens drift and is *not* discharged.
- **R-B4 — `covered` enumeration is new machinery (codex finding 6).** No slice-local
  coverage reader exists; `scan_coverage` is per-req-across-slices and `key.slice` is
  unvalidated. B·P3 must add a slice-local reader returning distinct reqs and refusing
  `key.slice != S` in S's own file. Small, but real new surface — a B·P3 task at `/plan`.

## 9. Quality Engineering & Validation *(sketch — to expand)*

- **REQ-112** — writer applies each move; **exactly one REC per act**; accept writes
  status via the B·P1 setter; redesign escalates with empty-delta REC + no instance
  write.
- **REQ-113** — gate refuses undischarged residual drift; F12 `reconcile→done`
  topology stays hard independent of the drift check. **D-B5 scope:** gate set =
  `covered ∪ declared ∪ reconciled`; an `extra_reqs`-declared req *and* a
  REC-`reconciled` req each block closure on residual drift same as a covered one; a
  slice with no `[gate]` table in `slice-044.toml` → `declared = ∅`, gates on
  `covered ∪ reconciled` (additive floor). **Discharge VTs (D-B5 step 3):** (i) an
  `owning_slice==S`, `move==accept`, `to==current` REC whose `evidence_ref` covers the
  residual evidence discharges; (ii) the *same* REC does **not** discharge once a
  post-REC coverage cell introduces fresh drift (evidence-coverage clause); (iii) a
  `revise`/`redesign` REC does **not** discharge (move==accept clause); (iv) a foreign
  `owning_slice` REC does **not** discharge.
- **REQ-114 / NF-001** — info-flow, layered (D-B7, §5.6): **(structural)**
  `select_status`'s signature excludes every coverage-derived type — a unit/compile
  check that the fn is called with only `(--to, prior)`; **(structural)** the
  `Verdict` is consumed by the prompt builder, absent at the write; **(behavioural)** a
  verdict-independence VT — hold `--to` fixed, vary *all* coverage-derived handler
  inputs (`Composite`/staleness, not just the `Verdict` discriminant), assert the
  written status never moves.
- **REQ-116 / NF-003** — a REC + commits reconstruct a requirement's current status
  with no runtime-state recourse; the on-demand scan resolves "last act" from
  authored REC ids alone.
- Lint/format gates per house rules (`cargo clippy` zero-warning bins/lib, `just
  check`).

## 10. Review Notes

- **/consult (2026-06-12)** — override representation. Resolved: D-B1 (affirm
  reframe), D-B2 (per-slice gate ownership), D-B3 (on-demand scan over stored link,
  ADR-004 anti-desync). Recorded in §5.1/§5.5/§7.
- **OPEN-1..4 closed (2026-06-12).** D-B5 (OPEN-1) D-B6 (OPEN-4) D-B7 (OPEN-3)
  D-B8 (OPEN-2). D-B6/B7/B8 resolved by code-grounded reasoning, then stress-tested
  below.
- **Codex adversarial pass (gpt-5.5, 2026-06-12) — 3 blockers + 4 majors, all
  actioned:**
  - *Finding 1 (blocker)* — D-B7's type-level claim was **false** (the verdict→status
    launder compiles). Reframed twice: first to a behavioural test (still too weak —
    misses non-`Verdict` channels), then to the **layered info-flow** wall — signature
    isolation (`select_status` excludes coverage types) + verdict-consumed-by-prompt +
    a residual-site VT. §5.6, D-B7. (User pressed the point that no type substitution
    can fix an info-flow invariant — drove the signature-boundary framing.)
  - *Finding 2 (blocker)* — discharge predicate let stale RECs excuse live drift.
    Strengthened: latest-REC + `move==accept` + `evidence_ref ⊇ residual keys`. §5.5
    step 3, R-B3.
  - *Finding 3 (blocker)* — gate was opt-in for uncaptured reqs. Added the
    **`reconciled`** term to the gate set. §5.5 step 1, R-B2.
  - *Finding 4 (major)* — gate policy in the evidence tier. **User re-decided**
    (after the pass): home moved `coverage.toml` → **`slice-044.toml`**. D-B5.
  - *Finding 5 (major, moot)* — `CoverageFile` render would drop `[gate]`. Dissolved
    by the finding-4 move (no `[gate]` in coverage.toml).
  - *Finding 6 (major)* — `covered` needed a precise definition + a slice-local
    reader + `key.slice==S` validation. §5.5 step 1, R-B4.
  - *Finding 7 (major)* — scope still carried import-edge language. Reconciled in
    `slice-044.md`.
  - *Finding 8 (minor)* — `doc/spec-entity-spec.md:325` documents an ordered
    lifecycle; D-B6 caveat added (free setter scoped to reconcile).
  - *Not-a-finding:* redesign back-edge logic confirmed sound (slice leaves
    `reconcile`, so the `reconcile→done` gate can't fire — no dead logic).
- External adversarial pass — **DONE** (above). All findings actioned; design
  complete. Remaining before lock: user sign-off on the post-pass deltas, then
  `/plan`.
