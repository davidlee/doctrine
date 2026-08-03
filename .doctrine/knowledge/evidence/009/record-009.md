# EVD-009: Wall-clock cost is equal between mechanisms; bundle's larger transfer trips git auto-maintenance

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Datum — cost is not a discriminator

Measured on the heavy fixture (a ~169 MB clone of the doctrine repository),
F-P05-10:

| heavy cell | setup | capsule | run | TOTAL |
|---|---|---|---|---|
| happy path, `verify: true`, **M-A** (fetch) | 0.40s | 0.83s | 1.3s | **~2.5s** |
| happy path, `verify: true`, **M-B** (bundle) | 0.40s | 0.69s | 1.47s | **2.55s** |

A 2% difference on a 2.5s cell. The prior fear — that a heavy cell would cost
minutes and force the heavy column out of the matrix — was wrong for a reason
worth keeping: `git clone --no-hardlinks` of the 169 MB fixture measures
**0.2s**, because the filesystem reflinks it. Three clones per cell is not the
cost driver anyone expected.

(Separately: a *full* heavy verify end-to-end, running the real suite rather
than `verify: true`, is 376–413s. That cost is the client project's test suite,
identical under either mechanism.)

## Datum — the asymmetry is fragility, not speed

M-B's harvest moves a **27 MB pack** into a fresh quarantine object store. Across
H15's three killed harvests that is enough to trip **git's own auto-maintenance
inside the quarantine**, which repacks and prunes unreachable objects. The heavy
fixture carries a six-file **incremental commit-graph chain** inherited from the
real repository it was cloned from, and that chain names commits unreachable in
the clone. After the prune, `git fsck` reads the graph, cannot parse the pruned
commit, and exits 16 — so the harvester refused `harvest/fsck-failed`.

The M-A arm, put through the same three kills, did **not** detonate it:
`c89b124a` survived, fsck exited 0, in-place resume exited 0.

| probe | result |
|---|---|
| fixture / `canonical` / `capsule/repo` — chain present, commit present | fsck **exit 0** |
| quarantine after H15's three kills — chain present, commit **gone** | fsck **exit 16** |
| same quarantine, `-c core.commitGraph=false` | fsck **exit 0** — sole cause |
| all 4 packs `verify-pack`; `multi-pack-index verify` | **OK** — no object damage |
| M-A arm, same three kills | commit **survives**, fsck **0**, resume **exit 0** |

Every heavy M-B cell was one auto-gc away from the same spurious refusal until
`quiesce_clone` (`control/pipeline.sh`, `710f94e5`) dropped the inherited graph
and disabled `gc.auto` / `maintenance.auto` on `canonical` and `quarantine`.

## The honest boundary of this claim

F-P05-28's own ruling: **the landmine is the fixture's commit-graph, not the
capsule model.** This is N=1, on one fixture, and the trigger was a derived cache
that fixture inherited. Do **not** generalise it to "bundles are fragile".

What *does* generalise, and is the operational-friction input QUE-200 asked for:
M-B pushes materially more bytes through a fresh object store per harvest, so it
is the arm that reaches git's background-maintenance thresholds first. A real
deployment inherits whatever the client repository's object store carries,
including derived caches nobody authored. M-B therefore arrives with a
provisioning obligation — quiesce the quarantine — that M-A did not exhibit.
That obligation is cheap and now written; but it is a line of setup that exists
only because of the mechanism, and it was discovered by a red, not by design.

## Related

- [[safe-capsule-ingestion-mechanism]] — QUE-200, the question this informs.
- EVD-007 — the ambiguous `harvest/fsck-failed` token that made this cost a
  session to diagnose.
- SL-241 PHASE-05 T4a step 1, T4b; findings F-P05-10, F-P05-28.
