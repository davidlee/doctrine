# IMP-187: Dispatch candidate worktree should stage generated embed assets (web/map/dist) before gate

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

A freshly-forked dispatch candidate worktree fails `just check` with:

```
error[E0599]: no associated function or constant named `get` found for struct
`map_server::assets::Assets`  --> src/map_server/assets.rs:37
```

Cause: `src/map_server/assets.rs` derives `#[derive(RustEmbed)] #[folder = "web/map/dist/"]`.
`web/map/dist/` is gitignored generated npm output (`.gitignore:71`), so it is
absent in a fresh fork. `rust_embed` then generates no `get` → compile error. The
main tree builds fine because the assets were built there earlier.

This makes any **in-worktree gate unreliable** for crates carrying a RustEmbed
folder of generated assets — the failure is environmental, not a code defect, and
masks (or fakes) real gate results during dispatch/audit. During the SL-159 audit
the gate was unblocked by copying `web/map/dist/` from the main tree (then exit 0).

**Options:**
- worktree provisioning (`doctrine worktree fork` / dispatch candidate create)
  copies generated embed dirs from the parent tree, or
- the `just check`/`gate` recipe builds the map UI (npm) when `web/map/dist/` is
  missing, or
- document the manual copy step in the dispatch/audit skill.

Related memory: `crane-strips-non-rust-embeds` (the nix-flake analogue of the same
embed-asset class). Surfaced by: RV-172 F-4 (SL-159 audit).
