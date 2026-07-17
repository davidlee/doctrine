# Design SL-221: Unify dispatch boundary writes on the object-db ref

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->
<!-- Rescoped after RV-278: the localised "merge in commit_boundaries" design
     (git history: design aab267fd) was shown unsound (F-2 tie-break, F-3
     ordering) and is superseded by the seam collapse below (D-B1). -->

## 1. Design Problem

`dispatch sync --prepare-review` can delete phase-boundary rows that
`dispatch_conclude_phase` committed object-db-only, then halt its own completeness
gate on the gap (ISS-225). The root cause is not a bug in one function but **two
boundary writers with two source-of-truth models** joined by a lossy read. Collapse
the seam: one object-db writer, the ref as sole truth, no working-tree boundaries
ledger.

## 2. Current State

`.doctrine/dispatch/<slice>/boundaries.toml` has two writers and one sync reader:

| Path | Site | Writes | Source model |
|---|---|---|---|
| `dispatch_conclude_phase` → `conclude_boundary_commit` | `mcp_server/dispatch.rs:543` | **ref** (object-db RMW + `commit_on_behalf`) | ref is truth |
| `dispatch record-boundary` → `run_record_boundary` | `dispatch.rs:848` | **working tree** (`ledger::record_boundary`) | working tree is truth |
| `dispatch sync --prepare-review` → `commit_boundaries` | `dispatch.rs:2795` | ref (splices the **working-tree** file whole over the tip) | reads working tree |

`conclude_boundary_commit` (SL-199) is the primary writer; `record-boundary` is a
rare manual escape hatch (`dispatch/SKILL.md:97`). `commit_boundaries` reads the
working tree and splices it *whole*, so a stale coord checkout (the normal state
after an object-db conclude) **replaces and deletes** the ref's newer rows — the
ISS-225 sequence.

The object-db commit machinery lives in `mcp_server/dispatch.rs`:
`commit_on_behalf` (280) → `commit_tree_as` (241), keyed by a two-identity
`Provenance` (181: `Import` / `Conclude`) + `dispatch_identity` (372).
`mcp_server/dispatch.rs` already depends on `crate::dispatch::` (one-way; no reverse
import) — so these primitives sit *above* the CLI that also needs them.

**RV-278** (inquisition) closed the localised merge design: a working-tree row
differing from the ref is ambiguous — stale checkout (ref wins) vs escape-hatch
correction (working wins) — indistinguishable by content, so no silent merge rule
is safe (F-2); and merge append-order breaks `plan_phases`'s strict row-order phase
chaining (F-3). Both dissolve once there is only one writer.

## 3. Forces & Constraints

- **Both writers must keep functioning** — conclude (primary) and record-boundary
  (escape hatch) — but through **one** source model.
- **ADR-001 layering** — leaf ← engine ← command, no cycles. `dispatch.rs` (CLI)
  cannot call *up* into `mcp_server`; the shared primitive must relocate *down*.
- **SL-199 working-tree-free conclude** — preserved and generalised (now *all*
  boundary writes are working-tree-free).
- **Behaviour-preservation gate** (AGENTS.md) — the existing `mcp_server` +
  `e2e_dispatch_sync.rs` suites are the proof; relocation + conclude delegation must
  keep them green *unchanged*.
- **SL-064 §4.1 invariant** — sync reads the ledger from the ref; this slice makes it
  hold with no exception (retires the last working-tree reader).
- **STD-001** — no new magic strings; the relocated dispatch id constants
  (`DISPATCH_NAME`/`DISPATCH_EMAIL`) and ref-prefix stay single-sourced.

## 4. Guiding Principles

One writer, one truth. The `dispatch/<slice>` ref is the sole source of committed
boundary rows; nothing reads or writes a working-tree boundaries ledger. The escape
hatch and the funnel converge on the same object-db compose primitive.

## 5. Proposed Design

### 5.1 System Model

Target write/read graph:

```
dispatch_conclude_phase ─┐
                         ├─▶ dispatch::land_boundary_row ─▶ commit_on_behalf ─▶ dispatch/<slice> ref
dispatch record-boundary ┘                                                          │
                                                                                    ▼
                        dispatch sync --prepare-review ─▶ read_ledger (ref) ─▶ plan_phases
```

No node reads or writes the working-tree `boundaries.toml`. `commit_boundaries` and
the whole "splice the uncommitted working ledger" step are gone.

### 5.2 Interfaces & Contracts

