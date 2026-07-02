# Hymn Corpus — Authoring Convention

## Band Registry

| Band      | Segment     | Purpose                                |
|-----------|-------------|----------------------------------------|
| preamble  | `preamble`  | Universal preface (every resolution)   |
| harness   | `harness`   | Harness-specific guidance              |
| model     | `model`     | Model-family / exact-model notes       |
| role      | `role`      | Orchestrator vs worker envelope        |
| stage     | `stage`     | Phase-contextual prose                 |
| project   | `project`   | Project-specific / user-authored       |

## Path → Slot Rule

`<band>/<label>.md` → slot `{ band, label }`. Model labels are the full relative key
(e.g. `anthropic/claude-sonnet-4`). The filename stem (minus `.md`) becomes the slot
label. Directory structure under a band maps exactly to slot labels.

## Sidecar Overlay

A `.toml` file adjacent to the `.md` overlays selector axes. Example:

```toml
# harness/claude.toml
harness = "claude"
model = "anthropic/_default"
```
The sidecar overrides the path-derived defaults; undeclared axes keep their default.

## Seal / Expose

- **sealed** slots: the framework (embedded) snippet is authoritative. User disk
  twins are dropped.
- **exposed** slots: user disk copies are projected from the framework and win the
  equal-specificity provenance tiebreak.

## Provenance

- `Framework`: shipped with the binary (embedded under `install/hymns/`).
- `User`: on-disk under `.doctrine/hymns/`. Always wins at equal specificity.
