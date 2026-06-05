# Design SL-012: memory-record symlink tolerance

## 1. Design Problem

`doctrine memory record` cannot run in any repo with a committed symlink — the
born-anchor capture (`src/git.rs::capture` → `reject_unsupported_modes`,
SL-007 D8) hard-errors on mode 120000. The doctrine repo commits symlinks by
design (11 slice `nnn-slug` links + `CLAUDE.md`), so the memory producer is
undogfoodable in its own home. Fix this **without breaking the byte-for-byte
conformance** with the external decision register's frame algorithm.

## 2. Current State

`capture()` (`src/git.rs:548`) runs `reject_unsupported_modes` (`:731`)
unconditionally at step 5, **before** the clean/dirty split, scanning the whole
`git ls-files --stage` index. Any 120000 entry → `CaptureError::Symlink`.

This is a verbatim copy of the external decision register's gate
(`the external decision register/src/git_context.rs:727`, **m12**), same position, same semantics —
confirmed identical this session. The rejection is **upstream conservatism**, not
a doctrine bug.

Consumer wiring already diverges:
- `record` (`src/memory.rs:746`) and `verify` (`:1164`): `capture(&root)?` — hard-fail.
- `retrieve` (`src/retrieve.rs:472`): `capture(root).ok()` — soft-fails to a `None` frame.

## 3. Forces & Constraints

- **Conformance contract (D2/D7, VT-3):** `repo_id`/`checkout_state_id` must stay
  byte-identical to the external decision register so frames dedup at the interop seam. The golden
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
- **Single-source-of-truth:** the algorithm is the external decision register's; doctrine mirrors it.
  Diverging doctrine's *acceptance domain* unilaterally would fork the mirror.

## 4. Guiding Principles

Fix at the source of truth, not in the mirror. doctrine's frame algorithm is a
byte-for-byte reproduction of the external decision register's; the symlink-tolerance decision
belongs **in the external decision register**, then propagates here as a re-sync — the DEC-005-D
coordination pattern, not a local fork.

## 5. Proposed Design

### 5.1 System Model

**Decision: direction 3 — upstream-first (D1).** Sequence:

1. **the external decision register un-defers m12 — LANDED (DE-010).** Confirmed against
   `the external decision register/src/git_context.rs`: `reject_unsupported_modes` → `reject_submodules`
   (only 160000 now; symlinks supported). Untracked symlinks hash by **link text**
   — `sha256(readlink(2) target bytes)`, never following the link (`symlink_target_hash`,
   unix + a non-unix lossy fallback, DEC-010-07). Tracked symlinks ride `index_tree`/
   `worktree_fingerprint` as their 120000 blob. **Normalizer tag unchanged**
   (`forget.checkout.v1`) — regular-entry encoding byte-identical (DEC-010-06), so
   symlink-free csids do not move. The lighter (gate-narrowing) form, as predicted.
2. **doctrine re-syncs — THIS EXECUTION.** Mirror the rename + link-text untracked
   hashing; flip the `symlink_entry_is_rejected` test to the external decision register's FR-001/NF-001
   behavioural set. No literal golden csid exists to re-sync — conformance is
   structural (identical composition + VT-1 remote table copied verbatim).

Plan folded in (mechanical byte-mirror; the existing capture test suite is the
gate). Slice **unblocked → in_progress**.

### 5.2 Interfaces & Contracts

No public-surface change anticipated. `capture()` keeps its signature; the body's
`reject_unsupported_modes` is replaced with whatever the external decision register adopts. Consumer
call-sites (`record`/`verify` `?`) are left as hard-fail **iff** capture no longer
errors on the doctrine repo's (tracked-unchanged) symlinks — which the narrowed
gate guarantees. (If upstream instead keeps erroring on some dirty cases, revisit
soft-failing `record`/`verify` to match `retrieve` — deferred sub-decision.)

### 5.3 Data, State & Ownership

Frame schema unchanged in the expected (gate-narrowing) form. If the external decision register
changes the hash composition (true symlink content encoding), a normalizer bump
(`forget.checkout.v2`) and a wider re-sync follow — flagged as the heavier branch.

### 5.4 Lifecycle, Operations & Dynamics

Blocked → (the external decision register m12 lands) → re-sync algorithm + golden vector → verify
`memory record` succeeds in the doctrine repo → unblock the `doc/memories` port.

### 5.5 Invariants, Assumptions & Edge Cases

- **VT-3 stays green unchanged** for symlink-free trees (the existing fixture).
- Submodule (160000) / multi-root rejection unchanged.
- Assumption: the external decision register will choose the gate-narrowing form (no hash change).
  Verify against the external decision register's actual landed change before re-sync.

## 6. Open Questions & Unknowns

- **Q1 — the external decision register's chosen form? RESOLVED:** narrow-the-gate, no hash change,
  no normalizer bump (DE-010 / DEC-010-06). Light re-sync.
- **Q2 — who drives the external decision register? RESOLVED:** landed upstream (DE-010 phase 1, green).
- **Q3 — `record`/`verify` soft-fail? RESOLVED: leave hard-fail.** With link-text
  untracked hashing, capture no longer errors on any symlink (tracked or untracked);
  the remaining `CaptureError`s (submodule, multi-root, ambiguous-remote, git
  failure) are *genuine* faults that should surface, not be swallowed. `retrieve`'s
  `.ok()` is its own read-path leniency, untouched.

## 7. Decisions, Rationale & Alternatives

- **D1 — fix upstream first, then re-sync (CHOSEN).** Keeps doctrine a faithful
  mirror; avoids forking the conformance contract. Cost: blocked on the external decision register.
- **Alt A — record-level soft-fail only.** Untouched seam; `None` anchor in the
  doctrine repo. Rejected: discards a valid clean anchor; entrenches the
  retrieve/record inconsistency.
- **Alt B — doctrine-local seam narrowing (move/narrow the gate here).** Unblocks
  immediately but unilaterally widens doctrine's acceptance domain beyond
  the external decision register's — a mirror fork the user explicitly declined.

## 8. Risks & Mitigations

- **R1 — indefinite block.** the external decision register may not prioritise m12. *Mitigation:*
  `doc/memories` stays the interim store; re-port is the gated follow-up, not lost.
- **R2 — re-sync drift.** Mirroring upstream by eye risks a byte mismatch.
  *Mitigation:* the shared golden vector is the gate — re-sync is done iff it
  matches the external decision register's reference bytes.

## 9. Quality Engineering & Validation

- Conformance golden vector (VT-3) re-synced to the external decision register's new reference,
  including a symlink-bearing fixture; passes byte-identical.
- `doctrine memory record` succeeds in the doctrine repo (symlinks present).
- `just check` green.

## 10. Review Notes

(pending adversarial pass — deferred while blocked on upstream)
