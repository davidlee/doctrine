# Pre-warm dispatch worker fork target via reflink copy

Each dispatch worktree (worker fork AND the coordination tree) builds into its own
gitignored in-tree `target/` — no shared `CARGO_TARGET_DIR` by design
([[mem_019f026cffd27a43b8db3cf6728130b5]]). A fresh fork therefore has an EMPTY
target → a cold build of the full dep graph (~1766 crates here), which routinely
exceeds the subprocess arm's `timeout 300` (`/dispatch-subprocess`) and kills the
worker mid-build.

**Fix:** right after `doctrine worktree fork … --dir "$D"`, copy the main worktree's
warm target into the fork with a reflink (copy-on-write) clone:

```sh
cp -a --reflink=auto target "$D/target"
```

On a CoW filesystem (this repo's `/workspace` nvme supports it) this is near-instant
and space-free — ~18s wall, 0.07s user, the 14G is shared by extent. Cargo then
rebuilds only the workspace crate (`doctrine`) + the integration-test crates (their
own path changed); the 1766 registry deps keep their fingerprints and are NOT
recompiled. Worker build drops from minutes-cold to ~tens of seconds.

If the fs lacks reflink support `--reflink=auto` silently falls back to a full copy
(slower, uses disk) — still correct, just not free.

**How to apply.** Pre-warm the fork before the `pi`/`codex` spawn; also pre-warm the
COORDINATION tree before the funnel's verify beat (it too is cold after
`dispatch setup`). Distinct from [[mem_019eec3285e471c287a0c3d74c235b25]] (fork
*omits gitignored artifacts* → the symptom this prevents) and
[[mem_019ec39d5df97cd1bbdfe99d7a2027b7]] (a SHARED target causes false-RED — the
opposite hazard; reflink gives each tree its own physical-logical copy, no sharing).
Born SL-165 PHASE-01 dispatch, 2026-06-27.
