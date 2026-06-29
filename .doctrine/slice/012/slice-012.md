# memory-record symlink tolerance

## Context

`doctrine memory record` captures a **born git anchor** through the impure git
seam (`src/git.rs`). Before hashing the frame, `reject_unsupported_modes`
(`src/git.rs:731`) scans the *entire* staged tree (`git ls-files --stage`) and
hard-errors on any submodule (160000) or symlink (120000) entry —
`CaptureError::Symlink`, surfaced to the user as
`Error: unsupported: symlink entry (mode 120000)`.

This gate is deliberate (SL-007 **D8**, `slice/007/design.md:341`): the anchor's
`checkout_state_id` implements the frozen `forget.checkout.v1` normalizer
**byte-for-byte**, proven by a conformance golden vector (SL-007 **VT-3**).
Symlink hashing is **deferred in the frozen contract itself**, so the frame normalizer
has no defined symlink encoding — rather than emit an unstable/divergent anchor,
SL-007 chose to refuse the whole frame.

**The contradiction:** doctrine's own repository commits symlinks *by design* —
every slice `nnn-slug` symlink and `CLAUDE.md` itself (`git ls-files -s | awk
'$1==120000'` → 12 entries). The whole-tree scan means a single committed
symlink anywhere poisons capture, so **`memory record` cannot run in the
doctrine repo at all**, and breaks in any embedding repo that commits one
symlink. The memory producer (SL-005/SL-007) is undogfoodable in its own home.

Discovered while porting the four `doc/memories/*` notes into the real memory
store (`memory record`) — every record call failed on this gate.

## Scope & Objectives

Make `memory record` succeed in a repo containing committed symlinks, **without
breaking the frozen-frame conformance contract** (SL-007 VT-3 golden vector must
stay byte-identical for symlink-free trees).

**DECIDED (design D1): direction 3 — contract-first.** The frozen-frame contract
un-defers m12 (the symlink-gate change); doctrine
then applies the revised gate + the VT-3 golden vector. **Slice is `blocked`
on the contract revision.** Directions 1 (record soft-fail) and 2 (doctrine-local
seam narrowing) were rejected: both fork the byte-for-byte contract ad-hoc.
See `design.md` §7.

Candidate directions (decided — direction 3):

1. **Soft-fail the frame** — a capture that hits an unsupported mode degrades to
   a `None`/partial anchor at the `record` layer instead of aborting the command.
   Memory records; born anchor is absent or reduced. Cheapest; loses anchor
   fidelity for any symlink-bearing repo.
2. **Partial frame** — split the anchor: keep `repo_id` (`forget.remote.v1`,
   touches only remotes, symlink-agnostic) and the commit-reachability anchor;
   omit only `checkout_state_id` (`forget.checkout.v1`, the tree-hashing half)
   when the tree carries a symlink. Preserves staleness-by-commit; drops
   worktree-fingerprint precision in symlink repos.
3. **Un-defer symlink encoding** — define a conformant symlink hashing in the frozen
   `forget.checkout.v1` contract, then implement it byte-for-byte
   (the DEC-005-D coordination pattern). Highest fidelity, highest cost, blocks
   on a contract revision — likely a separate future slice, not this one.

Design must decide which, and what `verify` / staleness do when the anchor is
partial or absent.

## Non-Goals

- **No locally-invented symlink hash.** Hashing the symlink target text *only in
  doctrine* would diverge from `forget.checkout.v1` and break the conformance
  golden vector. Out of scope unless the frozen contract un-defers (direction 3).
- **No change to submodule (160000) or multi-root rejection** — those stay hard
  errors (genuinely unstable to anchor; D8 unchanged for them).
- **Not removing the committed symlinks** from the doctrine repo — they are
  load-bearing (slug navigation, `CLAUDE.md`). The fix is in the seam, not the
  tree.
- **No re-port of `doc/memories` in this slice** — porting resumes once `record`
  works; tracked as a follow-up.

## Affected Surface

- `src/git.rs` — `reject_unsupported_modes` (:731), `CaptureError::Symlink`
  (:481), and the frame-capture entry point that composes `repo_id` +
  `checkout_state_id`.
- The `memory record` layer that calls the seam and decides hard-error vs
  degraded anchor (PHASE-04 producer wiring, SL-007).
- `memory verify` / staleness ranking (`src/git.rs:768` reachability) — behaviour
  when the stored anchor is partial/absent.
- Tests: the SL-007 conformance golden vector (must stay green unchanged) + new
  symlink-tree fixtures.

## Risks / Assumptions / Open Questions

- **R1 — conformance regression.** Any change near the frame normalizer risks the
  byte-identity vector. *Mitigation:* the existing VT-3 golden vector is the gate;
  it must pass unchanged for symlink-free trees.
- **R2 — silent fidelity loss.** A soft/partial anchor must be *visible* (stored
  marker, surfaced in `show`/`verify`) — never a silent downgrade that reads as a
  full anchor. Open: how is a degraded anchor represented and attested?
- **Q1 — does `verify` refuse, warn, or pass** against a partial anchor?
- **Q2 — is the symlink gate even semantically right for `repo_id`?** `repo_id`
  (`forget.remote.v1`) never touches the index tree; only `checkout_state_id`
  does. The whole-tree scan may be over-broad relative to what actually hashes
  the tree.
- **A1 —** the frozen `forget.checkout.v1` contract still defines no symlink encoding
  (assumed; verify against the frozen-frame reference before direction 3).

## Verification / Closure Intent

- `doctrine memory record` succeeds in the doctrine repo (symlinks present) and
  scaffolds a well-formed item.
- SL-007 conformance golden vector passes **unchanged** (byte-identity preserved).
- A degraded/partial anchor (if chosen) is represented explicitly and handled by
  `verify` per the design decision; covered by a symlink-tree fixture test.
- `just check` green.
- The four `doc/memories/*` notes can then be ported (follow-up slice/task).

## Follow-Ups

- Port `doc/memories/{customizable-governance-surface, doctrine-just-module,
  engine-identity-and-claim-seam, backend-contract}` into
  the memory store once `record` works.
- Possible follow-up slice: un-defer symlink encoding in the frozen
  `forget.checkout.v1` contract (direction 3) for full anchor fidelity.
