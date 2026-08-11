# DEC-192: Capsule Git is host-initiated in both directions; git-daemon is deleted

**Accepted** 2026-08-11 by the owner — *"committing to the inversion, removing
git daemon"* — in the `RFC-025` capsule round, on `EVD-016` (measured) and
`EVD-017` (the escalation the inversion removes). It answers `QUE-212`.

## The decision

Capsule Git transport is **host-initiated in both directions**:

* **Result out** — `git fetch ssh://agent@<guest>/work/<repo>` into a fresh,
  host-authored quarantine repository, with `--no-tags` and an explicit refspec
  into `refs/capsule/<name>/*`.
* **Provisioning in** — `git push ssh://agent@<guest>/work/<repo>` into a guest
  repo seeded with `receive.denyCurrentBranch = updateInstead`. Fallback for
  re-provisioning over a dirty worktree: a bare `/work/origin.git` plus a
  guest-side clone.

No bundle, and no git-daemon. `DEC-135` is satisfied on **both** legs by
construction rather than by confinement: each side runs git only in a repository
its own side authored, which is what that rule is actually about — execution
context, not format.

## What this deletes

Deletion rather than mitigation, which is the shape `RSK-231` asked for. The
inventory below is **spike-side** (`/workspace/microvm-spike`), recorded here
because it is the price signal the round exists to produce; none of these paths
are doctrine's.

| where | what goes |
|---|---|
| `perimeter/default.nix` | `gitd`, `pushGuard`, `git-daemon-export-ok`, `sharedRepository=group`, the `gitPort` param, its half of the port preflight, the `base-path=` stray reap — ~60 of 297 lines |
| `host/services.nix` | the `capsule-gitd` unit, the `capsule-git` user+group, the 2775 setgid tmpfiles rule, `UMask=0002`, the group-membership onboarding — ~70 of 323. `capsule-proxy` untouched |
| `net.nix` | `gitPort` |
| `vm/capsule.nix` | `remote`, `capsule-clone`, `capsule-push`, the seed's clone and its wait-online ordering, two motd lines. The guest ends with **zero** capsule-specific programs |
| `justfile` | `branches`, `fetch`, and `status`'s gitd-listener and mirror-ref checks — reshaped onto the quarantine repo |
| `README` + host flake | *"the only two ports"* → one; `allowedTCPPorts = [3128 9418]` → `[3128]`. The host flake is outside the spike repo — flag, do not patch |
| `PLAN_C.md` | "per-capsule proxy and git daemon" → proxy only. The *"one gitd uid or N"* question stops existing rather than getting answered |

It lands **before** `capsules.nix` is written rather than after, which is why it
was worth confirming ahead of anything downstream.

## The claim that must not travel

*"The guest never initiates a connection to the host at all"* is **false**, and
the form of it in `IMPORT.md` contradicts its own next line. The proxy remains,
it is guest-initiated, and tinyproxy is the larger C parser of guest-authored
input of the two services involved.

Correct form: **the host runs no service that parses guest-authored Git input.**
One guest-initiated channel remains — the proxy.

Consequence for the round: `REQ-448`'s third demonstration (`IMP-426` P1c)
becomes *absence of a git channel*, not *absence of a channel*. Still stronger
than the present *refusal on a channel*, and it needs no update hook to make it.

## What it costs — three residuals, none closed here

1. **`index-pack` still parses hostile bytes host-side.** True of every candidate
   including bundles (`EVD-006`), so not new — but the receive-side
   `receive.maxInputSize` knob goes away with the receive side, and the
   replacement ceiling has no natural source. Open as `QUE-213`.
2. **No artifact.** A live fetch produces nothing to hash, retain, or
   deterministically re-ingest. `ASM-010` has already removed host-side disk
   forensics, so a **retained quarantine repo is the only candidate exhibit**
   under `DEC-133`. A retention question, not a security one; carried into
   `IMP-426` P1d, deliberately not settled here.
3. **The agent loses `capsule-push`** — agent-initiated publication. Under the
   inversion only the host pulls. This is a workflow regression and the one thing
   the inversion makes strictly worse; whatever replaces it is control-plane
   work, not perimeter work.

One posture change that is not a cost: **guest sshd becomes perimeter-load-bearing**
rather than a convenience. The trust direction is right — the host initiates and
authenticates the guest's host key — and under the netns design the socket path
*is* the identity (`NOTES.md` item 17), which dissolves the `known_hosts` problem
rather than relocating it.

## Unverified at decision time

Named so this is not read as resting on more than it does: git over the netns
unix-socket `ProxyCommand` (only `socat` has crossed that bridge); whether the
pushed worktree is populated; behaviour and cost under `transfer.fsckObjects`;
and every `P1a`/`P1b` figure, which still waits on a firecracker boot with its
tap inside a namespace.

## Related

`QUE-212` (answered by this), `EVD-016`, `EVD-017`, `QUE-213` (its own residual),
`DEC-135` (the rule satisfied by construction), `ASM-010` (which removed the
third candidate), `DEC-133` (the retention residual), `RSK-231`, `IMP-426`,
`RFC-025`.


## Amended 2026-08-11 — the implementation shape, as built

The decision as ruled said *fresh* quarantine repository. The spike is building
**persistent per capsule, fetched into incrementally**, with `--no-tags`,
`-c transfer.fsckObjects=true`, and a write ceiling from `target.nix`. Recorded
rather than left as silent divergence.

It should win, and the execution-context rule survives it: the quarantine is
host-created and host-configured, the guest cannot write its `config` or
`hooks/`, and the refs it lands under are chosen by the host's refspec. What
persistence buys is an incremental collect and — more importantly — the exhibit.

**Pin versus exhibit, which residual 2 left implicit.** The ref and sha that
`capsule-collect` prints are the **pin**: the durable, cheap journal entry
`DEC-133` wants on the trusted side. The **exhibit** is the persistent quarantine
repository itself. That makes `DEC-133`'s deliberately-unspecified retention
concrete for the first time — the exhibit expires when the quarantine is reaped,
so *when to reap* is now a knob someone has to set rather than an open axis.

**The mirror goes with the daemon.** It existed to keep the serving uid away from
the human's tree; with no serving uid there is nothing to keep away, and
`EVD-017` is the argument — the mirror *was* the two-uid repository.
`capsule-provision` pushes from `target.path` directly.

**Consequence worth stating: provisioning carries committed objects only.** A
push moves refs and objects, so uncommitted and gitignored working state never
reaches a capsule. Today's dispatch provisions via `.worktreeinclude`, which
copies exactly that class. Whatever the provisioning manifest turns out to be
(`QUE-204`, and `RFC-025` next-design-question 2), the Git channel is not its
carrier.

**Guest seeding depends on `HEAD`.** `updateInstead` fires only for the branch
`HEAD` points at — see `EVD-016`'s coupling section. Seeding must point `HEAD` at
the ref being provisioned rather than inherit it.


**Two of the four "unverified at decision time" items have since resolved.** Git
*does* cross the netns unix-socket `ProxyCommand` host-initiated, so P0's result
is not specific to the old tap shape; and the pushed worktree *is* populated
(`EVD-016`). The guest→host direction over the proxy was broken and has been
repaired, but it is being removed by this decision regardless. Still unverified:
cost under `transfer.fsckObjects`, and every P1a/P1b figure.
