# Doctrine file map and layout

Where things live — use this to *locate*. The authoritative layout block is in
`CLAUDE.md`; the evergreen internals are in `doc/*`. This signpost points; it
does not restate them.

- `.doctrine/slice/nnn/` — one dir per slice: `slice-nnn.{toml,md}` (metadata +
  scope), `design.md`, `plan.{toml,md}`, `notes.md`, `audit.md`, and the
  gitignored `handover.md` / `phases` symlink. Glob `.doctrine/slice/**`.
- `.doctrine/adr/nnn/` — project-global ADRs (`adr-nnn.{toml,md}`); status lives
  in the TOML.
- `.doctrine/spec/product/nnn/` and `.doctrine/spec/tech/nnn/` — product /
  technical specifications: `spec-nnn.{toml,md}` + `members.toml` (the
  requirements they compose).
- `.doctrine/backlog/` — work-intake items (issue / improvement / chore / risk /
  idea), the intake surface upstream of a slice.
- `.doctrine/memory/items/nnn/` — the memory store (`memory.{toml,md}` + a
  `mem.<key>` symlink). `.doctrine/memory/shipped/` is the gitignored synced
  global corpus. See [[concept.doctrine.memory-model]].
- `.doctrine/state/` — runtime tracking: phase sheets, `boot.md`, the `phases`
  symlink. GITIGNORED, disposable, `rm -rf`-able.
- `.doctrine/governance.md` — user-owned governance pointer, projected into the
  boot snapshot.
- `doc/*` — evergreen, authoritative specs (`slices-spec.md`, `memory-spec.md`,
  `skills-spec.md`, `entity-model.md`, `glossary.md`, …). Learn internals here.
- `install/` — sources copied into `.doctrine` by the installer.
- `src/` — the Rust shell (e.g. `src/git.rs`, the impure capture seam;
  `src/boot.rs`, the snapshot generator).

What is committed vs disposable is the storage tiers:
[[fact.doctrine.storage-tiers]] and [[concept.doctrine.storage-model]]. The
lifecycle artifacts under `slice/nnn/` are sequenced in
[[signpost.doctrine.lifecycle-start]].
