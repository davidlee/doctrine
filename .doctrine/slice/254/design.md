<!-- doctrine:section sec-1 -->
# Design SL-254: Collapse dispatch onto one subprocess arm

<!-- Reference forms (glossary.md § reference forms): entity ids padded
     (SL-254, ADR-011, DEC-207); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§8), INV-1 (§5.5). -->

## 1. Design Problem

Dispatch has two worker-spawn shapes. The codex/pi shape is a confined
subprocess: `worktree fork --worker`, a bwrap wrap around the harness exec, and
an orchestrator that imports the worker's working-tree diff. The claude shape is
an in-session `Agent` tool call, and everything around it — a disk marker for
identity, a `SubagentStart` hook to write that marker's sibling allowlist, a
`PreToolUse` wall standing in for OS confinement, and a gated `worker_commit`
MCP tool letting the worker commit through a wall it cannot commit past — exists
only because `ADR-011` concluded that for claude *"the only viable backend is
the in-session `Agent` tool"*.

`EVD-023` falsified the billing half of that conclusion: `claude -p` draws from
subscription usage limits. `DEC-202` took the consequence — run the claude
worker as a confined subprocess like the other two, and delete the apparatus
that existed only because it could not be one.

### The four mechanisms are one workaround

| mechanism | the `ADR-011` clause it exists for | what replaces it |
|---|---|---|
| disk marker as sole worker identity | `D2`/`D4` — `Agent` has no env channel | `--setenv DOCTRINE_WORKER 1`, already in the bwrap prefix |
| `SubagentStart` nominate / `SubagentStop` denominate | `D3` — the wall needs an orchestrator-exemption list and `Agent` has no exec wrapper | nothing: the read side dies with the wall (`DEC-205`) |
| `worktree pretooluse` confinement wall | `D3` — *"OS confinement: none — `Agent` is not a subprocess to wrap"* | nested bwrap, as on pi |
| gated `worker_commit` MCP tool | a hook-confined in-session worker cannot self-commit | nothing: the orchestrator's incumbent import already does this on pi (`DEC-204`, `DEC-213`) |

Restore the subprocess and all four causes disappear, rather than their symptoms
being managed. The work is therefore **substitution and deletion, not
construction**: the target shape runs in production today on the pi arm.

### What this design has to produce

1. A confined `claude -p` spawn at parity with `scripts/pi-spawn-confined.sh`,
   including the macOS `sandbox-exec` sibling, and a typed hand-back
   (`DEC-215`).
2. A deletion boundary drawn precisely enough to execute: which modules die,
   which survive with legs removed, and in what order (`DEC-206` fixes the
   order — re-home before delete).
3. Worker identity as a property of the process, not of a tree (`DEC-207`).
4. One arm, failing closed where confinement is unavailable (`DEC-208`).
5. A governance landing — one `REV` over `ADR-011`, `ADR-006`, `SPEC-021`,
   `SPEC-012` plus a hand source-anchor sweep (`DEC-211`). This is the larger
   half of the slice, not a tail.

### What this design must NOT produce

Clone provisioning, worker self-commit, fetch-from-clone in place of the import
belt, and the branch-point guard's re-homing onto fetched refs are **`SL-255`'s**
(`DEC-213`). Workers here stay on **linked worktrees** with the **incumbent
import transport**. The claude arm becomes a pi arm — no more and no less.

The confined-orchestrator mediated-write tier (`REQ-335`) stays pending; the
capsule contract (`ADR-020`, `SPEC-030`) is untouched; solo `/worktree`
isolation is untouched.

<!-- doctrine:section sec-2 -->
## 2. Current State

### 2.1 The two arms, as they run today

```
                         ORCHESTRATOR (claude session, coord worktree)
                                        |
            +---------------------------+---------------------------+
            |                                                       |
      CLAUDE ARM (in-session)                            CODEX/PI ARM (subprocess)
            |                                                       |
  dispatch arm-spawn --base B --slice N --phase PH         scripts/pi-spawn-confined.sh
    writes <coord>/.doctrine/state/dispatch/spawn/{base,jail.toml}    |
            |                                             doctrine worktree fork
  cd <spawn dir>   (positional discriminator)                --base B --branch BR
            |                                                --dir D --worker
  Agent{subagent_type: dispatch-worker, isolation: worktree}          |
            |                                             PREFIX = bwrap --ro-bind / /
  harness WorktreeCreate hook                                --bind $HOME/.pi ...
    -> doctrine worktree create-fork                          --bind $D --chdir $D
       Fork arm: fork at base, run_provision,                 --die-with-parent
       provision_jail_policy, provision DispatchRecord,       --setenv DOCTRINE_WORKER 1
       write_marker                                          (macOS: worktree jail-prefix
            |                                                 emits a sandbox-exec prefix)
  worker runs IN-SESSION, confined by                                 |
    PreToolUse(Bash)       -> nested-bwrap command rewrite    timeout $BACKSTOP "${PREFIX[@]}"
    PreToolUse(Edit|Write) -> realpath-subset-of-cwd deny       pi --mode rpc --no-extensions
    PreToolUse(Agent)      -> nomination gate                          |
    PreToolUse(Workflow)   -> blanket deny                     typed agent_end on the
            |                                                  RPC event stream
  worker calls worker_commit MCP tool                                 |
    (server-side, unconfined: 6 belts, then git commit)      orchestrator imports the
            |                                                 WORKING-TREE DIFF
  orchestrator: verify-worker -> dispatch_import                      |
            |                                                        |
            +---------------------------+---------------------------+
                                        |
                        funnel: import -> verify -> conclude -> reap
                          (identical on both arms, `dispatch next`)
```

The funnel below the dashed line is already arm-agnostic. Everything above it is
duplicated, and the claude column is the duplicate that has to justify itself.

### 2.2 The cut between reusable core and hook shell already exists

`src/worktree/jail.rs` (2218 lines) holds the pure confinement core:
`bwrap_core_argv` (`:537`), `bwrap_argv` (`:561`), `validate_policy` (`:451`),
`select_jailer` (`:664`), `resolve_with_policy` (`:792`).
`src/worktree/pretooluse.rs` (1113 lines) is the hook shell over it.
`src/worktree/jail_prefix.rs` (190 lines) is the *subprocess arm's* shell over
the same core — its own header calls it *"the subprocess (pi) arm's analog of the
claude `PreToolUse` hook"* — and it emits the NUL-delimited bwrap prefix the
spawn script consumes.

Four primitives sit on the wrong side of that line. `jail_prefix.rs:39-49`
imports them **from `pretooluse.rs`**, cfg-split by platform:

| platform | symbols imported from the dying module | site |
|---|---|---|
| non-macOS | `REASON_NO_BWRAP`, `have_bwrap` | `pretooluse.rs:83`, `:314` |
| macOS | `REASON_PROFILE_WRITE_FAILED`, `write_seatbelt_profile` | `pretooluse.rs:91`, `:361` |

Deleting `pretooluse.rs` breaks the **surviving** arm unless these move first.
This is the slice's single most likely breaks-unexpectedly moment, and its
mitigation is pure sequencing (`DEC-206`).

`jail_prefix.rs` deliberately does **not** use `probe_backend`, whose macOS
branch reads the per-arming disk policy at `JAIL_SUBPATH`
(`.doctrine/state/dispatch/jail`, `create.rs:258`). So the disk-policy chain —
`arm-spawn`'s `jail.toml`, `provision_jail_policy` (`create.rs:332`),
`probe_backend` — belongs entirely to the dying arm, and the inline-policy path
`jail_prefix.rs` uses is the one that survives.

### 2.3 Worker identity today

`describe_mode` (`marker.rs:88`) is a three-input truth table:

```rust
let marker_leg = is_linked && marker_present;      // marker.rs:89
let cause = match (marker_leg, env_set) {
    (true, true)  => Cause::Both,
    (true, false) => Cause::Marker,
    (false, true) => Cause::Env,
    (false, false) => Cause::None,
};
StatusLine { refused: marker_leg || env_set, cause, is_linked }
```

