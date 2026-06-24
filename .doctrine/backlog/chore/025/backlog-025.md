# CHR-025: Clear pre-existing rustfmt debt in boot.rs, main.rs, memory.rs

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced during SL-150 PHASE-01 dispatch. `cargo fmt --check` flags long-line
debt outside that slice's delta:

- `src/boot.rs:2753`
- `src/main.rs:791`
- `src/memory.rs:541`, `:2749`, `:2789`, `:2816`, `:4859`

These predate SL-150 and were kept out of the PHASE-01 commit (file-disjoint
worker contract). The hazard: `just gate` / `just check` run a **mutating**
`cargo fmt`, so any agent gating in those files silently rewrites this debt into
an unrelated commit. Fix in isolation: a single `cargo fmt` pass + commit, no
behaviour change.
