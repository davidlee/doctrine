# Design SL-012: memory-record symlink tolerance

## 1. Design Problem

`doctrine memory record` cannot run in any repo with a committed symlink — the
born-anchor capture (`src/git.rs::capture` → `reject_unsupported_modes`,
SL-007 D8) hard-errors on mode 120000. The doctrine repo commits symlinks by
design (11 slice `nnn-slug` links + `CLAUDE.md`), so the memory producer is
undogfoodable in its own home. Fix this **without breaking the byte-for-byte
conformance** with the frozen frame algorithm.

## 2. Current State

`capture()` (`src/git.rs:548`) runs `reject_unsupported_modes` (`:731`)
unconditionally at step 5, **before** the clean/dirty split, scanning the whole
`git ls-files --stage` index. Any 120000 entry → `CaptureError::Symlink`.

This gate is part of the frozen-frame contract (the symlink-gate change,
**m12** / DE-010), at a fixed position with fixed semantics —
confirmed this session. The rejection is **contract conservatism**, not
a doctrine bug.

Consumer wiring already diverges:
- `record` (`src/memory.rs:746`) and `verify` (`:1164`): `capture(&root)?` — hard-fail.
- `retrieve` (`src/retrieve.rs:472`): `capture(root).ok()` — soft-fails to a `None` frame.

## 3. Forces & Constraints

- **Conformance contract (D2/D7, VT-3):** `repo_id`/`checkout_state_id` must stay
  byte-stable so frames dedup at the frame seam. The golden
  vector is a *symlink-free* fixture.
- **What is actually deferred is narrow:** only the *worktree* symlink encoding —
  `git diff --binary` of a changed symlink, or `git hash-object` of an *untracked*
  symlink (follows the link, hashes target content → the real non-determinism). A
  **tracked-unchanged** symlink enters only `index_tree` as a stable git blob oid;
  never hashed under the deferred path. (A clean checkout computes no
  `checkout_state_id` at all.)
- **Governance lean:** doctrine prefers soft degradation over hard refusal for
  unstable input (`drift-spec.md:225,286`). But here a *real* anchor is available
  for clean trees — degrading to `None` would discard it.
- **Single-source-of-truth:** the frozen frame algorithm is the contract.
  Diverging the *acceptance domain* ad-hoc would fork the contract.

## 4. Guiding Principles

Fix at the source of truth — the frozen-frame contract itself, not an ad-hoc local
patch. doctrine's frame algorithm is output-conformant (DEC-010-06) to that contract;
the symlink-tolerance decision is a contract revision (the symlink-gate change,
DEC-010), then applied here — the DEC-005-D coordination pattern, not a local fork.
(The revision took the symlink half but deliberately left the IMPR-003 untracked-hash
batching unported — see A-1 in `audit.md` and the drift note in `notes.md`.)

## 5. Proposed Design

### 5.1 System Model

**Decision: direction 3 — contract-first (D1).** Sequence:

1. **The frozen-frame contract un-defers m12 — LANDED (DE-010).** Confirmed against
   the frozen-frame reference: `reject_unsupported_modes` → `reject_submodules`
   (only 160000 now; symlinks supported). Untracked symlinks hash by **link text**
   — `sha256(readlink(2) target bytes)`, never following the link (`symlink_target_hash`,
   unix + a non-unix lossy fallback, DEC-010-07). Tracked symlinks ride `index_tree`/
   `worktree_fingerprint` as their 120000 blob. **Normalizer tag unchanged**
   (`forget.checkout.v1`) — regular-entry encoding byte-identical (DEC-010-06), so
   symlink-free csids do not move. The lighter (gate-narrowing) form, as predicted.
2. **doctrine applies the revision — THIS EXECUTION.** Apply the rename + link-text
   untracked hashing; flip the `symlink_entry_is_rejected` test to the FR-001/NF-001
   behavioural set. No literal golden csid exists to re-sync — conformance is
   structural (identical composition + VT-1 remote table copied verbatim).

Plan folded in (output-conformant (DEC-010-06), not source-identical — IMPR-003
batching deliberately unported, see A-1; the existing capture test suite is the
gate). Slice **unblocked → in_progress**.

### 5.2 Interfaces & Contracts

No public-surface change anticipated. `capture()` keeps its signature; the body's
`reject_unsupported_modes` is replaced per the contract revision. Consumer
call-sites (`record`/`verify` `?`) are left as hard-fail **iff** capture no longer
errors on the doctrine repo's (tracked-unchanged) symlinks — which the narrowed
gate guarantees. (If upstream instead keeps erroring on some dirty cases, revisit
soft-failing `record`/`verify` to match `retrieve` — deferred sub-decision.)

### 5.3 Data, State & Ownership

Frame schema unchanged in the expected (gate-narrowing) form. If the contract
changes the hash composition (true symlink content encoding), a normalizer bump
(`forget.checkout.v2`) and a wider re-sync follow — flagged as the heavier branch.

### 5.4 Lifecycle, Operations & Dynamics

Blocked → (the frozen-frame contract revision lands, the symlink-gate change) → re-sync algorithm + golden vector → verify
`memory record` succeeds in the doctrine repo → unblock the `doc/memories` port.

### 5.5 Invariants, Assumptions & Edge Cases

- **VT-3 stays green unchanged** for symlink-free trees (the existing fixture).
- Submodule (160000) / multi-root rejection unchanged.
- Assumption: the contract revision takes the gate-narrowing form (no hash change).
  Verify against the actual landed change before re-sync.

## 6. Open Questions & Unknowns

- **Q1 — the contract's chosen form? RESOLVED:** narrow-the-gate, no hash change,
  no normalizer bump (DE-010 / DEC-010-06). Light re-sync.
- **Q2 — who drives the contract revision? RESOLVED:** landed (DE-010 phase 1, green).
- **Q3 — `record`/`verify` soft-fail? RESOLVED: leave hard-fail.** With link-text
  untracked hashing, capture no longer errors on any symlink (tracked or untracked);
  the remaining `CaptureError`s (submodule, multi-root, ambiguous-remote, git
  failure) are *genuine* faults that should surface, not be swallowed. `retrieve`'s
  `.ok()` is its own read-path leniency, untouched.

## 7. Decisions, Rationale & Alternatives

- **D1 — revise the frozen-frame contract first, then apply (CHOSEN).** Keeps the
  frame algorithm faithful to the contract; avoids forking the conformance contract.
  Cost: blocked on the contract revision.
- **Alt A — record-level soft-fail only.** Untouched seam; `None` anchor in the
  doctrine repo. Rejected: discards a valid clean anchor; entrenches the
  retrieve/record inconsistency.
- **Alt B — doctrine-local seam narrowing (move/narrow the gate here).** Unblocks
  immediately but ad-hoc widens the acceptance domain beyond the frozen
  contract — a contract fork the user explicitly declined.

## 8. Risks & Mitigations

- **R1 — indefinite block.** the contract revision may not be prioritised. *Mitigation:*
  `doc/memories` stays the interim store; re-port is the gated follow-up, not lost.
- **R2 — re-sync drift.** Applying the revision by eye risks a byte mismatch.
  *Mitigation:* the golden vector is the gate — re-sync is done iff it
  matches the frozen reference bytes.

## 9. Quality Engineering & Validation

- Conformance golden vector (VT-3) re-synced to the frozen new reference,
  including a symlink-bearing fixture; passes byte-identical.
- `doctrine memory record` succeeds in the doctrine repo (symlinks present).
- `just check` green.

## 10. Review Notes

(pending adversarial pass — deferred while blocked on upstream)
