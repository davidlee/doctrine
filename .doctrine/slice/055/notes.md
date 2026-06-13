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

## Dogfood log

Status transitions for SL-055 were driven by the verb as we went
(`proposed→design→plan→ready`), and PHASE-01 flipped `completed` — exercising the
exact hygiene BATCH 2 adds. If it felt natural here, that is the evidence the
reminder belongs in the skills.
