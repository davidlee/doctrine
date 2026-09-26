# IMP-487: Shipped-corpus delivery freshness is unguarded: a skipped re-embed or sync is invisible to every gate

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

Every shipped sub-corpus reaches a client through a delivery step that **no gate
covers**:

- `memory/` masters → `cargo build` (RustEmbed) → `doctrine memory sync` →
  `.doctrine/memory/shipped/`
- `install/` assets → `cargo build` (RustEmbed) → read through
  `doctrine library show`
- `plugins/` skills → `cargo build` → `doctrine install`

Skip the delivery step and the change is invisible while `doctrine doctor`,
`doctrine check gate`, `doctrine publication validate` and the e2e suites all
stay green: they read the sources, or the embedded assets, never the materialised
corpus a client actually reads.

## Evidence

`SL-267` (shipped-corpus conformance) closed with a stale delivery. PHASE-06's
final commit (`795317bdb`) repaired four shipped memory masters and never re-ran
`memory sync`, so the materialised `.doctrine/memory/shipped/` copies still
carried the pre-repair repo-private paths (`install/`, `install/hymns/`,
`ADR-019`, `memory/`). The closing audit only found it by re-running the
delivery: `cargo build && doctrine memory sync` reported `0 new, 9 changed`.
Design `R5` predicted exactly this invisibility. Evidence: `RV-395` `F-13`;
observation `01a0db8e-0ace-7640-9457-f0ca4f5077c2`.

## The fix

A check — run by `doctrine doctor` / `doctrine check gate`, or folded into the
`ISS-309` part 2 shipped-corpus integrity gate — that refuses when an embedded
corpus asset differs from its materialised delivery. **Distinct from the citation
gate** (`ISS-309` part 2): that one reads the corpus's *content*, this one reads
its *freshness*. Neither subsumes the other; both must land.

## Boundaries

- Not the citation gate (`ISS-309` part 2) and not the accuracy axis
  (`CHR-080`) — though all three share the same corpus and the same follow-up
  slice.
- The engine check is the deliverable; no corpus edit can substitute for it.
- The three known-unreachable signposts are `ISS-215`, a separate defect.

