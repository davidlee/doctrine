# Doctrine file map and layout

Where things live — use this to *locate*. The authoritative layout block is in
`CLAUDE.md`. This signpost points; it does not restate it.

- `.doctrine/slice/nnn/` — one dir per slice: `slice-nnn.{toml,md}` (metadata +
  scope), `design.md`, `plan.{toml,md}`, `notes.md`, `audit.md`, and the
  gitignored `handover.md` / `phases` symlink. Glob `.doctrine/slice/**`.
- `.doctrine/adr/nnn/` — project-global ADRs (`adr-nnn.{toml,md}`); status lives
  in the TOML. See [[mem.signpost.doctrine.adrs]].
- `.doctrine/spec/product/nnn/` and `.doctrine/spec/tech/nnn/` — product /
  technical specifications: `spec-nnn.{toml,md}` + `members.toml` (the
  requirements they compose). See [[mem.signpost.doctrine.specs]].
- `.doctrine/backlog/` — work-intake items (issue / improvement / chore / risk /
  idea), the intake surface upstream of a slice. See [[mem.signpost.doctrine.backlog]].
- `.doctrine/review/nnn/` — adversarial review ledgers (RV kind): `review-nnn.{toml,md}`.
  See [[mem.signpost.doctrine.review]].
- `.doctrine/rec/nnn/` — reconciliation records (REC kind): `rec-nnn.{toml,md}`.
- `.doctrine/revision/nnn/` — revision change-axis records (REV kind, ADR-013):
  `revision-nnn.{toml,md}`. See [[mem.signpost.doctrine.revisions]].
- `.doctrine/policy/nnn/` — governance policies (standing rules):
  `policy-nnn.{toml,md}`. See [[mem.signpost.doctrine.policies-standards]].
- `.doctrine/standard/nnn/` — governance standards (conventions of practice):
  `standard-nnn.{toml,md}`. See [[mem.signpost.doctrine.policies-standards]].
- `.doctrine/knowledge/nnn/` — durable knowledge records (assumption / decision /
  question / constraint): `knowledge-nnn.{toml,md}`.
  See [[mem.signpost.doctrine.knowledge]].
- `.doctrine/memory/items/nnn/` — the memory store (`memory.{toml,md}` + a
  `mem.<key>` symlink). `.doctrine/memory/shipped/` is the gitignored synced
  global corpus. See [[mem.concept.doctrine.memory-model]] and
  [[mem.signpost.doctrine.recording-memories]].
- `.doctrine/state/` — runtime tracking: phase sheets, `boot.md`, the `phases`
  symlink. GITIGNORED, disposable, `rm -rf`able.
- `.doctrine/governance.md` — user-owned governance pointer, projected into the
  boot snapshot. See [[mem.concept.doctrine.boot-snapshot]].
- `.doctrine/using-doctrine.md` and `.doctrine/glossary.md` — shipped reference
  docs (ADR-005 PULL tier). See [[mem.signpost.doctrine.reference-docs]].
- `install/` — sources copied into `.doctrine` by the installer. See
  [[mem.signpost.doctrine.install]].

What is committed vs disposable is the storage tiers:
[[mem.fact.doctrine.storage-tiers]] and [[mem.concept.doctrine.storage-model]]. The
lifecycle artifacts under `slice/nnn/` are sequenced in
[[mem.signpost.doctrine.lifecycle-start]].
