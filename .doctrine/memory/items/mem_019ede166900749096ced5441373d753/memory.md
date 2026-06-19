- Design-stage timeout specifications must include three concrete elements:
  1. **Value** — a falsifiable number (e.g. 300s wall-clock)
  2. **Mechanism** — how the timeout is enforced (e.g. `timeout` wrapper, RPC abort)
  3. **Abort semantics** — what happens when the deadline is hit (grace → SIGTERM, retry interaction rules)
- "The orchestrator should impose a deadline" is not a design decision — it is a wish
- Include retry-interaction rules: wall-clock deadline is inclusive of retries (not per-attempt)
- Source: RV-090 (inquisition of SL-108 D1 timeout)

See also: [[mem_019ed08526777250a3e6a7087821e41d]] — design decision tables must agree with scope exclusions
