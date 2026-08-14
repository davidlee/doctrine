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
5. A governance landing — one `REV` over **six** entities (`ADR-011`,
   `ADR-006`, `ADR-008`, `ADR-012`, `SPEC-012`, `SPEC-021`) plus a hand
   source-anchor sweep. `DEC-218` re-derives that set from the entities
   themselves, superseding `DEC-211`'s enumeration of four. This is the larger
   half of the slice, not a tail, and it has grown at every pass.

### What this design must NOT produce

Clone provisioning, worker self-commit, fetch-from-clone in place of the import
belt, and the branch-point guard's re-homing onto fetched refs are **`SL-255`'s**
(`DEC-213`). Workers here stay on **linked worktrees** with the **incumbent
import transport**. The claude arm becomes a pi arm — no more and no less.

The capsule contract (`ADR-020`, `SPEC-030`) is untouched; solo `/worktree`
isolation is untouched.

`REQ-335`'s confined-orchestrator tier stays pending **as a contract** — but its
one partial implementation, **Mode B**, retires with the arm. Mode B arms
`create-fork` through `arm-spawn` and drives the funnel record from a *bound*
fork; this slice deletes the first and `OQ-1` declines the second, and the two
are mutually exclusive (`DEC-217`). The retained landing path is **Mode A**, the
main-thread orchestrator, which applies the delta, commits, flips the phase and
records the boundary as separate acts, and never consults the funnel record.
That is what the pi arm runs in production.


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
                     landing: import -> verify -> conclude -> reap
                       (Mode A — the main-thread orchestrator — on both
                        arms today, as separate unfunnelled acts)
