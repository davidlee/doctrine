# Review RV-042 — design of SL-077

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Lines of interrogation** — this Inquisition holds the design of SL-077
(requirement prose render in `spec show`) to the following doctrine:

1. **Internal consistency** — does the design agree with its own scope? D1's
table must not contradict scope §1; every decision must trace to the scope it
serves.

2. **Storage rule integrity** — the `.toml` / `.md` tier boundary is sacred.
When two carriers (the `description` field and the `## Statement` prose) overlap,
the design must define their relationship — silence invites dual maintenance and
contradiction.

3. **Algorithmic precision** — `prune_empty_headings` is a pure function that
determines visibility. The comment-detection contract must be explicit, not
implied — ambiguity here produces invisible false-negatives.

4. **Error-surface coherence** — the existing degrade-and-continue pattern (E5)
for dangling member FKs must be honoured or explicitly departed. A new read
(`load_body`) must not silently widen the failure surface of `spec show`.

5. **Column completeness** — every column in `spec req list` must have a defined
value for every row kind, including degraded rows. The `prose` column must not
vanish into undefined territory when `load()` fails.

**Invariants pinned to the accused:**
- Behaviour-preservation gate — existing suites stay green unchanged
- `read_slice` precedent — `read_spec` mirrors `(parsed, raw-toml, prose-body)`
- `second_parent` handling — must not be broken; `build_registry` must carry the
  finding, not hard-fail
- Pure/imperative split — `prune_empty_headings` and `render` touch no disk

**Areas under scrutiny:** `src/spec.rs` (run_show, render, show_json,
relation_edges, build_registry, req_rows, ReqListRow, ReqJsonRow, columns),
`src/requirement.rs` (load, load_body, prune_empty_headings),
`.doctrine/slice/077/design.md` (D1–D6).

## Synthesis

### Verdict

**The design is heretical in six particulars; none is irredeemable.** The
gravest sin — F-1 (BLOCKER) — is a contradiction between D1's table and the
scope it purports to serve. The table lists `build_registry` as a `read_spec`
call site; the scope explicitly excludes it because `second_parent`
classification requires an inline parse. This must be struck before any code is
written — a design that contradicts itself is no design at all.

Five major charges (F-2 through F-6) expose sins of omission: the
`description`/`## Statement` overlap is ungoverned; the comment-detection
algorithm is imprecise; the `prose` column is undefined for dangling rows;
`load_body`'s error surface is unresolved; the demo example presents an
aspirational future as though it were the present. Each is confessable and
correctable within the design artifact.

One nit (F-7) is venial — a path helper inconsistency against the `read_slice`
precedent, not a correctness concern. Follow-up fodder.

### Penance — ordered corrective acts

1. **Strike `build_registry` from D1's table** (F-1). The corrected table shows
two call sites (`relation_edges`, `run_show`), with a note referencing
`is_second_parent` as the reason `build_registry` keeps its inline parse.

2. **Add a `description` / prose reconciliation decision to D4** (F-2). State
that `description` is the structural summary rendered before the prose body;
`## Statement` is the full prose. Both render; neither is deprecated. The
rendering order is: `description` line → prose body (Statement section →
Rationale section). If `description` is absent, prose renders normally. If prose
is scaffold (empty headings), it is omitted entirely.

3. **Specify comment detection in D3** (F-3). Add the precise contract:
   - Per-line evaluation — one content line saves the section
   - Single-line HTML comments only: `trim().starts_with("<!--") && trim().ends_with("-->")`
   - Multi-line HTML comments are NOT detected (treated as content)
   - Non-HTML comment syntax (`//`, `#`) is treated as content
   - Add test: `prune_empty_headings_non_html_comment_is_content`

4. **Define dangling-member prose column value in D6** (F-4). A dangling FK
shall show `—` in the prose column. No `load_body()` is attempted — the
`Option<Requirement>` absence suffices. Add test:
`req_list_prose_column_dangling`.

5. **Choose degrade-and-continue for `load_body` in D2** (F-5). Change the
signature to `fn load_body(root: &Path, canonical_fk: &str) -> Option<String>`.
A missing `.md` returns `None` (renders as scaffold); a corrupt entity is
visible in output, not a silent error. Aligns with E5. Update the Known Edge
Cases section.

6. **Add aspirational caveat to D4** (F-6). One line: "The filled example above
is aspirational — all current requirements are scaffolds, so output is
byte-identical until IMP-057 authors real prose."

### Standing risks

- **Author discipline gap**: Until IMP-057 delivers a requirement authoring
skill, agents must hand-edit `requirement-NNN.md` to fill the `## Statement` and
`## Rationale` sections. The prose render is dead code in production until
authorship catches up — a known, accepted gap.
- **Heading inversion**: If an author nests `###` inside a requirement section,
the demoted `####` headings will be subordinate to an author-level `###` — a
cosmetic inversion. Acceptable; the design warns about it (Known Edge Cases).
- **Multi-line HTML comments**: If an author writes a multi-line HTML comment in
a requirement prose section, the intermediate lines will be treated as content
and the section will not be pruned. Acceptable — the scaffold uses single-line
comments only, and the failure mode is extra rendering, not hidden content.

### Tolerated taint

None consciously tolerated. Every finding is sentenced to correction before
implementation. The seven charges are confessed and the verdict is sealed.
