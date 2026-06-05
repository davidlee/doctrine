# Entity-engine identity + claim seam

Durable cross-cutting facts about `src/entity.rs` that outlive any one slice.
Decided in SL-005 design (`.doctrine/slice/005/design.md`); **D1/D7/D8/D9 built in
SL-005 PHASE-01** (commits `ffe18a0` rename + `b58318d` widening) — the numeric
suite passed unchanged (the behaviour gate).

## 1. The engine serves two identity shapes (not just numeric)

Until SL-005 the engine was uniformly numeric (`u32` id, `max+1`, `{:03}`,
reservation race-retry). SL-005 generalises it — driven by memory as the first
**string-identity, reservation-free** caller — to serve both shapes through one
materialiser. Three coupled types:

- **`EntityId<'a>`** (render context): `Numbered { id, canonical }` | `Named { name }`.
- **`MaterialiseRequest<'a>`** (runtime placement + payload): `Fresh` |
  `InExisting { id }` | `Named { name }`. **Replaced** the old const `Kind.mode`
  field + `Inputs.existing_id` Option-bag — so an invalid placement/payload pair is
  unrepresentable (D8). `materialise` dispatches on the request, not a Kind field.
- **`OwnedEntityId`** (return): `Numbered { id, canonical }` | `Named { name }`,
  inside `Materialised { eid, dir }`, with `numeric_id()` / `canonical_ref()`
  accessors (D9). Replaced the old non-optional `Materialised.id: u32`.

**Principle:** generalise only as far as the second identity shape forces — no
speculative identity-strategy framework (reservation-spec § "apparatus arrives with
the caller"). A named `Kind` sets `dir` to the directory that must directly parent
its named entities (memory's is `.doctrine/memory/items`), so `tree_root.join(name)`
is the entity dir with no extra parameter.

## 2. The claim seam is generic, not "reservation" (D7)

The atomic-claim trait was renamed **`Reservation` → `Claim`**, **`acquire` →
`claim`** (variants `Won` / `AlreadyHeld` kept; `LocalFs` unchanged). `mkdir` is
still the mechanism; only the *interpretation* of an existing claim differs:

- numeric callers: `AlreadyHeld` = **lost a race** → recompute id and retry.
- memory (named): `AlreadyHeld` = **duplicate** → hard error, no retry.

Reservation is *one caller's interpretation* of the generic claim, not the seam's
identity. Reconciles seam reuse with memory-spec § Identity: memory takes no
reservation namespace and does not arbitrate; it claims-or-fails.

## 3. uid is minted-once-and-stored, not content-derived

`memory_uid` is a client-minted UUID **minted once per logical entity and stored,
never regenerated** (memory-spec § Identity) — **not** content-derived. The
content-addressed / append-idempotent property belongs to `event_id` (deterministic
`uuid5`), which arrives with the deferred ledger seam. A stored UUID (memory uses
**v7**, time-ordered) is fully spec-compliant. Do not "fix" the uid to a content
hash; that is the event layer's job. See [[mem.fact.backend.forgettable-event-store]].

## Invariants that gate any engine change

- **Numeric callers (slice/design/plan/phases) stay behaviour-preserving.** Their
  suites are the gate; signatures may change mechanically, observable behaviour may not.
- **`write_fileset` is the sole path→fs joiner (H1)** and is transactional (tracks
  created paths/dirs, unwinds with `remove_dir` not `remove_dir_all`, H2). Read paths
  taking user input must also go through `fsutil::safe_join` (SL-005 codex-MAJOR-3).
- **Engine is unix-only** (`std::os::unix::fs::symlink`).
