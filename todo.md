  Doctrine CLI gaps re workflow conventions

  Reading the command surface against the workflow you actually run, ranked by impact:

  1. No status rollup — the biggest hole. The phase-tracking system produces rich state
  (phase-NN.toml status/started/completed/log) but nothing surfaces it. There's no doctrine
  slice status <id> showing "5: 2/6 complete, PHASE-01 in_progress". slice list shows only the
  hand-edited slice-nnn.toml status — which is not derived from phase completion. You built the
  tracking, but it's write-only without reading the state tomls by hand.

  2. audit.md has no scaffold. Every other artifact has a verb — new/design/plan/notes — and
  audit.md is a named workflow deliverable (/code-review & audit → audit.md), yet it's
  hand-made. Inconsistent. A doctrine slice audit <id> (parallel to notes) closes it. No
  audit.md template either.

  3. Slice lifecycle decoupled from phase reality. slice-nnn.toml status (proposed→…→done) is
  hand-edited and can silently drift from actual phase state. No transition command, no
  derivation from "all phases completed".

  4. No standalone plan validation. A malformed plan.toml only blows up when slice phases
  parses it. A slice plan --check (duplicate ids, PHASE-NN grammar, schema) would catch it
  before materialising tracking.

  Minor
  - slice phase --status accepts any transition freely — no guard (can't-complete-a-blocked,
  entrance-criteria check).
  - No slice show <id> (single-slice detail) or phase-sheet viewer — you cat the state tree.
  - [relationships] / plan [specs]/[requirements] are reserved-empty — known, that's the SL-008
  registry.

  The throughline: #1 and #3 are the real ones — the tool captures phase/lifecycle state but
  gives no derived view of it, so progress lives in your head or in manual toml reads. #2 is a
  cheap consistency fix. All four are natural future slices (a slice status/rollup slice
  especially).
