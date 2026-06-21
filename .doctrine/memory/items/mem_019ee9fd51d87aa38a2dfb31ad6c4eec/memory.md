# toml_edit root insert lands above child table headers

`doc.as_table_mut().insert("key", …)` (array OR scalar) renders the key at the
TOP of the document, **above** every child `[table]` / `[[array-of-tables]]`
header, regardless of insertion order. This is structural to TOML, not a
toml_edit accident: a bare key written *after* a `[header]` would parse as a
member of that table, so the encoder has no choice but to emit header-less root
keys first.

**Consequence:** a root **insert-if-missing** cannot tail-land inside a trailing
subtable — there is no corruption to fear. The F-1 "strict refuse / never
insert a missing key" bail in `apply_status` (`dep_seq.rs`) and `apply_tags`
(`backlog.rs`) rests on the opposite belief and is therefore **over-conservative
but harmless**. Relaxing it on a root-key write path (e.g. SL-136's
`apply_tags_set`) is safe.

**Scope of proof:** the encoder guarantee holds for *root* keys only. A key
inserted *into* an existing subtable still positions within that subtable —
unchanged. Same-file overlap (an entity carrying both root `status` and
`[relationships].tags`) relocates/edits independently and benignly.

**How to apply:**
- Need a root key present-or-create on an authored `.toml`? Insert at root; do
  not add an F-1 refuse-if-absent guard for the *root* path.
- Don't extend this to subtable-nested writes.
- pinned `toml_edit 0.22.27`.

**Evidence:** spike CHR-019 / RV-129 F-1 —
`tests/spike_chr019_root_tag_insert.rs`, 7/7 over the live corpus + worst-case
fixtures (SL-118 AoT→`[estimate]`→comment, SL-048 post-relation comment,
RFC-002 16×`[[relation]]`+live tags, ADR-014/POL-001 same-file overlap,
spec-016 in-place edit). See also [[mem_019ea4e4f03c72e1b9a0ef55dcde956d]]
(edit-preserving authored-TOML status writes).
