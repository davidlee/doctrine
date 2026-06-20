# D1a: route dispatch by actual body calls, not nominal kind

Route a kind's dispatch to the module its `run_*` body *actually* calls, NOT the
nominal kind name. Body relocation from inert main into a command-tier module
mints new production edges; nominal misroutes close cycles.

## Why

When a `run_*` body lives in an engine/leaf module (where the data/policy
dwells) and the kind's enum + dispatch arm are in `main.rs`, the nominal kind
name doesn't tell you where the body executes. Moving the enum + dispatch arm
into the nominal kind's module can create a **cycle-former**: the kind module
imports the dispatch target, which imports back.

## Known instances (exhaustive, confirmed codex round 4)

1. **`MemoryCommand::Sync` → `corpus`** — stays in the `commands/` sink shell,
   NOT `memory.rs`. `corpus` imports `memory`; having sync's dispatch live in
   `memory` creates a `memory→corpus→memory` cycle.
2. **`SpecReqCommand` → `spec.rs`** — own-module dispatch (zero edge), NOT
   `requirement.rs`. `run_req_*` bodies live in `spec.rs`, not `requirement.rs`.
   Routing to `requirement` would mint `requirement→spec` where none exists.

## Guard

- PHASE-03 of SL-115 enforces per-batch gate: `[tangle_baseline] command = 120`.
- Per-arm audit (PHASE-03 EN-2) is mandatory before each batch.
- Codex round 4 confirmed exactly two cycle-formers across an exhaustive
  per-kind sweep; the per-batch gate guards against a missed third.

## Established in

[[mem.dispatch-body-route]] was discovered during the SL-115 design adversarial
audit (codex rounds 2–4) and encoded as the D1a rule in the plan.

