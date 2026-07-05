# Inquisition — SL-199 PHASE-05 design delta (textual record)

**Subject:** `SL-199` PHASE-05 design delta — `design.md` @ `dispatch/199` (delta
commits `23116d7d` → `d944f3a9`).
**Posture:** adversarial inquisition (hostile design review).
**Reviewer:** codex MCP, GPT-5.5 (project default) — read-only source cross-check
against the coord tree. Corroborated by a second source-crosscheck agent; the main
thread adjudicated where the two witnesses disagreed.
**Date:** 2026-07-05.
**Outcome:** DO-NOT-RE-LOCK-as-written → design reconciled in place (delta A6–A10);
re-lock remains the User's.

> **Ledger note.** This review *deserved* the RV ledger (a locked slice's design under
> adversarial review), but the delta lives on `dispatch/199` — a fork-resolved root the
> `doctrine review` verbs refuse — and the primary tree (`edge`) carries only the
> *pre-delta* design. No clean RV home exists until the delta lands on edge/main. Tried
> in prose (this record) rather than force a broken ledger. If durable RV capture is
> wanted, open it on the parent tree post-integration.

## Charges (ranked)

| # | Sev | Charge | Verdict vs source | Reconciled by |
|---|---|---|---|---|
| I | major→blocker | A1 "never a bad land" is unsound — the fail-loud gates check consistency-with-the-ARM, not truth | CONFIRMED (`import_compose` recomputes `merge-base`, dispatch.rs:434; `worker_commit` checks `C^==record.base`) | A7 (§A/§D step1) |
| II | major | conclude-atomicity invariant contradicts shipped source *and* the delta's own D-B4 | CONFIRMED (dispatch.rs:499-509 is deliberately two-tier, self-healing) | A6 (§B table / D-B1 / D-B3 / §D step5 / §E) |
| III | major | design/handover speak prospectively of work already BUILT; "no implementation yet" false | CONFIRMED (create.rs:200-230,321-325; mcp_server/dispatch.rs funnel tools built) | A8 (header built-vs-owed ledger) |
| IV | major | §6 overclaims feasibility beyond the primitive it proved (VH-1 unwitnessed) | CONFIRMED (§6 leaps primitive→"confirmed feasible"; §5.E honestly owed) | A9 (§6), A6 (§E VH-1) |
| V | major | A1/A2 written in past tense of unbuilt work | CONFIRMED (arm-spawn `--base` still REQUIRED; worker def has no `isolation` field) | A8/A9 |
| VI | minor | naming relic — jail-POLICY conflated with dispatch-RECORD | CONFIRMED (policy `.../dispatch/jail/<name>.toml` vs record `.../dispatch/record/<name>.toml`) | A10 (§2) |

## Adjudicated disagreement between witnesses

- **`classify_create` confined trigger.** codex: BUILT (create.rs:173/321). Second agent:
  ABSENT (read the doc-comment offset, not the signature). **Main thread adjudicated:
  codex correct** — create.rs:200-207 carries `cwd_is_coord_root` + `coord_in_dispatch`;
  line 212 is the confined trigger; line 325 `consume_arming_base` is the one-shot consume.
- **`commit_on_behalf`.** Second agent: ABSENT (grepped git.rs only). **Present** at
  mcp_server/dispatch.rs:491. D-B4's composer is real.

## What survived (acquittals)

- §A confined Fork trigger + one-shot base consume — built, sound.
- `worker_commit` resolves by the per-worktree dispatch record; enforces `C^==B`.
- OS1/R1 branch guard sound — `coordinate()` always `add -b dispatch/<NNN>`, never `--detach`.
- D-B4 working-tree-free primitives all real: `commit_on_behalf`, `merge_tree` (git.rs:854),
  `commit_tree` (git.rs:828), scratch `GIT_INDEX_FILE` (git.rs:725/801).
- D-B0 scope belt preserved server-side (`classify_import` HARD-refuses undeclared, import.rs:133-136).

**Architecture is not heretical — the capstone is realizable.** The defects were
prose-vs-source honesty/staleness, fixed by reconciliation, not redesign.

## Still owed (outside this review's remit)

- **VH-1** — live integrated armed-loop witness (real orchestrator + real worker →
  worker_commit → import → conclude → reap). Not witnessed.
- **F8** — ambient `commit-gate-red` coord base (~600 doctor findings) blocks a clean live
  `worker_commit` in VH-1; clear/scope before the live run.
- **A1/A2** implementation (arm-spawn default-base; worker-def frontmatter isolation).
