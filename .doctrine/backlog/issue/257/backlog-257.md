# ISS-257: Memory staleness checks render an undeterminable stamp as clean

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Raised as **RV-313 F-2** during the SL-230 reconciliation audit. Pre-existing —
**not** an SL-230 regression; SL-230 PHASE-06's Check 4 inherits it from the
incumbent seam.

## The defect is at the call sites, not in the guard

`git::commits_touching` (`src/git.rs:2493`) opens with an ancestry guard:

```rust
let ancestry = run_git(root, &["merge-base", "--is-ancestor", since, target]).ok()?;
if !ancestry.status.success() {
    return None;
}
```

That guard is **correct and must stay**. `since..target` is a set difference, so a
non-ancestor `since` silently *over*-counts; returning `None` is the documented
no-over-trust posture (comment F2).

The defect is how both callers consume it. `memory_health_findings`
(`src/memory.rs`) binds with a let-chain in Check 2 (`:3521`) and Check 4
(`:3568`):

```rust
if !memory.anchor.verified_sha.is_empty()
    && let Some(commits_behind) = crate::git::commits_touching(...)
    && commits_behind > 0
{ findings.push(...) }
```

`None` falls out of the chain and emits **no finding**. So the check has three
real outcomes — *drift* / *no drift* / *cannot determine* — collapsed into two,
and an undeterminable row is reported identically to a verified-clean one.

## Measured reach

Independently measured at audit on the primary tree (HEAD `46c4eac83`):

| | count |
|---|---|
| item memories | 734 |
| anchored (`verified_sha` non-empty) | 115 |
| reachable (stamp is an ancestor of HEAD) | 48 |
| **NON-ancestor — silently exempt from BOTH checks** | **67** |

**~58% of the anchored corpus is exempt; reach is capped at 41.7%.** Stamps go
non-ancestor the ordinary way — verified on a branch or a linked worktree whose
commits never landed on the measuring tree's HEAD — so this is the common case in
a dispatch-heavy repo, not a corner.

No criterion in SL-230 states a reach limit, while design § 1's closure promise
reads corpus-wide. That gap is why this is filed rather than tolerated.

## Suggested direction (not a design)

Make the undeterminable case *visible* rather than removing the guard:

- lift the call sites to a tri-state (drift / clean / undeterminable), and report
  the third as its own finding class — an unverifiable attestation is a real
  corpus-health signal, arguably more actionable than drift;
- keep `commits_touching`'s `None` semantics exactly as they are;
- both Check 2 and Check 4 need it — fixing only Check 4 leaves the older hole.

Worth weighing against a body digest (SL-230 OQ-3, carried to SL-232), which
would answer the staleness question without depending on ancestry at all. If the
digest lands first, this issue narrows to Check 2 only.

## Conformance standing — settled by REV-041

Adjudicated at SL-230's reconcile, via **RV-313 F-6** → **REV-041**. This is a
**conformance fix, not an improvement.**

SPEC-007 § "Git-anchored staleness" was ambiguous about which surfaces its
explicit-state guarantee bound — its Overview tied staleness to the `find` /
`retrieve` reader while `responsibilities[20]` carried it as a separate,
engine-wide responsibility. REV-041 resolved the contradiction by splitting the
sentence along the seam:

- the **five-state render contract** (`fresh` / `stale` / `unknown` / `unanchored`
  / `reference`) binds the `find` / `retrieve` axis, where a state is rendered;
- the **prohibition on silent over-trust is surface-independent** and binds *any*
  git-anchored staleness computation, `memory validate`'s health checks included.

So the behaviour this issue describes — a silent "no drift" on a non-ancestor
anchor, across 67 of 115 anchored memories — is **non-conformant with SPEC-007**,
not merely suboptimal. The remedy is unchanged: the tri-state lift described above.
Note that `validate` emits *findings*, not states, and discharges the guarantee by
emitting a finding rather than by rendering `unknown`.

## Related

- **RV-313 F-2** — the raising finding, with the audit's full evidence.
- **RV-313 F-6 / REV-041** — the SPEC-007 amendment that makes this a conformance
  fix; carries the two-tier contradiction evidence.
- **SL-230** — PHASE-06 shipped Check 4; disclosed the 67-row exemption itself.
- **SL-232** — owns OQ-3 (body digest), the alternative route.
