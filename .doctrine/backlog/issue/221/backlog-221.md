# ISS-221: Reservation id scan reads all prefixes → global counter

## Symptom

Per-kind entity id sequences are not independent when the `GitRef` reservation
backend is active. Observed gaps/jumps once a remote became reachable:

- `backlog/risk/`: `001-014`, then `224-228`
- `backlog/issue/`: `001-059`, then `204-220`
- slice numbering gapped (`1-6,10,13,15,19,21,30…`)

Proof of a shared counter: SL-183's audit harvest created `IMP-223` **and**
`RSK-224` in one commit — consecutive ids across two different subtypes.

## Root cause

`src/reserve.rs::remote_reservation_ids` enumerated the WHOLE reservation
namespace (`refs/doctrine/reservation/`) and parsed the trailing `<NNN>` of
every ref, discarding the `<prefix>` segment. So the `GitRef` scan source
unioned ids from every kind, making `next_id` for any kind the GLOBAL max+1.

The claim WRITES per-kind (`refs/doctrine/reservation/{prefix}/{id}`) and
`survey` (the `reservation list` read path) filters by prefix via `held_prefix`
— only the ALLOCATION scan forgot to. `gitref_scan_source` never received the
`prefix` the `GitRef` struct already held.

`LocalFs` (no remote) was unaffected — identity scan over the per-kind tree dir
— which is why early ids are contiguous and the existing per-kind test
(`materialise_fresh_reserves_each_kind_in_its_own_namespace`, LocalFs) passed.

## Fix

Thread `prefix` into `gitref_scan_source` → `remote_reservation_ids`; scope the
enumeration to `refs/doctrine/reservation/{prefix}/` (mirrors the claim refname
and the survey filter). Regression test:
`reserve::tests::gitref_scan_source_is_scoped_to_its_own_prefix`.

## Not in scope

No migration of the existing gappy ids — renumbering breaks references. The
orphaned high-numbered dirs stay as-is (decision: David, no migration).