```

The landing below the dashed line is already arm-agnostic. Everything above it is
duplicated, and the claude column is the duplicate that has to justify itself.

**Two orchestration modes, and most prose does not say which.** `Mode A` is the
main-thread orchestrator: unconfined, raw git, applies the delta, commits, flips
the phase and records the boundary as *separate acts*, and **never consults the
funnel record**. `Mode B` is the confined-orchestrator arm (`REQ-335`,
`install/dispatch-mechanics.md` §"Mode B"): jailed to the coordination tree with
a read-only `.git`, driving the same pipeline entirely through the dispatch MCP
tools, with the funnel record as its state. Mode A is what both arms run in
production. `plugins/doctrine/skills/dispatch/SKILL.md` currently claims the
funnel "is driven by `doctrine dispatch next` … identical on both arms" — that
is false of an arm that never lands a funnel row, and it is corrected in §5.6
([[mem.fact.dispatch.mode-a-vs-mode-b-funnel]]).

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
- **Mode B's arming path is claude-arm-only.** `dispatch arm-spawn` writes the
  spawn dir that `create-fork`'s Fork arm consumes, and `run_create_fork_and_record`
  (`dispatch.rs:6931`) is what lands the Class-2 `Spawn` funnel row. Deleting the
  arm therefore deletes Mode B's entry into the funnel machine (`DEC-217`, §6
  `OQ-2`).

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

Read directly this stage rather than through the research round's quotations.
`DEC-218` carries the site-by-site detail and **supersedes `DEC-211`'s
enumeration**: the target set is six entities, derived by reading the entities
rather than by reading a prior count. The count has been wrong five times, always
low — treat the numbers below as a floor and re-derive at reconcile.

- **`ADR-011`** (harness-agnostic orchestrator spawn interface) — the ADR this
  slice falsifies, at **eleven** regions. `D2` is the core contract — its
  *"claude's `Agent` path has no worker env channel and cannot consume it"* is
  exactly what a confined `claude -p` subprocess falsifies — and `D6` is sixty
  lines of fail-closed altitude resting on mechanisms this slice deletes, whose
  *"not fail-closable"* conclusion inverts under `DEC-208`. Three regions were
  missed by every prior pass: Consequences/**Positive** still calls the marker
  half of the agnostic floor *"identical and golden-testable under
  claude/codex/pi"*; Consequences/**Neutral** scopes bwrap and the env arm
  *"codex/pi-only until a free claude env backend (`IDE-004`) lands"* — falsified
  by a subprocess, not by `IDE-004`; and **References** carries the `ADR-008`
  cross-reference *"nested bwrap (`D-B3`) is the codex/pi OS floor this ADR's
  claude cell lacks"* plus two stale `mem.pattern.dispatch.*` pointers. `D5` and
  `D7` are checked and already falsified/withdrawn on their own terms.
- **`ADR-006`** (worktree posture) — **nine** regions, not the two corrections
  earlier drafts recorded. The falsification reaches `D2a`'s **decision body**,
  not only `D2b`'s note: the `worker_mode` formula itself, the
  *"disk marker is the harness-agnostic primary, the env leg a codex/pi
  optimisation (claude has no env channel)"* clause, the `marker --stamp-subagent`
  verb-identity exemption, the unstamped-claude-worker fence (`SubagentStart`
  stamp failure, the `IMP-052` post-spawn check), and the SL-064 amendment whose
  *"`env DOCTRINE_WORKER` must NOT leak"* hazard is scoped to codex/pi and becomes
  universal. `D9`'s two amendments go with them — the SL-056 G2 claude
  `SubagentStart` rung, and SL-064's markerless coordination-tree creation, whose
  *one* difference from an ordinary fork is the marker this slice deletes. `D2b`
  still names `IMP-065` as *"the real positive-marker close"*; `IMP-065` was
  closed **obsolete** on 2026-07-02 via `REV-018` — superseded by confinement, not
  delivered — so that is a live forward-reference to a close that will never
  arrive. `D2b`'s main body, *"the harness does not confine workers to their
  worktree"*, becomes false for the claude dispatch path; the SL-181 note (line
  143) is the existing pattern to re-cut against rather than append to.
- **`ADR-008`** (project-local jail build isolation and worker confinement) —
  **seven** regions, and a target entity **no prior survey opened at all**. `D-B3`
  states the confinement is *"codex/pi-only: claude's `Agent` tool is not a
  subprocess and cannot be wrapped … so its worker-sole-writer stays
  accident-fenced + prompt-enforced"* — the sentence this slice exists to
  falsify — and *"ro-binds the marker only"*. `D-B6`, the nominated-unjailed
  orchestrator, is **entirely** built from mechanisms deleted here: the
  `SubagentStart` nominate hook, the `PreToolUse(Agent)` gate, `SubagentStop`
  hygiene, and invariants `I1`/`I2`; its ledger names the confined Mode-B
  orchestrator as the *"reversible escape hatch"*, and Mode B retires (`DEC-217`).
  `N1` sanctions `worker_commit` as an exception to a `PreToolUse` wall — both
  halves deleted. Consequences/Negative books `D-B6`'s *"standing obligation …
  forever"*, Verification pins the `I1` doctor-check fixture, and References makes
  `D2b`'s discharge conditional on `D-B3` landing *"(codex/pi, userns-permitting)"*
  with the marker-primary CLI guard as the fallback. **This is the entity where
  the deletion is most load-bearing and least visible**: it is project-local, so
  no `[[source]]` anchor and no `spec validate` leg points at it.
- **`ADR-012`** (dispatch integration topology) — **touched, at three regions**,
  reversing this design's earlier claim. Decision 3's harness-synthesis rule is
  normative on the deleted arm — *"on arms that do **not** return a per-worker fork
  branch — the Claude `Agent` arm (`ADR-011`), where the worker delta lands
  directly onto `dispatch/<slice>`"* — and Verification `M3` repeats it as a
  fixture; the *Boundary* section restates `D2a`'s marker-absence permission
  model. The collapsed arm **does** return an orchestrator-created fork branch, so
  the rule's case goes empty. The earlier claim that `ADR-012` is *"not touched,
  and this is load-bearing: it is what keeps the `REV` inside the surveyed set"*
  **inverts**: what the retained import transport preserves is `ADR-012`'s
  *integration* decisions (`D4`/`D5`), and the arm-topology clause is falsified
  regardless of transport, because the falsifying fact is that the arm now
  produces a fork branch at all. The amendment is in place, on the precedent
  `ADR-011` already sets.
- **`SPEC-012`** — **four active requirements** rewrite or narrow, not one
  responsibility line. `REQ-192` (the `worker_mode` formula and its marker-absent
  fail-closed leg) rewrites; `REQ-248` (`fork` *"stamps the worker marker before
  any spawn window"*) rewrites; `REQ-250` (`land` refuses a *"marker-bearing
  (`dispatch-fork`)"* fork) rewrites onto the branch-shape classifier; `REQ-252`
  narrows — *"delivery is subprocess-only (codex/pi)"* goes stale when every arm is
  a subprocess, but the requirement survives. Six prose regions go with them:
  responsibility line 18 (`fork = create + provision + stamp + emit per-wt env`)
  loses the *stamp*, responsibility line 19 is the guard itself, plus the Overview
  keystone, the *worker-mode guard* body section, the *per-harness altitude*
  section's whole **claude** bullet, and the *Concerns* "claude altitude is weaker"
  bullet. *"import as the belted dispatch funnel"* is unchanged: import survives,
  and `REQ-249` with it.
- **`SPEC-021`** — **four requirements**, not two. `REQ-288` retires (its premise
  is a choice between two arms; retirement also moots `ISS-347`, which records the
  requirement stating the env-marker as *"`.claude/` presence"* while the router
  actually tests `CLAUDECODE=1`). `REQ-291` **rewrites, not retires** — an altitude
  contract still exists, with one column — and it carries the enforcement-altitude
  change described in §5.2.5: the worker-side **mutating** commit gate is replaced
  by an orchestrator-side **non-mutating** prove gate. `REQ-384` and `REQ-387`
  **narrow**: the `spawned` and `worker-committed` funnel positions lose their
  production writers and the *"through mediation"* leg loses its entry point
  (`DEC-217`). Responsibility line 16 (arm routing) and the **first half** of line
  19 (enforcement altitude) change. Responsibility line 15's funnel cadence does
  **not** — the belt does not re-home, because the incumbent import transport is
  retained (`DEC-213`) — and `REQ-335` stays `pending` as a contract while its one
  partial implementation retires.
- **`POL-002`** (platform independence from host-project conventions) —
  assessed and **satisfied, not strained**. Deleting harness-specific hook code
  from the binary and keeping spawn mechanics at the script tier reduces
  host-convention coupling. The argv builder stays in the binary: `POL-002`
  forbids host conventions in the engine, not mechanism.
- **`STD-001`** (no magic strings) — governs the thoroughness of the
  `claude-force-subprocess-dispatch` deletion (struct field, `dtoml.rs`
  round-trip, `doctrine.toml` entry, commented example) and of the belt
  constants' single-sourcing.

Checked and **not applicable**: `ADR-020`, `SPEC-030` (which states positively
that *"harness-specific in-session subagent identity is not part of the capsule
contract"*), `REV-046` and the capsule programme's evidentiary bar. This is
incumbent simplification, not capsule construction. `SPEC-011`, `SPEC-023`,
`SPEC-024` and `ADR-018` are false positives on *in-session* / *nominated* /
*denominated* in unrelated senses — they are the sweep's positive control that
the search discriminates. `funnel-machine.md` is a **generated** artefact pinned
byte-for-byte to `src/funnel_machine.rs`'s table; the table does not change, so it
is not a `REV` target.

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
- **A survey that keeps under-counting.** The governance target set has been
  wrong five times — always low, always found by reading an entity end to end
  rather than by reading a summary of it (`DEC-218`). Two of the five were found
  by an external reviewer, and the largest (`ADR-008`, absent from the set
  entirely) by re-deriving from the corpus instead of from `DEC-211`. The tension
  is real and unresolved: a design cannot cite a survey it has no reason to trust,
  so §3.1's counts are stated as a **floor** and the `REV` phase re-derives.


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
dispatch. The *fail-closed* half is already true on both platforms — an empty
`PREFIX` aborts, and a missing `bwrap` binary is a non-zero exec. The *named*
half is true only on macOS, where the script shells `worktree jail-prefix` and
gets `REASON_PROFILE_WRITE_FAILED`; the Linux branch puts a literal `bwrap` token
in `PREFIX` and never calls `have_bwrap` or the prefix verb, so it fails closed
**unnamed** (`RV-355` `F-6`). This design adds the Linux capability probe rather
than weakening the claim: `have_bwrap` is one of the four primitives `DEC-206`
already re-homes to `jail.rs`, so the caller is free by the time the spawn phase
runs, and a named refusal is the fail-closed story the design actually tells
(§7.2 `D7`, `DEC-208`).

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
   |        => linked worktree at D. NO --slice/--phase: the fork stays
   |           UNBOUND, as the pi arm's already are (§6 OQ-1, DEC-217)
   |
   +-- PREFIX resolution        (harness + platform each carry an enumerated
   |                             asymmetry set — §5.2.1, RV-356 F-2)
   |     Linux  : have_bwrap probe -> REASON_NO_BWRAP if absent (D7), then
   |              inline bwrap array
   |              bwrap --ro-bind / / --dev /dev --proc /proc --tmpfs /tmp
   |                    --bind $HOME/<harness-cfg> $HOME/<harness-cfg>
   |                    --bind $D $D --chdir $D --die-with-parent
   |                    --setenv DOCTRINE_WORKER 1
   |     macOS  : doctrine worktree jail-prefix --dir $D --main-root $ROOT
   |                    --extra-rw $HOME/<harness-cfg> --out $D/.tmp/jail.argv
   |              (NUL-delimited sandbox-exec prefix, read back into PREFIX)
   |              sandbox_exec_argv's trailing `env` token carries BOTH
   |                TMPDIR=<tmp> and DOCTRINE_WORKER=1               (F-2)
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
| `worktree/jail.rs` | the confinement core: argv build, policy validation, backend selection, **plus** the four re-homed primitives | gains `REASON_NO_BWRAP`, `have_bwrap`, `REASON_PROFILE_WRITE_FAILED`, `write_seatbelt_profile`; `sandbox_exec_argv` gains `DOCTRINE_WORKER=1` in its trailing `env` token; loses `PRIVILEGED_AGENT_TYPES` and `probe_backend`'s disk-policy branch |
| `doctor_checks.rs` | the doctor check registry | check **#10 `SpawnSeamSymmetry`** deleted with its subject (it reads `PRIVILEGED_AGENT_TYPES`, `SubagentStart` matchers and the `PreToolUse` seam registry); check **#9 `AgentConformance`** re-cut — `TOOL_ALLOWED` names `mcp__doctrine__worker_commit`, and the confined worker holds no MCP token at all (`DEC-216`) |
| `worktree/jail_prefix.rs` | the **sole** shell over that core: emit a confinement prefix for a spawn script | imports re-point from `pretooluse` to `jail`; otherwise untouched |
| `worktree/pretooluse.rs` | — | **deleted** (1113 lines) |
| `worktree/subagent.rs` | — | **deleted** (671 lines) |
| `worktree/marker.rs` | worker-mode status rendering over **one** signal | collapses to the env predicate + `worktree status`; the marker file, its ops, `Cause`, `is_stale_marker`, `DUAL_CAUSE` and `run_marker_clear` all go |
| `worktree/create.rs` | provision a benign harness-created worktree | Fork arm deleted; Passthrough arm and `run_provision` survive |
| `worktree/fork.rs` | the **only** worker-fork writer: create + provision + bind | loses the `write_marker` call |
| `worktree/import.rs` | the belted import transport | **untouched** — `classify_import` remains the enforcing caller of the two hard-coded scope-belt floors (`.doctrine/**`, `.claude/**`). It never read `worker-forbidden-writes` and does not start now (`RV-356` `F-3`) |
| `mcp_server/worker_commit.rs` | — | **deleted** (1495 lines) |
| `scripts/spawn-confined.sh` | the one spawn shape, parameterised by harness | generalised from `pi-spawn-confined.sh` |

### 5.2 Interfaces & Contracts

**5.2.1 The spawn script contract.** One script, one shape, harness as a
parameter.

> **Reconciled (`RV-356` `F-2`).** Earlier drafts of this section, of §5.1's
> diagram and of `D4` all asserted that *everything* harness-specific reduces to
> two facts — the config directory to bind and the exec line. That claim is
> retired rather than patched: it was falsified in **both** directions, by
> `DOCTRINE_WORKER` (Darwin lacked it, `RV-355` `F-2`) and by `TMPDIR` (Linux
> lacked it, found live at `PHASE-09`), and a claim that has been corrected twice
> by counter-example is not a contract. What the script actually carries is an
> **enumeration** of asymmetries, on two independent axes. Both lists are floors,
> not ceilings — the standing obligation (`IMP-429`) is to enumerate every
> asymmetry, not to re-check one platform against the other.

*Harness axis* — pi vs claude, on either platform:

| asymmetry | pi | claude |
|---|---|---|
| config dir to bind | `$HOME/.pi` (a directory; wholesale covers everything pi writes) | `$HOME/.claude`, plus the sibling **file** `~/.claude.json` outside it (`OQ-3`) |
| exec line | `pi --mode rpc …` | `claude -p …` (below) |
| completion signal | never self-exits: fifo holds stdin, poll the tail for `agent_settled`/`agent_end` | process exit, with the stream-json result on stdout — no fifo, no keepalive, no `pi_await_and_reap` |
| host credential precondition | none beyond `$HOME/.pi` | a **materialised** `~/.claude/.credentials.json`, or `CLAUDE_CODE_OAUTH_TOKEN` in the environment. The bind *carries* a subscription credential; it does not *create* one (`F-9`, `DEC-210`) |

*Platform axis* — Linux `bwrap` vs Darwin `sandbox-exec`, on either harness:

| asymmetry | Linux | Darwin |
|---|---|---|
| `DOCTRINE_WORKER` | `--setenv` in the inline array | had to be taught to `sandbox_exec_argv`'s trailing `env` token (`RV-355` `F-2`, `VT-10`) |
| `TMPDIR` | unset by the prefix; `PHASE-09` used `/tmp` (tmpfs) deliberately | `<wt>/.tmp` — **inside the tree whose working-tree delta is imported**. Benign here only because `.gitignore:14` matches `*.tmp` |
| network | inline array carries no `--unshare-net`, so it is open | `jail-prefix` defaults `--network` to **deny** and the Darwin arm does not pass it, so `(deny network*)` lands in the profile and a `claude -p` worker cannot reach the API at all (`F-2`) |
| capability probe | `have_bwrap` → `REASON_NO_BWRAP`, a **named** refusal ahead of the fork (`D7`) | no `sandbox-exec` presence probe; still fails closed, but **unnamed**, and only after a fork has been minted |

The config-directory branch remains the one place the script itself switches on
harness:

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
claude -p --output-format stream-json --verbose \
   --strict-mcp-config \
   --permission-mode bypassPermissions \
   <"$PF" >"$OUT" 2>&1
```

Flag-by-flag justification, each tied to a decision rather than to taste:

| flag | why | record |
|---|---|---|
| `-p` | headless, non-interactive; the whole premise | `DEC-202` |
| `--output-format stream-json` | typed completion event stream at parity with pi's RPC `agent_end`; argv-tier, no code | `DEC-215` |
| `--verbose` | **mandatory companion** to the pair above, not a diagnostic: `claude -p --output-format stream-json` hard-refuses without it and spawns nothing. `PHASE-09` run #1 died on it in 1.4s. Pinned by `claude_arm_stream_json_carries_verbose` (`jail.rs`); reason inline at `spawn-confined.sh:236-245`. `DEC-215`'s account of the typed hand-back is incomplete without this row (`RV-356` `F-10`) | `DEC-215` |
| *(not)* `--json-schema` | deliberately unspent: nothing consumes a shaped hand-back today; adopting one would be building | `DEC-215` |
| `--strict-mcp-config` with no `--mcp-config` | no MCP at all in the worker, at parity with pi's `--no-extensions`; the orchestrator performs every privileged act | `DEC-216` |
| `--permission-mode bypassPermissions` | the analog of pi's `--approve`: inside the confinement the OS floor is the boundary, so an in-agent prompt buys nothing | `DEC-208`, `DEC-216` |
| *(not)* `--bare` / `ANTHROPIC_API_KEY` | an API key forfeits the subscription billing `EVD-023` establishes, which is the slice's whole authorisation | `DEC-210` |
| *(not)* a narrowed mount set | `../microvm-spike`'s set is the known refinement, not the requirement; its identity-section partial is unproven | `DEC-210` |

**A divergence from pi that is real and should be stated:** `pi --mode rpc` never
self-exits, so the pi arm holds stdin open with a fifo and polls the tail of the
event stream for **`agent_settled`** (`scripts/lib/pi-reap.sh:68-82`).
`claude -p` **exits on completion**. The claude profile therefore needs neither
the fifo, the keepalive subshell, nor `pi_await_and_reap` — its completion signal
is process exit, with the stream-json result on stdout. This is a simplification,
not a gap, and the two profiles diverge exactly here and nowhere else.

**Name the pi terminal event correctly, because the generalised script is where a
wrong name becomes a defect.** It is `agent_settled`, **not** `agent_end`
(`ISS-293`, measured). `agent_end` carries the accumulated conversation state, so
it is pushed arbitrarily far from EOF — 684,768 bytes on one census turn — and a
windowed poll for it never fires on a real turn: every spawn then runs to the
backstop holding a live pi and an open API session, *and nothing warns, because
the output lands on time so it looks clean*. The incumbent matches **either**
event for robustness, and the generalised profile must keep matching either.
Earlier drafts of this design named `agent_end` throughout, which is the folklore
`ISS-293` was raised to correct (`RV-355` `F-7`, §8 `R4`).

**macOS is the one place parity is *not* by construction.** `sandbox_exec_argv`
(`jail.rs:627-655`) emits the `-D` binds, `-f <profile>`, the `--` terminator and
a trailing `env TMPDIR=<tmp>` token — and **no `DOCTRINE_WORKER`**. Only the Linux
inline array sets it. macOS worker identity rides the disk marker today, so
`DEC-207`'s deletion would leave a confined macOS worker with *no* identity signal
and make `INV-1` false on that platform. The fix is one token: `sandbox_exec_argv`
sets `DOCTRINE_WORKER=1` in the same trailing `env` run as `TMPDIR`. It is a
**binary** change, not a script change (`RV-355` `F-2`, §6 `OQ-5`, `VT-10`).

This was drafted as *the only* such asymmetry. It is not — see the platform-axis
table above, which `PHASE-09` extended with `TMPDIR`, the network default and the
missing `sandbox-exec` probe (`RV-356` `F-2`). It remains the only one this slice
fixed.

**5.2.2 `worktree fork --worker` — the binding contract, now on both arms.**
Unchanged in code, but it becomes the **only** producer of a worker fork, so its
preconditions become the collapsed arm's preconditions (`fork.rs:216-246`): a
fork binds its `(slice, phase)` iff `--worker`, both `--slice` and `--phase` are
supplied, **and** `dir` resolves under `<coord>/.worktrees/<name>`. An unbound
fork is not a parse failure — it is a fork whose phase cannot be proven, which
`require_binding` turns into the typed `unprovable-fork` refusal.

**The collapsed arm supplies `--worker` alone**, exactly as
`pi-spawn-confined.sh:56` does today, so its forks are **unbound** and no
`DispatchRecord` is written (`OQ-1`, settled). That is a decision with a
consequence, not a detail: a `Spawn` funnel row requires a bound fork, so unbound
forks and a live Mode B are mutually exclusive, and Mode B retires with the arm
(`DEC-217`). Mode A, the retained landing path, never asks.

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

**5.2.5 The scope belt is untouched; the check gate moves altitude.** One
non-merge commit per phase with `C^ == B`, the two-tier scope belt's refusal set
and tokens, report-and-halt on any breach, and the phase state machine's
transitions all stay exactly as they are. `classify_import` (`import.rs:147`)
keeps its single-sourced constants from `import.rs:24` and remains the belt's
enforcing caller — so the **belt** does not move.

**The check gate does.** `worker_commit` ran a **mutating** `CheckKind::Commit`
gate *worker-side, before landing* (`worker_commit.rs:473-477`, and again at
`:563-581` before adopting a pre-existing commit). The retained path calls
`classify_import` (`import.rs:464-472`), applies the patch, then resolves and runs
a **non-mutating** `CheckKind::Prove` gate *orchestrator-side, post-import*
(`import.rs:258-286`, `:480-491`). Neither the cadence nor the mutation contract
survives unchanged: *worker-side mutating commit gate → orchestrator-side
non-mutating prove gate*.

Earlier drafts recorded the posture as wholly unchanged on the ground that
`classify_import` survives — which answers the belt and not the gate, and that
omission is precisely what justified keeping the posture change out of the `REV`
(`RV-355` `F-3`). It is an **enforcement-altitude** change, and it is
governance-visible. It needs no new `REV` target: `SPEC-021` `REQ-291` is already
being rewritten, and this is what its rewrite has to say. `INV-2` narrows
accordingly (§5.5).

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
   <coord>/.worktrees/<name> --worker`. **No `--slice` / `--phase`** — the fork
   stays unbound, as the pi arm's already are (`OQ-1`; earlier drafts of this step
   carried both flags and contradicted `OQ-1`'s settlement, `RV-355` `F-1`). Fails
   closed; the orchestrator halts on non-zero.
3. **Confine.** Resolve `PREFIX`. Linux probes `have_bwrap` first (`D7`) and then
   takes the inline array; macOS shells `worktree jail-prefix`. An empty `PREFIX`
   aborts (the existing fail-closed guard). No `bwrap` ⇒ `REASON_NO_BWRAP`; no
   writable seatbelt profile ⇒ `REASON_PROFILE_WRITE_FAILED`. **There is no
   unconfined fallback** (`DEC-208`).
4. **Spawn.** `timeout $BACKSTOP "${PREFIX[@]}" <harness exec>`, prompt on
   stdin, stdout to `$OUT`.
5. **Hand back.** claude: process exit + stream-json result. pi:
   **`agent_settled`** on the RPC stream (matching either it or `agent_end`, as
   the incumbent does — `ISS-293`, §5.2.1). Either way the delta is an
   **uncommitted working tree**.
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
| a phase check fails | `worker_commit`'s **mutating** `CheckKind::Commit` gate, worker-side, before the commit is created | the orchestrator's **non-mutating** `CheckKind::Prove` gate, post-import, reject-and-halt (§5.2.5) |
| host cannot confine, Linux | unnamed non-zero exec (the script never probes) | `have_bwrap` ⇒ `REASON_NO_BWRAP` before the fork is spawned (`D7`) |
| stale marker left behind | `worktree status --assert` + `marker --clear --operator` | unrepresentable |
| env leaked into a non-worker process | `DUAL_CAUSE` names two horns | one horn: unset it |

### 5.5 Invariants, Assumptions & Edge Cases

**INV-1 — worker-ness is a property of the process.** `refused ==
env_worker_set()`, everywhere, with no tree-shaped input. The orchestrator is
never *identified* as the orchestrator; it is simply not-refused. Removing one
of two positive legs cannot convert a not-refused into a coordination claim,
because no reader anywhere concludes *this is the coordination tree* from
marker absence (`DEC-207` rationale (1); `mem_019fa94118a37c33ab54b06dfe4b1131`).

**INV-2 — a worker still cannot skip the *scope belt*.** Narrowed deliberately to
what it actually covers. `classify_import` survives as the scope belt's enforcing
caller, single-sourced from `import.rs:24`, so **the belt** is unchanged and the
worker cannot skip it. The **check gate** is a different claim and does move: from
a worker-side mutating `CheckKind::Commit` to an orchestrator-side non-mutating
`CheckKind::Prove` (§5.2.5). That is an enforcement-altitude change, it is
governance-visible, and it lands in `SPEC-021` `REQ-291`'s rewrite (`RV-355`
`F-3`). The earlier unqualified form of this invariant — *a worker cannot skip a
belt, posture unchanged* — covered the belt and silently dropped the gate.

**INV-3 — the confinement prefix is the boundary.** `--ro-bind / /` plus
`--bind $D $D` is the write floor; every other guarantee in this design rests on
it rather than on cooperation.

**INV-4 — one coordination commit per phase, `C^ == B`.** Untouched.

**Edge case: `land.rs:173`'s cross-tree marker read.** `bears_marker` is the
only marker read that asks about *another* tree, and it gates
`LandRefusal::DispatchFork` (`land.rs:127`). Env cannot answer it — env
describes *this* process. Substitute `shared.rs::is_dispatch_fork_branch`
(`:112`): a `dispatch/` prefix **and** a non-numeric suffix — exactly what the
funnel mints, and what `coord_branch_suffix` already knows. This is **strictly
stronger** than the marker read: it fires whether or not the fork was ever
stamped, catching the unstamped-worker case `ADR-011` `D6`/`M2` confesses
(`DEC-207`).

Drafted as *"the branch-shape role classifier (`shared.rs:77`) returning
`"fork"`"*, which taken literally bricks the verb — `classify_worktree_role`
returns `"fork"` for **every** linked non-coordination worktree, and `land` only
ever operates on a linked worktree, so the substitution would refuse every
`land`, including the solo TDD branches the verb exists to serve. `PHASE-05`
caught it and shipped the conjunct above; the retarget also found
`land_refuses_dispatch_fork` going green on a `solo-df` branch plus a stamp while
`land` merged a fork it should have refused, now pinned by the new negative
`land_permits_a_solo_fork_that_is_not_a_dispatch_branch`. Intent and the
strictly-stronger claim are unchanged; only the mechanism named was wrong
(`RV-356` `F-13`).

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

**Edge case: `--output-format stream-json` and `--verbose`.** Drafted as "some
`claude` builds require `--verbose`… verify at execute". **Settled at `PHASE-09`,
in the strong direction:** the pair hard-refuses without it and spawns nothing —
run #1 died in 1.4s. It is mandatory, not build-dependent (§5.2.1, `RV-356`
`F-10`).

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
| `tests/e2e_dispatch_arm_spawn.rs` | — | subject deleted (`D2`); the file is `arm-spawn` end to end |

**Modify**

| path | change |
|---|---|
| `src/worktree/jail.rs` | receive the four cfg-split primitives; **add `DOCTRINE_WORKER=1` to `sandbox_exec_argv`'s trailing `env` token** (`:627-655`, `F-2`); **delete the whole pure decision layer** — see *The wall's blast radius* below, which is where `PRIVILEGED_AGENT_TYPES` (`:121`) and the disk-policy backend branch sit |
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
| `src/boot.rs` | drop `HookSpec::nominate`, `::denominate`, `::pretooluse` from the registry (`:1257-1263`); the eleven-entry comment becomes five; drop the `PRIVILEGED_AGENT_TYPES` consumers (`:639`, `:4573-4583`) |
| `src/doctor_checks.rs` | **delete check #10 `SpawnSeamSymmetry`** (`:728-830`) — it reads `PRIVILEGED_AGENT_TYPES`, `SubagentStart` matchers and the `SEAM_REGISTRY`, all deleted, so the deletion does not compile without it; **re-cut check #9 `AgentConformance`** — `TOOL_ALLOWED` is `mcp__doctrine__worker_commit` (`:489`), and a `--strict-mcp-config` worker holds no `mcp__*` token at all (`DEC-216`) |
| `src/finding.rs` | drop `Category::SpawnSeamSymmetry` (`:82`) and renumber the ordinal (`:123`); the `CATEGORY_ORDER` array and the severity test move with it |
| `src/commands/doctor.rs` | drop the #10 registration and the `AgentConformance`-only filter (`:169`) if the re-cut leaves it empty |
| `src/mcp_server/tools.rs`, `src/mcp_server/mod.rs` | unregister `worker_commit`; drop the marker disjunct in `repository_context` (`:1389`) |
| `src/commands/observation.rs` | drop the marker disjunct (`:509`) |
| `src/main.rs` | drop the write-class tests for the deleted verbs |
| `src/commands/guard.rs` | **the write-class registry, not `main.rs`** — delete the `Marker { stamp_subagent: true }`, `Nominate` and `Denominate` arms (`:285-307`); their `Command` variants go, so **the match does not compile** with them. Retire the bespoke `WriteClass::MarkerClear` class and its pass-through leg (`:499`) — it exists only for `worktree marker --clear`. In `worker_guard`, the marker leg of `resolve_mode` collapses to the env leg, so the two-branch `is_env_on_nonlinked` / named-verb message split (`:512-524`) degenerates to one branch |
| `justfile` | `validate`'s worker-context skip reads three signals; drop the `[ -f .doctrine/state/dispatch/worker ]` leg (`:37`). `DOCTRINE_WORKER=1` and `DOCTRINE_DISPATCH_GATE` survive and are what `DEC-207` leaves standing — a clean deletion, not a behaviour change |
| `src/worktree/dispatch_record.rs` | doc-anchor only: `:26`'s intra-doc link to `super::create::JAIL_SUBPATH` dangles once that const goes. Its own `provision_dispatch_record` (`:110`) is an independent mirror and survives |
| `src/test_support.rs`, `src/regression_run.rs` | drop marker-presence from filter/selection state — in `test_support` that is `WORKER_MARKER_REL` (`:62`, an `STD-001` carve-out duplicate of `marker_path`) and `worker_marker_at`'s marker leg (`:68`). **The helper contains the blast radius**: `under_worker_marker` has 31 test-file callers and none of them change |
| `scripts/pi-spawn-confined.sh` → `scripts/spawn-confined.sh` | generalise past `pi`; add the claude profile; rename (`DEC-209`); **add the Linux `have_bwrap` probe** ahead of the inline array so the Linux arm fails closed *named*, not merely closed (`D7`, `F-6`) |
| `scripts/lib/pi-reap.sh` | sourced only by the pi profile after the merge — the claude profile's completion signal is process exit (§5.2.1, `R4`). **Unchanged**: its `agent_settled`/`agent_end` either-match is correct and must survive the generalisation (`ISS-293`) |
| `.claude/settings.json` | remove six entries (four `pretooluse`, nominate, denominate) |
| `.doctrine/doctrine.toml` | remove the commented `claude-force-subprocess-dispatch` example (`:14`) — the only config-file site |
| `install/doctrine.toml.example` | re-word the `worker-forbidden-writes` doc comment, which names `worker_commit` as the key's consumer (`:100`). ~~the key itself **stays**, with `classify_import` as its enforcing reader~~ — **false, corrected at reconcile** (`RV-356` `F-3`): `classify_import` never read the key, only the two hard-coded floors, which are unaffected. The key stays as a **declaration with no production reader** (`dispatch_config.rs:93-95`, `doctrine.toml.example:113-115` both say so); supplying an enforcing reader — kernel-level extra `--ro-bind` paths at spawn, not a post-import Rust belt — is `SL-255`'s work (`IDE-051`). `DEC-204`/`DEC-213`'s consequence text carried the same error and is corrected with this |
| `plugins/doctrine/skills/dispatch/SKILL.md` | delete the arm-routing branch (step 4); **correct the front-matter claim that the funnel "is driven by `doctrine dispatch next` … identical on both arms"** — untrue of an arm that never lands a funnel row (`DEC-217`, `F-1`) |
| `plugins/doctrine/skills/dispatch-agent/SKILL.md` + `…/dispatch-subprocess/SKILL.md` | merge into one spawn skill; promote today's "Fallback (A)" to the primary and only landing path |
| `plugins/doctrine/skills/worktree/SKILL.md` | drop the `/dispatch-agent` cross-references (`:17-19`, `:134`, `:200`) |
| `install/dispatch-mechanics.md` | rewrite the arm-specific mechanics (`:73`, `:100`, `:118`); **retire the Mode B section** (`:152-200`) — its arming path is `arm-spawn` (`DEC-217`) |
| `tests/e2e_claude_install.rs` | the exact hook-count assertion moves by **six** |
| `tests/e2e_worktree_create_fork.rs` | retarget onto the Passthrough arm |
| `tests/e2e_worktree_status_marker.rs` | retarget onto the env-only predicate |
| `tests/e2e_dispatch_h1_integration.rs` | 14 `arm-spawn` references and 6 `verify-worker` ones — the integration path it drives is the deleted arm's. Retarget onto `fork --worker`, or delete with it |
| `tests/e2e_worker_guard.rs`, `tests/e2e_worker_guard_explicit_root.rs` | both assert the `DUAL_CAUSE` message and stamp the marker file directly (`e2e_worker_guard.rs:183`); the explicit-root one also drives `nominate`/`denominate`. Retarget onto the env-only signal |
| `tests/e2e_worker_gate_skip.rs` | `SL-225 PHASE-01`'s proof of the real `just validate` skip; its fixture writes the marker (`:117`) to simulate worker context. Moves with the `justfile` leg above |
| `tests/common/mod.rs` | the shared fixture helper *stamps* the marker into fixture roots (`:158-180`) — unlike `test_support`'s read-side helper, this one changes |
| `tests/e2e_mcp_server.rs` | the tool-registry assertion names `worker_commit` (`:208`) |

**The wall's blast radius in `jail.rs`.** `jail.rs` survives, but the row above
understates what it loses. After the collapse its only importers are
`jail_prefix.rs:40-43` (`resolve_with_policy`, `validate_policy`, `Backend`,
`JailPolicy`, `RealEnv`, `ResolveEnv`, `select_jailer`) and `mod.rs:30`'s
`JailPolicy` re-export for `dispatch.rs`'s `from_toml_str`. Everything outside the
transitive closure of those eight has **no surviving consumer** — `pretooluse.rs`
was its only caller — and `pub(crate)` with no caller is a `dead_code` warning, so
this is not optional tidying: `just gate` runs clippy at zero warnings.

The orphaned set is the wall's entire **pure decision layer**:
`is_privileged_agent_type` (`:126`, sole consumer `decide_agent`), `enum Decision`
(`:231`), `enum Target` (`:248`), `resolve_target` (`:377` — the four-arg jail one;
`boot.rs:875` and `relation_graph.rs:329` are unrelated same-named functions),
`decide_agent` (`:399`), `decide_workflow` (`:418`), `pathcheck` (`:433`, sole prod
consumer `decide_write:974`), `opaque_wrap` (`:472`, sole prod consumer
`decide_bash:942`), `shell_single_quote` (`:491`, sole consumer `opaque_wrap`),
`acquire_policy` (`:761`, sole consumer `resolve_inputs:831`), `resolve_inputs`
(`:825`), `seatbelt_backend` (`:839` — this is the "disk-policy backend branch"),
`decide_bash` (`:928`), `decide_write` (`:964`), and the five `REASON_*` constants
that feed only them (`:94`, `:95`, `:96`, `:103`, `:109`). That is roughly 300 of
`jail.rs`'s 981 production lines, and about half of its ~60 unit tests
(`resolve_target` ×5, `pathcheck` ×5, `opaque_wrap` ×2, `decide_bash` ×5,
`decide_write` ×3, `decide_agent` ×4, `decide_workflow` ×2, `resolve_inputs` ×6,
plus the `seatbelt_backend` rows).

This is a consequence rather than a defect: the wall decided *per tool call*, and
confinement now happens once at spawn through `jail_prefix`. But it moves the
deletion phase's size materially, and two things fall out of it worth stating.
**`F-2`'s fix is on live code** — `sandbox_exec_argv` is reached from
`jail_prefix.rs:168` through `Seatbelt::wrap_argv` (`jail.rs:529`), not from the
dying wall. And **`DEC-206`'s re-homing set is exactly right**: `jail_prefix.rs`
imports precisely the four primitives it names from `pretooluse.rs`
(`REASON_NO_BWRAP`, `REASON_PROFILE_WRITE_FAILED`, `have_bwrap`,
`write_seatbelt_profile`) and nothing else, so the re-home is complete and no fifth
primitive is hiding.

**Governance (the larger half)** — one `REV` of this slice's own, over **six**
entities. `DEC-218` carries the region-by-region derivation and supersedes
`DEC-211`'s enumeration; §3.1 carries the argument. In summary:

| entity | regions | shape |
|---|---|---|
| `ADR-011` | 11 | in-place amendment; `D2`/`D6` load-bearing; Consequences Positive/Neutral and References newly counted |
| `ADR-006` | 9 | `D2a`'s **decision body**, not only `D2b`'s note; both `D9` amendments; the dangling `IMP-065` forward reference |
| `ADR-008` | 7 | **new target** — `D-B3`'s codex/pi-only clause, `D-B6` entire, `N1`, and the `D2b`-discharge condition |
| `ADR-012` | 3 | `D3`'s harness-synthesis rule, Verification `M3`, the `D2a` restatement in *Boundary* |
| `SPEC-012` | `REQ-192`/`248`/`250` rewrite, `REQ-252` narrow + 6 prose regions | responsibility lines 18 and 19, Overview keystone, guard section, the claude altitude bullet, the Concerns bullet |
| `SPEC-021` | `REQ-288` retire, `REQ-291` rewrite, `REQ-384`/`387` narrow + responsibility lines 16 and 19a | `REQ-291` also carries §5.2.5's enforcement-altitude change |

Plus a by-hand source-anchor sweep: `spec-021.toml:30`'s dangling `[[source]]` at
`plugins/doctrine/skills/dispatch-agent/SKILL.md` resolves by deletion, and
`spec-012.toml:30` / `spec-022.toml:45` are comment lists naming
`pretooluse.rs` and `subagent.rs`.

**Treat every count here as a floor.** The survey has under-counted six times,
always low; the `REV` phase re-derives from the entities rather than from this
table (`DEC-218`).

**How the sixth sweep was done, and why it should be the last hand-built one.**
Sweeps one through five read the code and asked *what does this change touch*.
That question kept missing consumers, because it is answered from the reader's
model of the system rather than from the symbol graph. The sixth sweep inverted it:
enumerate every `pub` item in the three dying modules and every `pub` item in
`jail.rs`, then grep each one across `src/` and `tests/` and subtract the consumers
that are themselves dying. It reproduced the `doctor_checks.rs` find as a positive
control and then found the `jail.rs` decision layer, `commands/guard.rs`, the
`justfile` leg and six unlisted test files. Two limits to record: a negative grep
result is only trustworthy against a positive control (`e2e_priority_golden.rs`'s
three `denominate` hits are the arithmetic denominator, not the verb), and the
method sees Rust symbols — the `justfile` leg was found by grepping the *string*
`.doctrine/state/dispatch/worker`, which is how non-Rust consumers surface at all.
The plan phase should run this census mechanically against the real deletion
rather than re-reading this table (`R8`).

**Reconciled: the table was still a floor, and the registry is the load-bearing
copy (`RV-356` `F-8`).** `slice conformance` at audit read undeclared 96 /
undelivered 0 / conformant 40. Undelivered zero is the good half — nothing was
promised and dropped. The undeclared cell was the thirteenth under-count: the
`design-target` selector registry, which is what `conformance` actually reads,
never learned what this table missed, even where a plan criterion named the path
explicitly (`guard.rs`, `doctor_checks.rs`, `finding.rs`, `install.rs`,
`justfile`). Fifty-one selectors were declared at reconcile — the surfaces above
plus `src/commands/{guard,doctor,cli}.rs`, `src/{doctor_checks,finding,install}.rs`,
`src/mcp_server/dispatch.rs`, `src/worktree/{allowlist,claim_lock,dispatch_record}.rs`,
`justfile`, `plugins/doctrine/hooks/hooks.json`,
`plugins/doctrine/skills/{execute,dispatch-spawn}/SKILL.md`,
`install/hymns/role/{orchestrator,worker}.md` and its `.doctrine/` twin,
`install/agents/claude/{dispatch-orchestrator,dispatch-probe,dispatch-worker}.md`,
`install/workflows/drive-slice.js` and its `.doctrine/` twin,
`install/manifest.toml`, `publication/manifest.toml`, `.doctrine/governance.md`,
`scripts/{pi-agent,pi-respawn-nofork.sh,pi-review.sh}`, `tests/common/mod.rs`,
`tests/e2e_worker_confinement.rs`, `tests/e2e_dispatch_arm_spawn.rs`, and the
retargeted `tests/e2e_*` set — taking it to undeclared 45 / undelivered 0 /
conformant 91, with every remaining undeclared path in the governance-entity
class a code-surface selector list is not meant to hold.

One path sits outside the declared blast radius and is declared as deliberate
incidental reach: `crates/doctrine-control/src/backend/bubblewrap.rs`, a one-line
doc-comment rename in a byte-parity note (`PHASE-02`, `c66cd2c65`). The collapse
reached the capsule crate, which is worth recording even though it changed
nothing.

**The standing lesson, for the next slice rather than for this one.** A prose
table and a machine-read registry are two copies of the same claim, and this
slice under-counted the prose one twelve times before the registry made the
thirteenth mechanical. Prose corrections do not propagate: correcting §5.6 alone
would have left conformance red. The registry is the load-bearing copy.


<!-- doctrine:section sec-10 -->
## 6. Open Questions & Unknowns

`OQ-1` is settled below. The rest are bounded facts an implementer can settle,
none of them blocking, each stated so that settling it changes a line rather
than a design.

**`OQ-1` — settled: the collapsed arm's forks stay unbound.** `fork.rs:216-246`
binds a fork to its `(slice, phase)` only when `--worker`, both `--slice` and
`--phase`, **and** a `dir` under `<coord>/.worktrees/<name>` all hold — and
`bind_dispatch_record` is the sole production writer of a `DispatchRecord`, so
an unbound fork leaves **no record at all**, not an incomplete one.
`scripts/pi-spawn-confined.sh:56` passes `--worker` alone, so unbound forks are
already the surviving arm's production posture; the collapse inherits it rather
than adopting something new.

What that forecloses is exactly the two readers of the binding, and this slice
removes or orphans both on independent grounds: `worker_commit` (deleted,
`DEC-204`) and MCP `dispatch_import` (`mcp_server/dispatch.rs:303`, which
additionally requires `ForkExpect::Advanced` — a committed fork tip the
collapsed arm cannot produce; that is `OQ-2`). Nothing on the retained path
reads it: `worktree import` never touches `dispatch_record`, `slice
record-delta` (`slice.rs:3043`) takes `PHASE-NN` as an explicit argument, and
`reap_fork` / `gc` tolerate a missing record. The funnel row is named by the
orchestrator, not by the binding.

*Alternative declined:* have the generalised spawn script pass both flags and
constrain `$D`. It would write a durable record for a consumer this slice
deliberately leaves without a producer, and it would move the production pi
arm's runtime behaviour — records where none existed — against §9.1's gate. The
binding belongs with the transport, and the transport is `SL-255`'s surface.

*What settling it costs, stated plainly:* a `Spawn` funnel row requires a bound
fork, so this settlement and a live Mode B are mutually exclusive. `OQ-2` below
records the consequence. Reversing it means reversing this question, not amending
prose (`DEC-217`).

**`OQ-2` — settled: Mode B retires with the in-session arm.** Earlier drafts
recorded this as *one tool loses its producer*. That understates it by a tier, and
the correction is `RV-355` `F-1`'s.

`funnel_machine`'s `TABLE` admits only `Transition::Spawn` from `current: None`,
so a phase with no funnel row can reach no other position. All three production
`Spawn` writers go: `land_spawn_row` (`dispatch.rs:6958`, deleted with
`create-fork`'s Fork arm), `worker_commit` (deleted), and `dispatch_import`'s
heal-forward (retained, but it refuses without the durable binding `OQ-1`
declines — `mcp_server/dispatch.rs:286-315`). Mode B additionally rides
`arm-spawn` for its create-fork arming, which `D2` deletes. So what the collapse
retires is **Mode B's entry point**, not merely one tool's producer.

**Mode A is unaffected, and Mode A is the retained path.** The main-thread
orchestrator applies the delta, commits, flips the phase and records the boundary
as separate acts; it never consults the funnel record, and
`install/dispatch-mechanics.md` names "a fork with no funnel row" as an explicitly
supported case for the reap oracle. This is what the pi arm runs in production
today, and it is what the collapsed claude arm lands on. A reading of
`dispatch next`'s prescription as universal is exactly what produced `RV-355`
`F-1`'s claim that the collapse "cannot drive verify, conclude, or reap" — true of
Mode B, false of the retained path.

The MCP funnel tools and `funnel_machine` are still **retained, not deleted**
(`D6`): `funnel-machine.md` is generated from the code table, the table does not
change, and `REQ-335`'s tier stays `pending` as a contract. What lands in the
`REV` is `SPEC-021` `REQ-384` and `REQ-387`, narrowed (`DEC-217`, `DEC-218`). If a
reviewer judges retiring Mode B unacceptable, the argument to make is against
`OQ-1`'s unbound-fork settlement — not against this entry.

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

**`OQ-5` — re-cut: macOS parity is *not* by construction, at exactly one place.**
Earlier drafts claimed the claude profile changes exactly the two tokens the pi
profile parameterises, so parity held by construction. `RV-355` `F-2` falsified
that: `sandbox_exec_argv` (`jail.rs:627-655`) never emits `DOCTRINE_WORKER`, and
macOS worker identity rides the disk marker `DEC-207` deletes. So the macOS path
needs a change the Linux path does not, and it is **in the binary rather than the
script** — the trailing `env` token gains `DOCTRINE_WORKER=1` beside `TMPDIR`
(§5.2.1, `VT-10`).

That is the whole of the exception, and it is now a design item rather than an
open question. What remains open is unchanged and is *execution* evidence: whether
the claude arm actually reaches the seatbelt path on a real mac, which needs a
different host. `IMP-429` owns the larger question of where the prefix lives and
macOS parity with it. `VT-10` is deliberately a pure argv assertion over
`ResolvedMac`, so it is Linux-testable and does not wait on that host.

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
| `DEC-211` | one `REV` of this slice's own plus a hand anchor sweep — **method retained, enumeration superseded by `DEC-218`** | §3.1, §5.6 |
| `DEC-212` | (narrowed) the confinement control, its skip-not-pass rule, and the authored-sink evidence requirement | §9 |
| `DEC-213` | re-scope to the arm collapse alone; linked worktree, incumbent import | §1, §5.1 |
| `DEC-215` | typed hand-back by argv (`--output-format stream-json`); `--json-schema` unspent | §5.2.1 |
| `DEC-216` | no MCP in the worker now; privileged worker tools later live **outside** the confinement in their own binary | §5.2.1, §4 |

### 7.1b Records this review stage takes

Two decisions were settled *during* the review pass, from `RV-355`. They are
**proposed, not accepted** — the owner has not attested them — and they are cited
throughout on that basis.

| record | what it settles | where it lands here |
|---|---|---|
| `DEC-217` | Mode B retires with the in-session arm — a consequence of `OQ-1`'s unbound-fork settlement, not a new judgement | §1, §2.1, §2.5, §5.2.2, §6 `OQ-2` |
| `DEC-218` | the `REV` target set, re-derived from the entities, at six | §3.1, §5.6 |

`DEC-214` records that the standing `governance-confirmed` attestation means
*confirmed against a superset of the current scope* — a narrowing re-scope does
not oblige re-confirmation.

**An earlier draft of this subsection got the mechanism wrong and is corrected
here.** It argued that `DEC-218`'s growth of the `REV` target set obliged
re-confirmation, and that declaring `inq-13`/`inq-14` had invalidated the act.
Both halves were false. `governance-confirmed` binds the **governing edge set** —
this slice's outbound `governed_by` edges plus its `references` edges in role
`concerns` — which is what *binds* the design, not the `REV` target set, which is
what the design *modifies*. The two overlap but are different sets. And the act's
own coverage is inert by construction (`gate.rs:565`): the observed edge set does
all the invalidating work, so declaring an inquiry node cannot touch it.

**What actually obliged re-confirmation was the graph being wrong.** §3.1 named
`POL-002` and `STD-001` as governing this design, and neither had an edge;
`ADR-012` had been recorded as `references(concerns)` where every sibling slice
and this slice's own three other ADRs use `governed_by` — an artefact of
`--descriptor` being valid only on a `concerns` edge. Those three edges are now
corrected, which moves the observed fingerprint and expires the attestation at the
gate. Re-attesting is therefore not ceremony: it is confirmation over a governing
set that genuinely changed, and the first one taken over a set that matches §3.1.

One projection caveat for whoever reads this next: `doctrine design resume` and
`design show` render `governance-confirmed — current` regardless, because that
column is computed from the snapshot alone (`render/envelope.rs:902`) and never
sees the derived edge fingerprint. Only the gate compares them
(`gate.rs:1402-1408`). Do not read the projection as evidence the act still binds.

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
implementation this slice exists to remove. The decision holds; its stated ground
does not. Harness does **not** reduce to two tokens — §5.2.1 now enumerates four
harness asymmetries and four platform ones (`RV-356` `F-2`). One script is still
right, because the alternative is two scripts carrying the same enumeration twice.

**`D5` — `NextKind::Spawn`'s prose loses its arm split.** `dispatch.rs:6702`
is arm-agnostic by design but currently enumerates two arms; after the collapse
it names one. This is one of the few binary-tier strings the change actually
reaches.

**`D6` — the MCP funnel tools are retained, not deleted.** See `OQ-2`. The
boundary argument survives `DEC-217` intact: `SPEC-021`'s funnel-*cadence*
responsibility (line 15) still does not change, the transport does not move, and
deleting a machine whose governing prose is out of scope would put the code and
the spec out of step in the direction that does not fail loudly. What `DEC-217`
adds is honesty about what retention buys — the machine is retained *unreachable*,
and `REQ-384`/`REQ-387` say so in the `REV`.

**`D7` — the Linux arm gains a `have_bwrap` capability probe.** `DEC-208`'s
fail-closed argument leans on a *named* refusal, and §4 claimed the surviving arm
already delivers one. It does not on Linux: `pi-spawn-confined.sh:116-140` puts a
literal `bwrap` token in `PREFIX` and execs it under `timeout`, never calling
`have_bwrap` or shelling `worktree jail-prefix`; the named path exists only on the
macOS arm via `jail_prefix.rs:114-140`. Missing `bwrap` still fails **closed** — a
missing binary is a non-zero exec, and the empty-`PREFIX` guard is separate — but
**unnamed**. *Alternative considered:* restate §4 and §5.4 step 3 as "fails closed
at exec, unnamed on Linux". Rejected — `have_bwrap` is one of the four primitives
`DEC-206` already re-homes to `jail.rs`, so the caller is free by the time the
spawn phase runs, and the probe is a handful of lines against a claim the design
leans on twice. The claim could not stand as written either way (`RV-355` `F-6`).

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

**`R4` — the two harness profiles diverge at the reap, and the design named the
wrong event.** `pi --mode rpc` never self-exits and is polled for
**`agent_settled`** — **not** `agent_end` (`ISS-293`, `scripts/lib/pi-reap.sh:68-82`);
`claude -p` exits. A generalised script that keeps pi's fifo/keepalive/reap
machinery on the claude path would hang to the backstop on every phase, and one
that drops it for pi would break the production arm.

*The sharper hazard is the contract, not the machinery.* Earlier drafts named
`agent_end` as the pi arm's completion signal in §5.2.1, §5.4 step 5 and here.
`agent_end` carries the accumulated conversation state, so it sits arbitrarily far
from EOF — 684,768 bytes measured on one census turn — and a windowed poll for it
**never fires on a real turn**: every spawn runs to the backstop holding a live pi
and an open API session, *and nothing warns, because the output lands on time so it
looks clean*. Retaining `pi_await_and_reap` unchanged avoids an immediate
regression, but `R4`'s whole point is that the generalised script is where the two
profiles merge — and a merge implemented from the design's stated contract would
reintroduce the documented full-backstop burn. *Mitigation:* the correct event is
now named in §5.2.1, §5.4 step 5 and here, with `ISS-293` cited so the next reader
meets the measurement rather than the folklore; the incumbent's either-match
(`agent_settled|agent_end`) is called out as behaviour to preserve, not to tidy;
and the behaviour-preservation gate (§9) catches a regression on the pi side
(`RV-355` `F-7`).

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

**`R8` — this design's own surveys under-count.** Six times now, always low, and
twice found only by an external reviewer (`RV-355` `F-4`, `F-5`). It has fired on
**both halves**. On the governance half the largest miss — `ADR-008`, seven
regions, absent from the target set entirely — was found only by re-deriving from
the corpus instead of from `DEC-211`. On the code half, §5.6's hand-built impact
list missed `doctor_checks.rs` (a compile-breaker), `jail.rs`'s whole decision
layer (a clippy gate-breaker, ~300 production lines), `commands/guard.rs`, a
`justfile` leg and six test files.

The failure mode is structural, not careless, and it is the same one twice: a
survey answered from the author's model of the system inherits that model's blind
spots, and prose does not fail to compile. `ADR-008` is the worst case — project
local, so no `[[source]]` anchor and no `spec validate` leg points at it, and
nothing would have gone red.

*Mitigation:* derive, don't recall, and record the derivation method rather than
only its result. `DEC-218` does that for governance; §5.6's closing note does it
for code. §3.1 and §5.6 state their counts explicitly as a **floor**, the `REV`
phase re-derives from the entities, and the plan phase runs the symbol census
mechanically against the real deletion rather than reading §5.6's table. `VH`
(§9.4) is the leg that checks the governance half; the code half is checked by
the build itself, which is why the two misses that would not compile matter less
than the ones that would.

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
| `VT-10` | the macOS confinement prefix carries worker identity | `sandbox_exec_argv(&ResolvedMac)` emits `DOCTRINE_WORKER=1` in its trailing `env` token. **Pure over `ResolvedMac`, so Linux-testable** — it does not wait on a mac (`F-2`, `OQ-5`) |
| `VT-11` | the Linux arm refuses by name, not merely by exec failure | with `bwrap` absent from `PATH`, the spawn script exits non-zero **naming `REASON_NO_BWRAP`** before any fork is spawned (`D7`, `F-6`) |
| `VT-12` | the check gate moved altitude and the scope belt did not | `classify_import` still refuses the `.doctrine/`/`.claude/` touch (pre-apply, hard); the post-import gate resolves `CheckKind::Prove`, never `CheckKind::Commit` (§5.2.5, `INV-2`) |

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

The fork binding is **not** a precondition of this leg (§6 `OQ-1`): the fork
stays unbound as the pi arm's already are, the funnel row is named by the
orchestrator's explicit `PHASE-NN`, and no reader on the retained path consults
the binding.

### 9.4 By human (`VH`)

Two claims, and the second is the one `R8` exists for.

1. `ADR-011`'s corrected text and the `SPEC-012` / `SPEC-021` requirement
   dispositions accurately describe the shipped arm, and no `[[source]]` anchor
   points at a deleted file (the anchor sweep, `R7`).
2. **The `REV`'s target set was re-derived from the entities at the `REV` phase,
   not read off §3.1's table.** The sweep of `.doctrine/adr`, `.doctrine/spec`,
   `.doctrine/policy` and `.doctrine/standard` for the deleted mechanisms was
   re-run against the corpus as it then stood, every hit was read in context, and
   any entity or region beyond the six is recorded rather than absorbed
   (`DEC-218`, `R8`). A `REV` that matches this design's counts exactly is a
   result worth stating; a `REV` that merely *asserts* it re-derived is not.

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

2. **`OQ-1`, the fork binding, and what it costs.** Settled at drafting: the
   collapsed arm's forks stay unbound, exactly as the pi arm's already are. The
   claim worth attacking is the census, not the conclusion — a third reader of
   `require_binding` reachable on the retained path reopens it. Note the price
   `RV-355` `F-1` surfaced: unbound forks and a live Mode B are mutually
   exclusive, so this settlement *is* `DEC-217`.

3. **`OQ-2`, Mode B's retirement.** Retiring an orchestration mode as a
   *consequence* of a fork-binding choice is the largest thing this design does
   that its scope does not name. Mode A — the main-thread orchestrator, which
   never consults the funnel record — is the retained path and is what the pi arm
   runs in production, so the collapse is not the funnel break it can look like
   from `dispatch next`'s prescription. A reviewer may reasonably judge that a
   retired mode should be deleted rather than left unreachable, or that `SL-255`
   should widen. That is a scope argument, and it belongs here rather than at
   audit.

4. **`R4`, the reap divergence.** The generalised script has exactly one
   structural difference between profiles. If the merge is done carelessly, the
   pi arm's production reap breaks or the claude arm hangs to its backstop every
   phase. The behaviour-preservation gate catches the first; only the `VA` leg
   catches the second.

5. **The governance target set (§3.1, `DEC-218`, `R8`).** This is where the
   design has been wrong most often and most quietly. `ADR-011`'s count went four
   → five → eight → **eleven**; `SPEC-012` went one responsibility line → four
   falsified requirements; `ADR-012` went "not touched" → three regions;
   `ADR-006` went two corrections → nine; and `ADR-008` went **unlisted** → seven
   regions and a sixth target entity. Every correction came from reading an entity
   end to end rather than a summary of it, and two came from an external reviewer.
   A reviewer who reads only the decisions will inherit whichever count they land
   on. **Read the entities.** `ADR-008` is the one to read first — it is
   project-local, nothing anchors at it, and `D-B6` is built entirely from
   mechanisms this slice deletes.

6. **What is *not* here.** Clone provisioning, worker self-commit, the fetch
   transport, capsule work, `REQ-335`'s mediated-write tier, and solo
   `/worktree` isolation. A finding that this design should have addressed any of
   them is a finding about `DEC-213`'s split, not about the draft.


