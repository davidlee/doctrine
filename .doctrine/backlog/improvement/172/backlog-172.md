# IMP-172: Derived per-phase file-set nav view

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

**Gated on SL-154** (reliable conformance-registry population + `provenance` on
`BoundaryRow`, D12).

The registry stores per-phase OIDs, not a path list — a navigation consumer must
re-diff to get the files a phase touched. Once SL-154 makes the registry reliably
populated and self-describing (`provenance` routes funnel→`dispatch/NNN`/`review`,
solo→`edge`), expose a **derived** per-phase file-set view so an agent can jump
straight to a phase's changed paths without re-deriving:

- a `slice show --phase-files` projection, or a gitignored derived cache.

Keep it derived (no new authored/durable tier — storage rule). Source of the idea:
SL-154 design §5.6 (efficiency/nav lens). Do not start before SL-154 lands.
