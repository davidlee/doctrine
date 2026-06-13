# Notes SL-055: Holistic skills review & token-efficient improvements

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

The research base (`research/` — FINDINGS.md, 00-index.md) is gitignored and
regenerable. What follows is the **verified, durable** distillate so the slice
carries its own evidence if the research dir is wiped.

## Verified findings (ground-truth checked 2026-06-13)

- **`doctrine slice status <id> <state>` EXISTS** (states `proposed…done,
  abandoned`; refuses the closure seam out of order + leaving a terminal). Every
  skill claim of "no transition verb" / "hand-edit the status" is therefore
  **stale**. (`--help` confirmed.)
- **`close` skill stale-debt cluster (all confirmed against the file + boot):**
  its `> Tooling gaps` callout is flat false (verb exists; terminal set is
  `{done, abandoned}`, not `{done}`); Process step 3 wrongly says hand-edit TOML
  to `done`; its input `audit.md` is retired post-RV-rewire (see `/audit`). This
  is CHR-004 and a bit more. **Closing this slice closes CHR-004.**
- **IMP-023 is OPEN** — review-skill RV rewiring (code-review / inquisition
  corpus integration) is genuinely not done; route those findings there, not to
  prose.

## Primary finding (the slice's reason to exist)

The ADR-009 lifecycle (`proposed→design→plan→ready→started→audit→reconcile→done`)
and its verb are documented in the boot snapshot, but **no core lifecycle skill
(slice / design / plan / execute / audit / close) invokes the verb.** Each
describes a skill-to-skill handoff ("hand off to `/design`") without the matching
`doctrine slice status` transition, so the status is left hand-edited — the
source of the SL-009 `⚠` rollup divergence. SL-055 BATCH 2 wires the verb into
every stage handoff; BATCH 5 adds the cross-artifact reminders (`/close` → close
the originating backlog item; merged-worktree GC). This is not recorded as a
standalone memory by request — it is being fixed in this slice.

## PHASE-02 outcome (2026-06-13, paired session)

All six worklist batches adjudicated and landed (commits 2bf4783, f28ae6b,
0623b53, b03ba74, 2532935 — plus BATCH 2 edits swept into the concurrent
SL-054 session's commits 5330700/0c7e7f6 on the shared main worktree;
content intact, attribution commingled, expected per memory):

- **BATCH 1** — close skill: false tooling-gaps callout deleted, closure seam
  (`reconcile`→`done`) replaces hand-edit, input re-pointed to the RV ledger,
  new close-the-originating-backlog-item step. **CHR-004 resolved/fixed.**
- **BATCH 2** — lifecycle verb wired into every stage handoff: slice→`design`,
  design-lock→`plan`, plan→`ready`, execute entry→`started`,
  phases-done→`audit`, audit-done→`reconcile`, close→`done`. Seam ownership
  adjudicated: `/audit` owns `→reconcile`, `/close` confirms + owns `→done`.
  dispatch gained a conclude-the-slice step (status→audit, /audit from parent
  tree, worktree GC). R5-22 (worktree cleanup ownership) → **IMP-041**.
- **BATCH 3** — route trimmed ~94→55 lines; table/guardrails now pointed at the
  boot snapshot, route-unique content kept (anti-rationalization block, spec
  routing, no-stricter-ceremony, freshen ritual). Boot digest source confirmed:
  `install/routing-process.md`.
- **BATCH 4** — "runtime phase sheet" canonical (was 5 variants, 10 files);
  VA/VH added to plan+execute criteria vocabulary (R2-04/O7); design *locks* /
  plan *approved* split (R2-03); canon says "boot snapshot" (R2-09);
  spec-product description capitalised (O5). Skipped as adjudicated:
  R2-07/08/10/11 (mostly-consistent or reference-layer drift, not skills).
- **BATCH 5** — fully delivered via batches 1/2 (no separate edits).
- **BATCH 6** — R5-23/O3 → **IMP-042** (code-review corpus integration beyond
  RV rewire, `after IMP-023`); R5-24 already covered by IMP-023; R3-13 current.

Re-embed (`doctrine skills install` + touch `src/skills.rs`) deliberately held
for PHASE-03.

## Dogfood log

Status transitions for SL-055 were driven by the verb as we went
(`proposed→design→plan→ready`), and PHASE-01 flipped `completed` — exercising the
exact hygiene BATCH 2 adds. If it felt natural here, that is the evidence the
reminder belongs in the skills.
