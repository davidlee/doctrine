# Notes SL-254: Collapse dispatch onto one subprocess arm

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-13 · proposed (design run `dr-019ff653` live, stage exploring rev 13) · 124c5e44b

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

### Learned
- mem_019ff650d94a7960a638913a40416165 — collide "what calls this" research findings against "what should exist" decisions at synthesis. **Second instance this slice** (DEC-204 vs thread 2's `worker_commit` reading) — strengthen from incident to standing hazard
- The reuse finding: `crates/doctrine-control` already implements clone-inside-bwrap (`provision.rs:949`, `backend/bubblewrap.rs:1110`) — see QUE-215
- `worker_commit` is **six belts**, not a bare ro-git bypass; two are not topology-coupled, and its scope belt has exactly two callers (`classify_import` import.rs:147, worker_commit.rs:94) single-sourced from import.rs:24 (DEC-204)
- `nominate`/`denominate` are a **closed loop** with `pretooluse`'s gate legs — `is_nominated` has no other reader, so they die by construction (DEC-205)
- The confinement prefix already delivers the write floor (`--ro-bind / /`, scripts/pi-spawn-confined.sh:113-131); thread 5's gaps are defence in depth (DEC-209)
- ADR-006 §D2b's `SL-064` note identifies the **coordination tree by marker *absence***. Load-bearing against DEC-207 — see Open

### Open
- **The re-scope, PROPOSED AND UNCONFIRMED.** Owner's direction at end of session: scope SL-254 shy of carving out dispatch proper — separate concern, less clear-cut what stays useful under clone-backed workers, and more manageable split. Proposed line: SL-254 = *the claude arm becomes a pi arm* (confined subprocess, linked worktree, incumbent import transport); clone provisioning + worker self-commit + fetch transport split to a successor slice. Not yet executed
- **DEC-207 is contested by a later finding of my own.** It deletes the disk marker. Under linked worktrees (i.e. under the re-scope) that appears wrong: ADR-006 §D2b identifies the coordination tree by marker *absence*, so deleting the marker makes everything read as coord. Under a clone it was safe because topology distinguishes. **This is the next session's task** — trace and critically evaluate before acting
- Consequential on the re-scope: DEC-203, DEC-204, DEC-212 would be superseded and DEC-211's site list narrowed; DEC-205, DEC-206, DEC-208, DEC-209, DEC-210 survive untouched
- QUE-214 disposed by DEC-203 but **not settled** — deliberately, pending the re-scope. QUE-215 disposed by DEC-209, safe to settle
- Design run gate: `governance-confirmed` and `graph-reviewed` both outstanding, both require **user** authority (DEC-088). `blocking-set-declared` (agent half) is live
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

- **`ADR-011`** — **five** amendment sites, not four: Context (28), D1 (43-45),
  D3's table (89, 93), D4 (98-100), and a consequences-register restatement at 263.
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
