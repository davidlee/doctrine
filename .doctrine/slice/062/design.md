# Design SL-062: Uniform lifecycle-transition engine + transactional supersession verb

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-062, ADR-009, REQ-…); doc-local refs bare — OQ-1 (§9), D1 (§6), R1 (§8). -->

Scopes IMP-006. Carves the destructive-verb axis (delete/archive) out to a
follow-up (§9). Three locked decisions frame this design: engine altitude **C**
(re-home the FSM beside `conduct.rs` + share the setter core), **gov-first**
supersession, and a **typed-field** write target.

## 1. Design Problem

Two coupled absences:

1. **The edit-preserving authored-TOML status setter is duplicated 4×.** The
   identical write-core — read → `parse::<DocumentMut>` → no-op guard → F-1
   malformed refuse → `insert` → single `fs::write` — is hand-copied across
   `governance::set_status` (flat, GovKind-data-driven), `slice::set_slice_status`
   (FSM-gated), `backlog::set_backlog_status` (closed-enum + resolution coupling),
   and `requirement::set_status` (status-**only**: the requirement entity carries no
   `updated` field — git is the trail — so its managed key-set is a single `status`,
   `requirement.rs:339`). **There is no `spec` status setter** — `spec req status`
   (`spec.rs:837`) is a thin shell that delegates to `requirement::set_status`
   (codex C1; the earlier draft's "`spec::set_spec_status` + spec-req setter" was a
   double-count of one real seam). The
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

