# IMP-249: Shadow ~/.cargo/bin/doctrine with nix-built binary bound into the jail — kill stale-bind hook/MCP breakage after cargo install

## Symptom

After `cargo install`-ing the PATH doctrine (`~/.cargo/bin/doctrine`)
mid-session, the Claude Code doctrine hooks and the
doctrine MCP tools break and stay broken until the session (jail) is reloaded.
Reload fixes it every time; nothing short of reload does.

## Root cause: RO-bind of a mutable file + atomic rename

bwrap bind-mounts `~/.cargo/bin/doctrine` **read-only into the jail namespace at
jail-creation time** — a bind against a specific inode/dentry. `cargo install`
does not edit in place: it builds to a temp file then **atomic-renames** over the
path. The rename unlinks the old inode and drops a new one at the same name. The
jail's bind still points at the old dentry — now stale/unlinked from the
namespace's view.

Asymmetry (why hooks die hardest):

- **Hooks** spawn a fresh `doctrine` per event → each exec re-resolves the path →
  hits the stale bind → ENOENT/ESTALE → hook fails.
- **MCP** server is long-lived, spawned once at session start → limps on the old
  code (or dies if it re-execs). Schema/protocol drift after the reinstall then
  surfaces as broken tools.

Reload = new jail = fresh bind against the new inode = fixed. The bind is
namespace state, not a PATH lookup, so nothing short of a jail cycle re-resolves
it.

This is the same underlying hazard as [[ISS-053]] (stale PATH binary silently
drops new `BoundaryRow` fields) and the "stale-binary verification hygiene" class
parked at RFC-005 OQ-6 / Tension 5 — but a different face: ISS-053 is
*stale-but-alive* (old code drops fields), this is *hard-break* (bound inode
unlinked, hooks/MCP die).

## Fix: swap the flake.nix jail bind from the mutable cargo file to the immutable crane output

Root principle: match the binary's mutability to the mount's refresh boundary. A
nix output is a content-addressed store path — **immutable**. A rebuild produces a
*new* store path; nothing ever renames over the existing one, so an RO bind
pointing at it never goes stale mid-session. The dentry the jail bound at creation
stays valid for the jail's whole life. Converts the failure mode from **hard
break** (rename → stale bind → ENOENT) into **benign staleness** (old store path
keeps working until the jail is cycled), which also structurally eliminates the
[[ISS-053]] / RFC-005 OQ-6 class on the jail surface.

**The one-line cure: `flake.nix:95`.** The jail *already* RO-binds the cargo path:

```
(try-readonly (noescape "~/.cargo/bin/doctrine"))   ← binds the MUTABLE cargo file
```

Change *what it binds* — point it at the crane package already built in flake.nix
(`srcWithDist`), i.e. `${doctrine}/bin/doctrine`, instead of the live mutable
cargo file. This is not adding a bind; it's swapping the existing bind's target
from mutable→immutable. One edit covers **every** consumer at once — PATH callers
and absolute-path callers alike — because they all resolve to the same bound path.
`cargo install` mid-session then no longer touches what the jail sees.

Why a single bind-over beats per-consumer retargeting: the census (below) found
**five** absolute-path hardcodes across the pi + codex arms, none reachable by
PATH order. Retargeting each = 5 edits + template regen + a standing rule that new
consumers must not hardcode. Swapping the mount target = one edit, no consumer
touched, no PATH-order dependency.

The user's parallel `.mcp.json` edit (Claude MCP `command`
`/home/david/.cargo/bin/doctrine` → bare `doctrine`) is retained as
belt-and-suspenders for the Claude arm.

## Scope, cost, accepted limitation

- **Not a mid-session hot-swap** — by design. Rebuild → new store path, but the
  jail still binds the OLD store path until relaunch. The fix makes the hook/MCP
  doctrine a *jail-lifecycle-scoped* artifact, changeable only by cycling the
  jail — which is exactly when the bind re-resolves. Alignment, not magic.
- **Dev iteration unaffected** — in-session CLI stays `./target/debug/doctrine`
  (the coord build); it's a real file the jail never bound over. The consumer
  split is correct: fast tooling → hot dev binary; hooks + MCP → pinned store
  path.
