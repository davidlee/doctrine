# ISS-447: Hymn stage band is unreachable — nothing passes --stage

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

The hymn cascade's `stage` band resolves correctly on demand and is never asked
for, so its content is shipped-but-dead.

## Evidence

- `.claude/settings.json:27` is the only wiring that resolves the cascade:
  `${DOCTRINE_BIN:-doctrine} prompt resolve --role orchestrator`. No `--stage`.
- A tree-wide grep for `--stage` returns only `git ls-files --stage` call sites.
  Nothing in `src/`, the skills, the scripts or the settings passes a stage to
  `prompt resolve`.
- Confirmed by resolution: `prompt resolve --role orchestrator` omits
  `install/hymns/stage/design.md`; adding `--stage design` includes it
  (`# Design stage invariants`). The band works — it is simply never selected.

So `install/hymns/stage/design.md` ships inside the binary and reaches no agent
context by any current path.

## Why it is not simply a duplicate of the design-prompts store

`src/design_run/prompt.rs:42-46` records the split as deliberate: the four
`install/design-prompts/` fragments (inquiry, drafting, reviewing, delegation)
are *intra-design obligations*, whereas `KNOWN_STAGE_LABELS` is an enforced
lifecycle vocabulary. The stage hymn is the cross-turn-invariants layer above
the per-obligation fragments. The layering is sound; only its delivery is
missing. Note the design-prompts store is also deliberately override-free
(`prompt.rs:4-9`), so the stage band is the only user-overridable home for
lifecycle-stage guidance.

## Candidate remedies

1. Have each lifecycle skill invoke `doctrine prompt resolve --stage <label>`
   in its own body — zero Rust, and it makes stage content arrive at the moment
   the stage begins rather than at session start.
2. Extend the SessionStart hook with a stage argument. Cheaper but wrong-shaped:
   a session has no single stage.
3. Retire the stage band and fold its content elsewhere, if the layer earns
   nothing once (1) is priced.

## Provenance

Found during `SL-258` design while assessing the hymn cascade as a delivery
vehicle for that slice's convention-only deliverables. Not on `SL-258`'s path —
it uses the `project` band, which the SessionStart hook does deliver.