`ADR-011` `D1` justifies the marker as primary — *"identity rides disk, the one
medium every harness has, not a process env seam, which not every harness
has"* — and `D3` grades `DOCTRINE_WORKER` as a codex/pi-only optimisation. Once
every arm is a subprocess, every arm has the env seam.

Census of the marker surface (verified 2026-08-13, recorded in `DEC-207`):

- **writers**: `fork.rs:192` (`fork --worker`), `land.rs:173` region, plus
  `subagent.rs:230` (`run_stamp_subagent`) and `worker_commit.rs:175` (the
  clear/restore dance around its own commit). `gc.rs` and `import.rs` import
  the symbols and use none of them, masked by a module-wide
  `expect(unused)` at `gc.rs:1` left from `SL-116 PHASE-02` (`CHR-062`).
- **readers**: `resolve_mode`/`describe_mode`; `inventory.rs:199` (the marker
  column) and `:73` (the `Cause` override); `land.rs:173`'s `bears_marker`,
  which gates `LandRefusal::DispatchFork` at `land.rs:127` and is the **only**
  marker read that asks about *another* tree rather than about self;
  `mcp_server/tools.rs:1389` and `commands/observation.rs:509`
  (`repository_context`), neither of which carries the `is_linked` conjunct —
  so a marker on the primary tree is honoured there while `resolve_mode` calls
  it inert.
- **derived surface**: `Cause`'s four variants, `is_stale_marker`,
  `worktree status --assert`'s stale exit, `run_marker_clear` (`marker.rs:209`)
  with its `--operator` accident-fence, and `DUAL_CAUSE` (`marker.rs:171`).
  `ISS-028` — worker-marker confinement refuses CLI writes in a stamped fork —
  lives here.

Positive coordination identity already ships marker-free:
`classify_worktree_role` (`shared.rs:77`) requires linked **and** an all-numeric
`dispatch/<NNN>` branch, with live callers at `dispatch whereami`
(`dispatch.rs:862`, `:1308`) and `review`'s root guard (`review.rs:2362`).
`inventory.rs` carries a parallel implementation of the same classifier.

### 2.4 The claude arm's fork-provisioning is a substitute for `worktree fork`

`fork.rs:216-246` documents the **durable fork binding** as *"the SUBPROCESS
arm's binding path"*: `worktree fork --worker --slice N --phase PHASE-NN` binds
the fork to its `(slice, phase)` when `--worker`, both flags, and a `dir` under
`<coord>/.worktrees/<name>` all hold. `DispatchRecord` / `ForkBinding`
(`dispatch_record.rs:31-95`) are shared machinery: `dispatch_import`'s
heal-forward names the funnel row from the binding, and `require_binding` turns
an unbound fork into the typed `unprovable-fork` refusal rather than a guess.

The claude arm cannot run a fork command before its worker exists, so
`arm-spawn` (`dispatch.rs:691`) plus `create-fork`'s Fork arm
(`create.rs:583`, `CreateAction::Fork`) reproduce the same three writes from a
hook. `run_create_fork_and_record` (`dispatch.rs:6931`) additionally lands the
Class-2 `Spawn` funnel row, because on this arm nothing else knows both the
harness-assigned fork name and the base.

That is a parallel implementation of `worktree fork --worker`, forced by the
`Agent` tool. It has no reason to exist once the worker is a subprocess.

### 2.5 Routing and config

- `claude-force-subprocess-dispatch` is **dead config in the binary**: defined
  at `dispatch_config.rs:61`, defaulted at `:109`, exercised only by its own
  unit tests and one `dtoml.rs:168` round-trip. **No production read.**
- Arm routing is entirely skill-level: `/dispatch` step 4 reads the config key
  and then routes on `CLAUDECODE=1`
  (`plugins/doctrine/skills/dispatch/SKILL.md:26-32` — the authored source; `.agents/` is the gitignored install projection).
- `REQ-288` and `SPEC-021` responsibility 2 state the env-marker as *"`.claude/`
  presence"*, which is factually wrong today (`ISS-347`); the router tests
  `CLAUDECODE=1`.
- `dispatch next`'s `NextKind::Spawn` prose (`dispatch.rs:6702-6707`) names both
  arms and is deliberately arm-agnostic — one of the few binary-tier strings
  that has to change.

### 2.6 Installed hook set

`.claude/settings.json` carries **eleven entries across five events**, asserted
exactly by `e2e_claude_install.rs` and seeded from `boot.rs:1257-1263` (seven
`HookSpec`s):

| event | entries | fate |
|---|---|---|
| `PreToolUse` | 2 × `memory surface` (`Read|Edit|Write`, `Bash`) | **keep** — unrelated capability |
| `PreToolUse` | 4 × `worktree pretooluse` (`Bash`, `Edit|Write`, `Agent`, `Workflow`) | delete |
| `SessionStart` | 2 (`prompt resolve`, `memory sync`) | keep |
| `SubagentStart` | 1 × `worktree nominate` | delete |
| `SubagentStop` | 1 × `worktree denominate` | delete |
| `WorktreeCreate` | 1 × `worktree create-fork` | keep the **entry**; delete its Fork arm (§5.6, `D1`) |

Six entries go, not four — the count assertion moves by the nominate/denominate
pair as well as the four wall matchers (`DEC-205`, `DEC-212`).

<!-- doctrine:section sec-3 -->
## 3. Forces & Constraints

### 3.1 Governance that binds

Read directly this stage rather than through the research round's quotations;
`DEC-211` carries the site-by-site detail.

- **`ADR-011`** (harness-agnostic orchestrator spawn interface) — the ADR this
  slice falsifies, at **eight** regions: Context (21-25), `D1` (39-56), `D2`
  (58-74), `D3`'s table (82-92), `D4` (94-111), `D6` (146-207), Consequences
  (248-259), Verification (265-275). `D2` is the core contract — its *"claude's
  `Agent` path has no worker env channel and cannot consume it"* is exactly what
  a confined `claude -p` subprocess falsifies — and `D6` is sixty lines of
  fail-closed altitude resting on mechanisms this slice deletes, whose *"not
  fail-closable"* conclusion inverts under `DEC-208`. `D5` (113-144) and `D7`
  (209-232) are recorded as considered and deferred.
- **`ADR-006` §D2b** (worktree posture) — its main body, *"the harness does not
  confine workers to their worktree"*, becomes false for the claude dispatch
  path. A new note on the existing `SL-181` note pattern (line 143), re-cut
  rather than appended, plus the second correction at line 308: D2b still names
  `IMP-065` as *"the real positive-marker close"*, and `IMP-065` was closed
  **obsolete** on 2026-07-02 via `REV-018` — superseded by confinement, not
  delivered. A live forward-reference to a close that will never arrive.
- **`SPEC-021`** — `REQ-288` retires (its premise is a choice between two arms);
  `REQ-291` **rewrites, not retires** (an altitude contract still exists, with
  one column). Responsibility 16 (arm routing) and the **first half** of
  responsibility 19 (enforcement altitude) change. Responsibility 15's funnel
  cadence does **not** change — the belt does not re-home, because the incumbent
  import transport is retained (`DEC-213`).
- **`SPEC-012`** — responsibility at line 18 loses the *stamp* from
  `fork = create + provision + stamp + emit per-wt env`. *"import as the belted
  dispatch funnel"* is unchanged: import survives.
- **`POL-002`** (platform independence from host-project conventions) —
  assessed and **satisfied, not strained**. Deleting harness-specific hook code
  from the binary and keeping spawn mechanics at the script tier reduces
  host-convention coupling. The argv builder stays in the binary: `POL-002`
  forbids host conventions in the engine, not mechanism.
- **`STD-001`** (no magic strings) — governs the thoroughness of the
  `claude-force-subprocess-dispatch` deletion (struct field, `dtoml.rs`
  round-trip, `doctrine.toml` entry, commented example) and of the belt
  constants' single-sourcing.
