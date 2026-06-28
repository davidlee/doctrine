# IDE-023: Binary delivery & release tech spec

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Deferred at SL-174 reconcile (RV-187 F-3, OQ-3). SL-174 shipped binary
*delivery* — the tag-triggered GitHub-release pipeline (`release.yml`), the
no-compile install channels (`install.sh` curl|sh + `[package.metadata.binstall]`),
and the asset-name contract (`doctrine-<triple>.tar.gz` + `.sha256`) shared across
three consumers. This is durable evergreen surface that lives **outside SPEC-009**,
which owns only embed + lay-down into a project.

The reconcile decision was to **record the boundary** (in SL-174 design §6 OQ-3)
rather than extend SPEC-009 or author a spec now. Author a dedicated delivery /
release tech spec (under PRD-006) **if/when the delivery surface grows** — e.g.
Linux/Windows artifacts, a vanity install domain, signing/notarization, or a
second delivery channel. Until then the mechanism is documented in
`.doctrine/slice/174/notes.md` and the design.

Source of record: SL-174 design.md §5, notes.md; RV-187 synthesis.
