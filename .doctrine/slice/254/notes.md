# Notes SL-254: Collapse dispatch onto one subprocess arm

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-13 · proposed (design run `dr-019ff653` live, stage **drafting** rev 35, ten sections declared + materialised, `draft.selectors` discharged; stage deliberately NOT advanced — the draft is unvalidated) · 5a68bfd9e

### Produced
- SL-247 abandoned at design; scope `## Summary` carries the dissolution (e788e1520)
- minted: DEC-202 — retire the claude in-session arm for a confined `claude -p` subprocess; EVD-023 — `claude -p` is subscription-billed, falsifying ADR-011's premise (e788e1520)
- SL-254 scoped with selectors + relations, carrying a reconcile/close survey (61c54752f)
- minted: ISS-347 — REQ-288 and SPEC-021 responsibility 2 misstate the env-marker as `.claude/` presence; it is `CLAUDECODE=1` (95a00592c)
- marker-identity scope drift corrected against DEC-202 (eca536780)
- research round complete — 5 threads, `research/research.md` assembled. **Runtime tier, gitignored**: it does not survive a state wipe, so anything load-bearing must be lifted before then
- 2 friction observations recorded; the second (`019ff631…`) is **uncommitted**
- verification gate: `doctrine check gate` NOT run this session — no code was modified, `doctrine check commit` green at each commit
- design run `dr-019ff653` opened; 10 inquiry nodes declared and all 10 resolved; exploring runbook discharged in full (124c5e44b)
- minted: DEC-203..DEC-212 — one per node; see `## Design surface triage` below for the node→record table
- minted: IMP-428 — harden the worker confinement prefix, deferred out by DEC-209
- DEC-202's `choice` corrected in place (it read as a claim about worktrees, not dispatch workers)
- 2 further friction observations recorded and committed — exploring runbook invisible until a stage transition refuses; `agent_declaration` apply emits no event row
- marker/confinement due-diligence trace: full reader/writer census of the marker
  surface. `DEC-207` amended (context + rationale + consequences 3 → 7);
  `DEC-211` `choice` extended with a second `ADR-006` §D2b correction site;
  minted `CHR-062` — prune the `SL-116` extraction `expect(unused)` in worktree
  `gc`/`import`
- `mem.fact.dispatch.worker-confinement-is-actor-based` strengthened with the
  process-vs-tree generalisation and re-attested; scoped to `shared.rs`/`land.rs`
- **both design-run gates cleared** — `governance-confirmed` and `graph-reviewed`
  accepted by the user (rev 13 → 15); `blocking-set-declared` already current
