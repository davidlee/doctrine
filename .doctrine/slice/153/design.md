# Design SL-153: CLI verbs for spec-internal edges (descends_from, parent, interactions)

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Three spec-internal relation edges have no CLI verb and are authored by hand-
editing TOML — the last hand-edit-only edges in the corpus:

- `descends_from` (SPEC→PRD) — scalar in a tech spec's identity TOML.
- `parent` (SPEC→SPEC, and PRD→PRD per SL-065) — scalar in a spec's identity TOML.
- `interactions` (SPEC→SPEC) — `[[edge]]` rows in a tech spec's `interactions.toml`.

Goal: author and remove all three through `doctrine spec` verbs — edit-preserving,
idempotent, forward-validated — eliminating the hand-edit gap.

## 2. Current State

- `descends_from`/`parent` ship as **commented examples** in `spec-tech.toml`
  (`install/templates/spec-tech.toml`) — legitimately absent at rest, never seeded
  as live keys. `parent` also renders/validates on **product** specs (SL-065:
  `Spec.parent`, `render`, `build_registry` `on_product`), but the product template
  carries no `parent` example and `RELATION_RULES` declares no PRD-parent row.
- `interactions.toml` is a comment-only seed (tech specs only); `[[edge]]` rows
  carry `target` + free-text `type` + optional `notes` (`Interaction` struct).
- Readers exist and stay unchanged: `read_spec`, `read_interactions`,
  `relation_edges`, `interaction_types`, `build_registry`/`validate`.
- `RELATION_RULES` (`src/relation.rs`) declares all three as `Tier::Typed` +
  `LinkPolicy::TypedVerbOnly`: `descends_from` SPEC→Kinds([PRD]), `parent`
  SPEC→Kinds([SPEC]), `interactions` SPEC→Kinds([SPEC]).
- Dispatch for `SpecCommand` lives in **`src/spec.rs`** (the slice scope's
  `src/commands/spec.rs` is stale — that file is empty).

**Available seams (no parallel implementation):**

| Need | Ride |
|---|---|
| scalar set/clear (edit-preserving) | NEW pure core in `dep_seq.rs`, mirroring `apply_status` core/IO split |
| `[[edge]]` append | `spec.rs::append_member` (toml_edit AoT push, tolerant `entry().or_insert`) |
| `[[edge]]` remove | NEW `spec.rs::remove_interaction_edges` (top-level `edge` AoT, canonical-target match, index-collect/reverse-remove). `dep_seq::remove_after` does NOT serve — it is bound to `[relationships].after` inline-tables keyed by `to` and F-1-bails if absent (E2). |
| atomic write | `fsutil::write_atomic` |
| ref → (subtype, id) | `spec.rs::resolve_spec_ref`; canonicalise via `canonicalize_spec_ref` |

## 3. Forces & Constraints

- **ADR-010** — typed-verb tier: these labels are authored by bespoke verbs, NOT
  generic `link`. Non-Goal: no `link` writability (no `RELATION_RULES` change).
- **ADR-001** — leaf ← engine ← command, no cycles. `dep_seq` (leaf) gains the
  scalar seam, imports only `toml_edit`/`anyhow`/`std`; `spec.rs` (command) calls
  down.
- **Pure/imperative split** — no clock/rng/git/disk in the pure layer; pure cores
  take a held `&mut DocumentMut`, the shell does I/O.
- **Behaviour-preservation gate** — existing `spec`/`relation`/`dep_seq` suites are
  the proof for the shared machinery; they stay green unchanged.
- **CHR-019** (closed/done, in-tree evidence, pinned toml_edit 0.22.27): root
  `insert` lands the new key **above** all trailing subtables / `[[relation]]` AoT
  and re-parses correctly. This de-risks creating an absent scalar key even when a
  spec carries `governed_by`/`consumes` `[[relation]]` rows.
- Edit-preserving everywhere; idempotent; no-op holds mtime.

## 4. Guiding Principles