- **Cost** — build step becomes `nix build` on host + jail relaunch (nix is
  absent in-jail). Slower than `cargo install`, but it's the only thing that
  fixed the break — trade a fast-but-broken path for a slower-but-clean one, no
  regression in real capability.

## Consumer census (`cargo/bin` refs, done 2026-07-03)

Live exec/config refs to `~/.cargo/bin/doctrine` (prose docs and the disposable
`.doctrine/state/dispatch/candidate/…` worktree copies excluded). `rtk` does NOT
exist — the earlier "rtk hook" was a bad assumption; drop it.

**Absolute-path hardcodes — bypass PATH entirely (the reason bind-over > PATH-shadow):**

- `.codex/hooks.json:7` — codex orchestrator hook `command`
- `.pi/extensions/doctrine/index.ts:8` — pi orchestrator `execSync`
- `.pi/extensions/doctrine/mcp.ts:27` — pi `BIN_PATH` const
- `.pi/extensions/doctrine/mcp.ts:343` — pi `DOCTRINE_BIN ||` fallback (regenerated artifact — see below)

**Install-time injection (the real source of the literal home path):** the deployed
`.pi/extensions/doctrine/mcp.ts:27` `const BIN_PATH = "/home/david/.cargo/bin/doctrine"`
is not authored — `doctrine claude install` (pi arm) resolves the current PATH
doctrine and bakes it via `--define BIN_PATH=…`. The `templates/mcp.ts` source
fallback was itself a literal `/home/david/…` (POL-002 violation: ships one
machine's home into every install); **fixed to bare `'doctrine'`, uncommitted**.
Open decision — bake nothing (PATH / `DOCTRINE_BIN`) vs resolved absolute — is
owned by [[IMP-234]] (the installer `--dev` posture), where "don't bake exec
paths" is now recorded as the second `--dev` axis alongside marketplace source.
Under the flake.nix:95 bind-swap the baked absolute is harmless in-jail but
non-portable for raw out-of-jail use.

**Still hardcoded absolute (no template/`--define` seam — need manual edit or PATH):**

- `.pi/extensions/doctrine/index.ts:8` — pi orchestrator `execSync("…/doctrine prompt resolve…")`
- `.codex/hooks.json:7` — codex orchestrator hook `command`

**PATH-preferring (fine as-is once nix doctrine is on PATH, or moot under the bind swap):**

- `scripts/pi-spawn-confined.sh:29` — `command -v doctrine || $HOME/.cargo/bin/doctrine` (PATH-first)
- `scripts/pi-spawn.sh:13` — `DOCTRINE=~/.cargo/bin/doctrine` (older non-confined; hardcoded but low-traffic)

**The mount + PATH setup (where the fix lands):**

- `flake.nix:95` — `(try-readonly (noescape "~/.cargo/bin/doctrine"))` — **the stale RO bind; swap its target here**
- `flake.nix:98` — `(add-path "/home/david/.cargo/bin")` — cargo bin on PATH (leave, or point at nix)

**Docs (prose, no exec surface — update for accuracy, not correctness):** `AGENTS.md`
(the "run from coord build, not `~/.cargo/bin/doctrine`" notes), a few memories.

Under the `flake.nix:95` bind swap, all five absolute hardcodes are cured without
edits — they resolve to the bound path, which now points at an immutable store
path. Per-consumer retargeting becomes optional hygiene, not required.

### Regenerated Claude MCP command (already applied)

`.mcp.json` doctrine server `command` was `/home/david/.cargo/bin/doctrine`, now
edited to bare `doctrine`. Note `doctrine claude install` may rewrite `.mcp.json`
— confirm the installer emits bare `doctrine` (or the nix path) so a reinstall
doesn't reintroduce the absolute cargo path.

## Source

Diagnosed live during SL-191 PHASE-05. Durable memory to capture once MCP is
healthy: `mem.pattern.env.jail-ro-bind-stale-after-reinstall` (the memory tools
route through the very MCP at risk).
