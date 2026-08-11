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
