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

- **D-P5-1 — the registry stores CANONICAL-STRING id sets, not numerics
  (PHASE-05).** `Registry { requirements: BTreeSet<String>, tech_specs:
  BTreeSet<String>, members, interactions }` holds `"REQ-NNN"`/`"SPEC-NNN"`. FKs are
  stored canonical too, so every check is a direct `BTreeSet::contains` — no
  FK→numeric parse at the check site (`requirement::id_from_fk` and
  `resolve_spec_ref` stay out of the pure checks). The **tech-only interaction rule
  falls out for free**: a `PRD-*` target is simply absent from `tech_specs` → flagged
  as "resolves to no tech spec" by the same membership test (no subtype branch).

- **D-P5-2 — registry seam split (PHASE-05).** `registry.rs` = the pure index type
  + check fns (disk-free, unit-tested over literal `Registry` values) + the
  `validate(scope)` aggregator that encodes the scope policy (3 outbound/intra-spec
  checks always; orphan corpus-only). `spec.rs` = `build_registry` (the impure scan,
  rides its own private `read_members`/`read_interactions`, no widening) +
  `run_validate` (the verb). This is the relation-index *seed*; the cache and cycle
  detection arrive with the feature DAG (deferred).

- **D-P5-3 — the scan reaches members on BOTH subtypes; interactions tech-only.**
  Products member requirements too, so dangling-member + orphan coverage iterates
  product *and* tech trees. No product *id set* is materialised (no check resolves
  against one — "generalise only as far as forced"). `tech_specs` is the only spec
  id set built (interaction targets resolve against it).

- **D-P5-4 — exit code via `bail!` after the report.** Findings → stdout report
  (`Vec<String>` + `concat()`, F-7), then `anyhow::bail!("validate: N hard
  finding(s) in <target>")` → anyhow's `fn main() -> Result` reporter exits non-zero.
  Clean → `validate: <target> clean` to stdout + `Ok(())` → 0. `<target>` is the
  canonical scope ref or `"corpus"`.

- **F-10 — two widenings P5 added (PHASE-05).** `SpecSubtype::canonical_id(self, id)
  -> String` (the inverse of `resolve_spec_ref`; DRYs `run_new`'s print + the scan)
  and `requirement::tree_root(root) -> PathBuf` (the requirement tree dir for
  `scan_ids`, keeping `REQUIREMENT_DIR` private). `resolve_spec_ref` stayed `fn`
  (private) — the scoped-validate ref parse lives in `spec.rs::run_validate`, so the
  long-flagged `pub(crate)` widening (F-4) was **not** needed after all.

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

## PHASE-06 — canon sweep + close-out

- **D-P6-1 — canon reconciled to the as-built peer-entity model (four-file
  sweep).** `doc/spec-entity-spec.md` rewritten (status deferred→SHIPPED; thesis
  pathology/diagnosis kept; decomposition/identity/schemas/serde/lifecycle/risks
  rewritten); `doc/entity-model.md` three spots (spec-family row, identity/
  references, edges); `doc/relation-index.md` facet-row count (:52) + the edge-
  tables list (:93); `doc/glossary.md` additively gained `requirement`/`REQ-` +
  the `FR-`/`NF-` membership-label rows. The overturned creed in one line: a
  requirement is a **reserved numeric peer entity `REQ-NNN`** (not a compound-key
  facet row), membered by a spec-side `members.toml` `[[member]]` carrying the
  durable FK + a **mobile** `FR-`/`NF-` label + advisory `order`; `collaborators.toml`
  dissolved (cross-spec reuse = deferred `spec req link`); render is ephemeral
  one-way `spec show` (D8/D9); FK integrity is `spec validate`, not creed. This
  **inverts** the draft's Option A (which froze a compound key by *forbidding*
  requirement moves — the peer model makes moves safe instead).

- **D-P6-2 — intended grep-gate residue (VT-1).** Gate
  `grep -rnE 'rows, not artefacts|SPEC-[0-9]+\.(FR|NF)' doc/` returns ONLY
  `spec-entity-spec.md:12` and `:121` — both deliberate superseding callouts
  ("the old draft said `SPEC-110.FR-001`; the shipped model uses `REQ-NNN`").
  Named historical keeps, not live creed. No `rows, not artefacts` survives.

- **D-P6-3 — out-of-scope `collaborators`/old-model residue, deliberately left.**
  `slices-spec.md:99` and `drift-spec.md:153` carry `specs.collaborators` — that
  is the **slice** spec-scope field (primary ∪ collaborators, coverage-gate
  scope), a *different* concept from the dissolved spec `collaborators.toml`; both
  are outside P6's named four-file scope. `drift-spec.md` is gate-clean already (no
  `SPEC-N.FR` / rows-not-artefacts) but still describes the old block model in
  places — a future drift-ledger slice reconciles it; not P6's mandate.