**(a) Relocate the object-db compose engine** — move `commit_on_behalf`,
`commit_tree_as`, `Provenance`, `Identity`, `dispatch_identity`, `CommitOutcome`,
`CommitRefusal`, and `DISPATCH_NAME`/`DISPATCH_EMAIL` from `mcp_server/dispatch.rs`
into `dispatch.rs` (engine tier). `mcp_server/dispatch.rs` references them as
`crate::dispatch::…` (the existing one-way edge). Generalise the CAS target:

```rust
// was: derives the branch ref from `coord_root`'s HEAD (couples to running in the coord worktree)
// now: explicit target ref — object-db + ref update work from any worktree sharing the common git dir
pub(crate) fn commit_on_behalf(
    git_root: &Path, target_ref: &str, expected_old: &str,
    tree: &str, message: &str, prov: &Provenance,
) -> anyhow::Result<CommitOutcome>
```

The empty-delta refusal, `commit-tree` object-db-only compose, and CAS lost-ref-race
guard are unchanged; only the target-ref source changes (explicit arg, not HEAD).

**(b) Shared boundary-write helper** in `dispatch.rs`:

```rust
/// UPSERT `row` (by phase) into the committed boundaries at `tip` and land it on
/// `coord_ref` with one working-tree-free commit. The single boundary writer for
/// both the funnel (conclude) and the CLI escape hatch (record-boundary).
pub(crate) fn land_boundary_row(
    git_root: &Path, coord_ref: &str, tip: &str, slice: u32, row: BoundaryRow, prov: &Provenance,
) -> anyhow::Result<CommitOutcome> {
    let path = format!(".doctrine/dispatch/{slice:03}/boundaries.toml");
    let mut b = read_ledger::<Boundaries>(git_root, tip, &format!("{slice:03}"), "boundaries.toml")?;
    match b.rows.iter_mut().find(|r| r.phase == row.phase) {
        Some(existing) => *existing = row,   // UPSERT by phase (funnel + escape hatch alike)
        None => b.rows.push(row),
    }
    let tree = git::tree_with_file(git_root, &tree_of(git_root, tip)?, &path, &b.to_toml()?)?;
    commit_on_behalf(git_root, coord_ref, tip, &tree, &funnel_message(slice, &phase), prov)
}
```

**(c) `conclude_boundary_commit`** (`mcp_server`) → resolves the coord + tip as
today, then delegates to `crate::dispatch::land_boundary_row(coord.root,
&dispatch_ref(slice), coord.tip, slice, row, &Provenance::Conclude{…})`. Behaviour
identical (same UPSERT, same commit).

**(d) `run_record_boundary`** (`dispatch.rs`) → resolves the `dispatch/<slice>` ref
tip (`resolve_commit`) and calls `land_boundary_row(root, &coord_ref, &tip, slice,
row, &prov)` instead of `ledger::record_boundary`. Its second write —
`state::record_source_delta` (the arm-neutral primary-tree registry) — is
**unchanged**.

**(e) `prepare_review`** (`dispatch.rs:1959`) → delete the
`live_worktree_for_ref → commit_boundaries` block; `let tip = tip0;`. `plan_phases`
reads boundaries from the ref via `read_ledger` (already does). Reword the §5.2-step-1
doc-comment.

**(f) Deletions** — `dispatch::commit_boundaries`, `ledger::read_boundaries_file`,
`ledger::record_boundary` (+ their unit tests) once unreferenced.

### 5.3 Data, State & Ownership

