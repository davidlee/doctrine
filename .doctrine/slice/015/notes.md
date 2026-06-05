# SL-015 Implementation Notes

Durable, committed record of decisions/findings that outlive a phase sheet.
Design rationale lives in `design.md`; phase criteria in `plan.toml`; this is for
cross-phase implementation decisions the design didn't pin.

## Cross-phase decisions

- **D-1 — requirement `kind` is template-seeded, then overwritten post-reserve.**
  `ReqKind` (functional|quality) is not carried by `entity::ScaffoldCtx`, and the
  engine must stay unchanged (R6 gate). So `install/templates/requirement.toml`
  seeds a default `kind = "functional"`, and `spec req add --kind` (PHASE-03) sets
  the real value after reservation via an edit-preserving `toml_edit` write —
  exactly the `adr::set_adr_status` pattern (status seeded `proposed`, later
  flipped). Spans PHASE-01 (seed) → PHASE-03 (overwrite). No engine edit.

- **D-2 — staged-landing lint bridge.** A module landing one phase ahead of its
  first production caller (PHASE-01 `requirement.rs` before PHASE-03 `spec req
  add`) is `dead_code` in the bins/lib build. Bridge each pending item with
  `#[cfg_attr(not(test), expect(dead_code, reason = "first caller in PHASE-NN"))]`.
  Bare `#[allow]` is a hard error in this repo — see the recorded pattern memory
  `mem.pattern.lint.expect-not-allow` (`doctrine memory retrieve --query expect
  allow`). PHASE-02 is expected to unwind it as `spec.rs` references the
  requirement types; `expect` makes any stale bridge a build error, so it cannot
  rot.

- **D-3 — `Source` shape (PHASE-02, user-confirmed).** `design.md` §5.3 declares
  `sources: Vec<Source>` (tech-only) but never defines `Source`. Resolved to the
  legacy-canon shape (`doc/spec-entity-spec.md:202`): `Source { language: String,
  identifier: String, #[serde(default)] module: Option<String> }` — a code anchor.
  Tech-only, seeded-empty in P2; payload first rendered by `spec show` (PHASE-04).
  The design should be back-filled with this when §5.3 is next touched (P6 canon
  sweep is a natural home).

- **D-4 — `spec list` renders per-subtype sections, not one table.** `PRD-001` and
  `SPEC-001` co-exist (independent reservation namespaces), so a single `id` column
  would be ambiguous. `run_list` prints a `product` block then a `tech` block, each
  a labelled `id status slug #members` table over the shared `meta::render_table`.
  Empty subtype → suppressed. (PHASE-02.)

- **D-1 — CLOSED (PHASE-03).** The seeded `kind = "functional"` is overwritten
  post-reserve by `requirement::set_kind` (edit-preserving `toml_edit`, the
  `set_adr_status` shape). Verified on the built binary: a `--kind quality` add
  lands `kind = "quality"` on the reserved requirement. No engine edit.

- **D-2 — CLOSED (PHASE-04).** `spec show`'s `render` reads every spec parse struct
  (`Spec`/`SpecStatus`/`C4Level`/`Source`/`Interaction`) and `requirement::load`
  reads `Requirement`, so all `cfg_attr(not(test), expect(dead_code …))` bridges
  are erased. Verified: `rg "expect\(\s*dead_code" src/` finds only the *other*
  modules' bridges (git/memory), none in spec/requirement. `Interaction`'s bridge
  reason said PHASE-05 but render (P4) was its real first caller — dropped in P4.

- **D-P4-1 — "statement" in `spec show` = the requirement's `description` field,
  NOT its `.md` prose.** Design §5.4's requirement entry wants "kind, statement,
  acceptance criteria"; §5.3 puts the full statement in `requirement-NNN.md` prose.
  The storage rule (§3, "tooling never parses prose structure") forbids extracting
  the `## Statement` section, and render is a pure fn over *parsed facets*. So the
  structured `description` (`Option<String>`) is rendered as the statement line;
  absent → no line. The spec's *own* `spec-NNN.md` is still emitted **verbatim**
  (whole-body dump is not a structural parse). No `/consult` — the storage rule
  resolves the design tension cleanly.

## Findings

