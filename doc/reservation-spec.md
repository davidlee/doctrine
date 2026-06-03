# Reservation &amp; leasing specification

## Overview

Heresiarch courts parallel agents and multiple teams. The immediate hazard is
**numbering collisions** — two agents in separate clones both mint `slice-003`
and collide at merge (slices-spec § Known risks). A later hazard is **concurrent
edits** of the same entity.

Both are *coordination* problems — who holds what, across filesystems — and both
reduce to one primitive: a **lease**, a compare-and-swap claim on a named key in
a shared store. This spec ships the half that solves the immediate hazard
(**reservation** — permanent claims) and specifies the other half
(**transient leasing** — expiring claims for edit-exclusion) as a deferred
extension of the same primitive.

The primitive is **generic** (any entity kind, any key), **coordination-only**
(it never stores entity content — entities stay in the working tree and in PRs),
and **backend-pluggable** (a shared git ref by default; a local directory with no
remote; a hosted store such as Postgres later). It is the Heresiarch form of
lazyspec's RFC-030 (reservation) and RFC-035 (leases), collapsed into one
primitive.

## The unification

A lease is an *atomic exclusive claim on a name*. Heresiarch already has one:
slices-spec allocates ids by `mkdir`-ing the numeric directory — the directory
**is** the claim, and `EEXIST` **is** the compare-and-swap failure.

So id reservation is a single backend-agnostic algorithm:

```
reserve_next(namespace):
  loop:
    candidate = max(list(namespace)) + 1     # 001 if empty
    if acquire(namespace, candidate):        # atomic, exclusive
      return candidate
    # lost the race — someone claimed it between list and acquire; retry
```

`mkdir` is simply the **local backend's `acquire`**. Point the same algorithm at
a **shared git ref** and the identical logic becomes safe across teams. The
*algorithm* never changes; the backend's **reach and failure modes** do — local
`acquire` is offline and instant (an `EEXIST` syscall), git-ref `acquire` costs a
network round-trip (`fetch --prune`) and fails by push-rejection. So callers must
sit behind the `acquire` seam itself (not a bare `fs::create_dir`), or the
"swap the backend, not the caller" promise holds only on paper — see § Code
seam.

### Code seam

The unification is real only if the claim is written against an `acquire`
operation, not inlined as a filesystem call. v1's slice code (`reserve_create`)
inlined `fs::create_dir` + `ErrorKind::AlreadyExists` as a shortcut; that is the
one thing that must be lifted to a one-method seam —

```rust
fn acquire(&self, key: &str) -> Result<Acquired>;  // Won | AlreadyHeld
```

— with the local `mkdir` as its sole implementation, **before** a second backend
or a second caller arrives. The retry loop, the candidate scan, and the
materialisation then compose over the seam unchanged when `git-ref` lands. This
is tracked as the first deliverable of the entity-engine work, not deferred with
the `git-ref` backend itself.

## v1 scope

**Ships:** id reservation (permanent claims) over two backends (`local`,
`git-ref`), the `acquire` / `read` / `list` operations, and the `heresy reserve`
/ `heresy lease list` CLI. Enough to make `heresy slice new` collision-free
across teams.

**Deferred** (§ Deferred: transient leasing): TTL, heartbeat, release,
force-acquire, clock-skew handling, crash recovery, and the per-kind
write-gating they serve. None of it is needed for reservation — a permanent claim
never expires, so it has no recovery, heartbeat, or clock semantics. Specifying a
clock-skew protocol now would serve a caller that does not yet exist.

This is the deliberate "simplest thing that'll probably work" line: the apparatus
arrives with the caller that needs it.

## Concepts

### Lease

A claim on a **key**, recorded in the active backend:

```toml
holder   = "agent-7"                 # § Holder identity (attribution)
acquired = "2026-06-03T10:00:00Z"
# expires — omitted in v1. Present only for transient leases (§ Deferred).
```

A v1 lease is **permanent** — a reservation. A claimed id is taken forever; it is
never released. This decouples *claiming a number* from *the entity existing
yet*: a team can reserve `slice-042` now and author it later. An abandoned
reservation (the agent died, the slice was never written) is a harmless gap, not
a fault to recover from.

### Key

A namespaced string, `<kind>/<facet>/<name>`. Kinds register their own
namespaces; the primitive is blind to meaning:

| Caller | Key pattern | v1 |
|---|---|---|
| Slice id reservation | `slice/id/<n>` | yes |
| (future) spec-family id reservation | `prd/id/<n>`, `spec/id/<n>`, `rev/id/<n>` | trait-ready |
| (future) entity write-claim | `<kind>/claim/<id>` | deferred (transient) |

