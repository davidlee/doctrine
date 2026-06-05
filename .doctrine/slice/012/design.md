# Design SL-012: memory-record symlink tolerance

## 1. Design Problem

`doctrine memory record` cannot run in any repo with a committed symlink — the
born-anchor capture (`src/git.rs::capture` → `reject_unsupported_modes`,
SL-007 D8) hard-errors on mode 120000. The doctrine repo commits symlinks by
design (11 slice `nnn-slug` links + `CLAUDE.md`), so the memory producer is
undogfoodable in its own home. Fix this **without breaking the byte-for-byte
conformance** with forgettable's frame algorithm.

## 2. Current State

`capture()` (`src/git.rs:548`) runs `reject_unsupported_modes` (`:731`)
unconditionally at step 5, **before** the clean/dirty split, scanning the whole
`git ls-files --stage` index. Any 120000 entry → `CaptureError::Symlink`.

This is a verbatim copy of forgettable's gate
(`forgettable/src/git_context.rs:727`, **m12**), same position, same semantics —
confirmed identical this session. The rejection is **upstream conservatism**, not
a doctrine bug.

Consumer wiring already diverges:
- `record` (`src/memory.rs:746`) and `verify` (`:1164`): `capture(&root)?` — hard-fail.
- `retrieve` (`src/retrieve.rs:472`): `capture(root).ok()` — soft-fails to a `None` frame.

## 3. Forces & Constraints

- **Conformance contract (D2/D7, VT-3):** `repo_id`/`checkout_state_id` must stay
  byte-identical to forgettable so frames dedup at the interop seam. The golden
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
- **Single-source-of-truth:** the algorithm is forgettable's; doctrine mirrors it.
  Diverging doctrine's *acceptance domain* unilaterally would fork the mirror.

## 4. Guiding Principles

Fix at the source of truth, not in the mirror. doctrine's frame algorithm is a
byte-for-byte reproduction of forgettable's; the symlink-tolerance decision
belongs **in forgettable**, then propagates here as a re-sync — the DEC-005-D
coordination pattern, not a local fork.

## 5. Proposed Design

### 5.1 System Model

**Decision: direction 3 — upstream-first (D1).** Sequence:

1. **forgettable un-defers m12.** Upstream narrows `reject_unsupported_modes` (or
   defines a conformant symlink encoding) so a symlink-bearing tree captures a
   frame instead of erroring. Expected lighter form: *narrow the gate* to reject
   only symlinks participating in the dirty fingerprint (diff ∪ untracked),
   leaving `checkout_state_id` composition unchanged → no normalizer version bump,
   existing frames/vectors stay valid.
2. **doctrine re-syncs.** Mirror forgettable's updated gate byte-for-byte and
   re-sync the conformance golden vector (VT-3) to forgettable's new reference
   values, adding a symlink-bearing fixture.

This slice is **blocked** until step 1 lands.

### 5.2 Interfaces & Contracts

No public-surface change anticipated. `capture()` keeps its signature; the body's
`reject_unsupported_modes` is replaced with whatever forgettable adopts. Consumer
call-sites (`record`/`verify` `?`) are left as hard-fail **iff** capture no longer
errors on the doctrine repo's (tracked-unchanged) symlinks — which the narrowed
gate guarantees. (If upstream instead keeps erroring on some dirty cases, revisit
soft-failing `record`/`verify` to match `retrieve` — deferred sub-decision.)

### 5.3 Data, State & Ownership

Frame schema unchanged in the expected (gate-narrowing) form. If forgettable
changes the hash composition (true symlink content encoding), a normalizer bump
(`forget.checkout.v2`) and a wider re-sync follow — flagged as the heavier branch.

### 5.4 Lifecycle, Operations & Dynamics

Blocked → (forgettable m12 lands) → re-sync algorithm + golden vector → verify
`memory record` succeeds in the doctrine repo → unblock the `doc/memories` port.

### 5.5 Invariants, Assumptions & Edge Cases

- **VT-3 stays green unchanged** for symlink-free trees (the existing fixture).
- Submodule (160000) / multi-root rejection unchanged.
- Assumption: forgettable will choose the gate-narrowing form (no hash change).
  Verify against forgettable's actual landed change before re-sync.

## 6. Open Questions & Unknowns

- **Q1 — forgettable's chosen form:** narrow-the-gate (no hash change) vs
  define-symlink-encoding (hash change + version bump)? Determines re-sync size.
- **Q2 — does anyone drive the forgettable change**, and on what timeline? (This
  session's user owns the call.)
- **Q3 — `record`/`verify` soft-fail:** still wanted as defence-in-depth even
  after the gate narrows, for *other* symlink-changing repos? Or leave hard-fail
  and let those genuinely error (mirroring forgettable)?

## 7. Decisions, Rationale & Alternatives

- **D1 — fix upstream first, then re-sync (CHOSEN).** Keeps doctrine a faithful
  mirror; avoids forking the conformance contract. Cost: blocked on forgettable.
- **Alt A — record-level soft-fail only.** Untouched seam; `None` anchor in the
  doctrine repo. Rejected: discards a valid clean anchor; entrenches the
  retrieve/record inconsistency.
- **Alt B — doctrine-local seam narrowing (move/narrow the gate here).** Unblocks
  immediately but unilaterally widens doctrine's acceptance domain beyond
  forgettable's — a mirror fork the user explicitly declined.

## 8. Risks & Mitigations

- **R1 — indefinite block.** forgettable may not prioritise m12. *Mitigation:*
  `doc/memories` stays the interim store; re-port is the gated follow-up, not lost.
- **R2 — re-sync drift.** Mirroring upstream by eye risks a byte mismatch.
  *Mitigation:* the shared golden vector is the gate — re-sync is done iff it
  matches forgettable's reference bytes.

## 9. Quality Engineering & Validation

- Conformance golden vector (VT-3) re-synced to forgettable's new reference,
  including a symlink-bearing fixture; passes byte-identical.
- `doctrine memory record` succeeds in the doctrine repo (symlinks present).
- `just check` green.

## 10. Review Notes

(pending adversarial pass — deferred while blocked on upstream)