- **`ADR-012`** (dispatch integration topology) — **not touched**, and this is
  load-bearing: it is what keeps the `REV` inside the surveyed set. It stays
  untouched only while the transport does not move, which is precisely the
  `SL-254`/`SL-255` boundary.

Checked and **not applicable**: `ADR-020`, `SPEC-030`, `REV-046` and the capsule
programme's evidentiary bar. This is incumbent simplification, not capsule
construction.

### 3.2 Constraints on the change

- **Behaviour preservation.** The codex/pi arm is production and does not move.
  Its existing suites must stay **green unchanged** — achievable as written
  because the claude arm changes *into* the pi arm's shape rather than the pi
  arm changing (`DEC-213` restoring the scope's original gate over `DEC-212`'s
  observable-contract restatement).
- **The confinement control must skip, not pass, without `bwrap`.** A vacuous
  pass is the same fail-open shape the slice is deleting, and it is the one test
  whose failure mode is invisible (`DEC-212`).
- **No new dependency.** `crates/doctrine-control` is Linux-only, outside every
  default test selection (`just capsule-check`), and under active development
  as the capsule programme's crate. Generalise the incumbent prefix; port no
  hardening (`DEC-209`). The hardening delta is filed as `IMP-428`.
- **Subscription credential, wholesale bind.** `ANTHROPIC_API_KEY` would forfeit
  the very billing property `EVD-023` establishes. Bind `$HOME/.claude` the way
  the pi arm binds `$HOME/.pi` — wholesale, read-write, on both the Linux inline
  array and the macOS `--extra-rw` path. Narrowing is deferred (`DEC-210`).
- **No MCP in the worker.** Parity with pi's `--no-extensions`; the orchestrator
  performs every privileged act (`DEC-216`).
- **Two `PreToolUse` hooks are `memory surface`, not confinement.** Deleting
  them would be a silent regression of an unrelated capability.
- **Evidence lands in an authored sink.** The `VA` leg's live dispatch run
  records into `notes.md` or an `EVD`, never the gitignored scratchpad —
  a `VA` criterion over runtime state leaves nothing an audit can re-derive
  (`mem_019fd1d862887d42b7a1f88c28fd28a7`).

### 3.3 Forces in tension

- **Deletion breadth vs. blast radius.** The honest deletion set is larger than
  the scope's list: `arm-spawn`, `create-fork`'s Fork arm, `verify-worker` and
  the `Spawn`-row recorder are all claude-arm-only. Keeping them "just in case"
  leaves a live fork trigger that, with the marker deleted, would mint an
  unmarked and unconfined worktree — strictly worse than deleting it.
- **Mid-cutover posture.** `RFC-025`'s discipline is that every slice but the
  last lands *beside* the incumbent arms. This slice removes one. Accepted
  knowingly (`DEC-202`); softened but not dissolved by keeping the import
  transport (`DEC-213`). Say it plainly rather than claim it away.
- **Line-count vs. risk.** The three claude-arm modules total ~4000 lines, and
  the deletion is nearly mechanical. The *governance* half is smaller in bytes
  and much larger in judgement, because prose does not fail to compile.

<!-- doctrine:section sec-4 -->
## 4. Guiding Principles

**Delete the cause, not the symptom.** Each of the four mechanisms traces to one
clause of `ADR-011` `D3`. The design's job is to remove the clause's premise and
let the mechanisms fall, not to improve them.

**Substitution, not construction.** The target already runs. Every choice
resolves toward *what does the pi arm do*, and a divergence from it has to be
argued for rather than drifted into (`DEC-215`). Where the pi arm's answer is
imperfect — it is — that is a separate slice's problem, not a licence to invent
a third shape here.

**Re-home before delete.** The one ordering that is a design fact rather than an
implementer's preference: the four jail primitives move to `jail.rs` **first**,
while the change is behaviour-preserving and the whole existing suite is its
proof (`DEC-206`). Sequencing is the entire mitigation for the slice's sharpest
breakage risk, so it is stated here and carried into the plan as its own phase.

**Identity is a property of a process.** The disk marker models worker-ness as a
property of a *tree*, and that single mismatch is the origin of the whole
stale-marker error surface. `DOCTRINE_WORKER` has no stale class by
construction: it dies with the process, and it is set by the **same bwrap argv**
that establishes the write floor — one actor, one moment, atomic with
confinement itself (`DEC-207`).

**Confinement is the boundary; a cooperative flag never was.** `ADR-006`'s own
`SL-181` note already concedes it — *"The cooperative marker was never the
fence; the OS floor now is."* `REV-018` retired `IMP-065`'s positive marker on
exactly that argument, and the argument applies verbatim to the negative one.

**A gated tool is privileged only if it sits on the far side of a boundary the
agent is inside.** `worker_commit` relied on this implicitly and `ADR-011` never
states it. Anything reached from *inside* the confinement is exactly as confined
as the agent, so it may be argued on convenience but never on privilege
(`DEC-216`).

**Harness specifics at the script tier; mechanism in the binary.** The bwrap
argv builder, its policy validation and its backend selection are validated
mechanism and stay in `jail.rs`. What harness gets exec'd, and which config
directory gets bound, are script-tier facts. This is what makes the collapse
satisfy `POL-002` rather than strain it (`DEC-206`, `DEC-209`).

**Fail closed, and name the reason.** An environment that cannot confine cannot
dispatch. That is not new behaviour to author: the surviving arm already fails
closed at spawn through `REASON_NO_BWRAP` on Linux and
`REASON_PROFILE_WRITE_FAILED` on macOS (`DEC-208`).

**One thing at a time.** The hardening delta (`IMP-428`), the prefix's permanent
home and macOS parity (`IMP-429`), the narrowed `~/.claude` mount set, and clone
provisioning (`SL-255`) are all real and all deferred. Each was declined on
scope, with a card, not overlooked.

<!-- doctrine:section sec-5 -->
## 5. Proposed Design

### 5.1 System Model

After the collapse there is **one** worker-spawn shape. The claude column of
§2.1 is deleted, not rewritten; the pi column is parameterised by harness.

```
ORCHESTRATOR (any harness, coordination worktree — unconfined, sole writer)
   |
   |  scripts/spawn-confined.sh <harness> <B> <BRANCH> <DIR> <PROMPT> [BACKSTOP]
   |
   +-- doctrine worktree fork --base B --branch BR --dir D --worker
   |        --slice N --phase PHASE-NN            (durable fork binding, §5.3)
   |        => linked worktree at D, DispatchRecord written coord-side
   |
   +-- PREFIX resolution                          (harness parameterises ONE token pair)
   |     Linux  : inline bwrap array
   |              bwrap --ro-bind / / --dev /dev --proc /proc --tmpfs /tmp
   |                    --bind $HOME/<harness-cfg> $HOME/<harness-cfg>
   |                    --bind $D $D --chdir $D --die-with-parent
   |                    --setenv DOCTRINE_WORKER 1
   |     macOS  : doctrine worktree jail-prefix --dir $D --main-root $ROOT
   |                    --extra-rw $HOME/<harness-cfg> --out $D/.tmp/jail.argv
   |              (NUL-delimited sandbox-exec prefix, read back into PREFIX)
   |     empty PREFIX => abort. bwrap/seatbelt unavailable => fail closed (DEC-208)
   |
   +-- timeout $BACKSTOP "${PREFIX[@]}" <harness exec>
   |        pi     : pi --mode rpc … --no-extensions      (typed agent_end)
   |        claude : claude -p --output-format stream-json --strict-mcp-config …
   |                                                      (typed event stream)
   |
   +-- hand-back: typed completion event on stdout; worker leaves an UNCOMMITTED
   |              working-tree delta in D (ro .git — it cannot commit)
   |
   +-- doctrine worktree import --base B --from-worktree D --slice N
   |        classify_import scope belt (hard) -> apply onto B, NON-committing
   |        -> reject-and-halt prove gate
   |
   +-- one coordination commit, then boundary + funnel verbs (unchanged)
```

