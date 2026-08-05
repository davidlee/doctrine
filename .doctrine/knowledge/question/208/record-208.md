# QUE-208: Entity id allocation from inside an execution capsule

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The question

Doctrine entities carry sequentially allocated zero-padded ids — `SL-NNN`, `DEC-NNN`,
`QUE-NNN`, `RV-NNN`. A capsule inhabitant is a full agent: it may phase-plan, navigate
entities, mint knowledge records, and capture memories. Two capsules running in
parallel that each mint an entity of the same kind will allocate the same id, because
neither can see the other and neither can reach canonical state.

How does a capsule allocate a durable entity id?

## Why it is not obvious

Allocation today is `max(local ∪ trunk) + 1` (`entity.rs:215-218`) over a shared
namespace, arbitrated by one of two claim backends: `LocalFs`, where `mkdir` is the
claim (`entity.rs:71`), or `GitRef`, a zero-oid create-CAS against
`refs/doctrine/reservation/*` (`reserve.rs:188-213`). Capsules remove both — there is
no shared filesystem and no reachable remote.

Worse, the current path fails **open** rather than refusing: `trunk_entity_ids`
degrades to an empty vector when the trunk is unreachable, so allocation silently
proceeds from a partial view. That defect is `ISS-319` and is independent of this
question, but it means the naive path does not merely fail — it produces collisions
quietly.

`doctrine reservation` does not help as it stands: it is `list`-only, a survey of held
remote reservation refs, not an allocator.

Governance is silent. ADR-020, SPEC-030, DEC-133 through DEC-137, DEC-153 and REV-046
say nothing about capsule-side id allocation. ADR-006 D3 asserts "minting is a
trunk-side act", and the RFC-025 mechanism census marks reservation refs KEEP
(`mechanism-census.md:83`) — but as "orthogonal to dispatch", a judgement made about
cross-clone collisions, not about capsules.

## Framing: this is a claim-backend question

The three options are usually posed as competing architectures. They are not. A
one-method claim interface already ships:

```rust
pub(crate) trait Claim {                                    // src/entity.rs:51
    fn claim(&self, ctx: &ClaimCtx<'_>) -> anyhow::Result<Acquired>;  // Won | AlreadyHeld
}
```

`reserve.rs:1` states the intent: *"Routing the 11 Fresh call sites through one helper
… is what lets the second backend drop in behind a single signature."* Two options
below are a third implementation of that trait. One is not, and that asymmetry should
drive the answer.

### Option A — uuid until admission

Capsule-minted entities stay uuid-addressed and provisional; the control plane assigns
sequential ids at admission. This is the mechanism memories already use
(`memory.rs:1838`) and observations already use (`observation/store.rs:28-46`).

**Governance appears to refuse it as specified.** REQ-454's third acceptance criterion:
*"The journaled verified identity equals the later admitted identity."* Admission-time
renaming rewrites directory names, TOML `id` fields, symlink slugs, relation targets,
and free-text citations in prose — so the admitted commit is not the verified commit.
No reference-rewriting tooling exists. It becomes viable only if REQ-454 is amended to
separate content identity from id normalisation.

It is also the only option that changes the id **shape**, and therefore the only one
that reaches every citation, slug, relation target, and the reference-form convention
STD-002 and the boot snapshot both encode.

### Option B — pre-allocated blocks

The control plane reserves a range per capsule at provisioning. Preserves the identity
chain and needs no rewriting. Costs: abandoned capsules leak permanent gaps (allocation
never backfills, `entity.rs:208`); block size is unknowable in advance, since a phase
may mint zero entities or twenty; and pre-allocation is new functionality.

### Option C — a doorbell-answering allocator

The capsule requests an id from the control plane and receives a number.

The census's B8 note is more permissive here than a first reading suggests
(`mechanism-census.md:116-127`): *"What genuinely survives is the narrow cross-boundary
interface idea — reborn as **doorbell + harvest**, not as mediated writes."* What B8
deletes is mediated **writes**, not a narrow cross-boundary interface.

The distinction that matters, because governance treats the three oppositely:

| Shape | Direction | Standing |
|---|---|---|
| Doorbell as notification | capsule → control plane, payload-minimal | Settled. SPEC-030: *"Notification is a payload-minimal doorbell, not a verdict or identity source."* |
| Allocator | capsule requests an id, receives a number | Ungoverned. Interprets no capsule-authored document — the request carries a kind, not content. |
| Entity writer | capsule sends entity content, trusted code parses and commits it | This is B8's deleted mediated-write tier and RT-1's hazard shape: every field becomes a trusted-side parser surface, and results split into two streams (source by bundle, entities by RPC) that must be reconciled. |

An allocator is the middle row. Its costs are the ones QUE-207 warned about for a
persistent service — lifecycle, availability, crash recovery, version negotiation —
and DEC-153 deliberately kept the daemon axis separate from binary count, so choosing
one here does not reopen that.

## Deferred, deliberately (2026-08-06)

Parked until needed, not neglected. It does not block `SL-248` (capsule provisioning
and the Linux backend), whose four requirements do not touch entity minting. It becomes
live for the ingestion and conformance slice, and unavoidable by the recovery slice —
see RFC-025 § State of play next-action 2 for the decomposition.

`ISS-319` is separable and can be fixed on its own schedule; it is a defect in the
current allocation path regardless of what this question decides.

## What would settle it

1. Whether v0 permits capsules to mint entities at all. If entity minting stays a
   control-plane act — a capsule returning an intent rather than an entity — the
   question evaporates. ADR-020 says the capsule worker may "edit local Doctrine state
   and create local commits", which suggests it does not, but no requirement was found
   that binds capsule-side entity writes either way. **This is the first thing to
   settle; the rest is downstream of it.**
2. Whether an allocator can be synchronous without a persistent process — e.g. the
   control plane satisfying requests at the same moments it already services the
   doorbell, rather than a daemon that must be up whenever a capsule is running.
3. Measured minting rates from a real capsule phase. Nobody knows whether a phase mints
   zero entities or twenty, and option B's viability turns entirely on that.

## Review ledgers are harder than the rest

`RV-NNN` is the worst case and **no option above fixes it**. Ledgers are turn-based with
mutable dispositions; two capsules disposing findings in separate bundles produce a
merge with no merge logic. Unique ids do not help — the conflict is in the ledger body,
not the identifier. The likely answer is that review creation stays
control-plane-originated, which is consistent with ADR-007 treating adversarial review
as a coordination primitive. Worth settling separately.

## Related

- `ISS-319` — the existing fail-open this question sits on top of.
- `QUE-207` / `DEC-153` — binary topology; the daemon axis was explicitly deferred there.
- `REQ-454` — the verified-equals-admitted criterion that constrains option A.
- SL-248 research round, thread 3 — `.doctrine/slice/248/research/research.md`.
