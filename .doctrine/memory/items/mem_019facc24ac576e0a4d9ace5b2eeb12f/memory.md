After authoring criteria into `plan.toml`, the obvious read-back commands are
both mutating:

- `doctrine slice plan <N>` is the **authoring** verb. On a slice whose plan
  exists it refuses — `Refusing to overwrite existing plan.toml` — which is the
  right refusal, but only after a mutating verb has been aimed at a slice
  mid-dispatch.
- `doctrine slice phase <N> <PHASE>` is the **state-transition** verb and
  requires `--status`.
- `slice show` prints the slice header only. No verb prints a phase's
  `entrance_criteria` / `exit_criteria` / `verification`.

So verifying authored criteria means parsing `plan.toml` directly, which is the
raw-file route the boot guardrail otherwise forbids. That is an accepted
exception here, not a lapse: use a TOML parse to confirm the file is
well-formed and the ids are present, and say so when you do.

The same gap exists for review findings — `review show` prints the brief and a
finding *count*, never the findings, so a responder must open
`.doctrine/review/NNN/review-NNN.toml`. Both are recorded as RFC-011 friction
observations.

Related: [[mem.pattern.doctrine.no-standalone-plan-validation]].