- **Four status setters, one write-core** (codex C1 — corrected from "five"):
  `governance::set_status` (`src/governance.rs:372`), `slice::set_slice_status`
  (`src/slice.rs:~497`), `backlog::set_backlog_status`, and `requirement::set_status`
  (`src/requirement.rs:339` — status-**only**, no `updated` stamp). `src/spec.rs:711`
  is `append_member` (NOT a status setter — it mirrors `set_adr_status`'s *shape* in a
  doc comment, which seeded the draft's miscount); `src/spec.rs:837` is the
  `spec req status` shell that delegates to `requirement::set_status`. The mechanical
  body is identical; the variation is the gate before the write and the managed
  key-set (gov/slice → status+updated; backlog → +resolution; requirement → status only).
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
       crosses_closure_seam · edge table
     NO disk/clock — classify takes &str, total over its inputs.
     NOT moved (P3/P4 — not FSM, would taint the kind-agnostic leaf):
       is_terminal_status — stays in slice.rs beside its sole consumer is_divergent
         (divergence semantics, not a transition edge; classify uses
         is_transition_terminal, NOT is_terminal_status).
       transition_label — slice-CLI presentation (feeds status_line); stays in
         slice.rs. The leaf exports Transition (data); each consumer renders its own.

SHARED IO SEAM  the edit-preserving authored-TOML mutators (IO — engine tier).
  Reuse the EXISTING dep_seq::append STRING-ARRAY path (SL-060), do NOT re-roll it
  (no parallel implementation). NB (codex C2): dep_seq::append is axis-SPECIFIC, not
  generic — `needs` dedupes by string membership, `after` by structural {to,rank}
  equality (dep_seq.rs:~91/~145). supersedes/superseded_by are STRING arrays, so the
  reuse target is precisely the `needs` (string-membership) path, parametrized by the
  field name — NOT "any seeded array" (which would wrongly sweep in the {to,rank} and
  any future array-of-tables axis). The seam splits into PURE mutators on a held
  `&mut DocumentMut` + thin read/parse/write wrappers (P1/P2 — one mutation core, no
  parallel implementation, no double-parse):
       // pure cores — mutate a held doc, NO disk:
       fn apply_status(doc: &mut DocumentMut, managed: &[(&str,&str)], hint) -> Result<bool>
            // no-op (all match → false) → F-1 (any key absent → bail!(hint)) → insert each.
       fn apply_string_append(doc: &mut DocumentMut, field, value) -> Result<bool>
            // value domain: one &str; idempotence: string membership (present → false);
            // F-1: the field array must be scaffold-seeded ([]) — absent = bail (never
            // create); the existing needs-path contract, lifted to operate on a held doc.
       // thin single-file IO wrappers (read → parse → core → write-once):
       fn set_authored_status(path, managed, hint) -> Result<bool>   = read+apply_status+write
       fn append_string_array(path, field, value) -> Result<bool>    = read+apply_string_append+write
  Single-mutation callers (slice/gov/backlog/requirement) use the wrappers; the
  supersede verb composes the PURE cores over docs it parsed ONCE and writes ONCE
  (§5.4) — so it shares the mutation logic without a third read/parse/write body
  (kills the `with_authored_doc` second-seam — P1). Exact module name/placement =
  execution detail (OQ-3): fold into the dep_seq leaf (rename to an
  authored-TOML-mutation seam) or a sibling leaf; binding constraint: ONE mutation
  core, the needs-path string-append reused.

CONSUMERS (command tier — keep their distinct GATE, delegate the WRITE):
  slice::run_status        gate: lifecycle::classify + RV close-gate → set_authored_status
  governance::set_status   gate: none (flat)                         → set_authored_status
  requirement::set_status  gate: none (flat, status-ONLY key-set)    → set_authored_status
  backlog::set_*_status    gate: resolution coupling (validate)      → set_authored_status

NEW VERB  doctrine supersede <NEW> <OLD>   (top-level, sibling of link/needs/after)
  resolve both · same-kind · supersede_policy Some (ADR only today) · not-already-
  superseded guard · transaction: parse both docs once, compose the pure cores
  (apply_string_append + apply_status), write each file once
```

### 5.2 Interfaces & Contracts

**`set_authored_status` — the shared single-file write wrapper (gate-free; lives in the
IO seam, NOT in the pure `lifecycle.rs` — F-A). It is a thin read→parse→`apply_status`→
write over the pure `apply_status` core (P1); the supersede verb calls the core directly
on its held docs instead of this wrapper:**

```rust
/// Edit-preserving authored-TOML status write. NO classification — the caller's
/// shell gates BEFORE this (slice via classify, backlog via resolution coupling).
/// Single responsibility: read → parse → apply_status (no-op → F-1 refuse → insert)
/// → write.
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

**Per-kind managed key-sets** (codex C1 — `requirement` is status-only, no `updated`;
there is no `spec` setter): slice/gov → `[("status", to), ("updated", today)]`;
backlog → `[("status", to), ("updated", today), ("resolution", res)]`; requirement →
`[("status", to)]` (single key — the variable-length `managed` slice is exactly what
lets one helper serve both the stamped and unstamped shapes). **All coupling decided
in the caller** (F-G): backlog's shell validates the resolution coupling AND the
D9-clear (a back-edge passes `res=""` to clear — `resolution` is scaffold-seeded `""`
in both backlog templates, so F-1 passes and the empty value writes; reopen clears via
`validate_transition` returning `""` before the no-op/F-1 checks — codex C10 verified),
then composes its own post-transition print string from its own logic — the setter only
returns `wrote: bool`.

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
- **FSM ownership moves** to `lifecycle.rs` (`classify`/`Transition`/
  `is_transition_terminal`/`crosses_closure_seam`/edge table). `SLICE_STATUSES` +
  `SliceStatus` `ValueEnum` stay in `slice.rs` (read-filter + CLI-arg authority —
  command-tier vocab, not FSM logic); the drift canary stays beside the enum.
  `is_terminal_status` and `transition_label` ALSO stay in `slice.rs` (P3/P4 —
  divergence semantics and CLI presentation respectively, not transition logic).

### 5.4 Lifecycle, Operations & Dynamics — the supersession transaction

`doctrine supersede <NEW> <OLD>` (top-level), ADR-first:

The transaction is **parse-once / hold-both / write-once** (P1/P2 — one mutation core,
no double-parse, no second write seam):

1. **Pre-flight — parse BOTH docs, HOLD them (P2/codex C3).** Resolve `<NEW>` and
   `<OLD>`; assert both resolve, are the **same kind**, and `supersede_policy(kind)` is
   `Some` (else refuse: POL/STD/slice → "supersession not yet supported for <kind>
   (follow-up F2)"; cross-kind → refuse; `NEW == OLD` → refuse). Parse both TOML files
   into `DocumentMut` **once** and keep them in scope; verify every key/array the
   transaction will touch is scaffold-present: `NEW.[relationships].supersedes` (seeded
   `[]`), `OLD.[relationships].superseded_by` (seeded `[]`), `OLD.status`, `OLD.updated`.
   A missing key here is a malformed file — bail (F-1, non-destructive hint), **before
   any write**. Because steps 2–3 mutate and write THESE SAME held docs (no re-read),
   the only remaining failure class after a clean pre-flight is a mid-write I/O error —
   there is no later re-parse for F-1 to re-fire at (P2 closes the throwaway-parse hole
   the prior draft left open).
   **Not-already-superseded guard (F-D):** with `<OLD>` parsed — if its status is already
   `superseded`, allow only the idempotent re-run, where no-op requires BOTH
   `OLD.superseded_by == [NEW]` **and** `NEW.supersedes ∋ OLD` (codex C4 — check both
   files, not just OLD). Otherwise refuse: a *different* supersessor (`superseded_by` holds
   someone else `<X>`) → "<OLD> already superseded by <X>; reopening is deferred" (the
   FromTerminal analog); `superseded_by` **empty or malformed** while `status==superseded`
   (a hand-drifted record — no `<X>` to name) → refuse as drift: "<OLD> status is
   superseded but its superseded_by carve-out is empty/inconsistent — run `doctrine
   validate`" (P5 — do not silently self-heal). (`resolve-every-ref-before-pure-compare`
   — shrink the write window.)
2. **Mutate the held docs (no IO)** — `apply_string_append(&mut new_doc, "supersedes",
   OLD)` (membership-idempotent); then on the held `<OLD>` doc, `apply_string_append(&mut
   old_doc, "superseded_by", NEW)` **and** `apply_status(&mut old_doc,
   [("status","superseded"),("updated",today)], hint)`. All three are the PURE cores
   shared with the single-file wrappers — no parallel body, no `with_authored_doc`
   second seam (P1; supersedes the prior draft's separate helper).
3. **Write each file once** — `fs::write(NEW_path, new_doc.to_string())` then
   `fs::write(OLD_path, old_doc.to_string())`. Two writes total, one per file; ordering
   NEW-then-OLD (see caveat). The held docs from step 1 are written directly — no
   re-read, no re-parse (P2).

**Atomicity caveat (R1, accepted).** Two files, no FS transaction. Parse-once/hold-both
+ the pre-flight verify + the not-already-superseded guard reduce the remaining failure
class to a mid-write I/O error (P2/codex C3 — parse/missing-key failures are caught
pre-write on the held docs, and there is no re-parse before the writes for F-1 to
re-fire at). Write ordering is NEW-then-OLD, so any torn state is
`<NEW>.supersedes ∋ OLD` without the reciprocal `<OLD>.superseded_by` (OLD-written
implies NEW-written — this ordering is what makes the F-D no-op check sound, codex C4).
That torn state is **detectable by a subsequent top-level `doctrine validate`** — the
SHIPPED SL-048 PHASE-05 cross-check (`relation_graph::validate_relations`, invoked from
`main.rs`; stored `superseded_by` vs the derived `supersedes` in-edge reciprocal;
IMP-032 is that check's correction, not its origin). It is **not** auto-run by the verb
(codex C5 — recovery contract is "run `doctrine validate`", not silent self-heal). The
verb surfaces a precise error naming both refs + the recovery command; re-run is
**idempotent** (no-op guard + membership append). Documented edge, not a silent hazard.

### 5.5 CLI surface

```
doctrine supersede <NEW> <OLD>      one transaction: NEW supersedes OLD,
                                    OLD.superseded_by += NEW, OLD.status = superseded
```

Top-level sibling of `link`/`needs`/`after`. ADR refs today; POL/STD/slice refs refused
with the ADR-first message. (`backlog edit`, `slice status`, `<gov>` status verbs are
unchanged — they delegate their write to the shared setter; their CLI is untouched.)

## 6. Design Decisions

- **D1 (REVISED post-review, F-A + P1/P2) — split by purity, single mutation core.**
  `lifecycle.rs` is a **pure leaf** hosting ONLY the FSM
  (classify/Transition/`is_transition_terminal`/`crosses_closure_seam`/edge table) — the
  ADR-009 pairing becomes two pure leaves beside `conduct.rs`. The edit-preserving
  mutation lives in the authored-TOML seam as **pure cores on a held `&mut DocumentMut`**
  (`apply_status` + `apply_string_append`, the latter reusing `dep_seq::append`'s
  string-array path — SL-060, not re-rolled) wrapped by thin single-file read/parse/write
  helpers (`set_authored_status`/`append_string_array`). Single-mutation callers use the
  wrappers; the supersession transaction composes the pure cores over docs it parses
  once and writes once (§5.4) — so there is exactly ONE mutation body, no
  `with_authored_doc` second seam (P1) and no throwaway pre-flight parse (P2). Rationale:
  pure/imperative split + ADR-001 + no-parallel-implementation. (Superseded the original
  "both in lifecycle.rs" framing and the interim `with_authored_doc` helper.)
- **D2 — altitude C, not B.** Reject a unified `StatusPolicy`-as-data: gov/requirement
  are flat (no edges to model — "all legal" is degenerate) and backlog's resolution
  coupling is a second-field invariant, not a transition edge. One type wearing three
  unrelated gate shapes is leaky. Extract the mechanism; keep the policies per-kind.
- **D3 — the gate leaves the setter.** Today `set_slice_status` classifies inside;
  after, the shell classifies then calls the gate-free setter. Single responsibility;
  matches gov's already-gate-free setter.
- **D4 — ADR-first via a per-kind policy fn (`supersede_policy → Option`) — F-C.**
  Honest framing (codex C7): this is ADR-only **today**, expressed as a small
  kind-dispatching fn, not as `GovKind` data — `GovKind` carries only
  `kind/stem/statuses/hidden`, no supersession metadata. It returns `Some` for ADR
  (the only kind whose vocab has `superseded`), `None` for POL/STD/slice. It is
  data-shaped (returns a `SupersedePolicy` record the shell consumes blindly) but the
  capability boundary is a hardcoded match, not a field. The F2 follow-up grows the
  deferred kinds' vocab and flips them on with zero verb-shell change; promoting the
  capability into `GovKind` data is an F2 option, noted there.
- **D5 — typed-field write target (Q3 (i)).** The verb writes the current canonical
  typed fields; the `[[relation]]` migration is downstream (§9 F3). Transaction shape
  is storage-agnostic — the migration repoints `supersedes_field`, contained.
- **D6 — top-level `supersede` verb.** Uniform across kinds (sibling of `link`),
  matches "ideally uniform across kinds that supersede" (IMP-006); not a per-gov subcmd.

## 7. Verification Alignment

**Behaviour-preservation (the gate) — existing suites green, ASSERTIONS unchanged**
(import paths update when the FSM moves modules; "behaviour-preserving" is about
outcomes, not byte-identical test text — F-E): slice FSM (classify edges, closure-seam,
FromTerminal/SeamBreach refuse, RV close-gate), gov/**requirement**/backlog setter
suites (codex C1 — `requirement::set_status`, incl. its existing no-`updated` /
status-only round-trip + malformed-refuse tests, stays green unchanged). FSM tests move
to `lifecycle.rs` or stay in `slice.rs` driving through the re-export (OQ-1).
**Footnote (3rd-pass nit):** retiring `requirement`/`gov` onto the shared core changes
their F-1 *message wording* (today "regenerate via the scaffold" / "regenerate via
`<kind> new`" — both violate the SL-060 non-destructive lesson). This is the right moment
to fix them; a malformed-refuse test that asserts the **old** string must update to the
new non-destructive hint (assertion text changes; the refuse *behaviour* is preserved).

**New — `set_authored_status` (VT, driven through `run()`/shell, NOT the pure
helper — `invariant-test-must-drive-the-write-seam`):**
- no-op guard: all-match → `false`, mtime/content hold.
- F-1 refuse: missing managed key → `bail!`; assert the message does **not** say
  "regenerate via new" (the SL-060 non-destructive lesson, pinned as a test).
- multi-key path (backlog status+updated+resolution) round-trips; `[relationships]`/
  `[[relation]]`/comments preserved.
- **status-only path** (requirement, single managed key, no `updated`) round-trips —
  proves the variable-length `managed` slice serves the unstamped shape (codex C1).

**New — `supersede` verb (VT), ADR refs:**
- happy path: NEW.supersedes ∋ OLD, OLD.superseded_by ∋ NEW, OLD.status == superseded
  — assert **all three** surfaces (`conformance-asserts-surface-not-just-envelope`).
- idempotent re-run: second `supersede NEW OLD` is a no-op (all three already hold).
- not-already-superseded guard (F-D): `supersede NEW2 OLD` where OLD already superseded
  by NEW1 → refused with the "already superseded by NEW1" message.
- POL/STD/slice refs refused (ADR-first message); cross-kind refs refused; `NEW==OLD`
  refused.
- **pre-flight malformed refuse (codex C3 / P2)**: an OLD missing a seeded array/key
  (e.g. `superseded_by` absent) is refused in pre-flight with NO write to `<NEW>` —
  assert `<NEW>.supersedes` is untouched (proves parse-and-verify on the held docs
  precedes any write).
- **drift refuse (P5)**: OLD with `status==superseded` but `superseded_by==[]` → refused
  with the "carve-out is empty/inconsistent — run `doctrine validate`" message (no `<X>`
  to name; not silently healed).
- both blocks + comments preserved.
- partial-write detectability: a hand-induced NEW-without-OLD state is flagged by a
  subsequent top-level `doctrine validate` (`relation_graph::validate_relations`), NOT
  by the verb itself (codex C5) — the SHIPPED SL-048 PHASE-05 cross-check (F-F).

**Gate:** `just gate` green, zero clippy warnings.

## 8. Risks & Assumptions

- **R1 — supersession non-atomicity** (accepted, §5.4): two-file write, parse-and-verify
  pre-flight (both docs, all touched keys/arrays — reduces the residual failure class to
  mid-write I/O, codex C3) + not-already-superseded guard (both-files no-op check, codex
  C4) + idempotent re-run + detectability via a subsequent top-level `doctrine validate`
  (NOT verb-auto, codex C5) mitigate; documented edge.
- **R2 — FSM re-home must stay behaviour-preserving.** One consumer, but the move
  touches a well-tested core. 3rd-pass verified the moved set (`classify`,
  `is_transition_terminal`, `crosses_closure_seam`, `Transition`, edge table) is
  genuinely pure `&str`-in (no `SLICE_STATUSES` / slice-type reference) and that
  `run_status`'s gate/RV/drift/conduct ordering survives the split; `is_terminal_status`
  + `transition_label` deliberately stay in `slice.rs` (P3/P4 — not FSM). Mitigation:
  existing suites green (assertions unchanged, imports update — F-E); move tests with the
  code or re-export.
- **R3 — reusing `dep_seq::append`'s string-array path must not regress SL-060**
  (F-A/F-B; refined by codex C2). The reuse is the `needs` (string-membership) path
  parametrized by field name — NOT a widening to "any array" (which would entangle the
  `after` `{to,rank}` struct-eq path). SL-060's needs/after suites are the
  behaviour-preservation proof and stay green unchanged; the new `append_string_array`
  carries its own value-domain/idempotence/F-1 contract (§5.1), not assumed-free.
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
  `superseded_by=[]`; concretized `append_string_array` (F-1-guarded). §5.2/§5.4.
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

### Codex external pass (GPT-5.5, repo-read, integrated)

Hostile pass over design + scope with read-only repo access; claims verified against
source. Dispositions (all accepted unless noted):

- **C1 (significant, ACCEPTED)** — the "five setters" enumeration was factually wrong:
  `spec::set_spec_status` does not exist (`spec.rs:711` = `append_member`; `spec.rs:837`
  = the `spec req status` shell delegating to `requirement::set_status`). Real setters
  are four: gov/slice/backlog + `requirement::set_status` (status-**only**, no `updated`
  — `requirement.rs:339`). USER DECISION: requirement is **in scope** as a consumer (its
  single-key set exercises the helper's variable `managed` generality). Fixed §1, §2,
  §5.1, §5.2, §7.
- **C2 (significant, ACCEPTED)** — "generalize `dep_seq::append` to any seeded array"
  overbroad: append is axis-specific (`needs` string-membership vs `after` `{to,rank}`
  struct-eq, dep_seq.rs ~91/~145). Narrowed to reuse of the string-array (`needs`) path
  parametrized by field, renamed `append_string_array` with an explicit
  value-domain/idempotence/F-1 contract. Fixed §5.1, D1-adjacent, R3.
- **C3 (significant, ACCEPTED)** — R1's "only I/O can fail post-pre-flight" was false;
  parse/missing-seeded-key failures fire at mutation time. USER DECISION: add a **real
  parse-and-verify pre-flight** over both docs (all touched keys/arrays) — now the claim
  holds. Fixed §5.4 step 1, R1, §7 (new pre-flight refuse test).
- **C4 (medium, ACCEPTED)** — idempotent-rerun no-op checked OLD only; the torn state is
  asymmetric. Made the no-op require BOTH `OLD.superseded_by==[NEW]` and
  `NEW.supersedes∋OLD`, and documented that NEW-then-OLD write ordering is what makes the
  one-sided guard sound. §5.4.
- **C5 (medium, ACCEPTED)** — detectability is via a subsequent top-level `doctrine
  validate` (`relation_graph::validate_relations`), NOT verb-auto. Recovery contract
  reworded honestly. §5.4, R1, §7.
- **C6 (medium, ACCEPTED)** — the "thin transaction helper" was design fiction (no
  `DocumentMut`-level composable API exists). Named it as a NEW helper
  `with_authored_doc(path, |doc| …)`. §5.4 step 3.
- **C7 (minor, SOFTENED)** — `SupersedePolicy`-as-`Option` is ADR-only via a kind-match
  fn, not `GovKind` data; D4's "not a hardcode" oversold. Reworded to honest framing;
  GovKind-data promotion noted as an F2 option. D4.
- **C8 (minor, ACCEPTED)** — scope.md "uniform status-transition engine" overstates
  (`run_status` composes classify + RV gate + drift + conduct). Reconciled scope wording
  to "shared pure FSM primitives + shared write seam". slice-062.md.
- **C10 (confirmation, NO CHANGE)** — backlog `resolution=""` clear path verified
  grounded (both templates seed it; reopen clears via `validate_transition` before the
  no-op/F-1 checks). Noted inline §5.2.
- **C9** = the C8 scope-wording point (codex listed it under the design's "uniform"
  claim); same disposition.

### Third pass (Opus sub-agent, repo-read, integrated)

Targeted at the surfaces the first two passes under-attacked (FSM re-home R2) and the
claims THIS integration introduced (pre-flight, `with_authored_doc`, requirement-in-scope).
Found two significant defects — both artifacts of the codex-round integration itself —
plus three medium trims. Dispositions:

- **P1 (significant, ACCEPTED)** — the C6 `with_authored_doc` helper became a SECOND
  write seam: `set_authored_status` owns read→parse→write, but `with_authored_doc` did
  too, hand-rolling the same scalar-insert (parallel-implementation smell) and bypassing
  the no-op/F-1 contract for the OLD-status flip. Fix: split the seam into PURE cores on
  `&mut DocumentMut` (`apply_status`/`apply_string_append`) + thin IO wrappers; the verb
  composes the cores. One mutation body. Supersedes C6. §5.1, §5.4, D1.
- **P2 (significant, ACCEPTED)** — the C3 pre-flight double-parsed: steps 2–3 re-read
  from disk, throwing away the pre-flight's parsed docs, so a concurrent edit could still
  surface F-1 at mutation time → "only I/O can fail" was still overstated (TOCTOU). Fix:
  **parse-once / hold-both `DocumentMut` / write-once** — the writers operate on the held
  docs, no re-parse. Now the claim holds. Collapses with P1 (one parsed doc per file,
  written once). §5.4 steps 1–3, R1.
- **P3 (medium, ACCEPTED)** — `is_terminal_status` is NOT FSM (its only consumer is
  `is_divergent`; `classify` uses `is_transition_terminal`). Keep it in `slice.rs` beside
  `is_divergent`; trim the move set. §5.1, §5.3, R2.
- **P4 (medium, ACCEPTED)** — `transition_label` is slice-CLI presentation (feeds
  `status_line`), not transition logic. Keep in `slice.rs`; the leaf exports `Transition`,
  each consumer renders its own. §5.1, §5.3.
- **P5 (medium, ACCEPTED)** — the F-D guard had no specified behaviour for the
  `status==superseded` but `superseded_by==[]`/malformed drift case (no `<X>` to name).
  Specified: refuse as drift, point at `doctrine validate`, do not self-heal. §5.4, §7.
- **Nit (ACCEPTED)** — `requirement`/`gov` existing F-1 messages ("regenerate via …")
  violate the SL-060 non-destructive lesson; retiring them onto the shared core is the
  moment to reword (assertion text changes, refuse behaviour preserved). §7 footnote.
- **CLEARED by the 3rd pass (verified against source, no change needed):** FSM purity of
  the moved set; `run_status` gate/RV/drift/conduct ordering survives the split; ADR-001
  no-cycle for `lifecycle.rs`; `append_string_array` ≈ dep_seq needs-path (real refactor,
  correctly deferred to OQ-3); requirement-on-shared-helper has zero behaviour drift;
  supersession detectability end-to-end (`supersession_pair` reads the typed fields the
  verb writes → `validate_supersession`); torn-state re-run convergence; ADR-first vocab
  + template seeds. adr/pol/std already share `governance::set_status` (the "four setters"
  count is exact — no further gov-kind duplication to collapse).