- **the re-scope EXECUTED** (`DEC-213`): `SL-254` narrows to the arm collapse;
  `SL-255` minted for clone provisioning. `DEC-203`/`204`/`211`/`212` corrected
  in place (all `proposed`, per this slice's own `DEC-202` precedent);
  `QUE-214`/`QUE-215` settled; scope document re-cut in six places
- 1 friction observation recorded — `memory edit` resets attestation on any field
  change, so one logical memory change costs a commit per step
- `governance-confirmed` left standing after the re-scope (owner's call): the
  confirmed survey was over a strict superset, and the clone half's governance
  goes to `SL-255`, which does not inherit
- `DEC-211`'s `ADR-011` site enumeration corrected in place — four → **eight**
  regions, `D5`/`D7` deferred; 1 friction observation (line-number citations into
  an authored `.md` are unstable and self-check nothing)
- **stage advanced `exploring` → `inquiring`** (rev 17), and the inquiring
  runbook cleared in full (rev 18, 19)
- `explore.research` had **regressed** — the re-scope re-cut `slice-254.md` under
  the research stamp. Repaired by superseding `research.md` in place
  (section-by-section marks for what the re-scope refuted or relocated) then
  re-baselining — **not** by a bare restamp, which would have asserted currency
  over conclusions known wrong
- minted `DEC-214` — a *narrowing* re-scope does not stale a governance
  confirmation (superset argument); a *widening* one always does. The asymmetry
  is the durable content
- scope reconciled against the accepted decisions (`inquire.scope`): two
  `ADR-011` under-counts corrected, `ADR-006` §D2b de-hedged, `SPEC-012`'s
  responsibility added as a `REV` target, `OQ-1`/`OQ-3`/the `REV-046` question
  closed out, `A2` narrowed to its hand-back leg
- minted: IMP-429 — give the confinement prefix a home, and macOS parity with it
  (46507a9e0); the sufficiency judgement's area (2), deliberately out of slice
- minted: DEC-215 (typed hand-back by argv, at parity with the pi arm) and
  DEC-216 (privileged worker tools live outside the confinement, in their own
  binary), via `inq-11`/`inq-12` — the sufficiency judgement's area (1). Scope
  objective 1 and the Non-Goals updated to match
- **inquiry closed 2026-08-13** — `graph-reviewed` and `sufficiency-accepted`
  both taken (user, in-session), stage advanced `inquiring` → `drafting` at rev
  26. All four evidence acts current. The `sufficiency-accepted` basis records
  what became of each of the three offered areas, so the judgement is auditable
  without this file
- ISS-328 (an unknown key inside `CreateRecord` is dropped, not refused) already
  had this filed from `SL-249`; added the second sighting — the first with a real
  cost. Minted ISS-349 for the comma splitter, which had a standing memory and no
  card (40f521540)
- 3 further friction observations — a concurrent agent's broad `git add` captured
  my observation record seconds after capture (symptom is an *absence* in
  `git status`, so it reads as a failed capture); and `inquire.scope` regresses
  `explore.research` by construction, because one step edits the file the other
  hashes; and `design apply`'s `CreateRecord` silently swallowed an unknown
  `facets` key (the field is `facet`), landing two records with every facet
  empty and reporting success — while `Declaration` one struct out carries
  `deny_unknown_fields` for exactly that failure mode
- **design drafted in full 2026-08-13** — ten sections declared, materialised at
  rev 34, `draft.selectors` discharged at rev 35 (35 design-target selectors over
  the §5.6 code-impact table). Research baseline restamped: `design.md`'s *arrival*
  tripped the verifier, and nothing in `research.md` needed marking
- 1 further friction observation — section document order is *creation* order and
  a batched `design apply` claims `seq` in **id-sorted** order, so `sec-10` landed
  ahead of `sec-6`; no reorder verb exists, so the repair was rotating five bodies
  onto the ids already in the right positions. Section ids now mismatch their
  headings (`sec-10` holds §6, `sec-6` holds §7, …) — cite sections by **heading**,
  not by id

### Learned
- mem_019ff650d94a7960a638913a40416165 — collide "what calls this" research findings against "what should exist" decisions at synthesis. **Second instance this slice** (DEC-204 vs thread 2's `worker_commit` reading) — strengthen from incident to standing hazard
- A gated tool is privileged **only** if it sits on the far side of a boundary the
  agent is inside. `worker_commit` relied on that implicitly and `ADR-011` never
  states it; an in-jail MCP server is exactly as confined as the agent and buys
  nothing (`DEC-216`). Corollary the user drew: the six belts were the security
  content of exposing a privileged tool from a server that also exposes
  *everything else* — a narrow binary makes most of them unnecessary rather than
  merely enforced
- The script tier shows what the orchestrator **prints**, not what it **reads**.
  `pi-spawn-confined.sh`'s `tail -40 "$OUT"` reads as an untyped hand-back; the
  actual channel is `pi --mode rpc`'s typed event stream and its `agent_end`
  completion. Nearly landed `DEC-215` backwards — prose-tailing would have been a
  *regression* dressed as the conservative option
- The reuse finding: `crates/doctrine-control` already implements clone-inside-bwrap (`provision.rs:949`, `backend/bubblewrap.rs:1110`) — see QUE-215
- `worker_commit` is **six belts**, not a bare ro-git bypass; two are not topology-coupled, and its scope belt has exactly two callers (`classify_import` import.rs:147, worker_commit.rs:94) single-sourced from import.rs:24 (DEC-204)
- `nominate`/`denominate` are a **closed loop** with `pretooluse`'s gate legs — `is_nominated` has no other reader, so they die by construction (DEC-205)
- The confinement prefix already delivers the write floor (`--ro-bind / /`, scripts/pi-spawn-confined.sh:113-131); thread 5's gaps are defence in depth (DEC-209)
- ~~ADR-006 §D2b's `SL-064` note identifies the **coordination tree by marker *absence***. Load-bearing against DEC-207~~ — **refuted 2026-08-13.** It is a doc-level contrapositive of the refuse predicate, not a mechanism: no reader anywhere concludes "coord" from absence. See `DEC-207` rationale (1)
- Worker-ness is a property of a **process**; the disk marker models it as a property of a **tree**. That single mismatch is the origin of the whole stale-marker surface (`Cause::Marker`, `--assert`, `marker --clear --operator`, `ISS-028`). `DOCTRINE_WORKER` has no stale class by construction — it dies with the process, and is set by the *same bwrap argv* that establishes the write floor
- Positive coordination identity **already shipped, marker-free**: `classify_worktree_role` (`shared.rs:77`) — linked + all-numeric `dispatch/<NNN>`. `inventory.rs` carries a **parallel implementation** of the same classifier (`STD-001`)
- `IMP-065` — which `ADR-006` §D2b still names as "the real positive-marker close" — was **closed obsolete** 2026-07-02 (`REV-018`): a positive marker is a cooperative flag not a boundary, and confinement (`SL-182`/`183`/`185`) is the genuine close. The argument that retired it retires the *negative* marker identically

### Open
- ~~**The re-scope, PROPOSED AND UNCONFIRMED**~~ — **EXECUTED 2026-08-13
  (`DEC-213`).** `SL-254` = *the claude arm becomes a pi arm* (confined
  subprocess, linked worktree, incumbent import transport). `SL-255` minted for
  clone provisioning + self-commit + fetch transport, carrying `A1` as the
  assumption it exists to verify and `DEC-204` as the analysis that becomes live
  again there
- ~~**DEC-207 is contested**~~ — **RESOLVED 2026-08-13 by the marker/confinement
  trace. `DEC-207` stands, and is now topology-INDEPENDENT** — it survives
  unchanged whether workers ride clones or linked worktrees, so the re-scope no
  longer threatens it. The trace also found one consequence `DEC-207` had missed
  (`land.rs:173`'s `bears_marker` is the only marker read that asks about
  *another* tree; it needs the branch-shape classifier as substitute, which is
  strictly stronger) and one `REV` site `DEC-211` had missed (`ADR-006` §D2b's
  dangling forward-reference to the obsolete `IMP-065`). Both are now recorded
- ~~Consequential on the re-scope~~ — **discharged.** All four corrected **in
  place**, not superseded: they are `proposed`, and `DEC-203` set that precedent
  when it corrected `DEC-202`'s choice text on exactly that ground. `DEC-204`'s
  hazard *dissolved* rather than narrowing — `classify_import` survives as the
  scope belt's enforcing caller, so `worker_commit` is deleted outright and
  nothing re-homes. `DEC-205`/`206`/`207`/`208`/`209`/`210` untouched
- ~~**`governance-confirmed` may deserve re-taking.**~~ — **DECIDED 2026-08-13
  (`DEC-214`): leave it standing.** The machine still reports all
  three acts `current` after the re-scope — it did not stale them. But the user
  confirmed governance against the *wider* scope, and the picture has since moved
  (`DEC-211`'s REV narrowed, `SL-255` carries its own unsurveyed governance). The
  gate is not blocking; this is a judgement call for the owner, not a refusal
- ~~**`DEC-211` under-counts `ADR-011`**~~ — **CORRECTED IN PLACE 2026-08-13**,
  and the count was neither four nor five. Reading `ADR-011` end to end against
  `DEC-207`/`208` found four *further* regions beyond the four named: **`D2`**
  (58-74, the core contract — the marker leaves its agnostic core, and "claude's
  `Agent` path has no worker env channel" is the load-bearing claim a confined
  `claude -p` falsifies); **`D6`** (146-207, sixty lines of fail-closed altitude
  resting entirely on deleted mechanisms, marked *asserted* not proposed, whose
  "not fail-closable" conclusion inverts under `DEC-208`); **Consequences**
  (248-259, three falsified bullets); **Verification** (265-275, two `VA` criteria
  that invert — one forbids `claude -p` as a required element, one calls
  `DOCTRINE_WORKER` "never the identity"). **Eight regions**, plus `D5` and `D7`
  recorded as considered-and-deferred. Target set unchanged at four entities.
  The old "five" mis-cited line 263 as a consequences restatement; 263 is the
  `## Verification` heading, and Consequences and Verification are two distinct
  regions
- ~~QUE-214 / QUE-215 unsettled~~ — **both settled `answered` 2026-08-13.**
  `QUE-214` by the user (neither pole as stated — narrowed Bounded Pole B, remainder
  to `SL-255`); `QUE-215` by agent (keep the shell for `SL-254`; the provisioning
  half moves to `SL-255` `OQ-2`, where the `doctrine-control` build-gating
  complication deserves a fresh look rather than an inherited answer)
- ~~Design run gate outstanding~~ — **both cleared 2026-08-13**, rev 15. Stage is
  still `exploring`; the advance to `inquiring`/`drafting` is a separate `stage`
  submission and is deliberately not yet sent
- ~~**`sufficiency-accepted` outstanding — the last contract on the
  `inquiring → drafting` edge, and a user act.**~~ — **TAKEN 2026-08-13**, with
  `graph-reviewed`; stage now `drafting`. It asks the harder question:
  not "was every raised question dealt with" but "was the raised *set*
  adequate". Three areas nobody asked about, offered for that judgement rather
  than as findings: (1) **the hand-back contract** — `A2`'s one surviving leg;
  there is no typed subagent-return under `claude -p`, the `result` field and
  `--json-schema` are the channel, and no `DEC` says what the worker returns or
  who enforces its shape, though the funnel consumes it; (2) **macOS
  `sandbox-exec` parity** — objective 1 asserts it and `DEC-206` re-homes
  `write_seatbelt_profile`, but no decision covers whether the claude arm
  reaches the seatbelt path or merely inherits the assertion; (3) **what
  `doctrine install` seeds** — deleting the hook set changes install's seeded
  `.claude/settings.json`, and the shipped-asset leg is a `RustEmbed` root.
  (1) has real design weight; (2) and (3) are plausibly execution detail and
  cheap to dismiss.

  **All three dispositioned by the user 2026-08-13.** (1) → worked as `inq-11`
  and `inq-12`, settling `DEC-215` (typed hand-back by argv, at parity with the
  pi arm) and `DEC-216` (privileged worker tools live outside the confinement,
  in their own binary). (2) → out of the slice as `IMP-429` — the real question
  is *where the prefix lives*, which is construction, and the macOS leg needs a
  different host to verify. (3) → not lifted to a decision: it follows from what
  is already settled, and locks in `drafting` as a derived consequence rather
  than an open question
- **Design open questions, none blocking** (`design.md` §6). `OQ-1` is the sharp
  one and is a **precondition of the VA leg**: `worktree fork --worker` binds a
  fork's `(slice, phase)` only with both flags **and** a dir under
  `<coord>/.worktrees/<name>` (`fork.rs:216-246`), and
  `scripts/pi-spawn-confined.sh:56` passes neither — so the collapsed arm would
  inherit unbound, `unprovable-fork` forks. `OQ-2`: `dispatch_import` loses its
  producer (no confined worker can commit) and is deliberately **retained**, on
  `DEC-211`'s narrowing of the funnel cadence out of the `REV`. `OQ-3`
  `~/.claude.json` is a sibling *file* outside the `$HOME/.claude` bind. `OQ-4`
  does `create-fork`'s Passthrough have a live consumer. `OQ-5` macOS parity is
  asserted, not exercised (`IMP-429`)
- **Deletion boundary widened at drafting** (`design.md` §7.2 `D1`-`D3`, §8 `R6`).
  Four surfaces the scope's objective 3 does not name go too — `create-fork`'s
  Fork arm, `dispatch arm-spawn`, the `Spawn`-row recorder, `worktree
  verify-worker` — each being the claude arm's substitute for something `worktree
  fork --worker` already does. Carry as a design-time scope correction in the
  reconciliation brief
- SL-254 `OQ-2` — five backlog items plausibly dissolved rather than fixed (IMP-269, IMP-342, IMP-334, IMP-337, IMP-407), plus IMP-401 and IDE-024; confirm at reconcile
- Memory-corpus sweep at reconcile — carried in the scope's Follow-Ups (25+ stale-but-plausible claude-arm memories, one in the boot snapshot)

## Design surface triage
<!-- `explore.triage` runbook step, design run dr-019ff653, 2026-08-13 -->

Ten inquiry nodes declared and resolved: `DEC-203`..`DEC-212`.

### Shaping decisions

| node | record | settled |
|---|---|---|
| inq-1 | `DEC-203` | Bounded Pole B — worker side goes capsule-shaped, `ADR-012`'s coordination topology untouched |
| inq-2 | `DEC-209` | Generalise the incumbent prefix; take no `doctrine-control` dependency, port no hardening |
| inq-3 | `DEC-204` | `worker_commit`'s **transport** retires; two of its six belts re-home to the fetch/admit step |
| inq-4 | `DEC-210` | Subscription credential, `$HOME/.claude` bound wholesale like pi's `$HOME/.pi`; narrowing deferred |
| inq-5 | `DEC-205` | nominate/denominate die with `pretooluse` — closed loop, not a separate choice |
| inq-6 | `DEC-206` | Re-home four jail primitives to `jail.rs` as the FIRST step, before any deletion |
| inq-7 | `DEC-207` | Identity collapses to the env leg; delete the marker and `describe_mode`'s `is_linked` conjunct |
| inq-8 | `DEC-208` | One arm. No degraded in-session rung; fail closed at spawn as the pi arm already does |
| inq-9 | `DEC-211` | One REV over `ADR-011`, `ADR-006`, `SPEC-021`, `SPEC-012` + a hand anchor sweep |
| inq-10 | `DEC-212` | Behaviour-preservation restated as the funnel's observable contract, not "suites unchanged" |

### Governance constraining the surface

Read directly this stage rather than through the research round's quotations —
`DEC-211`'s body carries the site-by-site detail.

- **`ADR-011`** — **eight** amendment regions (superseding this section's earlier
  "five", which mis-cited 263): Context (21-25), `D1` (39-56), `D2` (58-74),
  `D3`'s table (82-92), `D4` (94-111), `D6` (146-207), Consequences (248-259),
  Verification (265-275). `D5` (113-144) and `D7` (209-232) considered and
  deferred. Site-by-site detail in `DEC-211`'s `choice`.
- **`ADR-006` §D2b** — thread 1's claim verified; the `SL-181` note pattern is real
  at line 143. Second site at 308. Its "degenerate case" list already contemplates
  a standalone clone — `SL-254` makes that the *normal* case, so the note re-cuts
  rather than appends.
- **`ADR-012`** — **not touched**, per `DEC-203`. Load-bearing boundary: it is what
  keeps the REV inside the surveyed set.
- **`POL-002`** — satisfied, not strained (`DEC-206` keeps the argv builder in the
  binary and harness specifics at the script tier).
- **`STD-001`** — governs the thoroughness of the `claude-force-subprocess-dispatch`
  deletion and of the belt consts' single-sourcing.

### Corrections this stage made to prior artefacts

- **`DEC-202`'s `choice`** — corrected in place: it read as a claim about worktrees.
- **The scope's behaviour-preservation gate** — "codex/pi suites stay green
  unchanged" is unachievable once the pi arm moves to clones too (`DEC-212`).
- **The scope's governance survey** — missed `SPEC-012`'s responsibility *prose*
  (`spec-012.toml:18`, `fork`'s "stamp", `import` as "the belted dispatch funnel")
  and `ADR-011`'s fifth site.
- **Research thread 3's auth recommendation** — `ANTHROPIC_API_KEY` forfeits the
  subscription billing `EVD-023` establishes (`DEC-210`).
- **Research thread 2's `worker_commit` reading** — "every check is meaningless in a
  clone" is wrong on two of six belts (`DEC-204`).

### Newly surfaced — NOT in the scope's reconcile survey

- **The memory corpus.** At least 25 memories describe claude-arm mechanisms this
  slice deletes — `SubagentStart` stamping, `PreToolUse` jail behaviour,
  `WorktreeCreate` provisioning, `worker_commit` resolution, marker identity —
  several at `high` trust / `high` severity. After the collapse they are
  **stale-but-plausible**, the most dangerous class: an agent retrieving
  `mem_019ebfb61ba870219aafc14f8dc7da3b` ("worker identity via `SubagentStart`
  hook") would act on a deleted mechanism. `mem.signpost.doctrine.dispatch-claude-arm-wrong-base`
  is indexed in the **boot snapshot**, so the staleness reaches every session's
  context. Needs a deliberate `/reviewing-memory` sweep at reconcile — carried in
  the scope's Follow-Ups.

### Open / deferred

- `IMP-428` — harden the worker confinement prefix (deferred out by `DEC-209`).
- Clone disk/time cost for this repo: still unmeasured. `POL-002` forbids baking a
  local measurement into the platform regardless.
- `../microvm-spike`'s narrowed `~/.claude` mount set: deferred by `DEC-210`; its
  identity-section partial is not definitively proven.
