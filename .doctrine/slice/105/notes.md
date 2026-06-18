# SL-105 Implementation Notes

## Dispatch Summary (2026-06-18)

3 sequential phases landed via `/dispatch` → 3 subprocess workers → funnel.

### Commits on `dispatch/105`

| Commit | Phase | Description |
|--------|-------|-------------|
| `b21b553c` | PHASE-01 | `remove_after` core + IO wrapper, 7 unit tests |
| `fe341bb9` | PHASE-02 | `--remove` flag + `resolve_dep_seq_src_path` refactor, 6 E2E goldens |
| `c030bfe5` | PHASE-03 | `--prune` probe-and-remove loop, 5 E2E goldens |

Review branch: `review/105` (`176e98ad`)

### Verification

- **Unit tests**: 28/28 dep_seq tests green (all phases)
- **E2E goldens**: 14/14 passed (4 original + 10 new)
- **Clippy**: zero warnings (all phases)
- **`just check` (Rust)**: green

### Audit (RV-084)

- All 23 automated criteria verified (VT-1 through VT-5 for each phase, EX-1 through EX-7)
- **VA-1**: PHASE-03 EX-5 — manual prune sweep of dangling `after` edges
  - Resolved at reconcile (see §"Reconcile" below); not the procedure originally
    documented here — see the correction.

### Open

- [ ] Land `review/105` onto `main`

## Reconcile (2026-06-19, RV-084)

### Reconcile write duty: no-op

RV-084 carried a single finding (F-1, `verified`/`acknowledged`) recording VA-1
as pending. No `## Reconciliation Brief` section, no per-slice prose edits, no
governance/spec REVs. The reconcile write duty was a **no-op**; the only gate
was the VA-1 verification.

### VA-1 executed — 14/15 cleared, 1 deliberately retained

Ran `doctrine backlog after <SRC> --prune` from `/workspace/doctrine` with the
`review/105` binary. **14 in-domain item→item edges cleared** (targets
resolved/fixed/done/mitigated — the sound signal SL-105's `--prune` was built
for):

- IMP-021→IMP-028, IMP-042→IMP-023, IMP-042→IMP-008, IMP-047→IMP-033,
  IMP-050→IMP-064, IMP-056→IMP-044, IMP-059→IMP-008, IMP-063→IMP-064,
  IMP-067→IMP-035, IMP-069→IMP-037, IMP-070→IMP-037, IMP-073→IMP-037,
  ISS-013→IMP-008, ISS-003→RSK-001

**1 edge deliberately retained:** `IMP-095 → SL-095` — a valid cross-kind
`after` edge that SL-105's feature cannot clear (see §"Cross-kind shortfall"
and `design.md` §7). Left as a reminder and incentive.

### VA-1 procedure correction

The procedure documented above (and in F-1's response, and the original
handover) was **wrong in two ways**:

1. **SRC direction inverted.** `--prune` operates on the SRC's own `after`-list
   (`run_after` reads `dep_seq::read(item_path)` for SRC and drops edges whose
   `to` target is terminal/absent). The documented SRCs were the *resolved
   targets* (IMP-008, IMP-028, …), not the edge *holders*. The correct SRC is
   the holder of the dangling edge.
2. **ISS-003 omitted.** `ISS-003 → RSK-001` (RSK-001 resolved/mitigated) is the
   15th override; the documented 9-item list never cleared it.

The corrected SRC set is the 14 edge-holders: IMP-021, IMP-042, IMP-047,
IMP-050, IMP-056, IMP-059, IMP-063, IMP-067, IMP-069, IMP-070, IMP-073,
ISS-013, ISS-003 (and IMP-095, whose edge is the retained cross-kind case).

### Cross-kind shortfall (design debt, out of scope for SL-105)

The actionability graph is being extended beyond backlog items to include
slices and other entities. SL-105's `--prune`/`--remove` and the overrides-
adapter handle **item→item** `after` only; cross-kind `after` (e.g.
`IMP-095 → SL-095`) is reported in the `overrides:` footer but cannot be
cleared — `--prune` declines on slice status `done` (∉ {`resolved`,`closed`}),
and `--remove` cannot target it (`parse_ref` rejects the `SL` prefix). Slice
relationships do not carry the same semantics and ordering behaviour as
item→item `after`; extending the verb is a design task, not a mechanical
widening.

**Disposition:** not solved in this slice's lifecycle. Recorded as
**IMP-099** (triaged, `area:backlog, area:relations, quality`) for systematic
treatment in a future slice. The shortfall and the retained edge are noted in
`design.md` §7 (the design-significant surface); this section carries the
reconcile-time detail. Not opened as an RV-084 finding (per reconcile D9 —
audit owns discovery; the shortfall was surfaced to the user and filed as a
backlog item under explicit direction).

### State after reconcile

- 13 backlog TOMLs pruned (the 14 cleared edges; IMP-095's edge retained) —
  uncommitted, to land as `chore(SL-105): prune dangling item→item after edges
  (VA-1)`.
- IMP-099 filed + triaged + linked to SL-105.
- `design.md` §7 + §3.2 caveat added; this section added.
- `review/105` worktree at `.doctrine/review-105` to be removed.
- Handoff to `/close` for landing `review/105` onto `main`.