**Module responsibilities after the change.**

| module | responsibility | change |
|---|---|---|
| `worktree/jail.rs` | the confinement core: argv build, policy validation, backend selection, **plus** the four re-homed primitives | gains `REASON_NO_BWRAP`, `have_bwrap`, `REASON_PROFILE_WRITE_FAILED`, `write_seatbelt_profile`; loses `PRIVILEGED_AGENT_TYPES` and `probe_backend`'s disk-policy branch |
| `worktree/jail_prefix.rs` | the **sole** shell over that core: emit a confinement prefix for a spawn script | imports re-point from `pretooluse` to `jail`; otherwise untouched |
| `worktree/pretooluse.rs` | — | **deleted** (1113 lines) |
| `worktree/subagent.rs` | — | **deleted** (671 lines) |
| `worktree/marker.rs` | worker-mode status rendering over **one** signal | collapses to the env predicate + `worktree status`; the marker file, its ops, `Cause`, `is_stale_marker`, `DUAL_CAUSE` and `run_marker_clear` all go |
| `worktree/create.rs` | provision a benign harness-created worktree | Fork arm deleted; Passthrough arm and `run_provision` survive |
| `worktree/fork.rs` | the **only** worker-fork writer: create + provision + bind | loses the `write_marker` call |
| `worktree/import.rs` | the belted import transport | **untouched** — `classify_import` remains the scope belt's enforcing caller |
| `mcp_server/worker_commit.rs` | — | **deleted** (1495 lines) |
| `scripts/spawn-confined.sh` | the one spawn shape, parameterised by harness | generalised from `pi-spawn-confined.sh` |

### 5.2 Interfaces & Contracts

**5.2.1 The spawn script contract.** One script, one shape, harness as a
parameter. Everything harness-specific reduces to two facts: the config
directory to bind, and the exec line.

```sh
# scripts/spawn-confined.sh <harness> <B> <BRANCH> <DIR> <PROMPT_FILE> [BACKSTOP]
case "$HARNESS" in
  pi)     CFG_DIR="$HOME/.pi"     ;;
  claude) CFG_DIR="$HOME/.claude" ;;   # DEC-210: wholesale, rw, subscription cred
  *) echo "[spawn] unknown harness: $HARNESS" >&2; exit 1 ;;
esac
```

The exec lines, side by side — the only other harness-specific tokens:

```sh
# pi (unchanged, byte-for-byte)
pi --mode rpc --thinking "${PI_THINKING:-off}" --session-dir "$D/.pi-session" \
   --no-extensions --no-skills --no-themes \
   --offline --approve --tools "${PI_TOOLS:-read,bash,edit,write,grep,find,ls}" \
   <"$PI_FIFO" >"$OUT" 2>&1

# claude (new)
claude -p --output-format stream-json \
   --strict-mcp-config \
   --permission-mode bypassPermissions \
   <"$PF" >"$OUT" 2>&1
```

Flag-by-flag justification, each tied to a decision rather than to taste:

| flag | why | record |
|---|---|---|
| `-p` | headless, non-interactive; the whole premise | `DEC-202` |
| `--output-format stream-json` | typed completion event stream at parity with pi's RPC `agent_end`; argv-tier, no code | `DEC-215` |
| *(not)* `--json-schema` | deliberately unspent: nothing consumes a shaped hand-back today; adopting one would be building | `DEC-215` |
| `--strict-mcp-config` with no `--mcp-config` | no MCP at all in the worker, at parity with pi's `--no-extensions`; the orchestrator performs every privileged act | `DEC-216` |
| `--permission-mode bypassPermissions` | the analog of pi's `--approve`: inside the confinement the OS floor is the boundary, so an in-agent prompt buys nothing | `DEC-208`, `DEC-216` |
| *(not)* `--bare` / `ANTHROPIC_API_KEY` | an API key forfeits the subscription billing `EVD-023` establishes, which is the slice's whole authorisation | `DEC-210` |
| *(not)* a narrowed mount set | `../microvm-spike`'s set is the known refinement, not the requirement; its identity-section partial is unproven | `DEC-210` |

**A divergence from pi that is real and should be stated:** `pi --mode rpc` never
self-exits, so the pi arm holds stdin open with a fifo and polls the event stream
for `agent_end` (`scripts/lib/pi-reap.sh`). `claude -p` **exits on completion**.
The claude profile therefore needs neither the fifo, the keepalive subshell, nor
`pi_await_and_reap` — its completion signal is process exit, with the stream-json
result on stdout. This is a simplification, not a gap, and the two profiles
diverge exactly here and nowhere else.

**5.2.2 `worktree fork --worker` — the binding contract, now on both arms.**
Unchanged in code, but it becomes the **only** producer of a worker fork, so its
preconditions become the collapsed arm's preconditions (`fork.rs:216-246`): a
fork binds its `(slice, phase)` iff `--worker`, both `--slice` and `--phase` are
supplied, **and** `dir` resolves under `<coord>/.worktrees/<name>`. An unbound
fork is not a parse failure — it is a fork whose phase cannot be proven, which
`require_binding` turns into the typed `unprovable-fork` refusal. See `OQ-1`:
today's `pi-spawn-confined.sh` passes neither flag.

**5.2.3 `describe_mode` collapses to one input.**

```rust
// before — marker.rs:88
pub(crate) fn describe_mode(is_linked: bool, marker_present: bool, env_set: bool) -> StatusLine

// after
pub(crate) fn describe_mode(env_set: bool) -> StatusLine
```

`StatusLine` loses `cause` and `is_linked` and becomes the single `refused`
bool already on it. There is no second cause to disambiguate: `DOCTRINE_WORKER`
set means *this process is a worker*, and if that is wrong the remedy is to
unset it. That is today's `DUAL_CAUSE` text with its first horn removed
(`DEC-207`).

**5.2.4 Command surface removed.**

| surface | kind | why it goes |
|---|---|---|
| `worktree pretooluse` | CLI (hook) | the wall |
| `worktree nominate` / `worktree denominate` | CLI (hooks) | closed loop with the wall's `Agent`/`Workflow` gate legs; `is_nominated` has no other reader (`DEC-205`) |
| `worktree verify-worker` | CLI | claude-arm-only; its `unstamped` leg reads the marker, and `fork --worker` is now the sole fork producer (`D3`) |
| `worktree marker --clear` | CLI | the stale-marker class dissolves with the marker (`DEC-207`) |
| `worktree status --assert` stale exit | CLI flag behaviour | same |
| `dispatch arm-spawn` | CLI | the claude arm's substitute for `worktree fork --worker` (`D2`) |
| `worker_commit` | MCP tool | the in-session arm's only commit path (`DEC-204`, `DEC-213`) |

**Command surface retained and re-pointed:** `worktree create-fork` keeps its
`WorktreeCreate` entry and its Passthrough arm, which provisions a benign
harness-created worktree and is what solo `/worktree` isolation rides. Only the
Fork arm goes (`D1`).

