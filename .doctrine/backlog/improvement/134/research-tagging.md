Report written to `/workspace/doctrine/.doctrine/backlog/improvement/134/research-tagging.md`.

**Key findings at a glance:**

- **3 kinds with full tagging** (template + struct + listing + write verb): backlog, memory.
- **2 kinds with create+list only** (tags exist and are searchable but no edit verb): knowledge, requirements.
- **5 kinds with dead tags** (seeded in template/struct but listing hardcodes empty, no write verb): ADR, policy, standard, RFC, spec.
- **1 kind completely tagless**: slice.
- **TOML storage is fragmented**: root-level (`backlog/knowledge/spec`), `[relationships]` (`adr/policy/standard/rfc`), `[scope]` (`memory`).
- **Two separate normalisation functions**: `tag::normalize_tag` (charset-restricted, `[a-z0-9_:-]`) vs `memory::validate_tags` (trim+lowercase+dedup only).

The report includes a summary matrix, per-kind integration point table, and two architectural approaches (generic vs per-kind verb) for IMP-134.