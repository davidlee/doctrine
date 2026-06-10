# Kind-identity registry dedup is deferred to SL-031 — do not pre-build it

`integrity::KINDS` hand-copies each numbered kind's `prefix` + `dir` from the
kind-owning module's `entity::Kind` const (`slice::SLICE_KIND`, `adr::ADR_KIND`,
`backlog::*_KIND`, `spec::*_SPEC_KIND`, `requirement::REQUIREMENT_KIND`,
`policy::POLICY_KIND`) — a **parallel copy with no compile-time link**. SL-032
code-review flagged this (review F-2: not "the single table" the prior memory
implies; F-5: `KindRef.has_runtime_state` is a bool with a hardcoded
`.doctrine/state/slice`). The R-b silent-escape (a new numbered kind absent from
KINDS slips past `validate`) is the live consequence.

**Disposition (SL-032 audit.md): deferred to SL-031, on purpose.** SL-031 wires
trunk-aware minting into every `*::run_new`, which must resolve each kind's `dir`
to call `git::trunk_entity_ids` — making SL-031 the **second consumer** of per-kind
identity. The single shared registry both consumers derive from (plus the
set-equality guard test and moving the state-dir onto the registry row) is SL-031's
job: the second consumer shapes the abstraction. Building it now, against one
consumer, guesses the shape then reshapes.

**How to apply (until SL-031 lands):**
- Adding a numbered kind ⟹ add its `KindRef` row to `integrity::KINDS` and update
  `kinds_table_*` — the interim manual step. Do NOT spin up the shared registry in
  isolation to "fix" the duplication; that work belongs to SL-031.
- Touching trunk-mint wiring or the kind registry in SL-031 ⟹ this is where F-2/F-5
  discharge; fold them in there.
- This thread expires when SL-031 closes (the dedup either landed or is re-routed).

See [[mem.pattern.entity.numbered-kind-identity-table]] (the table itself; note its
"single corpus-wide table" framing is the very claim F-2 corrected — it is the
single *assembly point*, not a linked source of truth).
