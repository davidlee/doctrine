 ▐▛███▜▌   Claude Code v2.1.198
▝▜█████▛▘  Fable 5 · Claude Max
  ▘▘ ▝▝    /workspace/doctrine


❯ /model
  ⎿  Set model to Fable 5 and saved as your default for new sessions

❯ context: [SL-206 phase 07]: direction doc unjail-direction.md written +
  linked from design.md §5 banner, committed f749f829. Captures: workflow form
  dead (3 walls), A2+orchestrator-unjail-nomination direction
  (SubagentStart→allowlist→pretooluse PassThrough), corrected harness facts
  (nested no-fire lie), ADR-008 security weight, and the POC (P1 unjail
  round-trip gates A2; P2 logger-shim settles D'). Evidence index complete.
  Awaiting operator POC → `/design` re-open on pass.

  backround: `doctrine rfc show 018`

  attempting to design a functional parametric workflow to drive slices to
  completion (ultimately), with e.g. alternating orchestrator + worker agents.

  also questioning in the back of my mind IMP-269 (`/fork` is unusable in
  doctrine + bwrap at present due to inability of forked agents to write), and
  wondering if any of this is evidence that the whole mechanism for detecting
  claude arm agent privilege is sane.

  Your role: read and understand the empirical findings & design space; see if
  you have a stroke of genius which solves everything neatly; failing that,
  provide analysis on current design options.

● Reading direction doc, design, RFC-018 first.

  Read 2 files, listed 1 directory (ctrl+o to expand)

● Read design + notes + IMP-269 next.

  Read 2 files (ctrl+o to expand)

● Now IMP-269 + case-notes + jail source (pretooluse decide).

  Searched for 1 pattern, read 2 files, ran 1 shell command (ctrl+o to expand)

● Read everything: direction doc, design §5 + PHASE-05/06/07 deltas, RFC-018,
  IMP-269, pretooluse.rs, dispatch_record.rs. No single stroke of genius that
  dissolves all walls — but found one probe-shaped gap in the direction doc that
  could make the unjail nomination unnecessary, one security hole in the
  nomination as specified, and one unification that answers your back-of-mind
  IMP-269/privilege-sanity question. In order:

  1. Gap: direction doc under-argues why confined A2 fails — pivotal unknown is
  nested WorktreeCreate, not the jail

  Doc §3 says A2 orchestrator "is confined by worktree-jail (wall #3 applies)".
  But wall #3 was measured on a workflow no-iso leaf inheriting primary cwd.
  Agent-tool no-iso orchestrator with main thread parked at coord root lands at
  coord root = linked worktree ⇒ Jail(coord), not deny — exactly SL-199
  placement (your own placement spike arm A proved this geometry, just via
  Workflow). From there, check what confined orchestrator actually lacks:

  - arm base: CLI write to coord .doctrine/state — in-jail OK
  - import/conclude/reap: server-side MCP — OK (Mode B, proven)
  - DispatchRecord: working-tree TOML at
  <coord>/.doctrine/state/dispatch/record/<name>.toml (dispatch_record.rs:40) —
  in-jail writable, hand-stamp possible
  - spawn worker: retains Agent tool (proven, ae3a5dd36e3ea8c41) and jail walls
  only Bash/Edit/Write, so Agent passes

  So confined A2 dies only if the nested worker spawn's WorktreeCreate hook
  no-fires (harness-born fork then mints detached/wrong-base, and
  resolve_agent's branch check needs dispatch/<name> — a git op no in-jail actor
  can fix). SubagentStart nested no-fire is proven; WorktreeCreate nested
  behavior is unprobed — not in RFC-018, no memory. That's the real gate, and
  it's cheaper than P1:

  P0 (do first, standing SL-209 rig): main thread parks at coord-209, Agent-tool
  spawns dispatch-orchestrator (no-iso), orchestrator nest-spawns one
  dispatch-worker(isolation:worktree) after arming. Observe: WorktreeCreate
  fired? fork at armed base on dispatch/<name>? worker_commit resolves? PASS ⇒
  zero confinement-boundary change, no ADR-008 amendment, ship A2 as-is.
  Notes.md already nominated almost exactly this probe; the direction doc
  dropped it when nomination arrived.

  2. Steelman for unjail anyway (why P0-pass might still not settle it)

  Unjail = relocate main thread into a nameable subagent. Payoff isn't just
  capability — it's harness-surface minimization: confined A2 depends on three
  undocumented behaviors (nested WorktreeCreate fire, nested base control,
  hand-stamp round-trip), all version-fragile per RFC-018's own thesis. Unjailed
  orchestrator needs exactly one proven fact (top-level SubagentStart fires)
  and can then run either shipped arm verbatim — including pi-style subprocess
  workers with explicit cwd binding, zero nested-hook dependence. If P0 passes
  on CC 2.1.198 you still own a fragility argument. My lean: P0-pass ⇒ ship
  confined A2 now, keep nomination as designed fallback with the POC banked;
  P0-fail ⇒ P1 immediately.

  3. Security hole in nomination as specified — escalation guard is the same lie
  you distrust

  Doc §5 gate (c): "only Agent-tool holders spawn — workers cannot spawn".
  Incomplete. Any general-purpose subagent holds Agent (jail walls only
  Bash/Edit/Write; Agent passes through). Jailed subagent → Agent(subagent_type:
  dispatch-orchestrator) → grandchild would be nominated → unjailed. What
  prevents this today is only §4.3-3: nested SubagentStart silently no-fires. So
  the boundary's escalation-resistance rests on the very undocumented no-fire
  the design elsewhere refuses to trust — and a harness release that fixes the
  no-fire (plausible; it looks like a bug) silently opens jailed-subagent →
  unjailed-grandchild. Fail direction of that change is an escape, not
  confinement.

  Fix, cheap and idiomatic: nomination requires pre-armed intent. Single-slot
  nomination token (existing arming pattern,
  mem.fact.dispatch.single-slot-arming-rendezvous): main thread arms "next
  dispatch-orchestrator spawn may be nominated", SubagentStart hook consumes
  token + writes allowlist entry; no token ⇒ no entry ⇒ jailed. Two-man rule:
  type match (identity) ∧ armed token (intent from the one unconfined
  principal). Token file under primary .doctrine/state — outside every jail,
  unwritable by confined agents (verified: coord jail binds only .dispatch/SL-n;
  Edit/Write pathcheck denies primary-root writes). This should go into §5
  before any /inquisition — it's the finding an inquisitor will land.

  4. The unification — your privilege-sanity question

  Is agent_id-keyed detection sane? As a floor, yes — it faithfully mirrors the
  harness's actual trust topology: exactly one unconfined principal
  (human-attended main thread), everything with an agent_id fail-closed. IMP-269
  is that floor working as designed.

  What's insane is treating it as the whole policy language. It can express
  three postures only: anonymous→god, worktree→jailed, else→deny. Every recent
  pain point is the same missing fourth thing — a per-agent posture table:

  - A2 nomination allowlist = posture unjailed keyed by agent_id
  - IMP-269 fork = wants posture readonly or jail(scratchpad-widened)
  - per-worktree jail/<name>.toml (already shipped, resolve_provisioned_policy)
  = posture-widening keyed by worktree basename
  - DispatchRecord = sibling trust anchor, same directory pattern

  These are four fragments of one agent-posture registry: agent_id → {unjailed |
  jail(wt)+policy | readonly | deny}, provisioned at spawn seams by trusted
  actors (hooks run harness-side, outside jail), default = today's inference,
  every provisioning miss degrading to more confinement. Rides the existing
  .doctrine/state/dispatch/{jail,record}/ pattern — DRY, not new machinery.
  Don't build it speculatively; but design the A2 allowlist as the first row of
  it (a TOML with posture = "unjailed", not a bare id list), so IMP-269's
  eventual carve-out is a vocabulary addition, not a third parallel mechanism.
  That's the ADR-008-amendment shape I'd propose: not "an unjail exception" but
  "explicit posture provisioning above the fail-closed floor".

  5. IMP-269 cheap probe before any carve-out

  Agent tool fork inherits parent cwd — but does /fork accept/honor isolation:
  worktree? If yes: forked agent lands in linked worktree ⇒ Jail(wt) ⇒ writes
  work today, config-level fix, IMP-269 closes without touching the wall. Worth
  one probe before designing postures for it. If no: fork posture row in the
  registry (§4) is the home.

  6. Loose ends

  - RFC-011 budget lever is dead in every A-form, not just workflow — Agent-tool
  spawns return no token telemetry to the parent. Receipt-boundary accounting
  is the fallback; the design.md §1 motivation paragraph needs rewriting at
  re-open, not just §5.
  - A1 vs A2 converge under unjail: unjailed orchestrator = relocated main
  thread, so the A1 skill body is the A2 orchestrator prompt. Layer it: package
  /drive-slice as skill (A1, shippable now, zero unknowns), nomination later
  makes same skill one-spawn-able. OQ-A2a resolves as "both, sequenced" rather
  than either/or.
  - P2 (logger shim) still worth running regardless of P0/P1 — settles D' and
  fills RFC-018's workflow-leaf hook-surface blank.

  Recommended order: P0 → (P1 iff P0 fails or fragility argument wins) →
  arming-token amendment to §5 → /design re-open with allowlist-as-posture-row
  shape → P2 opportunistically.

  [SL-206 phase 07]: unjail-direction analysis — P0 nested-WorktreeCreate probe
  proposed; nomination escalation gap + posture-registry unification


❯ updates as they come to light:
  P1 — PASS, in full. Both halves.

  P1a (mechanism, hook-boundary): allowlist membership flips a jailed subagent
  deny → PassThrough, and only for the listed agent_id (control denied). ✓

  P1b (end-to-end, live orchestrator): decisive — and it exceeded the spec:

  - SubagentStart FIRED — matched dispatch-orchestrator, carried
  agent_id+agent_type, wrote the allowlist before the orchestrator's first Bash
  (sync-blocking confirmed).
  - pretooluse saw agent_id∈allowlist → PassThrough ×3 (unconfined).
  - Orchestrator's Bash ran unwrapped: git rev-parse HEAD ✓, coord-tree read
  git -C .dispatch/SL-206 rev-parse → 9ba5c0a6 (a jailed agent cannot run
  this), empty commit landed 98835cc. RESULT: ALLOWED-and-committed.

  Surprise finding — contradicts a recorded memory. This session has
  CLAUDE_CODE_CHILD_SESSION=1, yet SubagentStart fired. mem_019ec84b §4.3 lie
  #3 claims it silently no-fires in a child session. Refuted here (or the true
  condition is narrower — no-fires only for spawns nested deeper than a
  child-session's own main thread). Also: the settings.local.json hook edit
  hot-reloaded — no restart needed.

● P1 full pass changes the board. Three consequences, one urgent.

  1. The refuted memory cuts both ways — and it arms the escalation hole now,
  not later

  (Security — writing this part plain.)

  The no-fire claim in mem_019ec84b was doing two jobs in the direction doc:
  §4.4 used it to explain why workers need hand-stamping, and — per my earlier
  analysis — it was the only thing standing between a jailed subagent holding
  the Agent tool and minting a nominated, unjailed grandchild
  (Agent(subagent_type: dispatch-orchestrator) → SubagentStart fires → allowlist
  entry → PassThrough). Your P1b result shows SubagentStart firing where the
  memory said it wouldn't. Two readings:

  - Narrow reading: the true no-fire condition is "spawns initiated by
  subagents", and a child session's own main thread is exempt. Then the
  escalation defense may still hold — but it is now an unverified inference,
  since the direction doc's worker-no-fire claim was derived from the same
  refuted memory, never measured directly.
  - Broad reading: SubagentStart fires for nested spawns generally. Then the
  escalation hole is live today: any jailed subagent with Agent can
  self-nominate a grandchild orchestrator, and absence-of-entry never saves you
  because the entry gets written.

  Either way the conclusion is the same: the arming-token (single-slot
  nomination intent, consumed by the SubagentStart hook) moves from defensive
  hardening to a ship-blocker for the nomination leg. Don't let the allowlist
  mechanism land in pretooluse without it. Type-match alone is
  forgeable-by-invocation; the two-man rule (matcher ∧ armed token from the
  unconfined principal) is what makes the boundary hold under either reading.

  2. Same probe now answers three questions — run it next

  One nested-spawn probe (orchestrator subagent →
  dispatch-worker(isolation:worktree), logger shims on SubagentStart +
  WorktreeCreate) settles:

  1. Escalation reading — does SubagentStart fire for a subagent-initiated
  spawn? (Fires ⇒ hole live, token mandatory. No-fires ⇒ narrow reading, token
  still cheap insurance.)
  2. P0 / confined-A2 viability — does WorktreeCreate fire nested? If both fire,
  the shipped worker machinery (create-fork provisioning, DispatchRecord,
  worker_commit) works verbatim from a confined orchestrator — hand-stamp
  unnecessary, §4.4's workaround moot.
  3. Direction-doc §4.3 correction scope — narrows or deletes lie #3.

  Cheap: same rig, shims already exist from P1.

  3. Bookkeeping the pass forces

  - Correct mem_019ec84b now (edit, don't delete — record the refuting probe +
  CLAUDE_CODE_CHILD_SESSION=1 context; condition unknown-pending-probe). It's
  load-bearing in unjail-direction.md §4.3/§4.4 — a future agent reading the
  stale version re-derives wrong designs.
  - Record new facts: nomination round-trip proven (P1a control + P1b commit
  98835cc); settings.local.json hook edits hot-reload, no restart. Both RFC-018
  §1d rows.
  - Odd datum worth one line in the memory: this session carries
  CLAUDE_CODE_CHILD_SESSION=1 at what you're treating as top level — so that env
  var is not a reliable "am I nested" discriminator, which also means hooks
  can't use it to distinguish top-level from subagent-initiated spawns for the
  token check. The token file is the discriminator that works.

  Net position update: A2-unjail is now mechanically proven end-to-end;
  confined-A2 may also be viable pending the nested probe. The remaining
  decision is pure design: harness-surface minimization (unjail) vs
  zero-boundary-change (confined) — with the arming token required in the unjail
  arm regardless. /design re-open is unblocked once the nested probe lands; §5
  security section needs the token + the refuted-memory rewrite before
  /inquisition.

  [SL-206 phase 07]: P1 PASS analysis — escalation hole now live-or-unverified,
  arming token promoted to ship-blocker; one nested-spawn probe settles
  escalation + P0 + memory correction

  [SL-206 phase 07]: clarified — direction doc =
  .doctrine/slice/206/unjail-direction.md


❯ does any of this suggest an obvious out for the `/fork`? afaict the
  user-facing API doesn't expose isolation:worktree, and typically it's run in
  a top-level session without that defined. it *might* be worth a probe on a
  subagent def with / without it defined on the subagent - but that hardly
  seems likely to rescue the common case of "I want this to just work in the
  claude main session thread".

● Yes — P1 built the out. Same seam, one more matcher: /fork from the main
  thread is a top-level spawn, exactly the case P1 just proved fires
  SubagentStart sync-blocking with agent_id + agent_type in the payload. If a
  fork spawn carries a matchable type (fork, or whatever the payload actually
  says), a SubagentStart matcher nominates it → allowlist → PassThrough → fork
  writes the primary tree like the main thread. No isolation:worktree needed, no
  user-facing API change, "just works" in the main session. That's IMP-269's
  carve-out candidate (c) ("narrow, opt-in main-tree write grant gated by
  something forge-proof") landing for free on the nomination machinery —
  posture-registry row #2.

  Two gates before it's real:

  1. Does /fork fire SubagentStart, and what does the payload say? One probe:
  logger shim, run /fork, dump full stdin JSON. Need the discriminating field
  (agent_type: "fork"? something else?). Cheap, independent of everything
  queued.

  2. The escalation condition — stricter here than for the orchestrator.
  (Security, plain prose.)

  The orchestrator nomination has three gates; the strongest is that
  dispatch-orchestrator is a committed, reviewed agent def. "Fork" is a built-in
  subagent type — any jailed subagent holding the Agent tool can invoke it, and
  the def can't be removed or audited. So if subagent-initiated spawns fire
  SubagentStart (the open question your P1 surprise raised), an unconditional
  fork matcher is a straight jailbreak: worker spawns a "fork", fork is
  nominated, fork writes the primary tree. For forks a top-level discriminator
  is therefore mandatory, not defense-in-depth — and you can't use the
  arming-token pattern to fix it, because the user types /fork directly and
  there's no wrapper moment to arm intent.

  Candidate discriminator (probe-able alongside gate 1): stamp the top-level
  session_id into runtime state via a SessionStart hook; the fork matcher
  nominates only when the SubagentStart payload's spawning-session id equals the
  stamp. Whether the payload carries a usable parent/session id is unknown —
  the same full-payload dump answers it. If the already-queued nested probe
  returns the narrow reading (subagent-initiated spawns no-fire), the
  discriminator becomes belt rather than load-bearing — but per RFC-018's own
  thesis, don't rest a primary-tree write grant on an undocumented no-fire; P1
  just refuted one of those memories.

  Trust framing worth writing down when this reaches design: a user-initiated
  fork is main-thread-equivalent by intent (it's "parallel me"); the
  multi-writer hazard in the primary tree is a coordination problem (AGENTS.md
  conventions), not a confinement problem. The confinement question is solely
  "was this fork actually user-initiated" — which is exactly what the
  discriminator answers.

  So: append the fork-payload dump + session-id field check to the nested-probe
  session. If both come back friendly, IMP-269 closes as one matcher + one hook
  stanza on machinery you've already proven.

  [SL-206 phase 07]: /fork out via nomination seam — needs fork SubagentStart
  payload probe + top-level discriminator (mandatory for built-in type)


❯ new results in:
  Fixed and validated. Both tasks done.

  P0 — nested-spawn probe: RAN, both questions settled (committed 249ddca0)

  Rig: confined dispatch-orchestrator @ coord .dispatch/SL-209 nest-spawned one
  dispatch-worker; observer SubagentStart + WorktreeCreate shims. No
  allowlist/boundary edit — natural confinement. Torn down after; logs in
  scratchpad.

  Q1 — escalation reading: LIVE. SubagentStart fired for the subagent-initiated
  worker spawn (2 log entries, one prompt_id). Payload carries no spawner id —
  a hook cannot tell main- from subagent-initiated. So a type-match-only
  nomination allowlist is escalatable: jailed Agent-holder →
  Agent(subagent_type:"dispatch-orchestrator") → auto-nominated grandchild. §5
  arming token = ship-blocker for the unjail leg, not optional. Also further
  contradicts the mem_019ec84b nested-no-fire claim (it fired nested here).

  Q2 — confined-A2 viability: STRONGLY SUPPORTED. The confined orchestrator:
  sealed out of shared .git (RO, index.lock EROFS) but wrote its own coord
  .doctrine/state/ (arming feasible); nested isolation:worktree spawn fired
  WorktreeCreate + forked at correct base (fa951846); worker_commit resolved +
  ran the full gate, refusing only at commit-gate-red = pre-existing red test
  on the SL-209 base (exit 101), not a confinement failure.

  Net — the board changed. Confined-A2 drives the shipped worker machinery with
  zero boundary change → the unjail/nomination path may be unnecessary (no
  ADR-008 amendment). Unjail is now only worth it for harness-surface
  minimization, and it costs the mandatory token. Design fork is now
  "confined-A2 (cheap, no amendment) vs unjail (surface-min, token-gated)". Two
  side-facts: dispatch/209 base is red (needs green before any real drive);
  worker_commit red-gate refusal embedded a ~295k-char transcript (RFC-011
  case-note filed).

  Records: mem_019f364eb2a5 (security), mem_019f364ec31e (viability). Direction
  doc §6 P0 RESULT + §7 triggers updated.

● Board read: confined-A2 wins on the evidence. Recommendation and fallout:

  Design fork — call it

  Confined-A2 now has every leg proven live: placement (P0: coord jail, arming
  write OK), nested WorktreeCreate + correct-base fork, auto-stamped worker,
  worker_commit resolving and running the full gate. Zero boundary change, no
  ADR-008 amendment, no token machinery, no inquisition. Unjail's remaining
  value is harness-surface minimization — but that argument just weakened: the
  surfaces it would insulate against (nested WorktreeCreate fire, nested base
  arming) moved from speculated to empirically proven on 2.1.198, and unjail now
  carries a mandatory arming token. Cost asymmetry is stark. My call:
  confined-A2 primary; bank P1 + the token design in the direction doc as the
  named fallback if a harness release breaks nested hooks. The /design re-open
  then targets §5 with the confined shape — mostly the shipped SL-199 model plus
  the Agent-tool driver framing, which is a much smaller re-lock than a
  confinement-boundary change.

  Also note what Q2 quietly retired: direction doc §4.4's hand-stamp workaround
  is moot — nested SubagentStart fires, so workers get auto-stamped by the
  existing matcher even when orchestrator-spawned. The direction doc's whole
  §4.4 reasoning chain was built on the refuted memory.

  The generalized security lesson (plain prose)

  P0's Q1 finding is bigger than the nomination leg: every SubagentStart
  type-matched grant is invocable by any jailed Agent-holder, and the hook
  cannot tell who spawned it. That's now a standing property of the seam, not a
  per-design footnote. Audit the existing grants against it:

  - Worker provisioning matcher (shipped): a jailed rogue subagent can spawn a
  "dispatch-worker" and acquire a provisioned fork + worker_commit rights.
  Bounded — commits land on an ephemeral dispatch/<name> fork branch behind the
  commit gate, nothing reaches coord/trunk without an orchestrator import.
  Acceptable, but worth one line in ADR-008's threat model so it's a decision,
  not an oversight.
  - Rule for the posture registry: grant power must scale with intent evidence.
  Type-match alone buys bounded, gated capabilities (worker tier); anything
  unbounded (PassThrough) requires an armed token. That's the sentence the
  eventual ADR amendment should carry.

  /fork fallout — the payload finding hurts

  "No spawner id in payload" closes the clean discriminator I proposed. Check
  one thing in the captured P0 logs before giving up: do the two SubagentStart
  entries (top-level orchestrator spawn vs nested worker spawn) differ in any
  field — session_id, cwd, env? If the nested spawn's payload carries the
  subagent's session id and a SessionStart hook can stamp the top-level id, the
  discriminator survives. If truly indistinguishable, IMP-269's "just works"
  degrades to armed-fork: a doctrine-side arming command the user runs before
  /fork, token consumed by the fork matcher. Ergonomics tax, but it's the only
  forge-proof shape the seam supports — same two-man rule, human as the arming
  principal.

  Loose ends

  - mem_019ec84b still uncorrected — you recorded two new memories but the
  refuted one is the load-bearing lie cited by unjail-direction.md §4.3, RFC-018
  §1d, and the shipped dispatch-mechanics reasoning. Edit it now (two
  refutations logged: P1b child-session fire, P0 nested fire); leave the
  original claim as a superseded note with probe ids.
  - 295k-char transcript in the worker_commit refusal — case-note filed, but
  that's a real defect, not just an observation: gate output should be
  truncated/summarized in the refusal detail. Backlog item (backlog new, issue)
  so it doesn't evaporate.
  - dispatch/209 base red — green it before the confined-A2 acceptance re-drive,
  else EX-1 stays unfalsifiable (every drive halts at commit-gate-red
  regardless of orchestration correctness).

  [SL-206 phase 07]: P0 settles design fork — recommend confined-A2 (zero
  boundary change), unjail+token banked as fallback; type-match grant rule for
  ADR-008; /fork degrades to armed-fork unless payload logs show a session
  discriminator

✻ Crunched for 1m 6s

❯ on SubagentStart

  === field-by-field (python) ===
  DIFF  agent_id:
        orch: add711b568b9d372e
        wrkr: afd0c43fe5815021e
  DIFF  agent_type:
        orch: dispatch-orchestrator
        wrkr: dispatch-worker
  DIFF  cwd:
        orch: /workspace/doctrine/.dispatch/SL-209
        wrkr:
  /workspace/doctrine/.dispatch/SL-209/.worktrees/agent-afd0c43fe5815021e
  SAME  hook_event_name:
  SAME  prompt_id:
  SAME  session_id:
  SAME  transcript_path:

● Payload diff confirms it: session_id/transcript_path/prompt_id all attribute
  to the top-level session regardless of nesting depth — SubagentStart carries
  no parentage at all. And cwd is no discriminator either (a no-iso subagent
  inherits the primary cwd, so "cwd = primary root" doesn't prove
  main-thread-initiated).

  But this points at the fix rather than killing it. The spawner's identity does
  exist — one seam earlier.

  PreToolUse fires on every tool call, including Agent, and its payload carries
  the caller's agent_id. Your own run_pretooluse comment states it fires on
  every call; the jail merely routes only Bash/Edit/Write today. So the bit
  SubagentStart lacks — main-thread vs subagent-initiated — is sitting in
  PreToolUse(Agent): no agent_id = the unconfined principal spawning; agent_id
  present = a subagent spawning. The harness doesn't expose parentage at
  start-time, but you can reconstruct it with a two-seam handshake:

  Option A — spawn-gate deny (my lean, simplest). Add matcher Agent to the jail
  hook: when agent_id present ∧ caller ∉ allowlist ∧ tool_input.subagent_type ∈
  privileged set (dispatch-orchestrator, fork), deny the spawn itself.
  Escalation closes at the source; SubagentStart type-match becomes trustworthy
  again, and the nomination needs no arming token at all — the token's whole job
  was compensating for unknown parentage. Nominated (unjailed) actors pass the
  gate by allowlist membership, consistent with their main-thread-equivalence.

  Option B — arm-and-consume. PreToolUse(Agent) with no agent_id writes a
  single-slot token typed by subagent_type; SubagentStart consumes it to
  nominate. Finer-grained, but has a genuine hole Option A doesn't: if
  user-typed /fork spawns without a tool call, no arming moment exists and
  you're back to absence-logic (untainted SubagentStart(fork) ⇒ user-initiated),
  which fails-closed under concurrency but is uglier.

  Note Option A handles /fork cleanly either way: a subagent-initiated fork必
  goes through the Agent tool ⇒ gated; a user-typed /fork either fires
  PreToolUse with no agent_id (passes) or bypasses tools entirely (never gated)
  — both land on "user forks pass, subagent forks denied", which is exactly the
  trust model. Then the fork SubagentStart matcher nominates unconditionally on
  type, safely.

  Two cheap probes to confirm before designing on it:
  1. PreToolUse(Agent) payload shape — does tool_input carry subagent_type
  verbatim, and does the call from a subagent carry that subagent's agent_id?
  (Expect yes on both; docs + one shim run.)
  2. Does user-typed /fork fire PreToolUse(Agent)? Either answer is fine for
  Option A; determines whether Option B is even coherent.

  If probe 1 lands, the security posture across the board simplifies: type-match
  nomination + spawn-gate = closed loop, no token, no ADR-008-amendment-sized
  machinery — and IMP-269's "just works in the main session" survives intact.
  Worth running before the /design re-open so §5 gets written once.

  One caveat to check in probe 1: whether PreToolUse fires for the Agent tool at
  all in your matcher config — the "fires on every tool call" claim is from
  your hook's own doc comment describing harness behavior; the shipped settings
  matcher only registers Bash|Edit|Write, so this needs the matcher widened, and
  /reload-plugins hot-reload proven for the new matcher shape.

  [SL-206 phase 07]: SubagentStart payload has no parentage — but
  PreToolUse(Agent) carries spawner agent_id; spawn-gate deny (Option A) closes
  escalation without arming token; 2 probes queued

