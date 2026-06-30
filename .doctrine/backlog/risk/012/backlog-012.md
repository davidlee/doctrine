# RSK-012: Closure gate-set scope is per-slice; a foreign Failed req can be omitted by not declaring it

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced by the SL-179 external adversarial pass (codex GPT-5.5), deferred as
out-of-scope for RSK-008.

The closure-gate requirement set is `covered ∪ declared ∪ reconciled`
(`src/slice.rs gate_requirement_set`, ~1249). A requirement carrying a live
`Failed`/`Blocked` cell owned by **another** slice is gated only if the closing
slice covers, declares (`[gate].extra_reqs`), or reconciles it. A slice can avoid
a foreign contradiction by simply not declaring the req.

Not a *silent* leak — un-declaring is a reviewed, committed `slice-NNN.toml` edit,
and SL-179 closes the silent paths (forget refusal, accept hardening). This is the
**breadth** of the gate set, a deliberate per-slice scope (SL-044 D-B2: "each gate
discharges only its own slice's drift"), confirmed during SL-179 design.

Open question for a future slice: should the closure gate also consider
requirements structurally implicated by the slice (REV targets, touched-requirement
relations, owning-REC `evidence_ref`) rather than only the three authored terms?
Relatedly, a `redesign` REC's empty `status_delta` keeps its req out of the
`reconciled` term — mostly mooted by the redesign back-edge (slice returns to
`design`, can't close), but part of the same gate-set-membership theme.

Origin: RSK-008 / SL-179 design. Refs: SL-044 D-B2/D-B5, SPEC-002 D8.
