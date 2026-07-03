# Jail RO-bind of a mutable binary goes stale after in-session reinstall; bind the immutable store path instead

**Symptom.** After `cargo install`-ing `~/.cargo/bin/doctrine` mid-session, the
Claude Code doctrine hooks and the doctrine MCP tools break and stay broken
until the jail is reloaded. Nothing short of reload fixes it.

**Root cause.** bwrap bind-mounts the path **read-only at jail-creation time** —
a bind against a specific inode/dentry. `cargo install` doesn't edit in place: it
builds a temp file then **atomic-renames** over the path, unlinking the old inode.
The jail's bind still points at the now-unlinked dentry → stale.

- **Hooks** spawn a fresh `doctrine` per event → each exec re-resolves → hits the
  stale bind → ENOENT/ESTALE → hook dies (hardest hit).
- **MCP** is long-lived (spawned once at session start) → limps on old code, or
  dies on re-exec; schema/protocol drift surfaces as broken tools.

Reload = new jail = fresh bind against the new inode = fixed. The bind is
namespace state, not a PATH lookup, so nothing short of a jail cycle re-resolves.

**Fix (root principle: match the binary's mutability to the mount's refresh
boundary).** Bind the **immutable** crane/nix store output over the cargo path
instead of the mutable cargo file. A store path is content-addressed — a rebuild
mints a *new* path; nothing renames over the existing one, so the bind never goes
stale mid-session. `flake.nix:95` — `ro-bind` two-arg, src≠dst:

```nix
(ro-bind "${doctrine}/bin/doctrine" (noescape "~/.cargo/bin/doctrine"))
```

src = store path (immutable); dst = the cargo path every PATH + absolute-path
caller already resolves. One bind-over cures **all** consumers (5 absolute
hardcodes across pi/codex arms + PATH callers) with zero per-consumer edits.

**Verify the bind held.** `ls -la ~/.cargo/bin/doctrine` inside the jail shows nix
store markers — uid/gid `65534`, mode `r-xr-xr-x`, timestamp `Jan 1 1970`. A
mid-session `cargo install` writes the host file underneath but the jail still
sees the store path (markers unchanged).

**Accepted limitation — jail-lifecycle-scoped, by design.** A rebuild produces a
new store path picked up only on the next jail cycle; not a mid-session hot-swap.
Converts the failure mode from **hard break** (rename → stale bind → ENOENT) to
**benign staleness** (old store path keeps working till relaunch). In-session dev
binary stays `./target/debug/doctrine` — a real file the jail never binds over,
so it's unaffected.

Same underlying hazard class as [[mem.pattern.jail.stale-binary-strips-registry-field]]
(ISS-053, *stale-but-alive*) but a different face: this is *hard-break* (bound
inode unlinked). Diagnosed + fixed under IMP-249 (flake.nix bind swap, committed
70ef2a40).
