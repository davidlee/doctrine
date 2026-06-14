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

Two homes, split by purity (D1, revised post-review — F-A):

```
NEW MODULE  src/lifecycle.rs  (PURE leaf, beside the pure conduct.rs)
  └─ FSM (moved from slice.rs — the ADR-009 pairing, now TWO pure leaves):
       enum Transition · fn classify(from,to) · is_transition_terminal
       crosses_closure_seam · is_terminal_status · transition_label · edge table
     NO disk/clock — classify takes &str, total over its inputs.

SHARED IO SEAM  the edit-preserving authored-TOML mutators (IO — engine tier).
  Generalize the EXISTING dep_seq::append (SL-060), do NOT re-roll it (no
  parallel implementation). One home for "edit-preserving authored-TOML mutation":
       fn set_authored_status(path, managed: &[(&str,&str)], hint) -> Result<bool>
       fn append_relationship_array(path, field, value) -> Result<bool>   (generalizes
            dep_seq::append from {needs,after} to any seeded [relationships] array)
  Exact module name/placement = execution detail (OQ-3): fold into the dep_seq
  leaf (rename to an authored-TOML-mutation seam) or a sibling leaf; the binding
  constraint is ONE seam, reused.

CONSUMERS (command tier — keep their distinct GATE, delegate the WRITE):
  slice::run_status      gate: lifecycle::classify + RV close-gate → set_authored_status
  governance::set_status gate: none (flat)                         → set_authored_status
  spec / spec-req        gate: none (flat)                         → set_authored_status
  backlog::set_*_status  gate: resolution coupling (validate)      → set_authored_status

NEW VERB  doctrine supersede <NEW> <OLD>   (top-level, sibling of link/needs/after)
  resolve both · same-kind · supersede_policy Some (ADR only today) · not-already-
  superseded guard · run transaction (set_authored_status + append_relationship_array)
```

### 5.2 Interfaces & Contracts

**`set_authored_status` — the shared write-core (gate-free; lives in the IO seam,
NOT in the pure `lifecycle.rs` — F-A):**

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
backlog → `[("status", to), ("updated", today), ("resolution", res)]`. **All coupling
decided in the caller** (F-G): backlog's shell validates the resolution coupling AND
the D9-clear (a back-edge passes `res=""` to clear — `resolution` is scaffold-seeded
`""`, so F-1 passes and the empty value writes), then composes its own post-transition
print string from its own logic — the setter only returns `wrote: bool`.

**`SupersedePolicy` — per-kind data (the ADR-first boundary — F-C):**

```rust
struct SupersedePolicy {
    supersedes_field:  &'static str,   // "supersedes"     typed array on NEW
    carveout_field:    &'static str,   // "superseded_by"  typed array on OLD
    superseded_status: &'static str,   // "superseded"     OLD flips to this
}
/// Some(..) ONLY for ADR today. POL/STD lack a `superseded` status
/// (POLICY_STATUSES/STANDARD_STATUSES verified — §9 F2); slice lacks it too.
/// All three join via the F2 vocab-addition follow-up. ADR seeds both
/// `supersedes=[]` and `superseded_by=[]` (adr.toml verified — F-B).
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

`doctrine supersede <NEW> <OLD>` (top-level), ADR-first:

1. **Pre-flight, no writes** — resolve `<NEW>` and `<OLD>`; assert both resolve, are
   the **same kind**, and `supersede_policy(kind)` is `Some` (else refuse: POL/STD/slice
   → "supersession not yet supported for <kind> (follow-up F2)"; cross-kind → refuse;
   `NEW == OLD` → refuse). **Not-already-superseded guard (F-D):** read `<OLD>` — if its
   status is already `superseded`, allow only the idempotent re-run (`superseded_by ==
   [NEW]` → no-op); a different supersessor (`superseded_by` holds someone else) → refuse
   "<OLD> already superseded by <X>; reopening is deferred" (the FromTerminal analog).
   (`resolve-every-ref-before-pure-compare` — shrink the write window.)
2. **Write `<NEW>`** — `append_relationship_array(NEW_path, "supersedes", OLD)` —
   the generalized dep_seq append (idempotent, F-1-guarded: the array is scaffold-seeded
   `[]`). One `fs::write`.
3. **Write `<OLD>`** — one `DocumentMut` load: `append_relationship_array`-style append
   of `<NEW>` to `superseded_by` **and** `set_authored_status`-style scalar-set
   `status=superseded` + `updated`, one `fs::write`. (Option I — two file writes total,
   one per file; `<OLD>` is touched once. A thin transaction helper composes the two
   mutators over the single loaded doc so the file is written once.)

**Atomicity caveat (R1, accepted).** Two files, no FS transaction. Pre-flight + the
not-already-superseded guard remove every failure except a mid-write I/O error. A
partial (`<NEW>` written, `<OLD>` not) leaves `supersedes` without its reciprocal
`superseded_by` — **exactly the state the SHIPPED SL-048 PHASE-05 `validate` cross-check
detects** (stored `superseded_by` vs the derived `supersedes` in-edge reciprocal;
IMP-032 is that check's correction, not its origin). The verb surfaces a precise error
naming both refs + recovery; re-run is **idempotent** (no-op guard + idempotent
array-append). Documented edge, not a silent hazard.

### 5.5 CLI surface

```
doctrine supersede <NEW> <OLD>      one transaction: NEW supersedes OLD,
                                    OLD.superseded_by += NEW, OLD.status = superseded
