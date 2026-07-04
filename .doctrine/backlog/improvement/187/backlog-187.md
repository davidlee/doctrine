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

## Scope note (RFC-011 analysis, 2026-07-04)

This is NOT `web/map/dist/`-specific — it is the generic class of "any RustEmbed
`#[folder]` that points at a gitignored built artifact." Witnessed 3+ times in the
case notes (SL-193-audit, SL-195, SL-192-audit) and in prior sessions (SL-159-audit,
SL-171-audit, SL-168-audit). The fix should:
- Enumerate embed roots by scanning `src/**/assets.rs` for `#[folder]` annotations
  (or use a config-driven allowlist of known embed dirs).
- Provision them generically in `worktree fork` / `candidate create`, not hardcode
  one path.
- Fail loudly with a provisioning hint if an embed root is missing, rather than a
  deep `E0599` compile error.

Related memory: `crane-strips-non-rust-embeds` (the nix-flake analogue of the same
embed-asset class). Surfaced by: RV-172 F-4 (SL-159 audit).
