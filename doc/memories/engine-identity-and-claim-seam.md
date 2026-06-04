# Memory: entity-engine identity + claim seam

Durable cross-cutting facts about `src/entity.rs` that outlive any one slice.
Decided in SL-005 design ([../../.doctrine/slice/005/design.md](../../.doctrine/slice/005/design.md));
**pending implementation** until SL-005 lands. Recorded here so the engine's next
caller inherits them without re-deriving.

## 1. The engine serves two identity shapes (not just numeric)

Until SL-005 the engine was uniformly numeric (`u32` id, `max+1`, `{:03}`,
reservation race-retry). SL-005 generalises it — driven by memory as the first
**string-identity, reservation-free** caller — to serve both shapes through one
materialiser. The three coupled types:

- **`EntityId<'a>`** (render context): `Numbered { id, canonical }` | `Named { name }`.
- **`MaterialiseRequest<'a>`** (runtime placement + payload): `Fresh` |
  `InExisting { id }` | `Named { name }`. This **replaced** the old const
  `Kind.mode` field + `Inputs.existing_id` Option-bag — so an invalid
  placement/payload pair is unrepresentable (SL-005 D8). `materialise` dispatches on
  the request, not on a Kind field.
- **`OwnedEntityId`** (return): `Numbered { id, canonical }` | `Named { name }`,
  inside `Materialised { eid, dir }`, with `numeric_id()` / `canonical_ref()`
  accessors (SL-005 D9). Replaced the old non-optional `Materialised.id: u32`.

**Principle:** generalise only as far as the second identity shape forces — no
speculative identity-strategy framework (reservation-spec § "apparatus arrives with
the caller"). The *next* non-numeric caller (e.g. a content-addressed artefact)
inherits this capability rather than forking a materialiser. A named `Kind` sets
`dir` to the directory that must directly parent its named entities (memory's is
`.doctrine/memory/items`), so `tree_root.join(name)` is the entity dir with no extra
parameter.

## 2. The claim seam is generic, not "reservation" (SL-005 D7)

The atomic-claim trait is being renamed **`Reservation` → `Claim`**, **`acquire` →
`claim`** (variants `Won` / `AlreadyHeld` kept). `mkdir` is still the mechanism;
only the *interpretation* of an existing claim differs per caller:

- numeric callers: `AlreadyHeld` = **lost a race** → recompute id and retry.
- memory (named): `AlreadyHeld` = **duplicate** → hard error, no retry.

Reservation is therefore *one caller's interpretation* of the generic claim, not the
seam's identity. This reconciles the seam's reuse with memory-spec § Identity (:128)
"No reservation namespace, no `acquire` call" — memory takes no reservation namespace
and does not arbitrate; it just claims-or-fails.

## 3. uid is minted-once-and-stored, not content-derived

`memory_uid` (and any future named id) is a client-minted UUID **minted once per
logical entity and stored, never regenerated** (memory-spec § Identity :268-270) —
**not** content-derived. The content-addressed / append-idempotent property belongs
to `event_id` (deterministic `uuid5`), which arrives with the deferred ledger seam.
A stored UUID (memory uses **v7**, time-ordered) is fully spec-compliant. Do not
"fix" the uid to be a content hash; that is the event layer's job.

## Invariants that gate any engine change

- **Numeric callers (slice/design/plan/phases) stay behaviour-preserving.** Their
  suites are the gate; signatures may change mechanically, observable behaviour may
  not.
- **`write_fileset` is the sole path→fs joiner (H1)** and is transactional (tracks
  created paths/dirs, unwinds with `remove_dir` not `remove_dir_all`, H2). Read paths
  that take user input must also go through `fsutil::safe_join` (SL-005 codex-MAJOR-3).
- **Engine is unix-only** (`std::os::unix::fs::symlink`).
