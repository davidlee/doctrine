# IMP-426: Evidence spike: fresh-per-transaction microVM capsule

Evidence, not product — the `SL-241` shape, and its go/no-go discipline. This is
the **first** move of the capsule rescue: `RSK-231` calls for an architectural
revision rather than another remediation slice, and this is the measurement that
revision needs before anything is designed.

## The question

Can a doctrine-driven Firecracker microVM capsule be **fresh per transaction** at
tolerable cost on this project's hardware?

`SPEC-030` `REQ-450` (FR-003) requires that every phase worker *and every
verifier* launch headlessly in fresh mutable capsule state, across `Axis`'s five
freshness axes — checkout, repository, runtime, temporary state, process.
`DEC-134` fixes fresh-context-per-phase as the v0 topology.

## Why it comes first

Conformance measures a capsule. **There is no per-transaction capsule yet.**
`/workspace/microvm-spike` is the opposite shape by design:

- one persistent VM with a persistent `/work` volume holding a real clone;
- `vm` is build-then-exec in the **foreground**, not a daemon — `pgrep -af
  'microvm@'` is "the entire inventory; nothing is registered anywhere";
- driven interactively over ssh;
- more than one capsule at a time is *"scoped, not started"* (`PLAN_C.md`).

If the answer is no, the conformance work has nothing to measure and the rescue
is moot. If yes, `DEC-190`'s kernel and `DEC-189`'s re-derived row set have a
subject.

## What the spike already gives it for free

`NOTES.md` item 17 has done the scoping and names the deciding cost — **the guest
image, not the plumbing**: the obvious design pays N image blobs (N × disk, N ×
*pack* time on every `dev-tools` bump, though not N × build, since the closure is
almost entirely shared). Getting the two per-instance values out of the closure —
the guest's address and the **base commit** — buys one image and N small runners.
A kernel param does not work for either: it lands in `toplevel`, so in the
closure.

And **netns-per-capsule has been spiked and it holds** (`spike/netns.sh`): two
capsules, a guest that already has root, no path to the upstream, no path to the
sibling, unreachable from outside even by something holding a route to it, while
a process in the namespace reaches the internet normally. Flipping the
namespace's `ip_forward` proves the switch is what does the work. It makes the
guest bit-identical across instances — one image, no DHCP, no boot-time step —
and gives a forward control that is *ours* rather than one shared with docker and
tailscale.

## What to measure

1. Wall-clock to a usable fresh capsule, cold and warm.
2. Disk and pack cost per instance under the one-image-plus-runners shape versus
   N blobs.
3. Whether all five freshness axes are actually satisfied, or only some.
4. Teardown: does the VMM exit, is the tap released, is state genuinely gone.
   (Known sharp edge: firecracker does **not** exit on guest poweroff — it halts
   the vCPU and keeps the tap, so the next start fails `Device or resource busy`.)
5. Two concurrent capsules, since `red-team.md` `RT-1` requires verification in a
   *separate* capsule from the worker.

## Scope discipline

No conformance suite, no doctrine integration, no production code. Bounded
measurement with a written go/no-go, on the `SL-241` precedent — whose verdict
was explicitly scoped and did **not** claim production readiness or portability.

## Round shape, 2026-08-11 — widened, and it runs in the spike

Owner's direction: **import the most important unproven requirements into the
spike and price them there** — cheaply, before doctrine's governance gears turn.
So this item is no longer only the freshness measurement; freshness is one of
four imports.

The round is directed by a **disposable context packet**,
`/workspace/microvm-spike/IMPORT.md`, which carries the probe list, the priority
order, the hazards and the reporting bar. It is deliberately not governance and
is deleted when its findings graduate into knowledge records and back into this
item.

**The bar is a price, not a proof.** Each import wants *"costs about this much,
here's the shape, here's what it breaks"* — not a working implementation and not
a conformance row.

**Imports chosen** — the VM changes the answer for each, and the spike can price
each cheaply:

| | requirement | why it is in |
|---|---|---|
| P1a | `REQ-450` — fresh mutable state, five axes, worker *and* verifier | the spike is presently the opposite shape by design |
| P1b | `REQ-454` — verification in a *separate* fresh capsule | `red-team.md` `RT-1`; and per `CPT-002` the highest-value item in the list — it is the substrate for the defence against the threat that actually arrives |
| P1c | `REQ-448` — the authority floor | `DEC-191` makes it invariant at every posture; mostly already true here, so demonstrate rather than argue |
| P1d | `REQ-451` / `REQ-452` — ingestion bounds | **blocked on `QUE-212`** |

**Blocked on `QUE-212`** (`needs` edge authored): the spike has no host
filesystem shares, so the result leaves over a channel, and the channel it picked
— a live `receive-pack` into a host mirror — is not `DEC-135`'s bundle snapshot.
`REQ-451`/`REQ-452` cannot be shaped until the transport is chosen, because their
subject may not exist. (An earlier form of this paragraph expected an outbox
block device to win; `ASM-010` has since ruled that out standing — the host never
hands guest-authored filesystem metadata to the host kernel, and `ro`/`nosuid`/
`nodev`/`noexec` do not mitigate it because the parse precedes them.)

**Out of scope this round:** the conformance suite (`DEC-189`/`DEC-190` — a suite
needs a capsule to measure), `REQ-455`–`REQ-458`, `REQ-461`, and `DEC-191`'s
front list.

## P0 landed, 2026-08-11 — the transport is priced

`QUE-212` is **answered** by `DEC-192` (`accepted` — the owner committed to the
inversion and to removing git-daemon), so this item's `needs` gate is clear.

* `EVD-016` — host-initiated fetch **and** host-initiated push both work over the
  existing capsule ssh channel, ~32 MiB each way at ~100 MiB/s. n = 1, hand-run,
  current tap shape, **not** the netns design.
* `EVD-017` — the shared mirror that guest-push requires re-opens, in reverse,
  the escalation the `capsule-git` uid split was built to close. The strongest
  argument for the inversion, and it was not available when the round opened.
* `DEC-192` — host-initiated in both directions; git-daemon leaves the perimeter.
  Carries the deletion inventory and three residuals it does not close.
* `QUE-213` — new blocker for **P1d**: the receive-side size ceiling goes away
  with the receive side, and its replacement has no natural source.

**Two corrections that must not travel downstream.** *"The guest never initiates
a connection to the host at all"* is false — the proxy remains and is
guest-initiated; the claim that holds is *the host runs no service that parses
guest-authored Git input*, which makes **P1c**'s third demonstration *absence of
a git channel*. And the host's refspec does not fully decide the destination
namespace without `--no-tags`.

**P1a / P1b are unblocked.** The namespace boot worked on 2026-08-11 —
firecracker comes up with its tap inside a capsule namespace on the host-module
path, the one step `NOTES.md` item 11 had reasoned to from pinned microvm.nix
source without running. Their figures — wall-clock cold and warm, disk and pack
per instance, which freshness axes actually hold, teardown, two concurrent
capsules — remain unmeasured, and are the work.

## Related

`RSK-231`, `RFC-025`, `SPEC-030` `REQ-450`, `DEC-134`, `EVD-015`, `DEC-189`,
`DEC-190`, `DEC-191`, `CPT-002`, `IMP-397` (capsule egress allowlist and
build-input provisioning, which this will brush against). Round P0:
`QUE-212` (blocking), `DEC-192`, `EVD-016`, `EVD-017`, `QUE-213`, `ASM-010`.