- **F-4 — `resolve_spec_ref` is the shared canonical-ref parser (`spec.rs`).**
  PHASE-03 added it: `<spec-ref>` (`PRD-NNN`/`SPEC-NNN`) → `(SpecSubtype, u32)`,
  bare numeric rejected (C4), prefixes derived from the two `Kind`s (single
  source). **PHASE-04 `spec show` and PHASE-05 `spec validate` take the same
  `<spec-ref>` — reuse this fn, do not refork.** It is currently `fn` (private to
  `spec.rs`); both later callers live in `spec.rs`/`registry.rs`, so widen to
  `pub(crate)` when P5's `registry.rs` needs it.

- **F-5 — `toml_edit` `Table` index-assign (`tbl["k"] = …`) trips
  `clippy::indexing_slicing`** (a repo deny). Use `Table::insert("k", value(…))`
  for the edit-preserving member append (and any future row writer). The
  array-of-tables `push` lands new rows *above* a file's trailing document
  comment — cosmetic, comment survives, valid toml (the edit-preserving guarantee
  is survival, not position).

- **F-1 — lint suppression form** captured durably as memory
  `mem.pattern.lint.expect-not-allow` (not repeated here; the storage rule).

- **F-2 — `entity::Kind.dir` is project-root-relative and MUST include
  `.doctrine/`** (`.doctrine/spec/product`, not `spec/product`). PHASE-02 first set
  it without the prefix; tempdir unit tests passed because they built their
  expected path from the same constant — only a **CLI smoke test against the built
  binary** exposed the misplacement. Durable lesson: smoke the real binary for any
  new `Kind`; a self-referential path assertion proves consistency, not
  correctness.

- **F-3 — the `#members` column needs zero `meta.rs` change.** `meta::read_metas`
  (stem-parametric) + `meta::render_table` (generic grid) already suffice; the
  derived cell is built in `spec.rs`, mirroring `slice.rs`'s `phases` cell. The
  shared 4-column `format_list` path is untouched — the strongest form of the
  behaviour-preservation gate (PHASE-02 VT-3).

- **F-6 — `requirement::load(root, "REQ-NNN")` is the by-FK reader seam (PHASE-04).**
  Parses the canonical FK with `REQUIREMENT_KIND.prefix` (single source, mirrors
  `resolve_spec_ref`) and reads `requirement-NNN.toml` → `Requirement`. `spec show`
  resolves each member through it; **PHASE-05 `spec validate` reuses it** for the
  dangling-FK check (currently `pub(crate)`). It opens only the membered dirs —
  never scans the requirement tree (EX-2 "no cross-corpus scan").

- **F-7 — String assembly: NOT `push_str(&format!(…))`, NOT `write!(…).expect()`.**
  This repo denies `clippy::format_push_string` AND `clippy::expect_used` /
  `unwrap_used` for **non-test** code (`Cargo.toml [lints]`), and
  `let_underscore_must_use` kills `let _ = write!(…)`. So the infallible-`fmt::Write`
  idioms are all closed. House style (cf. `retrieve::format_find`): build a
  `Vec<String>` of pre-formatted pieces (`parts.push(format!(…))` — `Vec::push` is
  not the lint) and `parts.concat()`. `render` is built this way. A memory was
  recorded; see `mem.pattern.lint.string-build-no-push-format`.

- **F-8 — `interactions.toml` uses `[[edge]]`, not `[[interaction]]`** (the seed
  template's array key). `read_interactions` parses via an `InteractionsDoc { edge:
  Vec<Interaction> }` wrapper, mirroring `read_members`/`MembersDoc`. A missing file
  → `[]` (product specs have none — absent, not empty), so render's
  empty-slice-omits-the-block rule (VT-3) covers product and zero-edge tech alike.

- **F-9 — render emits no H1 of its own.** The spec's `spec-NNN.md` prose body
  (dumped verbatim) already carries `# <ref>: <title>`; a synthetic identity H1
  would double it. So the identity line is non-H1 (`` `SPEC-001` — Title `` +
  a `slug · status · kind` line), and the prose's H1 is the sole one. Trade-off:
  if an author strips the prose H1, the rendered doc has no H1 — acceptable under
  "prose verbatim, structured identity is authoritative".
