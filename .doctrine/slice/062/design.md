# Design SL-062: Uniform lifecycle-transition engine + transactional supersession verb

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-062, ADR-009, REQ-…); doc-local refs bare — OQ-1 (§9), D1 (§6), R1 (§8). -->

Scopes IMP-006. Carves the destructive-verb axis (delete/archive) out to a
follow-up (§9). Three locked decisions frame this design: engine altitude **C**
(re-home the FSM beside `conduct.rs` + share the setter core), **gov-first**
supersession, and a **typed-field** write target.

## 1. Design Problem

Two coupled absences:

1. **The edit-preserving authored-TOML status setter is duplicated 4–5×.** The
   identical write-core — read → `parse::<DocumentMut>` → no-op guard → F-1
   malformed refuse → `insert` → single `fs::write` — is hand-copied across
   `governance::set_status` (flat, GovKind-data-driven), `slice::set_slice_status`
   (FSM-gated), `backlog::set_backlog_status` (closed-enum + resolution coupling),
   `spec::set_spec_status`, and the spec-req setter. The
   `mem.pattern.entity.edit-preserving-status-transition` memory documents the
   recipe and names adr+backlog as precedent — the duplication is known and growing.

2. **The lifecycle FSM is trapped in `slice.rs`.** `classify`, the `Transition`
   enum, the terminal/seam predicates (`is_transition_terminal`,
   `crosses_closure_seam`, `is_terminal_status`), `transition_label`, and the edge
   table are real, well-tested, slice-only state-machine code. ADR-009 ("Slice
   lifecycle state machine and conduct axis") has two halves: the **conduct axis**
   (Actor/Autonomy — *who* gates) already lives in `conduct.rs`; the **FSM** (*what*
   transitions) never left `slice.rs`. The pairing is half-homed.

3. **Supersession is half-built and gov-only.** ADR governance has a `superseded`
   terminal status, a typed `[relationships].superseded_by` carve-out (ADR-004 §5 /
   ADR-010 D4 — verb-written, never hand-authored), and a `supersedes` `LifecycleOnly`
   relation slot — but **no transactional verb** writes the three-part move. SL-048
   OD-3 deliberately excluded gov `supersedes` from the `[[relation]]` migration
   "until IMP-006 builds the transactional supersede verb" (`governance.rs:165`). The
   verb is the missing transactional owner.

## 2. Current State (code-verified)

- **Five status setters, one write-core.** `governance::set_status`
  (`src/governance.rs`), `slice::set_slice_status` (`src/slice.rs:~497`),
  `backlog::set_backlog_status`, `spec::set_spec_status` (`src/spec.rs:~711`,
  "Mirrors `adr::set_adr_status`'s parse → mutate → write shape"), spec-req setter
  (`src/spec.rs:~837`). The mechanical body is identical; the variation is the gate
  before the write and the managed key-set.
- **The FSM** lives in `src/slice.rs` (~`580–710`): `classify` (edge-table, NOT
  index arithmetic), `Transition`, `is_transition_terminal` (`{done,abandoned}`
  reopen-refusal), `crosses_closure_seam` (`audit→reconcile`, `reconcile→done`),
  `is_terminal_status` (`{done}`, divergence-only — deliberately distinct from the
  hide-set and the reopen-set; three predicates diverge by design). The slice shell
  `run_status` reads the from-status, calls `classify`, fires the RV close-gate scan
  on the closure seam, resolves the conduct posture, and prints `status_line`.
- **Slice vocab** — `SLICE_STATUSES` const + `SliceStatus` clap `ValueEnum`, pinned
  in lockstep by a drift canary. `SLICE_STATUSES` is also the read-filter authority
  (`validate_statuses`). **No `superseded` state** — slice supersession is unbuilt.
- **Gov supersession storage** — `supersedes` + `superseded_by` are the **typed**
  `[relationships]` fields (`governance.rs:168` `Relationships{supersedes,
  superseded_by, …}`); `related` already migrated to `[[relation]]`, `supersedes`
  excluded (`governance.rs:165`). `supersession_pair` reads them directly from the
  typed table. The `[[relation]]` `Supersedes`/`LifecycleOnly` rule slot
  (`relation.rs:287`) is reserved but **not yet wired into any reader/writer**.
- **No delete/archive verb** exists on any authored entity (only memory carries an
  `Archived` *status*; `corpus.rs` gc-prune + `entity.rs` failure-cleanup are
  internal `remove_dir_all`, not user verbs).

## 3. Forces & Constraints

- **ADR-009** — lifecycle FSM and conduct axis are the two halves of one decision;
  this design completes the pairing by re-homing the FSM beside `conduct.rs`.
- **ADR-004 §5 / ADR-010 D4** — `superseded_by` is the sanctioned reverse carve-out,
  **verb-written, never hand-authored**. The supersession verb is its sole writer.
- **ADR-001** — layering leaf ← engine ← command, no cycles. The new `lifecycle.rs`
  must take primitive inputs (`&Path`, key/value slices, `&str` edges) and depend on
  no kind module; `slice`/`governance`/`backlog`/`spec` are its command-tier consumers.
- **No parallel implementation** (user standard) — the duplicated write-core is
  *lifted* into one unit; the genuinely-different gates stay per-kind (NOT forced
  through one leaky `StatusPolicy` — altitude B rejected, §6 D2).
- **Behaviour-preservation gate** — every existing slice/gov/backlog/spec status
  suite stays green **unchanged**; that is the proof the extraction is honest.
- **Storage rule** — typed data in TOML; the setter preserves inert
  `[relationships]`/`[[relation]]` blocks, comments, unknown keys (never reserialises).
- **SL-060 non-destructive-refuse lesson** — an edit-preserving seam's F-1 refuse
  must never fall back to "regenerate via `<kind> new`" (it would nuke an authored
  entity's content/lifecycle/relations). The hint is kind-appropriate and points at
  the seeded-keys / scaffold, not regeneration.
- **`pure/imperative split`** — date injected by the shell (`clock::today()`), never
  read in the engine.

## 4. Guiding Principles

- Extract the **mechanism** (the byte-duplicated write-core); leave genuinely-
  different **policy** (the gates) where it lives. DRY the copy, do not over-unify.
- Re-home the FSM for **cohesion** (ADR-009's correct address), not speculative
  reuse — slice is its only consumer today; that is acceptable and stated.
- The supersession verb is **kind-agnostic shell over per-kind data** — gov-first is
  a data boundary (`None` for slice), not a hardcode. Growing a kind's vocab flips it
  on with zero shell change.
- Single responsibility: classification **gates**, the setter **writes** — the gate
  leaves the setter.

## 5. Proposed Design

### 5.1 System Model

```
NEW MODULE  src/lifecycle.rs  (beside conduct.rs — the ADR-009 pairing)
  ├─ FSM (moved verbatim from slice.rs):
  │    enum Transition · fn classify(from,to) · is_transition_terminal
  │    crosses_closure_seam · is_terminal_status · transition_label · edge table
  └─ shared write-core:
       fn set_authored_status(path, managed: &[(&str,&str)], hint) -> Result<bool>
       fn supersede_old(path, new_ref, status, today) -> Result<()>   (§5.4)

CONSUMERS (command tier — keep their distinct GATE, delegate the WRITE):
  slice::run_status      gate: lifecycle::classify + RV close-gate → set_authored_status
  governance::set_status gate: none (flat)                         → set_authored_status
  spec / spec-req        gate: none (flat)                         → set_authored_status
  backlog::set_*_status  gate: resolution coupling (validate)      → set_authored_status

NEW VERB  doctrine supersede <NEW> <OLD>   (top-level, sibling of link/needs/after)
  resolve both · assert same-kind · assert SupersedePolicy Some · run transaction
```

### 5.2 Interfaces & Contracts

**`lifecycle::set_authored_status` — the shared write-core (gate-free):**

```rust
/// Edit-preserving authored-TOML status write. NO classification — the caller's
/// shell gates BEFORE this (slice via classify, backlog via resolution coupling).
/// Single responsibility: read → parse → no-op → F-1 refuse → insert → write.
/// `managed`: scalar (key, value) pairs (status, updated, resolution…), all
/// scaffold-seeded. `hint`: kind-appropriate NON-DESTRUCTIVE malformed message.
/// Returns true if a write happened (false = no-op, mtime/content hold).
pub(crate) fn set_authored_status(
    toml_path: &Path,
    managed: &[(&str, &str)],
    hint: &str,
) -> anyhow::Result<bool>;
```

- **No-op guard** (before F-1): all managed values already equal → `Ok(false)`.
- **F-1 refuse**: any managed key absent → `bail!(hint)`. Never `insert` a missing
  scaffold key (a tail-insert lands it inside the trailing subtable = silent
  corruption). `hint` never says "regenerate via new".
- Then `table.insert` each pair, one `fs::write(doc.to_string())`.

**The gate stays in each shell.** Slice example (after extraction):

```rust
// slice::run_status (shell)
let from = read_status(slice_root, id)?;
match lifecycle::classify(&from, to) {            // GATE (was inside the setter)
    Transition::FromTerminal => bail!(…),
    Transition::SeamBreach   => bail!(…),
    _ => {}
}
if lifecycle::crosses_closure_seam(&from, to) { rv_close_gate_scan(…)?; }   // unchanged
lifecycle::set_authored_status(                   // WRITE (gate-free)
    &path, &[("status", to), ("updated", today)], SLICE_MALFORMED_HINT)?;
```

**Per-kind managed key-sets:** slice/gov/spec → `[("status", to), ("updated", today)]`;
backlog → `[("status", to), ("updated", today), ("resolution", res)]` (the resolution
coupling validated in backlog's shell before the call; backlog composes its own
post-transition print string from its own logic, not the setter's return).

**`SupersedePolicy` — per-kind data (the gov-first boundary):**

```rust
struct SupersedePolicy {
    supersedes_field:  &'static str,   // "supersedes"     typed array on NEW
    carveout_field:    &'static str,   // "superseded_by"  typed array on OLD
    superseded_status: &'static str,   // "superseded"     OLD flips to this
}
/// Some(..) for gov kinds; None for slice (no `superseded` state yet — §9 F2).
fn supersede_policy(kind: &entity::Kind) -> Option<SupersedePolicy>;
```

### 5.3 Data, State & Ownership

- **No storage shape change.** The verb writes the **current canonical typed**
  `[relationships].supersedes`/`.superseded_by` fields (Q3 decision (i)); status flips
  via the scalar setter. The `[[relation]]` migration is SL-048 OD-3, downstream (§9 F3).
- **Ownership** — `<NEW>` owns its outbound `supersedes` (ADR-004 outbound-only);
  `superseded_by` on `<OLD>` is the single sanctioned reverse carve-out (ADR-004 §5),
  written only by this verb.
- **Vocab unchanged** — `superseded` already in `ADR_STATUSES`; `SLICE_STATUSES`
  untouched this slice (slice gets no `superseded` state — §9 F2).
- **FSM ownership moves** to `lifecycle.rs`; `SLICE_STATUSES` + `SliceStatus`
  `ValueEnum` stay in `slice.rs` (read-filter + CLI-arg authority — command-tier
  vocab, not FSM logic). The drift canary stays beside the enum.

### 5.4 Lifecycle, Operations & Dynamics — the supersession transaction

`doctrine supersede <NEW> <OLD>` (top-level), gov-first:

1. **Pre-flight, no writes** — resolve `<NEW>` and `<OLD>`; assert both resolve, are
   the **same kind**, and `supersede_policy(kind)` is `Some` (else refuse: slice →
   "supersession not yet supported (follow-up)"; cross-kind → refuse). Read both docs.
   (`resolve-every-ref-before-pure-compare` — shrink the write window.)
2. **Write `<NEW>`** — array-append `<OLD>` to `[relationships].supersedes`
   (idempotent; reuse the typed-append seam). One `fs::write`.
3. **Write `<OLD>`** — `lifecycle::supersede_old`: load `<OLD>`'s `DocumentMut`
   **once**, array-append `<NEW>` to `[relationships].superseded_by` **and** scalar-set
   `status=superseded` + `updated`, one `fs::write`. (Option I — two file writes total,
   one per file; NOT three writes — `<OLD>` is touched once.)

**Atomicity caveat (R1, accepted).** Two files, no FS transaction. Pre-flight removes
every failure except a mid-write I/O error. A partial (`<NEW>` written, `<OLD>` not)
leaves `supersedes` without its reciprocal `superseded_by` — **exactly the state
IMP-032's `validate` cross-check detects**. The verb surfaces a precise error naming
both refs + recovery; re-run is **idempotent** (no-op guard + idempotent array-append).
Documented edge, not a silent hazard.

### 5.5 CLI surface

```
doctrine supersede <NEW> <OLD>      one transaction: NEW supersedes OLD,
                                    OLD.superseded_by += NEW, OLD.status = superseded
```

Top-level sibling of `link`/`needs`/`after`. Gov refs today; slice refs refused with
the gov-first message. (`backlog edit`, `slice status`, `<gov>` status verbs are
unchanged — they delegate their write to the shared setter; their CLI is untouched.)

## 6. Design Decisions

- **D1 — new module `lifecycle.rs` hosts BOTH the FSM and the setter.** They are the
  two halves of "move an authored status": classify (decide) + set (write). Even flat
  kinds use the setter (degenerate classify). Cohesive, beside `conduct.rs`.
- **D2 — altitude C, not B.** Reject a unified `StatusPolicy`-as-data: gov/spec are
  flat (no edges to model — "all legal" is degenerate) and backlog's resolution
  coupling is a second-field invariant, not a transition edge. One type wearing three
  unrelated gate shapes is leaky. Extract the mechanism; keep the policies per-kind.
- **D3 — the gate leaves the setter.** Today `set_slice_status` classifies inside;
  after, the shell classifies then calls the gate-free setter. Single responsibility;
  matches gov's already-gate-free setter.
- **D4 — gov-first as data (`SupersedePolicy: Option`).** Not a hardcode — slice
  returns `None`; the follow-up that grows slice vocab flips it on, zero shell change.
- **D5 — typed-field write target (Q3 (i)).** The verb writes the current canonical
  typed fields; the `[[relation]]` migration is downstream (§9 F3). Transaction shape
  is storage-agnostic — the migration repoints `supersedes_field`, contained.
- **D6 — top-level `supersede` verb.** Uniform across kinds (sibling of `link`),
  matches "ideally uniform across kinds that supersede" (IMP-006); not a per-gov subcmd.

## 7. Verification Alignment

**Behaviour-preservation (the gate) — existing suites green UNCHANGED:** slice FSM
(classify edges, closure-seam, FromTerminal/SeamBreach refuse, RV close-gate),
gov/spec/backlog setter suites. FSM tests move to `lifecycle.rs` or stay in `slice.rs`
driving through the re-export (OQ-1) — either way they assert the same outcomes.

**New — `lifecycle::set_authored_status` (VT, driven through `run()`/shell, NOT the
pure helper — `invariant-test-must-drive-the-write-seam`):**
- no-op guard: all-match → `false`, mtime/content hold.
- F-1 refuse: missing managed key → `bail!`; assert the message does **not** say
  "regenerate via new" (the SL-060 non-destructive lesson, pinned as a test).
- multi-key path (backlog status+updated+resolution) round-trips; `[relationships]`/
  `[[relation]]`/comments preserved.

**New — `supersede` verb (VT):**
- happy path: NEW.supersedes ∋ OLD, OLD.superseded_by ∋ NEW, OLD.status == superseded
  — assert **all three** surfaces (`conformance-asserts-surface-not-just-envelope`).
- idempotent re-run: second `supersede NEW OLD` is a no-op.
- slice refs refused (gov-first message); cross-kind refs refused.
- both blocks + comments preserved.
- partial-write detectability: a hand-induced NEW-without-OLD state is flagged by
  IMP-032's `validate` (links the two slices' invariants).

**Gate:** `just gate` green, zero clippy warnings.

## 8. Risks & Assumptions

- **R1 — supersession non-atomicity** (accepted, §5.4): two-file write, pre-flight +
  idempotent re-run + IMP-032 `validate` detectability mitigate; documented edge.
- **R2 — FSM re-home must stay behaviour-preserving.** One consumer, but the move
  touches a well-tested core. Mitigation: the gate is the existing suite, green
  unchanged; move tests with the code or re-export.
- **A1 — `is_terminal_status` was left at module scope precisely for this reuse**
  (slice.rs comment) — the extraction handhold is in place.
- **A2 — gov is flat any→any** — the supersession status flip needs no FSM gate;
  `superseded` is already in `ADR_STATUSES`. Confirmed code-side.

## 9. Open Questions & Follow-Ups

**Open (carried, non-blocking):**
- **OQ-1** — FSM tests move to `lifecycle.rs` or stay in `slice.rs` (re-export)?
  Decide at execution by smallest green diff.
- **OQ-2** — exact `hint` wording per kind. Trivial; pinned at execution.

**Follow-ups (mint at close):**
- **F1 — destructive verbs (delete/archive).** The carved-out IMP-006 axis (b):
  file-level destruction semantics for committed authored entities (archive-status vs
  git-rm vs tombstone — the R2 of the slice scope); shares the `entity.rs` claim seam.
- **F2 — slice→slice supersession.** Grow slice vocab: add `superseded` to
  `SLICE_STATUSES`+enum+`classify`, add the `superseded_by` carve-out field, flip the
  slice `supersedes` rule to `LifecycleOnly`; `supersede_policy(SLICE)` returns `Some`.
- **F3 — SL-048 OD-3 storage migration.** Gov `supersedes` typed→`[[relation]]`
  (the reserved `LifecycleOnly` slot); repoint the verb's `supersedes_field`.
