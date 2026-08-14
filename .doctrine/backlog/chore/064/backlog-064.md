# CHR-064: drop the base64 dependency orphaned by SL-254

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`base64` has no remaining reference in Rust source. `Cargo.toml:64`
(`base64.workspace = true`) and `:166-168` still declare it, and the declaring
comment names the reason it is gone:

    # Leaf-legal external (cf. glob) — pure base64 for the opaque bwrap command wrap
    # (SL-182 PHASE-02, INV-5). Standard alphabet matches the probe's `base64 -w0`/`-d`.

`SL-182` `PHASE-02` is the Claude-arm subagent write-confinement work. `SL-254`
deleted `src/worktree/subagent.rs` and `src/worktree/pretooluse.rs` whole, and the
sole consumer went with them. Verified 2026-08-14: zero matches across `src/`,
`tests/` and `crates/` (the two hits in `tests/mcp-bridge.test.ts` are the literal
string `'base64...'` in a TypeScript fixture, not the crate).

## Why it survived the slice

Cargo does not warn on an unused dependency, so no gate goes red — this is
invisible to `just gate` and to clippy. It was verified at the audit's `PHASE-05`
leg and deliberately left rather than touched mid-slice, since removing a manifest
entry is out of a phase's declared surface.

## The work

Remove both declarations, rebuild, and confirm the lockfile drops it. Check
whether `crates/` members declare it independently before removing the workspace
entry — the root package's `base64.workspace = true` and the workspace-level
`base64 = "0.22"` are two separate lines and only the first is provably orphaned
by this slice.

## Provenance

- `RV-356` — `SL-254`'s implementation audit; recorded in its Reconciliation
  Brief under "Backlog (cards, not edits)" and dispositioned in its
  `## Reconciliation Outcome`.
- `SL-254` — the slice whose deletions orphaned it.
- `SL-182` — the slice that introduced it.