Slices are the first caller; the primitive is not slice-specific.

### Holder identity

Recorded for attribution. Resolved by priority chain:

1. `$HERESY_AGENT_ID` — explicit, for orchestrators.
2. `$CLAUDE_SESSION_ID` — auto-detected under Claude Code.
3. `git config user.name` — fallback.

## Backends

A backend is the *reach* of a claim. One trait, selected by config:

```rust
pub trait LeaseBackend {
    fn acquire(&self, key: &str, holder: &str) -> Result<Acquired>; // create-CAS
    fn read(&self, key: &str) -> Result<Option<Lease>>;
    fn list(&self, prefix: &str) -> Result<Vec<(String, Lease)>>;
    // Deferred (§ Deferred: transient leasing) — added with the transient layer:
    //   heartbeat, release, force_acquire
}
// Acquired = Won | AlreadyHeld(Lease)
```

`acquire` is **compare-and-swap create**: it succeeds only if the key is unheld.
That single atomic operation is the only race arbiter; every guarantee rests on
it.

### Local backend (`local`)

Claims are directory entries under the project: a key maps to a path, `acquire`
is an exclusive `mkdir` / `O_EXCL` create (`EEXIST` ⇒ `AlreadyHeld`). This *is*
slices-spec's existing `mkdir` allocation, generalised — no new mechanism.

- **Reach: one working tree.** Cannot see other clones; the inter-team hazard
  remains. Correct and lock-free for parallel agents on one filesystem.
- No git, no remote, no network. Always available — the floor of correctness.

### Git-ref backend (`git-ref`)

Claims are git custom refs under `refs/heresy/lease/<key>`, each pointing at a
commit whose tree holds the lease record. Lifted from lazyspec RFC-035, minus the
expiry machinery (not needed for permanent claims):

- **Reach: every clone of the remote.** Solves the inter-team hazard.
- `acquire` pushes with `--force-with-lease=<ref>:<all-zeros>` — succeeds only if
  the remote ref is absent. Racing agents both see "absent" locally; exactly one
  push lands, the rest are rejected with stale-info and retry.
- `list` / `read` fetch the namespace with `--prune` before reading, so a
  deleted remote claim never survives as a stale local ref. The remote is the
  **linearization point**: a claim isn't held until the remote accepts the push;
  the local ref is a cache updated only on success.
- Refs are commits, not bare blobs (hosted platforms reject non-commit refs):
  `hash-object` → `mktree` → `commit-tree` → `update-ref`.

Nothing lands in the working tree, `git log`, or PRs — coordination is invisible
to the permanent record.

### Future backend: Postgres

The trait admits a hosted store. A `postgres` backend maps cleanly:
`acquire` = `INSERT … ON CONFLICT DO NOTHING` (the unique constraint is the CAS),
`list` = `SELECT … WHERE key LIKE prefix`, and — when the transient layer lands —
expiry/heartbeat become a timestamp column and a cron-free `WHERE expires < now()`.
Not built; the trait shape exists so it drops in without disturbing callers.

### Selection

```toml
# .doctrine/config.toml
[lease]
backend = "auto"        # auto | local | git-ref  (postgres later)
remote  = "origin"
```

`auto`: `git-ref` when `remote` is configured and reachable, else `local`,
emitting a one-time warning that cross-team reach is off. Explicit values
override. v1 makes the reachability check by attempting the fetch on the reserve
path itself — **one network round-trip per reserve, accepted**; a cached probe is
a later optimization (§ Open questions), not a v1 concern.

## Reserve operation

`reserve_next(namespace)` (§ The unification) over the active backend. With
`local` it is the `mkdir` loop; with `git-ref` it lists `slice/id/*` refs, takes
`max + 1`, and CAS-acquires a permanent ref, retrying on rejection. The returned
number is globally unique across every clone of the remote.

`heresy slice new` calls this instead of the bare local scan; the on-disk slice
shape (slices-spec) is unchanged.

## CLI

```
heresy reserve <namespace>          # allocate + claim next id, print it
heresy lease list [<prefix>]        # held claims: key, holder, acquired
```

Both accept `--agent-id <id>` (else the identity chain) and `--json`. The
transient verbs (`acquire --ttl`, `heartbeat`, `release`, `steal`) arrive with
§ Deferred.

## Architecture

Same pure/imperative split as the rest of Heresiarch:

| Pure (library, unit-tested) | Imperative (thin shell) |
|---|---|
| candidate from a key listing → `u32` | the backend `acquire` CAS (mkdir / ref push) |
| lease record (de)serialise ↔ struct | `fetch --prune`, ref read / list / update |
| backend selection from config + probe result | probe: remote reachable? |
| holder resolution from an env/config snapshot | read env / `git config` |

The key listing and probe results are **inputs** to the pure layer. The CAS
itself cannot be pure — only the syscall (or the remote) arbitrates the race — so
it lives behind the `LeaseBackend` seam, exactly as slices-spec's `mkdir` claim
does. Tests inject a mock backend and assert the retry logic without git or a
remote.

## Coordination-only boundary

This spec governs *claims*, never *content*. Leases reference entities by key and
never hold their bytes. Slices, specs, and every other entity remain ordinary
working-tree files, visible in `git log` and PRs. Heresiarch deliberately does
**not** adopt lazyspec's git-ref *storage* (hiding documents in refs); that would
suit a future ephemeral task/iteration entity, not the design artifacts this repo
produces. Out of scope.

## Deferred: transient leasing

When an entity write-claim caller exists (e.g. gating concurrent edits of one
slice), the primitive grows the expiring half — the same trait, three more
methods:

- A **transient lease** carries `expires`. `heartbeat` extends it; `release`
  drops it by holder; `force_acquire` reclaims one past `expires + grace`.
- **Crash recovery** is expiry: a dead holder's claim becomes reclaimable. A
  `grace` window plus a `max_clock_skew` bound absorb honest NTP drift; under
  `git-ref` the committer timestamp is a tamper-evident cross-check. Split-brain
  is reachable only if `|Δ_clocks| > ttl + grace`.
- **Write-gating** (refusing `heresy` mutations without a held claim) is wired
  **per kind**, not globally, and only when that kind's lifecycle defines
  mutations (slices-spec § Lifecycle currently has none).

Recorded here so the trait shape and the `expires` field are deliberate, not
retrofitted. Not v1.

## Out of scope

- **Transient leasing** and per-kind write-gating — § Deferred.
- **jj support.** This repo targets git. A jj-native ref mechanism is not
  pursued; under jj-without-git the `git-ref` backend is simply unavailable and
  `auto` falls to `local`.
- **Hook integration** (Claude Code claim/heartbeat/release) — arrives with the
  transient layer and an orchestration story.
- **Git-ref storage** of entities — see § Coordination-only boundary.

## Known risks

- **No remote ⇒ no cross-team safety.** The `local` backend reaches one working
  tree; teams in separate clones can still collide. `auto` warns when it falls
  back. Accepted: solo / single-tree work is correct and lock-free; cross-team
  safety requires a configured remote.
- **Reservation ref count.** One permanent ref per reserved id. Negligible into
  the low thousands (git `for-each-ref` over hundreds of refs is sub-millisecond;
  ref-advertisement cost bites only at ~10k+ refs, and chiefly for *loose* refs —
  cured by `git pack-refs`). No slice-like entity approaches that. A single
  high-water-mark counter is the escape hatch if some future kind ever reserves
  10k+ ids; rejected for v1 because it would add a second primitive (a counter)
  and break the reservation-is-a-lease unification to dodge a cliff that is not
  reachable here.
- **Direct edits bypass claims.** Reservation only guards id *allocation*; it
  does not stop a human editing a working-tree file. That is the transient
  layer's concern, and even then gates only writes through `heresy`.

## Open questions

1. **Cached reachability probe.** v1 spends one network round-trip per reserve to
   decide `auto` and fetch the namespace. A short-lived cached probe (or reusing
   a recent fetch) removes it. Deferred until reservation frequency makes the
   round-trip felt — explicitly fine for now.

## Testing

Unit tests (pure layer, mock backend):

- Reserve loop — empty namespace ⇒ `001`; a CAS conflict on the first `acquire`
  drives recompute and lands the next free id; gaps and max selection.
- Lease record round-trip — serialise then parse; `expires` absent (permanent).
- Backend selection — `auto` resolves to `git-ref` when the remote probe
  succeeds, to `local` (with warning) otherwise; explicit override wins.
- Holder resolution — env chain precedence.

The `git-ref` backend's real ref I/O sits behind the `LeaseBackend` seam (the
same pattern as `heresy install` / `heresy skills`), asserted via a mock that
records the push / fetch / CAS calls — no git and no remote in the test.