```

Top-level sibling of `link`/`needs`/`after`. ADR refs today; POL/STD/slice refs refused
with the ADR-first message. (`backlog edit`, `slice status`, `<gov>` status verbs are
unchanged — they delegate their write to the shared setter; their CLI is untouched.)

## 6. Design Decisions

- **D1 (REVISED post-review, F-A) — split by purity.** `lifecycle.rs` is a **pure
  leaf** hosting ONLY the FSM (classify/Transition/predicates/edge table) — the ADR-009
  pairing becomes two pure leaves beside `conduct.rs`. The edit-preserving IO mutators
  (`set_authored_status` scalar-set + `append_relationship_array`) live in the
  authored-TOML-mutation seam, **generalizing the existing `dep_seq::append`** (SL-060)
  rather than re-rolling it. Rationale: pure/imperative split + ADR-001 — keep IO out of
  the pure FSM module; one home for authored-TOML edit-preserving mutation. (Superseded
  the original "both in lifecycle.rs" cohesion framing.)
- **D2 — altitude C, not B.** Reject a unified `StatusPolicy`-as-data: gov/spec are
  flat (no edges to model — "all legal" is degenerate) and backlog's resolution
  coupling is a second-field invariant, not a transition edge. One type wearing three
  unrelated gate shapes is leaky. Extract the mechanism; keep the policies per-kind.
- **D3 — the gate leaves the setter.** Today `set_slice_status` classifies inside;
  after, the shell classifies then calls the gate-free setter. Single responsibility;
  matches gov's already-gate-free setter.
- **D4 — ADR-first as data (`SupersedePolicy: Option`) — F-C.** Not a hardcode — only
  ADR has a `superseded` status today, so `supersede_policy` returns `Some` for ADR
  only; POL/STD/slice return `None`. The F2 follow-up grows their vocab and flips them
  on with zero shell change.
- **D5 — typed-field write target (Q3 (i)).** The verb writes the current canonical
  typed fields; the `[[relation]]` migration is downstream (§9 F3). Transaction shape
  is storage-agnostic — the migration repoints `supersedes_field`, contained.
- **D6 — top-level `supersede` verb.** Uniform across kinds (sibling of `link`),
  matches "ideally uniform across kinds that supersede" (IMP-006); not a per-gov subcmd.

## 7. Verification Alignment

**Behaviour-preservation (the gate) — existing suites green, ASSERTIONS unchanged**
(import paths update when the FSM moves modules; "behaviour-preserving" is about
outcomes, not byte-identical test text — F-E): slice FSM (classify edges, closure-seam,
FromTerminal/SeamBreach refuse, RV close-gate), gov/spec/backlog setter suites. FSM
tests move to `lifecycle.rs` or stay in `slice.rs` driving through the re-export (OQ-1).

**New — `set_authored_status` (VT, driven through `run()`/shell, NOT the pure
helper — `invariant-test-must-drive-the-write-seam`):**
- no-op guard: all-match → `false`, mtime/content hold.
- F-1 refuse: missing managed key → `bail!`; assert the message does **not** say
  "regenerate via new" (the SL-060 non-destructive lesson, pinned as a test).
- multi-key path (backlog status+updated+resolution) round-trips; `[relationships]`/
  `[[relation]]`/comments preserved.

**New — `supersede` verb (VT), ADR refs:**
- happy path: NEW.supersedes ∋ OLD, OLD.superseded_by ∋ NEW, OLD.status == superseded
  — assert **all three** surfaces (`conformance-asserts-surface-not-just-envelope`).
- idempotent re-run: second `supersede NEW OLD` is a no-op (all three already hold).
- not-already-superseded guard (F-D): `supersede NEW2 OLD` where OLD already superseded
  by NEW1 → refused with the "already superseded by NEW1" message.
- POL/STD/slice refs refused (ADR-first message); cross-kind refs refused; `NEW==OLD`
  refused.
- both blocks + comments preserved.
- partial-write detectability: a hand-induced NEW-without-OLD state is flagged by the
  SHIPPED SL-048 PHASE-05 `validate` cross-check (F-F).

**Gate:** `just gate` green, zero clippy warnings.

## 8. Risks & Assumptions

- **R1 — supersession non-atomicity** (accepted, §5.4): two-file write, pre-flight +
  not-already-superseded guard + idempotent re-run + the SHIPPED SL-048 PHASE-05
  `validate` detectability mitigate; documented edge.
- **R2 — FSM re-home must stay behaviour-preserving.** One consumer, but the move
  touches a well-tested core. Mitigation: existing suites green (assertions unchanged,
  imports update — F-E); move tests with the code or re-export.
- **R3 — `dep_seq::append` generalization must not regress SL-060** (F-A/F-B). Widening
  it from `{needs,after}` to any seeded `[relationships]` array touches SL-060's seam:
  its needs/after suites are the behaviour-preservation proof and stay green unchanged.
- **A1 — `is_terminal_status` was left at module scope precisely for this reuse**
  (slice.rs comment) — the extraction handhold is in place.
- **A2 — ADR is flat any→any** — the supersession status flip needs no FSM gate;
  `superseded` is already in `ADR_STATUSES`. POL/STD/slice vocabs lack it (verified — F-C).

## 9. Open Questions & Follow-Ups

**Open (carried, non-blocking):**
- **OQ-1** — FSM tests move to `lifecycle.rs` or stay in `slice.rs` (re-export)?
  Decide at execution by smallest green diff.
- **OQ-2** — exact `hint` wording per kind. Trivial; pinned at execution.
- **OQ-3 (F-A)** — home + name of the shared IO mutation seam: grow `dep_seq.rs` into
  an authored-TOML-mutation leaf, or a sibling leaf reusing its append core. Binding
  constraint: ONE seam, `dep_seq::append` reused not re-rolled. Decide at execution.

**Follow-ups (mint at close):**
- **F1 — destructive verbs (delete/archive).** The carved-out IMP-006 axis (b):
  file-level destruction semantics for committed authored entities (archive-status vs
  git-rm vs tombstone — the R2 of the slice scope); shares the `entity.rs` claim seam.
- **F2 — supersession for POL / STD / slice.** Each lacks a `superseded` status (F-C).
  Grow vocab: add `superseded` to the kind's status const+enum (+`classify` for slice's
  ordered FSM), ensure a `superseded_by` carve-out field (POL/STD already seed it; slice
  needs it added + flip the slice `supersedes` rule to `LifecycleOnly`); then
  `supersede_policy(KIND)` returns `Some` — zero verb-shell change.
- **F3 — SL-048 OD-3 storage migration.** Gov `supersedes` typed→`[[relation]]`
  (the reserved `LifecycleOnly` slot); repoint the verb's `supersedes_field`.

## 10. Adversarial Review (internal pass — integrated)

Self-hostile pass on the v1 draft; findings integrated above. Codex external pass
pending (handover).

- **F-A (significant, integrated)** — D1 put the IO setter in the pure `lifecycle.rs`,
  violating the pure/imperative split and muddying the ADR-009 pairing. Revised: split
  by purity — FSM → pure `lifecycle.rs`; IO mutators → the authored-TOML seam,
  generalizing `dep_seq::append` (no parallel impl). See D1 (revised), §5.1, OQ-3, R3.
- **F-B (verified, integrated)** — typed-array append was hand-wavy ("reuse the seam")
  and depended on a seeded array. Verified `adr.toml` seeds `supersedes=[]` +
  `superseded_by=[]`; concretized `append_relationship_array` (F-1-guarded). §5.2/§5.4.
- **F-C (significant, integrated)** — "gov-first" was wrong: `POLICY_STATUSES` /
  `STANDARD_STATUSES` lack `superseded`; only ADR has it. Re-scoped to **ADR-first**;
  POL/STD folded into the F2 vocab follow-up. D4, §5.2/§5.4/§5.5, A2.
- **F-D (medium, integrated)** — no guard against re-superseding an already-superseded
  record. Added the not-already-superseded guard (FromTerminal analog) + a VT. §5.4/§7.
- **F-E (minor, integrated)** — "tests unchanged" overclaimed; assertions unchanged but
  import paths shift on the module move. Reworded §7/R2.
- **F-F (minor, integrated)** — detectability cited IMP-032 (open, a correction); the
  cross-check actually SHIPPED in SL-048 PHASE-05. Re-cited. §5.4/§7/R1.
- **F-G (minor, integrated)** — backlog resolution-clear path under the shared setter
  left implicit. Made explicit: all coupling decided in the caller; `res=""` clears. §5.2.
