# Review RV-204 — design of SL-184

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Lines of interrogation.** This Inquisition arrays SL-184's design and plan
against the law of the project — the ADRs, the standards, the conventions, the
memory corpus, and the cold light of the existing `listing.rs` column model.

The tribunal probes four fronts:

1. **Design coherence.** Does the `search_columns()` definition in `design.md`
   compile against the actual `Column<R>` type and `Candidate<'a>` row? Every
   column extractor, every paint closure — do they match the signatures that the
   live `listing.rs` demands?

2. **Completeness of the rename surface.** Every verb, every function, every test
   reference, every MCP tool description string — is the rename table in the
   design exhaustive, or does it leave stale `memory_find` tokens in the corpus?

3. **Spec-conformance of the shared spine.** The plan promises 15 columns, 8
   defaults, comfy-table rendering, and colour via `ColumnPaint`. Does the design
   hold to STD-001 (no magic strings)? Does it follow the precedent patterns from
   the backlog, knowledge, slice, and coverage surfaces?

4. **Plan-to-design fidelity.** Do the phase criteria map one-to-one onto the
   design decisions? Are there phantom exit criteria with no design counterpart,
   or design commitments the plan forgot to gate?

The bodies are likely buried in the `search_columns()` column definitions
(`design.md` §2) — the paint closures, the cell extractors, the column count.
Secondary sites: the MCP tool description cross-references, the `--columns` flag
bleed into `retrieve --help`, and the IMP-220/deprecation-behaviour discrepancy.

## Synthesis

### Verdict

**GUILTY OF HERESY — MINOR, REMEDIABLE.**

The design of SL-184 is sound in its architecture and phase decomposition. The
shared listing spine adoption is correctly patterned after the backlog, knowledge,
slice, review, and rec precedents. The rename surface is largely exhaustive. Two
compile-blocking defects in the `search_columns()` pseudocode betray a lack of
sufficient mortification of the flesh during authorship, but both are trivially
excisable. The remaining taints are traceability and completeness gaps — sins of
omission, not commission. No cardinal doctrine has been violated. No ADR has been
contravened. STD-001 (no magic strings) is upheld: column names are single-sourced
in the `Column` definitions and `SEARCH_DEFAULT` array. The design is FIT FOR
PLAN EXECUTION once the prescribed penance is performed.

### Penance Ordered

*Thou shalt perform these acts of contrition before any code is written:*

1. **EXCISE the duplicate `uid` column** from `design.md` §2, `search_columns()`
   body (F-1). The function must return 15 `Column` definitions, not 16.
   Verification: count the entries; run `select_columns()` with a default set and
   confirm exactly one `"uid"` column resolves.

2. **PURGE the extra `&`** from the type column's `ByValue` closure (F-2):
   `&c.memory.kind.as_str()` → `c.memory.kind.as_str()`. Verification: the closure
   must type-check against `fn(&Candidate) -> Option<DynColors>` when `as_str()`
   returns `&'static str` and `memory_type_hue` expects `&str`.

3. **RECONCILE IMP-220** (F-3): add a sentence to `design.md` §1 under
   "Deprecation alias" citing IMP-220 and stating the override rationale (silent
   clap alias chosen over stderr notice for simplicity). Update IMP-220 body to
   reflect the resolved decision.

4. **EXPAND the rename surface** (F-4): add three lines to the design's rename
   table covering the `memory_retrieve`, `memory_show`, and `memory_list` tool
   description prose references. Expand PHASE-01 EX-3 to cover "def + handler
   dispatch + onboard table + sibling tool description prose references."

5. **ADD help text** to the `--columns` field (F-5): clarify that the flag is
   scoped to the search table output and ignored by retrieve.
   `#[arg(long, help = "Column projection for search table output (ignored by retrieve)")]`.

### Standing Risks

- The `--columns` bleed into `retrieve --help` is tolerated. An agent or user
  passing `--columns` to `retrieve` will see it silently ignored. The help text
  mitigation reduces but does not eliminate this papercut. If clap's flatten model
  ever supports per-subcommand arg visibility, a cleaner separation becomes
  possible.

- The three `format_find_*` tests that asserted on the old hand-rolled layout must
  be rewritten in PHASE-02. The design correctly identifies this, but the
  rewrite scope is under-specified — column-projection tests should cover both
  the `search_columns()` definition and the `render_columns` output for at least
  the default and a custom column set.

### Taints Consciously Tolerated

- The shared `FindRetrieveArgs` struct is an acceptable architectural tradeoff:
  the cost of a separate args struct (code duplication, breaking the shared
  scope/filter contract) outweighs the UX papercut of an inert `--columns` flag
  in `retrieve --help`.

- The IMP-220/deprecation discrepancy is a traceability gap, not a correctness
  defect — the design is the authority per ADR-003.

### Harvest

- `mem.pattern.retrieve.search-columns-typeclosure` — record the `as_str()`
  double-reference footgun as a durable pattern memory, with the backlog
  `backlog_kind_hue(i.kind.as_str())` precedent as the correct form.

---

**BY THE AUTHORITY OF THE USER AND THE DOCTRINE OF THIS PROJECT, THIS DESIGN IS
REMANDED FOR PENANCE. LET THE FLAWS BE SCOURGED FROM THE TEXT BEFORE THE FIRST
LINE OF CODE IS WRITTEN. THE INQUISITION STANDS ADJOURNED.**

> **HERESIS URITOR; DOCTRINA MANET**

⚔️🔥
