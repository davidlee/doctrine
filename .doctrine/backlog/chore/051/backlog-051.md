# CHR-051: Repair the pi spawn scripts' poll, reap and model mapping

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Three independent defects in the `pi` spawn/agent surface, all found and fixed in
`scripts/pi-review.sh` during RV-324 (2026-07-30). The fixes are proven there;
this item is to carry them back to the scripts still running the originals.

## 1. The completion poll re-reads the whole growing stream

`scripts/pi-spawn-confined.sh` and `scripts/pi-spawn.sh` detect completion with

```bash
grep -qE '"(type|event)":"agent_end"|"agent_end"' "$OUT"
```

inside a 2-second loop. `pi --mode rpc` re-serializes accumulated conversation
state on every event, so `$OUT` grows super-linearly — **50–150MB for one ordinary
turn** (measured 54 / 56 / 130 / 139 / 152MB across seven review turns; none was a
loop, all were legitimate reasoning). The poll's I/O cost is therefore quadratic
in turn length and can exceed the model cost.

`agent_end` is at the end by construction, so bound the read:

```bash
tail -c 131072 "$OUT" | grep -qE '"agent_end"'
```

**Second-order cost, worth fixing for its own sake:** log size reads as a runaway
to anyone watching progress. It is not a progress signal. During RV-324 this
misled the orchestrator into killing four in-flight raisers; two had already
written their output. Any tooling that surfaces spawn progress by log size will
make the same mistake — surface typed events instead.

## 2. The reap leaves the real `pi` orphaned

The spawn chain is `timeout` → `bwrap` → pi wrapper → `libexec/pi`, so the shell's
`$!` is the **`timeout`** process. `kill -9 "$PI"` at `agent_end` fells `timeout`
and leaves the grandchild `pi` alive holding its API session. Observed: two
orphans still running **16 minutes** after both had written their findings and
reported `agent_end`. `bwrap --die-with-parent` did not cover it.

Worse than a leak: a background `wait` on the spawning script does not return
until the orphan dies, so completion notifications arrive minutes late and the job
looks like it is still running.

Fix (as applied in `pi-review.sh`): `set -m` before the background launch so it
forms its own process group, then `kill -9 -"$PI"` to signal the group, falling
back to the bare pid.

> **`setsid` is NOT available in this jail.** It is the obvious fix and it fails
> with `setsid: command not found`, breaking the spawn outright. Whoever does this
> pass will reach for it — do not.

A belt check is also worth carrying over: after the reap, warn if any process
still holds this spawn's `--session-dir`.

## 3. `pi-scout` and `pi-research` are wired to the opposite models from their docs

| script | header comment claims | actually resolves |
|---|---|---|
| `scripts/pi-scout` | `deepseek-v4-flash` | `.pi/agents/scout.md` → **`deepseek-v4-pro`** |
| `scripts/pi-research` | `deepseek-v4-pro` | `.pi/agents/researcher.md` → **`deepseek-v4-flash`** |

The mapping is inverted at both ends. `CLAUDE.md` and `AGENTS.md` both repeat the
wrong version ("`pi-scout` (quicker, cheaper)" / "`pi-research` (smarter)"), so an
agent routing a cheap mechanical thread to `pi-scout` silently pays for the pro
model, and a judgement thread sent to `pi-research` gets flash — the exact inverse
of the documented intent, on the two scripts the project tells agents to prefer
over subagents.

Also: `.pi/agents/researcher.md` carries `name: scout` in its frontmatter, a
copy-paste from `scout.md`.

Fix is a decision, not just an edit — either correct the frontmatter to match the
docs, or correct the docs (and `CLAUDE.md` / `AGENTS.md`) to match the frontmatter.
Pick deliberately; the names imply the former.

## Scope note

`scripts/pi-review.sh` already carries fixes 1 and 2 and needs no change. It does
not use the agent profiles, so 3 does not affect it.

## Links

- Fixes proven in `scripts/pi-review.sh` (commits `a69274c8`, `299b6a55`).
- Friction observations: `019fb0ef-fb66-7d02-aea8-227b30a20776` (model mapping),
  `019fb124-c831-7132-9619-2c13061aae6c` (poll), `019fb137-d615-7ae2-8476-e60d05e294b6` (reap).
- Operational context for the review fan-out that surfaced these: **IMP-024**.