DRY (ride seams, no parallel impl); thin impure shells over pure cores; fail fast
before any write; idempotent and edit-preserving; close the gap rather than half-
close it.

## 5. Proposed Design

### 5.1 System Model

Three verbs added to `doctrine spec`, all in `src/spec.rs`:

```
spec edit <ref>  [--descends-from PRD-NNN | --clear-descends-from]
                 [--parent <ref>          | --clear-parent]
spec interactions add    <SPEC-NNN> <SPEC-NNN> --type <text> [--notes <text>]
spec interactions remove <SPEC-NNN> <SPEC-NNN>
```

New clap shapes: `SpecCommand::Edit{…}` and `SpecCommand::Interactions{ command:
SpecInteractionsCommand::{Add, Remove} }` (mirrors `SpecReqCommand`). The scalar
write-core is the only new code outside `spec.rs`.

### 5.2 Interfaces & Contracts

**`dep_seq.rs` (leaf) — new pure core + IO sibling of `apply_status`:**

```rust
/// Set (Some) or clear (None) one top-level optional scalar, edit-preserving.
/// Unlike `apply_status`'s F-1 refuse, an ABSENT key is CREATED — these fields are
/// legitimately absent (commented in the scaffold). CHR-019 proved root insert
/// lands above trailing [[relation]]/subtables on toml_edit 0.22.27.
/// Returns whether the document changed (no-op guard: equal value / already-absent
/// → false, no mutation).
pub(crate) fn apply_scalar(doc: &mut toml_edit::DocumentMut, key: &str, value: Option<&str>) -> bool
```

`apply_scalar` is a **distinct contract** from `apply_status` in the same module:
`apply_status` F-1-*refuses* an absent seeded key; `apply_scalar` *creates* an
absent optional key (safe per CHR-019). The module doc gains a line distinguishing
the two so the divergence is explicit, not silent.

`spec.rs` shells (thin, impure):

```rust
fn run_edit(path, spec_ref, descends_from: Option<String>, clear_descends_from: bool,
            parent: Option<String>, clear_parent: bool) -> Result<()>
fn run_interaction_add(path, spec_ref, target, kind: String, notes: Option<String>) -> Result<()>
fn run_interaction_remove(path, spec_ref, target) -> Result<()>
```

`run_edit` reads + parses the spec TOML once, validates each requested field, calls
`apply_scalar` per field on the held doc, and `write_atomic`s **once** iff any field
changed (batched multi-field write; no-op holds mtime). `run_interaction_add` rides
`append_member`'s AoT-push shape against `interactions.toml`'s top-level `edge` AoT;
`run_interaction_remove` calls a NEW pure helper `remove_interaction_edges(doc,
canonical_target) -> usize` (index-collect/reverse-remove over the `edge` AoT,
matching `target` canonical-to-canonical, write iff count > 0). `dep_seq::remove_after`
is **not** reused — it targets `[relationships].after` inline-tables keyed by `to`
and bails when the array is absent (E2).

### 5.3 Data, State & Ownership

- `descends_from`/`parent`: top-level scalar string keys in `spec-NNN.toml`. Owned
  by `spec edit`. Stored canonical (`canonicalize_spec_ref`); the no-op guard
  compares **canonical-to-canonical**, so a non-canonical on-disk value (e.g.
  `parent = "SPEC-2"`) is *normalized* to `SPEC-002` on re-set — a deliberate write,
  not a strict no-op. The strict no-op holds when the stored value is already
  canonical and equal.
- `interactions`: `[[edge]]` AoT in `interactions.toml`, each `{ target, type,
  notes? }`. Owned by `spec interactions add|remove`. Target stored canonical.
