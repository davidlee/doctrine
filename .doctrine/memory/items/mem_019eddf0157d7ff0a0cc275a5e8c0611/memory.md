# Design-stage timeout specifications must be concrete: value + mechanism + abort

A design-stage timeout for subprocess workers must specify three things:

1. **Concrete value** — e.g. 300s (5 min), not "should impose a deadline"
2. **Enforcement mechanism** — e.g. `timeout(1)` wrapping the subprocess,
   subprocess monitoring, or RPC command-based abort
3. **Abort semantics** — what happens when the deadline fires: send RPC `abort`,
   wait grace period (5s), then SIGTERM

Additionally, if auto_retry is active, the deadline must be wall-clock inclusive
of retry delays.

"The orchestrator should impose a deadline" is not a design decision — it is
a hand-wave. Every design that spawns a subprocess must answer: how long, how
enforced, what happens on expiry.

Discovered during RV-090 (inquisition of SL-108 pi dispatch worker design, F-2).