`BoundaryRow` / `Boundaries` schemas unchanged. Row order is now **only** ever the
committed ref's order (conclude/record-boundary append in call order); there is no
second source to interleave, so F-3's ordering hazard cannot arise — `plan_phases`
sees exactly the order the single writer produced (as it does today for conclude).
`boundary::Provenance` (`Funnel`/`Manual`) stays a row field; the commit-identity
`Provenance` (`Import`/`Conclude`) is the git author/committer (see OQ-1 for the
escape hatch's identity).

### 5.4 Lifecycle, Operations & Dynamics

- Conclude: unchanged externally (delegates internally).
- Escape hatch: `record-boundary` now lands on the ref in one step (was: write
  working tree, hope prepare-review commits it). A correction UPSERTs the ref row
  directly — no ambiguity, no clobber, no halt.
- prepare-review: reads one committed source; the operator ritual `git restore …
  <ledger paths>` for the boundaries clobber is **obsolete**.

### 5.5 Invariants, Assumptions & Edge Cases

- **One writer, one truth** — no code reads/writes the working-tree boundaries
  ledger after this slice; SL-064 §4.1 holds unconditionally.
- **UPSERT-by-phase preserved** — both writers replace a phase's row in place; a
  re-conclude or a corrected re-record simply overwrites the ref row (the F-2
  ambiguity is gone because there is no competing working-tree copy).
- **`commit_on_behalf` invariants preserved** — empty-delta refusal, lost-ref-race
  CAS, byte-unchanged-on-refusal; the only change is the explicit target ref.
- **(E1)** record-boundary run when no `dispatch/<slice>` ref exists → clean refusal
  (ref unresolved), same failure mode as any funnel verb off a missing coord.
- **SL-199 generalised** — working-tree-free now covers *every* boundary write.

## 6. Open Questions & Unknowns

- **OQ-1** — commit-identity for the escape-hatch ref write: reuse
  `dispatch_identity()` (attributes the correction to "dispatch") or add an
  attributable operator/`Manual` identity? Default: reuse `dispatch_identity()`;
  revisit if attribution matters. Resolve at `/plan`.
- **OQ-2** — remaining callers of `ledger::read_boundaries` (the ref-parsing sibling)
  vs `read_ledger`: audit at `/plan` before deleting anything in `ledger.rs`.
- **OQ-3** — does generalising `commit_on_behalf` to an explicit ref perturb
  `dispatch_import`'s call (also HEAD-based today)? It must pass the coord ref
  explicitly; confirm import stays behaviour-identical.

## 7. Decisions, Rationale & Alternatives

- **D-B1: Collapse the seam — one object-db writer, retire the working-tree ledger.**
  *Chosen.* Kills ISS-225 by construction, dissolves RV-278 F-2 (no competing
  source) and F-3 (single order), retires the last violator of SL-064 §4.1, and net
  *removes* code (the merge/splice/working-tree read/write) rather than adding merge
  logic.
- **D-B1-alt (merge in `commit_boundaries`, ref-base ⊕ working overlay).**
  *Rejected at RV-278.* Unsound tie-break (F-2) and order hazard (F-3); keeps two
  source models alive.
- **D-B1-alt (conclude checkout-syncs the working tree).** *Rejected.* Reintroduces
  the working-tree write SL-199 removed and only closes one writer's gap.
- **D-B2: Relocate primitives down to `dispatch.rs`, not up into `mcp_server`.**
  Forced by ADR-001 (no `dispatch → mcp_server` cycle); also the natural engine home.
- **D-B3: Generalise `commit_on_behalf` to an explicit target ref.** Decouples the
  primitive from "run inside the coord worktree", letting the CLI escape hatch reuse
  it. Alternative (keep HEAD-derivation, run record-boundary in the coord worktree)
  rejected — fragile cwd coupling.

## 8. Risks & Mitigations

- **R1: relocation regresses the funnel.** *Mitigation:* phase the move as a pure
  relocation + delegation with the `mcp_server`/`e2e_dispatch_sync` suites green
  *unchanged* before any behaviour change (behaviour-preservation gate).
- **R2: `commit_on_behalf` contract change breaks import/conclude.** *Mitigation:*
  its provenance/CAS/empty-delta tests move with it and must still pass; import +
  conclude pass the coord ref explicitly (OQ-3).
- **R3: a hidden consumer still reads the working-tree ledger.** *Mitigation:* OQ-2
  audit + compiler (deleting `read_boundaries_file`/`record_boundary` fails to build
  if anything references them).

## 9. Quality Engineering & Validation

- **VT-1 (new, red-first): ISS-225 clobber is impossible.** conclude lands 01–07 on
  the ref, coord working tree is a stale prefix (or absent), `prepare-review` →
  resulting ref boundaries still 01–07. (No `commit_boundaries` to clobber.)
- **VT-2 (new): escape hatch lands on the ref.** `record-boundary PHASE-N` on a live
  coord → the row is present in the committed `dispatch/<slice>` boundaries (not the
  working tree); a corrected re-record UPSERTs it.
- **VT-3 (new): `land_boundary_row` UPSERT-by-phase** — new phase appends; existing
  phase replaces; used identically by conclude and record-boundary (one behaviour).
- **VT-4 (preserved): `commit_on_behalf` primitives** — empty-delta refusal,
  lost-ref-race CAS, byte-unchanged-on-refusal — green after relocation + the
  explicit-ref generalisation.
- **VT-5 (preserved): `e2e_dispatch_sync.rs`** idempotency / stale-ref / refused-row
  green *unchanged* (behaviour-preservation gate).

## 10. Review Notes

- **RV-278** (inquisition, codex/GPT-5.5): F-2 (tie-break, major), F-3 (ordering,
  major), F-4 (verification, minor) against the prior merge design — all accepted;
  resolved by the D-B1 collapse (F-2/F-3 dissolve with the single writer; F-4's gaps
  are covered by VT-1..VT-5). F-1 was a malformed duplicate of F-2. Dispositions on
  the ledger.