- **D-P6-4 — skills now point at shipped verbs.** `spec-product`/`spec-tech`
  dropped the "Not yet structural" caveat; each points at `spec new product|tech`,
  `spec req add`, `spec show`, `spec validate`, `spec list` and names the
  `spec-NNN.toml`/`.md` + `members.toml` (+ tech `interactions.toml`) fileset.

- **F-P6-1 — `Source` back-filled into spec-entity-spec.md.** Notes D-3's resolved
  `Source { language, identifier, module? }` (tech-only code anchor) now appears in
  the Metadata + Serde sections (the §5.3-undefined gap closed in the canon, the
  "natural home" D-3 flagged).

- **F-P6-2 — the new identity/edge seam recorded as memory** (distinct from the
  engine's `mem.system.engine.identity-claim-seam`): see
  `mem.system.spec.composition-seam` — durable `REQ-NNN` FK + mobile `FR-`/`NF-`
  label + `order` in `members.toml`, tech-only `interactions.toml [[edge]]`,
  `collaborators` dissolved, `spec validate` FK gate.

## Post-audit code-review (held-open close gate)

Code-review over `git diff 11359db..HEAD -- src/ install/` (the slice src diff,
post-audit). Two findings fixed before close; the rest named as follow-ups. No
slice re-open ceremony — surgical fix on a held-open slice, gate re-green.

- **F-P7-1 — FK identity was split between two readers (🟠, FIXED).** `spec show`
  resolved a member FK by *parsing* (`requirement::load` → `id_from_fk`, so
  `REQ-1` → dir `001`), but `spec validate` compared FK strings **byte-exact**
  (`build_registry` pushed the raw `m.requirement`; `Registry` does
  `BTreeSet::contains`). So a hand-authored non-canonical FK (`REQ-1`, `SPEC-2`)
  pointing at a real entity rendered fine in `show` yet was flagged **dangling**
  by `validate` — and its target double-flagged **orphan**. Two notions of
  identity for the same byte. **Fix:** unify on parse→reformat at registry-build.
  New `requirement::canonicalize_fk(&str) -> String` (best-effort:
  `id_from_fk().map_or_else(verbatim, canonical_id)`) — junk (`garbage`, `REQ-x`,
  wrong prefix) passes through verbatim, so genuinely-dangling FKs stay flagged.
  `build_registry` now canonicalizes `m.requirement` through it, and each
  interaction `e.target` through `resolve_spec_ref(..).map(|(s,n)| s.canonical_id(n))
  .unwrap_or(e.target)`. The canonicalizer matches `load`'s existing tolerance, so
  `show` and `validate` now share one identity notion. Test
  `build_registry_canonicalizes_member_and_interaction_fks` (spec.rs) +
  `canonicalize_fk_normalises_and_passes_through_garbage` (requirement.rs).

- **F-P7-2 — `render` reinvented `canonical_id` (🟡, FIXED).** `render`'s opening
  `format!("{}-{:03}", spec.kind.kind().prefix, spec.id)` duplicated
  `SpecSubtype::canonical_id` (which the *same fn* reuses for requirement refs 13
  lines down). Replaced with `spec.kind.canonical_id(spec.id)`.

- **F-P7-3 — named follow-ups, NOT fixed (out of surgical scope).**
  - `spec show` hard-fails the whole document on one dangling member FK
    (`run_show` propagates `requirement::load`'s error) — no partial/degraded
    render. Conscious (D-P4-1) but a resilience gap: the readable-whole is
    unreadable until the FK is hand-fixed. A `⚠ <REQ> unresolved` line would keep
    `show` usable. Candidate for the registry-surface slice (inbound refs, R3).
  - `as_str()` on `SpecStatus`/`C4Level`/`ReqStatus` hand-mirrors the
    `#[serde(rename_all = "kebab-case")]` mapping — compiler catches a *missing*
    variant (exhaustive match) but not a *typo* divergence, which would split
    `spec show` (uses `as_str`) from `spec list` (reads the raw toml string).
  - `read_members`/`read_interactions` are the same NotFound-tolerant
    parse-into-`Doc`-wrapper twice; `spec.rs` at 1.6k lines mixes parse types +
    four verbs + pure render + the impure `build_registry` scan. Cohesion debt,
    not a defect; `render` is the clean amputation point if a sixth verb lands.

- **F-P7-4 — gate re-green post-fix.** `cargo clippy` (bins/lib) zero warnings;
  `cargo test --bin doctrine` **406 passed** (404 audit baseline + 2 new); `cargo
  fmt --check` clean. Touched only `src/spec.rs` + `src/requirement.rs` — engine,
  `meta.rs`, `registry.rs` logic all unchanged (behaviour gate intact). e2e
  unaffected (pure-Rust change; the off-PATH e2e skips are the pre-existing
  `mem.pattern.testing.stale-cargo-bin-exe` condition, not a regression).
