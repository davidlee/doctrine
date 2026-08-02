# CHR-051: Repair the pi spawn scripts' poll, reap and model mapping

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Three independent defects in the `pi` spawn/agent surface, all found and fixed in
`scripts/pi-review.sh` during RV-324 (2026-07-30). The fixes are proven there;
this item is to carry them back to the scripts still running the originals.

> **Read §1 and §2 from `scripts/pi-review.sh`, not from this card's first
> draft.** The original §1 prescription below has been *corrected in place*
> (2026-08-02) — as first written it reproduced the bug. The script is the
> reference implementation; where the two disagree, the script wins.

**Which scripts, and which arm.** §1 and §2 are `--mode rpc` defects and apply
to exactly three scripts: `scripts/pi-spawn.sh`, `scripts/pi-spawn-confined.sh`,
and **`scripts/pi-respawn-nofork.sh`** (which carries the identical original
poll/reap and was missed by this card's first draft). They do **not** apply to
`scripts/pi-scout` / `scripts/pi-research`: those run `pi --print`, which
self-exits on stdin EOF and needs no fifo, poll, or reap — their termination
defect is ISS-266 and its fix is `</dev/null` plus a `timeout`. Do not port the
rpc machinery onto the `--print` arm. §3 applies only to the `--print` arm.

## 1. The completion poll re-reads the whole growing stream

`scripts/pi-spawn-confined.sh`, `scripts/pi-spawn.sh` and
`scripts/pi-respawn-nofork.sh` detect completion with

```bash
grep -qE '"(type|event)":"agent_end"|"agent_end"' "$OUT"
```

inside a 2-second loop. `pi --mode rpc` re-serializes accumulated conversation
state on every event, so `$OUT` grows super-linearly — **50–150MB for one ordinary
turn** (measured 54 / 56 / 130 / 139 / 152MB across seven review turns; none was a
loop, all were legitimate reasoning). The poll's I/O cost is therefore quadratic
in turn length and can exceed the model cost.

### Corrected prescription (2026-08-02) — the first draft's was wrong twice

This card originally prescribed:

```bash
tail -c 131072 "$OUT" | grep -qE '"agent_end"'    # WRONG — do not port this
```

on the reasoning that "`agent_end` is at the end by construction". It is not,
and that window is far too small. `pi --mode rpc` carries the accumulated
conversation state *on the `agent_end` event itself*, so `agent_end` is pushed
arbitrarily far back from EOF as the turn grows: **measured 684,768 bytes from
EOF** on one census turn, 5.2× outside the 128KiB window. A poll written to the
prescription above therefore **never fires on a real turn** — every raiser runs
to the full backstop holding a live `pi` and an open API session, and nothing
warns, because the findings file lands on time and the run looks clean.

The terminal event is **`agent_settled`**, a bare `{"type":"agent_settled"}`
17 bytes from EOF and robust to any window. Match either, so the poll survives
another ordering change. What `scripts/pi-review.sh:133-143` actually runs:

```bash
tail -c 4194304 "$OUT" 2>/dev/null | grep -qE '"(agent_settled|agent_end)"'
```

4MiB at a 2s cadence is still four orders of magnitude below the file. Source:
`scripts/pi-review.sh:118-132` (ISS-293).

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

> **`setsid` is NOT available in this jail.** It is the obvious fix and it fails
> with `setsid: command not found`, breaking the spawn outright. Whoever does this
> pass will reach for it — do not.

### Corrected prescription (2026-08-02) — the reap is four things, not one

This card's first draft described only "`set -m` before the background launch,
then `kill -9 -"$PI"`". `scripts/pi-review.sh` carries **four** elements, and
the three it omits are each load-bearing. Port all four:

1. **`set -m` before the FIRST background job, not just before the pi spawn.**
   `$KEEP` (the fifo-holding subshell) needs its own process group too: `kill -9
   $KEEP` fells only the subshell and **orphans its `sleep $BACKSTOP` child**,
   which inherits the script's stderr and so holds the pipe open for any caller
   that pipes or `wait`s on us. The script then exits promptly and the *caller*
   hangs to the backstop anyway — the same symptom, relocated. Observed: sleeps
   at `ppid=1`, 29 minutes after their raiser finished (ISS-293). So
   group-kill both: `kill -9 -"$KEEP" || kill -9 "$KEEP"`.
   (`pi-review.sh:63-73`, `:144-149`.)
2. **`wait "$PI" "$KEEP"` after the kills.** `kill -9` only *queues* the signal.
   Without the wait, both — this shell's own children — linger as zombies that
   `ps` still lists, and the belt check in (4) reports a survivor that is
   already dead (ISS-294). (`pi-review.sh:150-153`.)
3. **A bounded settle loop before believing `ps`.** The grandchildren
   (`timeout` → `bwrap` → wrapper → `pi`) are not this shell's children and
   cannot be waited on, so give the group kill a window to land — 5 × 0.2s,
   breaking early. (`pi-review.sh:156-162`.)
4. **The belt check, plus elapsed-vs-backstop reporting.** After the reap, warn
   if any process still holds this spawn's `--session-dir`. And print
   `reason=… after ${ELAPSED}s of ${BACKSTOP}s`, warning explicitly when the
   reason is `timeout`: **elapsed-vs-backstop is the only signal that
   distinguishes a clean early finish from a silent full-backstop burn** — the
   output file lands either way, so without it the operator has to catch the
   burn in `ps`. This is what made §1's defect invisible for as long as it was.
   (`pi-review.sh:164-177`.)

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

### Re-verified 2026-08-02 — still live, and the fix is smaller than it looks

Confirmed unchanged, and two facts narrow the work:

**(a) The inversion is PROJECT-LOCAL, and a correct mapping already exists next
to it.** Both scripts resolve `$PROJECT_ROOT/.pi/agents/*.md` — *not*
`~/.pi/agents/`. The home profiles carry the mapping the docs describe:

| profile | `~/.pi/agents/` | `/workspace/doctrine/.pi/agents/` |
|---|---|---|
| `scout.md` | `deepseek-v4-flash` ✅ | `deepseek-v4-pro` ❌ |
| `researcher.md` | `deepseek-v4-pro` ✅ | `deepseek-v4-flash` ❌ |
| `researcher.md` `name:` | `researcher` ✅ | `scout` ❌ |

So the decision this item asks for is already made everywhere except the two
project copies, and "correct the frontmatter to match the docs" is the option
the rest of the system has taken. Do not chase the home profiles or the docs.

**Corrected 2026-08-02:** this paragraph originally ended "Fix is two `model:`
lines", which contradicts (b) directly below it and understates the work. It is
two lines *for `scout.md` only* — one `model:` line, since the rest of the
project `scout.md` is a deliberate project-local variant (its description and
body differ from the home profile's and are the better prose; keep them). For
`researcher.md` it is the whole file: see (b).

**(b) It is not just `name:` that was copy-pasted.** The project
`researcher.md`'s entire frontmatter is `scout.md`'s — `name`, `description`
(*"Evidence-based repo analyst — reads code and structure to inform design
decisions"*), `tools`, and `defaultProgress` — with only `model` differing, and
differing *wrongly*. A researcher advertising `tools: read, write, grep, find,
ls, bash` has **no `web_search` / `fetch_content`**, which the home profile
does. So `pi-research` is not merely running the wrong model: it cannot do
the web research the script's own usage examples (*"What's new in Rust 2025?"*,
*"Compare Bun vs Node 2025"*) promise. That is a bigger defect than the model
mapping and nothing else records it.

Sharpened 2026-08-02: `diff .pi/agents/scout.md .pi/agents/researcher.md`
returns **one line** — the `model:`. The project `researcher.md` is not merely
a frontmatter copy but a *whole-file* copy of `scout.md`, body prose included,
with the model changed to the wrong one. There is no researcher profile in this
project; there is a second scout that costs less and is advertised as smarter.

**But (b)'s conclusion about `web_search` is wrong — corrected on the way to
fixing it.** (b) reasoned from the script's usage examples (*"What's new in Rust
2025?"*) that `pi-research` must be a web researcher and that the missing
`web_search` / `fetch_content` was the larger defect. It is the other way round:
those examples are themselves copy-paste from the *home* profile, of a piece
with the rest of this card. In this project both runners are repo-facing —
`.doctrine/governance.md` "Research agents" scopes `pi-research` to *"governance
applicability or when a thread needs judgement"* and has both piping stdout to
`.doctrine/slice/NNN/research/raw/`, and no skill or caller in the tree invokes
either for web research (`grep -rn 'pi-scout\|pi-research' .agents .claude
install` → no hits; `CLAUDE.md` and `governance.md` are the only references).

Flagged as an **inference, not a quotation**: `governance.md` does not say
"repository, not web" in as many words — it is silent on web access, and the
repo role is read off the stated purpose plus the absent callers. If that read
is wrong the correction is cheap and local (swap the profile's `tools:` line
and restore the web examples), but the whole-file rewrite was authored on it.

Resolved by giving the project a real researcher persona — judgement,
governance applicability, verified-vs-inferred discipline — on repo tools
(`read, write, grep, find, ls, bash`) and `deepseek-v4-pro`. `output:` is
deliberately NOT carried over from the home profile: these runners return on
stdout, and an `output:` key would divert the brief to a file.

### Root cause of the drift, and the residual — `.pi/` is gitignored

`.gitignore:6` ignores `/.pi`, so `.pi/agents/{scout,researcher}.md` are
**untracked local state**. This is why the mapping could invert against the docs
with nothing to catch it, and it leaves two live consequences:

1. **The fix above is per-machine.** It is applied in this working tree and
   cannot be committed. A fresh clone gets neither the corrected models nor the
   researcher persona.
2. **`pi-scout` / `pi-research` are broken by default on a fresh clone.** Both
   hard-exit with `ERROR: agent file not found: $PROJECT_ROOT/.pi/agents/*.md`
   — and these are the runners `CLAUDE.md` tells agents to prefer over harness
   subagents.

The two scripts are tracked and load-bear on these profiles, so the profiles
are not really local config. Un-ignoring `.pi/agents/` (keeping the rest of
`.pi/` ignored — it holds session and auth state) is the change that makes this
item stay fixed. Flagged for a decision rather than taken: editing `.gitignore`
to start tracking a directory is a repo-policy call, not a chore fix.

**A negative check on the home profiles is a false negative for this item** —
they read correct, and that is not the file the scripts load. Positive control:
`head -7 /workspace/doctrine/.pi/agents/researcher.md`.

## Constraint on the §1/§2 backport — do not hoist the `bwrap` block

`src/worktree/jail.rs` (`pi_spawn_core_tokens`, ~`:534`, exercised at `:1165`)
**scrapes `scripts/pi-spawn-confined.sh`'s `PREFIX=( … )` region** and asserts
byte-parity with the Rust `bwrap_core_argv()` builder. The §1/§2 backport does
not touch that region and is clear — but it means the obvious follow-on ("these
four scripts share one poll/reap, DRY them into a common runner") breaks that
test the moment it moves the `bwrap` flags. A prior worker did exactly this; see
[[mem.pattern.dispatch.pi-spawn-parity-scraper-couples-to-script-format]].
Keep the backport a literal per-script edit.

## Scope note

`scripts/pi-review.sh` already carries fixes 1 and 2 and needs no change — it is
the reference implementation to port *from*. It does not use the agent profiles,
so 3 does not affect it.

Sibling items on the same surface, all `--print` arm: **ISS-266** (stdin left
open, no timeout), **IMP-322** (no `--session-dir`, so sessions land in a
possibly-read-only `$HOME/.pi/agent/sessions`). §3 and both of those touch the
same two scripts and are best done in one pass. **IMP-104** is the umbrella
"one general non-dispatch runner" item — read the constraint above before
acting on it.

## Links

- Fixes proven in `scripts/pi-review.sh` (commits `a69274c8`, `299b6a55`).
- Friction observations: `019fb0ef-fb66-7d02-aea8-227b30a20776` (model mapping),
  `019fb124-c831-7132-9619-2c13061aae6c` (poll), `019fb137-d615-7ae2-8476-e60d05e294b6` (reap).
- Operational context for the review fan-out that surfaced these: **IMP-024**.
