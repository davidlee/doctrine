# SL-079 plan inquisition — domain_map

## Areas
- `.doctrine/slice/079/plan.toml`, `.doctrine/slice/079/plan.md` — phase objectives, criteria, and sequencing rationale under trial.
- `.doctrine/slice/079/design.md`, `.doctrine/slice/079/slice-079.md`, `.doctrine/slice/079/slice-079.toml`, `.doctrine/review/045/review-045.md` — locked design canon, scope, and prior verdict.
- `src/main.rs`, `src/listing.rs`, `src/tty.rs` — shared colour plumbing, `--color` resolution, list argument injection, and status colouring seam.
- `src/priority/mod.rs`, `src/priority/render.rs` — survey/next rendering and priority command injection.
- `src/adr.rs`, `src/policy.rs`, `src/standard.rs`, `src/knowledge.rs`, `src/revision.rs` — status-line lifecycle handlers.

## Invariants
- The plan is subordinate to the locked SL-079 design and RV-045 corrections.
- Pure/impure split: tty/clap/env capability resolution stays in the shell; pure render helpers receive plain inputs.
- Phase criteria must be executable and specific enough for `/phase-plan` and `/execute` without guessing command shapes or hidden scope.
- Existing colour-free piped/golden output remains byte-preserving unless the plan explicitly proves and verifies a deliberate change.

## Risks
- Scope text still mentions install/reconcile/corpus while the locked design and plan name standard/knowledge/revision.
- Priority command wording can accidentally include blockers/explain, which the design says are prose and not part of `render_columns`.
- TTY colour verification is easy to write as human observation only, leaving `--color` and `NO_COLOR` precedence unproven.