- **Target-as-PK** for interactions: at most one edge per target (add no-ops if the
  target is present; remove clears by target, matching canonical-to-canonical).
  **Both** add's dup-check and remove **canonicalize the existing on-disk row
  `target` before comparing** (E3) — a hand-authored `target = "SPEC-2"` already in
  the file is matched by `SPEC-002`, so add does not append a duplicate and remove
  clears it. (The reader already canonicalizes on read, `spec.rs:read_interactions`;
  the write path inspects the `toml_edit` row string, so it must canonicalize there
  too.)
  Consistent with the "degenerate dup-target" stance (`spec.rs` §5.5) and
  `interaction_types` last-wins. The add no-op **informs** ("edge to <target>
  already present; remove + add to change its type") — silence would mislead a user
  trying to re-type an edge.
- No storage-shape change; no migration of existing hand-edited data (Non-Goals).

### 5.4 Lifecycle, Operations & Dynamics

Per-verb gates, all evaluated **before any write**:

| Verb | Source gate | Target kind | Target exists | Idempotency |
|---|---|---|---|---|
| `edit --descends-from` | tech only | PRD | yes | re-set same → no-op |
| `edit --parent` | any subtype | == source subtype (tech→SPEC, product→PRD) | yes + **acyclic** (E1) | re-set same → no-op |
| `edit --clear-descends-from` | tech only | — | — | already absent → no-op |
| `edit --clear-parent` | any subtype | — | — | already absent → no-op |
| `interactions add` | tech only | SPEC | yes | target present → no-op |
| `interactions remove` | tech only | — (no validation) | — | no match → no-op (count 0) |

Forward-validation: `resolve_spec_ref(target)` → `(subtype, id)`; kind-check via
`relation::lookup` + `check_target_kind` for the **declared** rows (`descends_from`,
tech `parent`, `interactions` — E4: reuse the `RELATION_RULES` table rather than
re-encode the kinds inline); a narrow product-`parent` branch covers the one
undeclared case (R2 — closes when the follow-up table-honesty work lands). Then
assert the target dir `is_dir()`. `validate_link` itself cannot serve — it is gated
to `LinkPolicy::Writable` and these labels are `TypedVerbOnly` — but `lookup` /
`check_target_kind` are policy-agnostic and do.

**Parent acyclicity (E1) — before any write.** `--parent` additionally rejects a
self-parent (`target == source`) and any cycle the new edge would close. The shell
builds the prospective parent map (existing `parent` scalars across the source's
subtype family + the proposed edge) and walks from the target: if it reaches the
source, reject (`"parent edge SPEC-AAA → SPEC-BBB would form a cycle"`). This
forward-stops a corpus state that `spec validate` already treats as HARD-invalid
(`registry.rs::self_parent`, `parent_cycle`; REQ-087). `spec validate` remains the
backstop for pre-existing drift (D5).

`remove` does **not** validate the target (removing a dangling edge is valid).

Each shell prints a per-action confirmation: `Set SPEC-005 parent to SPEC-002` /
`Cleared SPEC-005 descends_from` / `Added interaction SPEC-005 → SPEC-007 (uses)` /
`Removed N interaction(s) SPEC-005 → SPEC-007`. A no-op prints an unchanged note.

### 5.5 Invariants, Assumptions & Edge Cases

- Every write is a `toml_edit` in-place mutate → `write_atomic`; comments, inert
  tables, `[[relation]]` rows, unknown keys survive verbatim. No serde reserialize.
- No-op → no write (content + mtime hold).
- clap: `--descends-from` ⊥ `--clear-descends-from`; `--parent` ⊥ `--clear-parent`;
  ArgGroup `required=true, multiple=true` (≥1 of the four).
- `--type` is free-text (no enum, per the relation schema).
- `descends_from` set **and** `--clear-descends-from` are both tech-only
  (symmetric); on a product spec → error ("descends_from is tech-only").
- `parent` cross-subtype (e.g. tech `--parent PRD-001`) → error.
- Multiple `edit` fields in one invocation → single read/parse/write-once pass.
- Behaviour-preservation: existing `spec`/`relation`/`dep_seq` suites stay green.

## 6. Open Questions & Unknowns

None open — D1–D6 resolved (§7). One assumption to confirm at implementation:
`interactions remove` returns a count and prints it (mirrors `dep_seq::remove`); the
shell reports "removed N edge(s) to <target>".

## 7. Decisions, Rationale & Alternatives

- **D1 — Subtype-aware `parent` (chosen B over tech-only A).** `spec edit --parent`
  serves both subtypes; target must equal the source subtype. Tech-only would re-
  create the hand-edit gap for product `parent`; subtype-awareness costs one branch.
  `descends_from` stays tech-only. `RELATION_RULES` is left untouched (Non-Goal) —
  the product-parent under-declaration predates this slice (SL-065) and is a
  separate cleanup (§8 follow-up).
- **D2 — Explicit `--clear-*` flags (over set-only / sentinel).** Unambiguous;
  `--<field>` always takes a value; completes the no-hand-edit goal.
- **D3 — Target-as-PK for interactions add/remove.** Remove by target; add no-ops
  on an existing target. One edge per target; re-typing = remove+add in v1. Matches
  the degenerate-dup stance; avoids an inconsistent remove-all vs add-dup split.
- **D4 — `--type` required, `--notes` optional, both free-text.** Matches the
  `Interaction` struct.
- **D5 — Validate kind-shape AND existence** (over shape-only). Same posture as
  `link`; trivial via `resolve_spec_ref` + `is_dir()`. `spec validate` still catches
  later drift.
- **D6 — `spec edit` (flags) + `spec interactions add|remove` (subcommand group)**
  over separate `spec set-descends-from`/`set-parent` verbs. One file pass,
  extensible, cleaner dispatch.

## 8. Risks & Mitigations

- **R1 — Root scalar insert corrupts a trailing `[[relation]]` block.** Mitigated:
  CHR-019 proved insert lands above trailing AoT on the pinned toml_edit; a unit
  test pins the worst-case shape (set into a spec carrying `[[relation]]`).
- **R2 — `RELATION_RULES` honesty drift** (product `parent` authorable but
  undeclared). Accepted for this slice; routed to a follow-up backlog item: a UX
  review of **all** relation-authoring CLI surfaces (consistency + coverage),
  absorbing the PRD-parent row + VT-1 golden fix. Closes the whole gap.
- **R3 — toml_edit version drift** could invalidate R1's premise. Mitigated: the
  worst-case unit test re-parses (not string-match) and fails loudly on regression.

## 9. Quality Engineering & Validation

**Unit — `dep_seq::apply_scalar` (the load-bearing new seam):**
- set creates an absent key, lands **above** a trailing `[[relation]]` block, re-
  parses to the intended structure (CHR-019 worst-case).
- set on a present key updates in place; comment / inert table / `[[relation]]`
  survive.
- set to the current (canonical) value → `false` (no-op); a non-canonical on-disk
  value re-set to its canonical form → `true` (normalized); clear present → `true`;
  clear absent → `false`.

**CLI/integration — `spec edit` + `spec interactions`:**
- `edit --descends-from` on tech → `spec show` reflects it; on product → error.
- `edit --parent` same-subtype OK; cross-subtype → error; nonexistent target →
  error; **self-parent (`SPEC-005 --parent SPEC-005`) → error, no write/mtime
  change** (E1); **2-node cycle (B already parents A; `A --parent B`) → error, no
  write** (E1).
- `--clear-parent` removes; clear-absent no-op; `--descends-from X
  --clear-descends-from` → clap rejects; no flags → ArgGroup rejects.
- multi-field one invocation (`--parent X --clear-descends-from`) → single
  read/parse/**write-once**; an all-no-op invocation holds mtime.
- `interactions add` appends (`--notes` optional); idempotent on target; `remove`
  by target; remove-absent → count 0 no-op; add on product → error; forward-
  validation (existence + kind) for add and edit.
- **Non-canonical existing row (E3):** with `target = "SPEC-2"` already on disk,
  `add SPEC-002` must NOT append a duplicate (canonical dup-match) and `remove
  SPEC-002` must clear that row.
- **`remove_interaction_edges` helper:** removes all canonical-target matches over
  the top-level `edge` AoT, preserves comments/inert rows, writes iff count > 0;
  asserts `dep_seq::remove_after` is untouched (E2).

**Behaviour-preservation:** full `spec`/`relation`/`dep_seq` suites green unchanged.

## 10. Review Notes

### Internal adversarial pass (2026-06-25)

Probed for vagueness, hidden assumptions, weak verification. Findings integrated:

- **A1 — `apply_scalar` contradicts `dep_seq`'s strict-refuse ethos.** Resolved:
  documented as a distinct contract (create vs refuse), grounded in CHR-019; module
  doc to carry the distinction (§5.2).
- **A2 — canonicalization vs no-op.** A non-canonical on-disk value is normalized on
  re-set (a write, not a strict no-op); compare/store canonical; `remove` matches
  canonical-to-canonical (§5.3, §5.4, §9).
- **A3 — silent add-to-existing-target no-op misleads a re-type attempt.** The no-op
  now informs (remove + add to change type) (§5.3).
- **A4 — output unspecified.** Per-action confirmation lines specified (§5.4).
- **A5 — verification gaps.** Added multi-field single-write batching + canonical-
  normalization tests (§9).
- **A6 — `--clear-descends-from` symmetry.** Both set and clear of `descends_from`
  are tech-only (§5.5).

Residual judgment call (no blocker): `apply_scalar` could live in `spec.rs` (YAGNI,
only spec uses it) rather than the `dep_seq` leaf. Kept in `dep_seq` — a top-level
scalar set is kind-neutral and at the leaf's altitude; `append_member`'s AoT shape
is spec-specific, this is not. Open to challenge at external review. **Closed:** the
codex pass was explicitly asked to pressure-test this placement and the
canonicalization/no-op contract (§10 E-pass prompt) and flagged neither — the leaf
altitude and the no-op guard stand.

### External review — codex (GPT-5.5) inquisition (2026-06-25)

Adversarial pass over the locked design + source. Four findings, all verified
against source, all **accepted** (none spurious):

- **E1 — BLOCKER — parent acyclicity gate missing.** `--parent` validated source
  subtype, target subtype, existence — but not self-parent or cycle, which
  `registry.rs::self_parent`/`parent_cycle` already treat as HARD-invalid (REQ-087).
  The typed verb could author a known-invalid corpus state. **Disposition:** accepted.
  Added a pre-write acyclicity gate (§5.4): self-parent + prospective-cycle walk
  before `apply_scalar`; tests for self and 2-node cycle (§9). This was the block.
- **E2 — MAJOR — wrong removal seam.** `dep_seq::remove_after` is bound to
  `[relationships].after` inline-tables keyed by `to` and F-1-bails on absence; it
  cannot serve the top-level `[[edge]]` AoT keyed by `target`. **Disposition:**
  accepted. Design now specifies a new pure `spec.rs::remove_interaction_edges(doc,
  canonical_target) -> usize` (§2 table, §5.2); `remove_after` explicitly not reused.
- **E3 — MAJOR — add dup-check must canonicalize existing on-disk targets.** A
  hand-authored `target = "SPEC-2"` would escape a raw-string dup-check and admit a
  duplicate edge, violating target-as-PK. **Disposition:** accepted. §5.3 now
  requires both add and remove to canonicalize the existing row target before
  comparing; test added (§9).
- **E4 — MINOR — inline kind-validation re-encodes `RELATION_RULES`.** `validate_link`
  can't serve (Writable-gated), but `relation::lookup` + `check_target_kind` are
  policy-agnostic and validate the declared `TypedVerbOnly` rows. **Disposition:**
  accepted. §5.4 reuses `lookup`/`check_target_kind` for declared rows + a narrow
  product-`parent` branch (R2), instead of an inline kind table (DRY).

Verdict from reviewer: "not ready to plan" on E1. E1 now resolved in-design →
**design ready to plan.**