**5.2.5 The funnel contract is untouched.** One non-merge commit per phase with
`C^ == B`, the two-tier scope belt's refusal set and tokens, report-and-halt on
any breach, and the phase state machine's transitions all stay exactly as they
are. `classify_import` (`import.rs:147`) keeps its single-sourced constants from
`import.rs:24` and remains the belt's enforcing caller, so `INV-2`'s posture —
a worker cannot skip a belt — does **not** change (`DEC-213`, which is why this
item left `DEC-211`'s `REV`).

### 5.3 Data, State & Ownership

**What worker identity is, after.** One datum: `DOCTRINE_WORKER=1` in the
worker process's environment, set by the same bwrap argv that establishes the
write floor. It has no persistent representation, no staleness class, and no
operator cure verb, because it dies with the process.

**Runtime state deleted.**

| path / datum | owner today | fate |
|---|---|---|
| `<wt>/.doctrine/state/dispatch/worker` (the marker) | `fork --worker`, `create-fork`, `run_stamp_subagent` | deleted |
| `<root>/.doctrine/state/orch-allowlist.txt` | `nominate` / `denominate` (`subagent.rs:422`) | deleted |
| `<coord>/.doctrine/state/dispatch/spawn/{base,jail.toml}` | `arm-spawn` (`dispatch.rs:691`) | deleted |
| `<coord>/.doctrine/state/dispatch/jail/<name>.toml` | `provision_jail_policy` (`create.rs:332`) | deleted |

**Runtime state retained.** `<coord>/.doctrine/state/dispatch/record/<name>.toml`
— the `DispatchRecord` with its `ForkBinding` — survives untouched. It is shared
machinery whose subprocess-arm writer is `fork.rs`, and `dispatch_import`'s
heal-forward names funnel rows from it. Deleting the claude arm's *substitute*
writer does not touch the record itself.

**Ownership after the change.** Every write to authored state remains the
orchestrator's. The worker's only outputs are files inside its own worktree —
which the OS enforces, not a hook — plus a typed event stream on stdout. That is
strictly narrower than today's claude worker, which could reach the coordination
tree whenever the fail-open `PreToolUse` wall failed open.

### 5.4 Lifecycle, Operations & Dynamics

Per phase, on every harness:

1. **Pre-spawn.** `doctrine check prove` on the base (unchanged, once per batch).
   Capture `B = git rev-parse HEAD` from the coordination tree.
2. **Fork.** `worktree fork --base B --branch dispatch/<name> --dir
   <coord>/.worktrees/<name> --worker --slice N --phase PHASE-NN`. Fails
   closed; the orchestrator halts on non-zero.
3. **Confine.** Resolve `PREFIX`. Linux takes the inline array; macOS shells
   `worktree jail-prefix`. An empty `PREFIX` aborts (the existing fail-closed
   guard). No `bwrap` ⇒ `REASON_NO_BWRAP`; no writable seatbelt profile ⇒
   `REASON_PROFILE_WRITE_FAILED`. **There is no unconfined fallback**
   (`DEC-208`).
4. **Spawn.** `timeout $BACKSTOP "${PREFIX[@]}" <harness exec>`, prompt on
   stdin, stdout to `$OUT`.
5. **Hand back.** claude: process exit + stream-json result. pi: `agent_end` on
   the RPC stream. Either way the delta is an **uncommitted working tree**.
6. **Import.** `worktree import --base B --from-worktree D --slice N` — the
   `classify_import` belt as a hard pre-apply gate, apply onto `B`
   non-committing, then the reject-and-halt prove gate. Any refusal is
   report-and-halt.
7. **Commit + record.** One commit on the coordination branch; `slice
   record-delta <SL> PHASE-NN --commit <S>` with `S` pinned at the code commit.
8. **Verify / conclude / reap** — unchanged.

**Failure modes and where they surface.**

| failure | today (claude arm) | after |
|---|---|---|
| host cannot confine | the wall silently fails open — `PreToolUse` denies are data, and a missing binary means no deny at all | spawn aborts with a named reason before the worker exists |
| worker writes outside its tree | denied by hook, if the hook ran | `EROFS` from the kernel |
| worker cannot commit | it commits through `worker_commit`'s six belts | it does not commit; the orchestrator imports the diff |
| stale marker left behind | `worktree status --assert` + `marker --clear --operator` | unrepresentable |
| env leaked into a non-worker process | `DUAL_CAUSE` names two horns | one horn: unset it |

### 5.5 Invariants, Assumptions & Edge Cases

**INV-1 — worker-ness is a property of the process.** `refused ==
env_worker_set()`, everywhere, with no tree-shaped input. The orchestrator is
never *identified* as the orchestrator; it is simply not-refused. Removing one
of two positive legs cannot convert a not-refused into a coordination claim,
because no reader anywhere concludes *this is the coordination tree* from
marker absence (`DEC-207` rationale (1); `mem_019fa94118a37c33ab54b06dfe4b1131`).

**INV-2 — a worker still cannot skip a belt.** `classify_import` survives as the
scope belt's enforcing caller, so the posture is unchanged and this is *not* a
governance-visible change (`DEC-213`).

**INV-3 — the confinement prefix is the boundary.** `--ro-bind / /` plus
`--bind $D $D` is the write floor; every other guarantee in this design rests on
it rather than on cooperation.

**INV-4 — one coordination commit per phase, `C^ == B`.** Untouched.

**Edge case: `land.rs:173`'s cross-tree marker read.** `bears_marker` is the
only marker read that asks about *another* tree, and it gates
`LandRefusal::DispatchFork` (`land.rs:127`). Env cannot answer it — env
describes *this* process. Substitute the branch-shape role classifier
(`shared.rs:77`) returning `"fork"`. This is **strictly stronger**: it fires
whether or not the fork was ever stamped, catching the unstamped-worker case
`ADR-011` `D6`/`M2` confesses (`DEC-207`).

**Edge case: `inventory.rs`.** Dropping the marker column (`:199`) and the
`Cause` override (`:73`) makes role fall through to branch shape, which
**cannot** degrade to `Coordination` — that still requires an all-numeric
`dispatch/<NNN>` suffix (`inventory.rs:80`). While there, fold
`inventory.rs::classify_worktree` into `shared.rs::classify_worktree_role`:
they are a parallel implementation carrying two `dispatch/` prefix tests and two
numeric-suffix tests (`STD-001`).

**Edge case: `repository_context`.** `mcp_server/tools.rs:1389` and
`commands/observation.rs:509` drop the marker disjunct. An inconsistency
dissolves with it — neither site carries the `is_linked` conjunct, so today a
marker on the primary tree is honoured there while `resolve_mode` calls it inert.

**Edge case: `~/.claude.json`.** `$HOME/.pi` is a directory, so binding it
wholesale covers everything pi writes. Claude also writes `~/.claude.json`, a
**sibling file** outside `$HOME/.claude`. Under `--ro-bind / /` it is readable
but not writable, so the claude profile likely needs a second bind. Named here
rather than assumed; carried as `OQ-3`.

**Edge case: `--output-format stream-json` and `--verbose`.** Some `claude`
builds require `--verbose` alongside `stream-json` under `-p`. Verify against
the installed binary at execute; it is an argv fact, not a design choice.

**Assumption `A2` (narrowed to its hand-back leg).** `claude -p` reaches the
worker's needs. Tool-surface scoping, permission modes and structured output all
exist as flags (research thread 3, confirmed). What was ambient under the
`Agent` tool — inherited auth, inherited MCP, a typed subagent return — is
answered by `DEC-210` (subscription credential, wholesale bind), `DEC-216` (no
MCP at all) and `DEC-215` (stream-json) respectively. The residue is
provisioning, not capability.

**Assumption: `create-fork`'s Passthrough has a live non-dispatch consumer.**
The `WorktreeCreate` entry is retained on that basis (`D1`). If it turns out
nothing creates harness-side worktrees in this repo, the entry is dead too —
cheap to confirm at execute, and it changes a deletion count, not a design.

### 5.6 Code Impact

Paths and intended change. This is the set the design-target selectors record.

**Authored vs projected.** Skills are cited at their authored source under
`plugins/doctrine/skills/`; `.agents/skills/` is the gitignored install
projection and is never edited directly. `.claude/settings.json` is the
exception — it is tracked, and `doctrine install` reconciles it against
`boot.rs`'s `HookSpec` registry, so both sides move together.

**Delete outright**

| path | lines | note |
|---|---|---|
| `src/worktree/pretooluse.rs` | 1113 | the wall; four primitives re-homed out first |
| `src/worktree/subagent.rs` | 671 | stamp, nominate/denominate, `verify-worker` |
| `src/mcp_server/worker_commit.rs` | 1495 | nothing re-homes (`DEC-213`) |
| `tests/e2e_worktree_stamp.rs` | — | subject deleted |
| `tests/e2e_worktree_verify_worker.rs` | — | subject deleted |

**Modify**

| path | change |
|---|---|
| `src/worktree/jail.rs` | receive the four cfg-split primitives; drop `PRIVILEGED_AGENT_TYPES` (`:121`) and the disk-policy backend branch |
| `src/worktree/jail_prefix.rs` | re-point the four imports (`:39-49`) from `pretooluse` to `jail` |
| `src/worktree/marker.rs` | collapse to the env predicate + status render; delete marker file ops, `Cause`, `is_stale_marker`, `DUAL_CAUSE`, `run_marker_clear` |
| `src/worktree/mod.rs` | drop the deleted subcommand arms (`Pretooluse`, `Nominate`, `Denominate`, `VerifyWorker`) and re-exports; retire the `marker_on_main` truth-table test |
| `src/worktree/create.rs` | delete the Fork arm, `ARMING_*`, `JAIL_SUBPATH`, `provision_jail_policy`; keep Passthrough + `run_provision` |
| `src/worktree/fork.rs` | drop the `write_marker` call (`:192`) |
| `src/worktree/land.rs` | replace `bears_marker` with the branch-shape classifier |
| `src/worktree/inventory.rs` | drop the marker column + `Cause` override; fold `classify_worktree` into `shared.rs` |
| `src/worktree/gc.rs`, `src/worktree/import.rs` | drop the unused marker imports and the `SL-116` module-wide `expect(unused)` (`CHR-062`) |
| `src/dispatch.rs` | delete `arm-spawn` and the `Spawn`-row recorder path; de-arm `NextKind::Spawn`'s prose |
| `src/dispatch_config.rs`, `src/dtoml.rs` | delete `claude_force_subprocess_dispatch` and its round-trip (`STD-001`) |
| `src/boot.rs` | drop `HookSpec::nominate`, `::denominate`, `::pretooluse` from the registry (`:1257-1263`); the eleven-entry comment becomes five |
| `src/mcp_server/tools.rs`, `src/mcp_server/mod.rs` | unregister `worker_commit`; drop the marker disjunct in `repository_context` (`:1389`) |
| `src/commands/observation.rs` | drop the marker disjunct (`:509`) |
| `src/main.rs` | drop the write-class tests for the deleted verbs |
| `src/test_support.rs`, `src/regression_run.rs` | drop marker-presence from filter/selection state |
| `scripts/pi-spawn-confined.sh` → `scripts/spawn-confined.sh` | generalise past `pi`; add the claude profile; rename (`DEC-209`) |
| `scripts/lib/pi-reap.sh` | sourced only by the pi profile after the merge — the claude profile's completion signal is process exit (§5.2.1, `R4`) |
| `.claude/settings.json` | remove six entries (four `pretooluse`, nominate, denominate) |
| `.doctrine/doctrine.toml` | remove the commented `claude-force-subprocess-dispatch` example (`:14`) — the only config-file site |
| `install/doctrine.toml.example` | re-word the `worker-forbidden-writes` doc comment, which names `worker_commit` as the key's consumer (`:100`); the key itself **stays**, with `classify_import` as its enforcing reader (`DEC-204`, `DEC-213`) |
| `plugins/doctrine/skills/dispatch/SKILL.md` | delete the arm-routing branch (step 4) |
| `plugins/doctrine/skills/dispatch-agent/SKILL.md` + `…/dispatch-subprocess/SKILL.md` | merge into one spawn skill; promote today's "Fallback (A)" to the primary and only landing path |
| `plugins/doctrine/skills/worktree/SKILL.md` | drop the `/dispatch-agent` cross-references (`:17-19`, `:134`, `:200`) |
| `install/dispatch-mechanics.md` | rewrite the arm-specific mechanics (`:73`, `:100`, `:118`) |
| `tests/e2e_claude_install.rs` | the exact hook-count assertion moves by **six** |
| `tests/e2e_worktree_create_fork.rs` | retarget onto the Passthrough arm |
| `tests/e2e_worktree_status_marker.rs` | retarget onto the env-only predicate |

**Governance (the larger half)** — one `REV` of this slice's own, over
`ADR-011` (eight regions), `ADR-006` §D2b (two corrections at one site),
`SPEC-021` (`REQ-288` retire, `REQ-291` rewrite, responsibilities 16 and 19a),
`SPEC-012` (responsibility 18, the stamp), plus a by-hand source-anchor sweep:
`spec-021.toml:30`'s dangling `[[source]]` at
`plugins/doctrine/skills/dispatch-agent/SKILL.md` resolves by deletion, and
`spec-012.toml:30` / `spec-022.toml:45` are comment lists naming
`pretooluse.rs` and `subagent.rs` (`DEC-211`).

<!-- doctrine:section sec-10 -->
## 6. Open Questions & Unknowns

None of these blocks drafting; each is a bounded fact an implementer can settle,
and each is stated so that settling it changes a line rather than a design.

**`OQ-1` — the generalised spawn script must bind the fork it creates.**
`fork.rs:216-246` binds a fork to its `(slice, phase)` only when `--worker`,
both `--slice` and `--phase`, **and** a `dir` under `<coord>/.worktrees/<name>`
all hold; otherwise `require_binding` yields the typed `unprovable-fork`
refusal. Today `scripts/pi-spawn-confined.sh:56` passes neither flag and takes
`$D` from its caller. Since the collapsed arm inherits this script as its only
spawn path, the script must either pass both flags and constrain `$D`, or the
design must say explicitly that the collapsed arm's forks are unbound and which
funnel verbs that forecloses. **Settle before the spawn phase**; it is a
precondition of the `VA` leg, not a follow-up.

**`OQ-2` — `dispatch_import` loses its producer.** The MCP funnel import
requires a *committed* fork tip. A bwrap-confined worker cannot commit, so after
the collapse no arm produces one, and the surviving landing path is the CLI
`worktree import --from-worktree` that the pi arm uses today (what
`/dispatch-agent` currently calls "Fallback (A)"). This design **does not delete**
`dispatch_import` or the funnel positions it heals: the funnel cadence is
governed prose that `DEC-211` explicitly narrowed *out* of this slice's `REV`,
and the transport is `SL-255`'s surface. So the tool is retained with no
producer, recorded here as a known residual rather than discovered later. If a
reviewer judges that unacceptable, the fix is to widen `SL-255`, not this slice.

**`OQ-3` — `~/.claude.json`.** `$HOME/.pi` is a directory; binding it wholesale
covers everything pi writes. Claude also writes `~/.claude.json`, a sibling
file, which `--ro-bind / /` leaves readable but not writable. Determine at
execute whether the worker needs it rw and add a second bind on both the Linux
inline array and the macOS `--extra-rw` path if so. Consistent with `DEC-210` —
match the pi arm's posture; do not narrow.

**`OQ-4` — does `create-fork`'s Passthrough arm have a live consumer?** The
`WorktreeCreate` hook entry is retained on the assumption that harness-created
worktrees (solo `/worktree` isolation) still ride it. Confirm at execute. A
negative answer deletes one more entry and moves the hook-count assertion by
seven rather than six; it changes no design.

**`OQ-5` — macOS parity is asserted, not exercised.** Objective 1 asserts the
`sandbox-exec` sibling stays at parity and `DEC-206` re-homes
`write_seatbelt_profile`, but no decision covers whether the claude arm actually
reaches the seatbelt path — and verifying it needs a different host. `IMP-429`
owns the larger question of where the prefix lives and macOS parity with it. The
design's position: the claude profile changes exactly the two tokens the pi
profile parameterises, so parity holds by construction, and the residual is
*execution* evidence rather than design uncertainty.

**Carried to reconcile, not open here.** Five backlog items plausibly dissolved
rather than fixed (`IMP-269`, `IMP-342`, `IMP-334`, `IMP-337`, `IMP-407`) plus
`IMP-401` and `IDE-024`; the `RFC-025` post-capsule finding inherited from
`SL-247`; and the memory-corpus sweep (§8 `R5`).

<!-- doctrine:section sec-6 -->
## 7. Decisions, Rationale & Alternatives

### 7.1 Accepted records this design implements

Twelve decisions were settled before drafting. They are the authority; this
section states what each binds in this document, not a re-argument.

| record | what it settles | where it lands here |
|---|---|---|
| `DEC-202` | retire the in-session arm; run `claude -p` confined | §1, §5.1 |
| `DEC-204` | `worker_commit` retires as transport; its analysis is preserved for `SL-255` | §5.2.4 |
| `DEC-205` | nominate/denominate die with the wall — a closed loop, not a separate choice | §2.6, §5.2.4 |
| `DEC-206` | re-home four jail primitives to `jail.rs` **first**, as its own phase | §4, §5.1, §9 |
| `DEC-207` | identity collapses to the env leg; delete the marker, topology-independently | §5.2.3, §5.3, §5.5 |
| `DEC-208` | one arm, no degraded in-session rung; fail closed at spawn | §5.4, §5.2.4 |
| `DEC-209` | generalise the incumbent prefix; no `doctrine-control` dependency, no hardening port | §5.2.1 |
| `DEC-210` | subscription credential; bind `$HOME/.claude` wholesale | §5.2.1, §5.5 |
| `DEC-211` | one `REV` over four entities plus a hand anchor sweep; `ADR-011` at eight regions | §3.1, §5.6 |
| `DEC-212` | (narrowed) the confinement control, its skip-not-pass rule, and the authored-sink evidence requirement | §9 |
| `DEC-213` | re-scope to the arm collapse alone; linked worktree, incumbent import | §1, §5.1 |
| `DEC-215` | typed hand-back by argv (`--output-format stream-json`); `--json-schema` unspent | §5.2.1 |
| `DEC-216` | no MCP in the worker now; privileged worker tools later live **outside** the confinement in their own binary | §5.2.1, §4 |

`DEC-214` records that the standing `governance-confirmed` attestation means
*confirmed against a superset of the current scope* — a narrowing re-scope does
not oblige re-confirmation. Read the evidence acts that way at reconcile.

### 7.2 Decisions this drafting stage takes

These are derived consequences of `DEC-208` ("one arm") rather than new
choices, but they are large enough that leaving them implicit would read as
scope creep to a reviewer.

**`D1` — `create-fork` keeps its hook entry and its Passthrough arm; the Fork
arm goes.** *Alternative considered:* retain the Fork arm as inert. Rejected —
with the marker deleted, a surviving Fork arm would mint an **unmarked,
unconfined** worktree from an `Agent` spawn, which is a live hazard rather than
dead code. Deleting the trigger is the fail-closed choice.

**`D2` — `dispatch arm-spawn`, the arming-dir contract, and the `Spawn`-row
recorder go with it.** `arm-spawn` plus the Fork arm are the claude arm's
substitute for `worktree fork --worker`, forced by the `Agent` tool's inability
to run a command before its worktree exists (`fork.rs:222` names the fork path
as "the SUBPROCESS arm's binding path"). `run_create_fork_and_record` exists
because on that arm nothing else knew both the harness-assigned fork name and
the base; on the surviving path `dispatch_import`'s heal-forward already lands
the `Spawn` row from the durable binding (`mcp_server/dispatch.rs:434-441`).
*Alternative:* keep `arm-spawn` as a generic pre-spawn verb. Rejected — it
writes only arming state that no surviving reader consumes.

**`D3` — `worktree verify-worker` goes with the arm.** Its `unstamped` leg reads
the marker, and its base/branch/isolation legs are guarantees `fork --worker`
now establishes at creation rather than verifies after the fact.
*Alternative:* keep it and re-cut `unstamped` onto branch shape. Rejected — the
verb has exactly one caller, `/dispatch-agent`, which is being deleted.

**`D4` — one generalised spawn script, not a claude sibling.** `DEC-209`
already says the script becomes the sole spawn shape for every harness and that
renaming it past `pi` is part of the work. A second script would be the parallel
implementation this slice exists to remove. Harness reduces to two tokens
(§5.2.1).

**`D5` — `NextKind::Spawn`'s prose loses its arm split.** `dispatch.rs:6702`
is arm-agnostic by design but currently enumerates two arms; after the collapse
it names one. This is one of the few binary-tier strings the change actually
reaches.

**`D6` — the MCP funnel tools are retained, not deleted.** See `OQ-2`. The
boundary argument is `DEC-211`'s: `SPEC-021`'s funnel-cadence responsibility no
longer changes in this slice, so the funnel is not this slice's surface, and
deleting a tool whose governing prose is out of scope would put the code and the
spec out of step in the direction that does not fail loudly.

### 7.3 Alternatives rejected at design, with their grounds

- **Keep the in-session arm as a degraded rung.** Its confinement *is* the wall
  being deleted, so the rung would offer unconfined dispatch under the name of
  confined dispatch (`DEC-208`).
- **Ride `crates/doctrine-control`.** Couples production dispatch to an in-flight
  Linux-only capsule crate outside every default test selection, and
  reintroduces the capsule-programme dependency `DEC-202` rejected waiting on
  (`DEC-209`).
- **Port the hardening into `jail.rs` without the dependency.** The write floor
  is already delivered by `--ro-bind / /`; the port buys defence in depth on a
  surface already scheduled for replacement (`DEC-209`, `IMP-428`).
- **`--bare` + `ANTHROPIC_API_KEY` + inline `--mcp-config`.** Forfeits the
  subscription billing `EVD-023` establishes, which is the slice's whole
  authorisation, and the MCP half is moot once `worker_commit` is deleted
  (`DEC-210`, `DEC-216`).
- **Let `claude -p` write prose and tail it.** Reads as the conservative option
  and is a **regression** against the arm being collapsed onto: `pi --mode rpc`
  already hands back typed completion (`DEC-215`).
- **Rebuild `worker_commit`'s six belts on the general MCP server when the need
  returns.** Belts are a general-purpose server apologising for its own surface;
  a narrow binary outside the confinement makes most of them unnecessary rather
  than merely enforced (`DEC-216`).

<!-- doctrine:section sec-7 -->
## 8. Risks & Mitigations

**`R1` — assurance-bar creep.** The likeliest failure is a reviewer applying the
capsule programme's evidentiary standard to an interim simplification. *Stated
up front (§1, §3.1):* the bar is parity with today's pi arm, not `SPEC-030`
conformance. A negative control was run at research — `grep -c
'dispatch-agent\|SubagentStart\|pretooluse'` over `SPEC-030`'s prose returns 0.

**`R2` — the billing premise is a pause, not a settlement.** `EVD-023` records
Anthropic *"working to update the plan"*, with notice promised before anything
takes effect. Accepted knowingly: `DEC-202` rests on the subprocess arm being
the better shape independently — uniform across three harnesses, real OS
confinement instead of a fail-open wall, net deletion of mechanism. Billing
removed a blocker; it is not the reason. Monitored, not designed around. *Second
edge (`DEC-210`):* if the change lands, the worker's billing changes with it and
the API-key alternative becomes live again.

**`R3` — mid-cutover posture on the incumbent.** `RFC-025`'s discipline is that
every slice but the last lands *beside* the incumbent worktree arms, so the repo
never sits half-cut-over. This slice breaks that by **removing** an arm.
Accepted in `DEC-202` on the grounds that it shrinks the eventual capsule slice
rather than growing it. *Softened, not dissolved,* by `DEC-213`: keeping the
import transport confines the breach to one axis — an arm goes, the funnel does
not move. It remains the slice's sharpest governance tension and this design
does not soften it further.

**`R4` — the two harness profiles diverge at the reap.** `pi --mode rpc` never
self-exits and is polled for `agent_end`; `claude -p` exits. A generalised
script that keeps pi's fifo/keepalive/reap machinery on the claude path would
hang to the backstop on every phase, and one that drops it for pi would break
the production arm. *Mitigation:* the divergence is named in §5.2.1 as the **one**
place the profiles differ, and the behaviour-preservation gate (§9) is what
catches a regression on the pi side.

**`R5` — a stale-but-plausible memory corpus.** At least 25 memories describe
mechanisms this slice deletes — `SubagentStart` stamping, `PreToolUse` jail
behaviour, `WorktreeCreate` provisioning, `worker_commit` resolution, marker
identity — several at `high` trust and `high` severity, and
`mem.signpost.doctrine.dispatch-claude-arm-wrong-base` is indexed in the **boot
snapshot**, so the staleness reaches every session. Stale-but-plausible is the
most dangerous class: an agent retrieving them acts on a mechanism that no
longer exists. *Mitigation:* a deliberate `/reviewing-memory` sweep at reconcile,
retiring or re-anchoring rather than editing in place where the memory's whole
subject is gone. Carried in the scope's Follow-Ups; triage detail in `notes.md`.

**`R6` — the deletion set is larger than the scope's list.** `arm-spawn`,
`create-fork`'s Fork arm, `verify-worker` and the `Spawn`-row recorder are all
claude-arm-only and none is named in the scope's objective 3. *Mitigation:* each
is recorded as a design decision with its own grounds (§7.2 `D1`–`D3`), so a
reviewer sees a reasoned boundary rather than drift. The reconciliation brief
should carry this as a design-time scope correction.

**`R7` — a silent governance regression.** `spec validate` does not catch a
dangling `[[source]]` anchor, so nothing goes red and the rot is silent.
*Mitigation:* `DEC-211` enumerates the anchors concretely rather than describing
them — `spec-021.toml:30` is the only true dangling anchor; `spec-012.toml:30`
and `spec-022.toml:45` are comment lists. The sweep is by hand and is a named
`REV` item, not an execution detail.

<!-- doctrine:section sec-8 -->
## 9. Quality Engineering & Validation

### 9.1 The gate

**The codex/pi suites stay green unchanged.** That arm is production and does
not move: the claude arm changes *into* its shape. This is achievable as
written under the narrowed scope (`DEC-213` restoring the scope's original gate),
and it is the strongest evidence available that the collapse disturbed nothing.

The `DEC-206` re-home is where the gate does its most useful work: after the
four primitives move to `jail.rs` and before any deletion, the **existing suite
must pass with no test edits**. That is the behaviour-preservation gate applied
at its cheapest point, and it is why the re-home is its own phase.

### 9.2 By test (`VT`)

| # | claim | shape |
|---|---|---|
| `VT-1` | the re-home is behaviour-preserving | existing suite green, **zero test edits**, after the `jail.rs` move |
| `VT-2` | a worker under the confinement prefix cannot write outside its own directory | spawn under `PREFIX`, attempt a write to the coordination tree, assert refusal from the **kernel** — not from a hook |
| `VT-3` | `VT-2` **skips, never passes**, on a host without `bwrap` | explicit capability probe; a vacuous pass is the fail-open shape being deleted (`DEC-212`) |
| `VT-4` | worker mode is a function of the environment alone | `describe_mode(env_set)` truth table — two rows, not eight; the `marker_on_main` test retires with its subject |
| `VT-5` | a stamped-but-unenvironed tree is no longer refused | the `ISS-028` case: shelling the doctrine CLI inside a fork succeeds |
| `VT-6` | `land`'s dispatch-fork refusal survives the marker | `LandRefusal::DispatchFork` fires on branch shape, including for a fork that was never stamped — strictly stronger than today |
| `VT-7` | the installed hook set is five entries | `e2e_claude_install.rs`'s exact count moves by **six** (four `pretooluse` + nominate + denominate) |
| `VT-8` | the config key is gone everywhere | no `claude-force-subprocess-dispatch` in `DispatchConfig`, `dtoml.rs` round-trip, `doctrine.toml` or its commented example (`STD-001`) |
| `VT-9` | the import belt is untouched | `classify_import`'s refusal set and tokens unchanged; its constants still single-sourced from `import.rs:24` |

**Test triage** (research thread 2's table is a **floor**, re-checked here
against the retained transport rather than a fetch path): 10 of 13
`e2e_worktree_*` files **keep** unchanged — the transport does not move, so the
retarget column that `DEC-212` opened is void (`DEC-213`).
`e2e_worktree_stamp.rs` and `e2e_worktree_verify_worker.rs` **delete** with
their subjects. `e2e_worktree_create_fork.rs` **retargets** onto the Passthrough
arm, and `e2e_worktree_status_marker.rs` **retargets** onto the env-only
predicate.

### 9.3 By agent (`VA`)

A **live dispatch phase driven end-to-end on the claude harness** through the
collapsed subprocess arm, concluding with the orchestrator's incumbent import of
the worker's working-tree diff. This is the only evidence that the assembled
path works; nothing in the unit suites exercises a real `claude -p` under bwrap.

**Its evidence lands in an authored sink** — `notes.md` or an `EVD` — never the
gitignored scratchpad. A `VA` criterion over runtime state leaves nothing an
audit can re-derive (`DEC-212`, `mem_019fd1d862887d42b7a1f88c28fd28a7`).

`OQ-1` is a precondition of this leg: an unbound fork cannot be named by the
funnel, so the binding question must be settled before the run rather than
diagnosed during it.

### 9.4 By human (`VH`)

`ADR-011`'s corrected text and the `SPEC-021` requirement dispositions
accurately describe the shipped arm, and no `[[source]]` anchor points at a
deleted file. This is the leg that the anchor sweep (`R7`) exists to make
checkable.

### 9.5 What is deliberately not verified here

The hardening delta (`IMP-428`), the narrowed `~/.claude` mount set, macOS
seatbelt exercise on a real mac (`IMP-429`, `OQ-5`), and every clone-topology
property (`SL-255`, carrying `A1` as the assumption it exists to verify).

<!-- doctrine:section sec-9 -->
## 10. Review Notes

Where a reviewer's attention is worth most, in order.

1. **The deletion boundary (§7.2 `D1`–`D3`, `R6`).** Four surfaces are deleted
   that the slice scope's objective 3 does not name. The argument is that each is
   the claude arm's substitute for something `worktree fork --worker` already
   does. Attack that claim directly: if `arm-spawn`, `create-fork`'s Fork arm,
   `verify-worker` or the `Spawn`-row recorder has a consumer on the surviving
   path, the boundary is wrong and the phase count changes.

2. **`OQ-1`, the fork binding.** The strongest single objection available to
   this design is that it collapses onto a spawn script which does not bind the
   forks it creates, and that the funnel's provability therefore rests on a
   precondition nobody has checked. It is stated rather than assumed, but stating
   it is not settling it.

3. **`OQ-2`, `dispatch_import` without a producer.** Retaining an unreachable
   tool is a deliberate choice with a governance argument behind it (`DEC-211`
   narrowed the funnel cadence out of the `REV`). A reviewer may reasonably
   judge that the code and the spec should move together and that `SL-255`
   should widen instead. That is a scope argument, and it belongs here rather
   than at audit.

4. **`R4`, the reap divergence.** The generalised script has exactly one
   structural difference between profiles. If the merge is done carelessly, the
   pi arm's production reap breaks or the claude arm hangs to its backstop every
   phase. The behaviour-preservation gate catches the first; only the `VA` leg
   catches the second.

5. **`ADR-011`'s eight regions (§3.1).** The count was four, then five, then
   eight, each correction found by reading the ADR end to end rather than
   through quotations. `D2` and `D6` are the load-bearing ones and were absent
   from the first two lists. A reviewer who reads only the decisions will
   inherit whichever count they land on; read `ADR-011` itself.

6. **What is *not* here.** Clone provisioning, worker self-commit, the fetch
   transport, capsule work, `REQ-335`'s mediated-write tier, and solo
   `/worktree` isolation. A finding that this design should have addressed any of
   them is a finding about `DEC-213`'s split, not about the draft.

