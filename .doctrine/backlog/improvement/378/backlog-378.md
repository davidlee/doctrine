# IMP-378: Version-keyed once-per-repo migration primitive for installed clients

Deferred out of SL-242 on 2026-08-01 by explicit decision — that slice cleans
this repo and stops the shipped corpus describing a retired delivery model, but
deliberately builds no mechanism to reach back into installed clients.

## Problem

`doctrine install` is write-if-absent and idempotent by contract. It can add;
it can never remove or refresh. So any state a client acquired from an older
doctrine persists forever across upgrades, with no path to correct it.

The concrete instance that surfaced this: repos installed before SL-227 /
ADR-019 carry projected `.doctrine/*.md` reference-doc copies that are no longer
projected, cannot refresh, and drift silently from their published masters (in
this repo, before cleanup: `dispatch-mechanics` 231 divergent lines,
`using-doctrine` 93, `glossary` 70). SL-242 also freshens the shipped
`[gitignore]` contract — additive, so a re-installing client picks the entries
up, but nothing untracks what that client already committed. Both leftovers need
the same missing primitive.

## Shape (not designed)

A migration that is:

- **version-keyed** — each migration names the release it belongs to, and a
  repo records which have been applied;
- **once-per-repository** — re-running is a no-op, and the applied-migration
  ledger has to survive a runtime-state wipe (runtime state is `rm -rf`-able by
  contract, so a ledger there would silently re-run everything);
- **isolated from doctrine's own dependencies** — a migration typically runs in
  a repo whose binary has just changed; it must not be breakable by the very
  upgrade that triggers it. The design has to say what "isolated" concretely
  forbids.

## Sharp edge

A client may have edited a projected copy in place. Any removal step must diff
against the published bytes and report divergence rather than delete silently —
otherwise the primitive's first use destroys user work.

## Neighbours

- **SL-242** — the slice that deferred this; cleans this repo only.
- **IDE-030** — the general form: framework-owned client docs go stale forever
  under write-if-absent install. Blocked on this primitive.
- **IMP-315** — the originating report.
- **ADR-019** — the projection/publication split that makes this a live problem.
