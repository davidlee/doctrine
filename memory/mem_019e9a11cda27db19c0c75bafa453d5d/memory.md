# Doctrine file map and layout

Where things live — use this to *locate*. The authoritative layout block is in
`CLAUDE.md`; the evergreen internals are in `doc/*`. This signpost points; it
does not restate them.

- `.doctrine/slice/nnn/` — one dir per slice: `slice-nnn.{toml,md}` (metadata +
  scope), `design.md`, `plan.{toml,md}`, `notes.md`, `audit.md`, and the
  gitignored `handover.md` / `phases` symlink. Glob `.doctrine/slice/**`.
- `.doctrine/adr/nnn/` — project-global ADRs (`adr-nnn.{toml,md}`); status lives
  in the TOML. See [[signpost.doctrine.adrs]].
- `.doctrine/spec/product/nnn/` and `.doctrine/spec/tech/nnn/` — product /
  technical specifications: `spec-nnn.{toml,md}` + `members.toml` (the
  requirements they compose). See [[signpost.doctrine.specs]].
- `.doctrine/backlog/` — work-intake items (issue / improvement / chore / risk /
  idea), the intake surface upstream of a slice. See [[signpost.doctrine.backlog]].
- `.doctrine/review/nnn/` — adversarial review ledgers (RV kind): `review-nnn.{toml,md}`.
  See [[signpost.doctrine.audit]].
- `.doctrine/rec/nnn/` — reconciliation records (REC kind): `rec-nnn.{toml,md}`.
- `.doctrine/revision/nnn/` — revision change-axis records (REV kind, ADR-013):
  `revision-nnn.{toml,md}`. See [[signpost.doctrine.revisions]].
- `.doctrine/policy/nnn/` — governance policies (standing rules):
  `policy-nnn.{toml,md}`. See [[signpost.doctrine.policies-standards]].
- `.doctrine/standard/nnn/` — governance standards (conventions of practice):
  `standard-nnn.{toml,md}`. See [[signpost.doctrine.policies-standards]].
- `.doctrine/knowledge/nnn/` — durable knowledge records (assumption / decision /
  question / constraint): `knowledge-nnn.{toml,md}`.
- `.doctrine/memory/items/nnn/` — the memory store (`memory.{toml,md}` + a
  `mem.<key>` symlink). `.doctrine/memory/shipped/` is the gitignored synced
  global corpus. See [[concept.doctrine.memory-model]] and
  [[signpost.doctrine.recording-memories]].
- `.doctrine/state/` — runtime tracking: phase sheets, `boot.md`, the `phases`
  symlink. GITIGNORED, disposable, `rm -rf`able.
- `.doctrine/governance.md` — user-owned governance pointer, projected into the
  boot snapshot. See [[concept.doctrine.boot-snapshot]].
- `.doctrine/using-doctrine.md` and `.doctrine/glossary.md` — shipped reference
  docs (ADR-005 PULL tier). See [[signpost.doctrine.reference-docs]].
- `doc/*` — evergreen, authoritative specs (`slices-spec.md`, `memory-spec.md`,
  `skills-spec.md`, `entity-model.md`, …). Learn internals here.
- `install/` — sources copied into `.doctrine` by the installer. See
  [[signpost.doctrine.install]].
- `src/` — the Rust shell (e.g. `src/git.rs`, the impure capture seam;
  `src/boot.rs`, the snapshot generator).

What is committed vs disposable is the storage tiers:
[[fact.doctrine.storage-tiers]] and [[concept.doctrine.storage-model]]. The
lifecycle artifacts under `slice/nnn/` are sequenced in
[[signpost.doctrine.lifecycle-start]].
