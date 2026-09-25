# Notes SL-265: Unified doctrine show: one verb for any canonical ref

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design triage (exploring, 2026-09-25)

Surface read: `src/commands/cli.rs` (`Command` variant, `FAMILIES` `explore`
row, dispatch arm), `src/commands/show.rs` (new router, command tier),
`src/commands/guard.rs` (one exhaustive-match arm), `tests/**` (equivalence +
totality), `.doctrine/spec/tech/013/**` + one new `REQ`.

Constraining governance: `ADR-001` (leaf ← engine ← command; command tangle
ratcheted `= 76`, top-level granularity); `ADR-013`/`ADR-003` (governance change
routes through a `REV`; specs reconciled, not edited aspirationally);
`SPEC-013` (`REQ-197` uniform `<kind> <verb>`, `REQ-198` kind-blind *list*
spine — a kind-blind *show* takes a new REQ); `REQ-113` (closure gate:
`Indeterminate` on a SPEC-013 member is residual drift until covered or
REC-excused); `REQ-136` (`spec req add` is the only REQ producer; `REV` stages
intent + frozen label); `REQ-192 §3` (`write_class` exhaustive — a new verb is a
compile error until classified); `POL-002` (owned contracts, no silent host
dependency — satisfied by pure delegation); `STD-001` (ride `KINDS`/`kinds::`
constants, no hard-coded prefixes); `ADR-005` (docs point at `--help`). Not
engaged: `ADR-004`, `ADR-016`, `ADR-019`, `POL-001`, `STD-003`.

Settled by research (not open): the module home is `src/commands/` (R1 — adds
no tangle edge, needs no tier row); the delegate contract is
`parse_resolvable_ref` → `kinds::canonical_id(prefix, id)` → `<kind>::run_show`
(every per-kind parser rejects a foreign prefix; `spec::resolve_spec_ref`
requires the hyphen); `outbound_for` models the three dispatch shapes and
carries the `debug_assert!(false)` fallthrough precedent.

Open questions: `OQ-1` fidelity (byte-identical vs normalised envelope);
`OQ-2` address scope (numbered prefixes only vs `mem_…`/observation/design refs);
`OQ-3` bare ids; `OQ-4` totality mechanism. Plus the governance-phase shape the
scope left as a footnote.

Risks: `R1` tangle (mitigated by home); `R2` census assertion churn (deliberate);
`R3` `REQ` is the one prefix not on `spec::run_show`; `R4` boot-map/help goldens
(member presence only; boot snapshot regenerated). Assumption `A1`: every
numbered prefix has a working `show` today — `OQ-4`'s guard is what surfaces a
counterexample.

Memory recalled: `mem_019fe0eacf…` (new variant needs a `guard.rs` arm);
`mem_019f20e3…` (bare-id ambiguity must be rejected, not first-match);
`mem_019f20c6…` (two id-parsing functions — `parse_resolvable_ref` is the
canonical-ref one); `mem_01a0b257…` (`<kind> show` is not cheap; derives the
comparison pipeline twice — irrelevant here since the router adds no render);
`mem_019f944d…` (forward-intent additions to a shipped spec must be `pending`
introduce-only); `mem_01a00982…` (`coverage record --mode VT` with no check
silently stores an attestation — the governance phase must bind a real check);
`mem_019f239c…` (a REV-only slice reports its authored deliverable as
`undeclared` in conformance — expected, dispose `aligned`).

## Review (design run, 2026-09-25)

Pass `RV-384` (design), concluded. Two `major` findings raised, both dispositioned
`route:control fixed` and integrated into the design and its records:

- `F-1` — the totality test iterated the sibling `ALL_KINDS` literal; the
  resolver reaches `route()` through `KINDS`, so the test must iterate `KINDS`
  (plus a negative control). Integrated into `sec-3`, `sec-7` and `DEC-298`.
- `F-2` — `coverage record` leaves a check-bearing cell `Planned`; the
  governance phase must also run `coverage verify 265` and exit on the
  `Verified` cell. Integrated into `sec-5` and `sec-7`.

**A further pass would probe** (superseded — an independent hostile pass ran):

1. Whether the router's `--json` shorthand resolves identically to each kind's
   own — `review`/`concept-map` do not flatten `CommonShowArgs` and resolve the
   shorthand in their own dispatch.
2. The bare-id scan cost.
3. The equivalence fixtures.

### Second pass — independent adversarial read (2026-09-25)

Ran `./scripts/pi-research` over `design.md`, the scope and the cited governance,
with a hostile prompt. It falsified 12 claims, all verified against source before
integration. Ledgered on `RV-384` as `F-4`–`F-15` (F-1/F-2 were the earlier
in-session findings):

| id | sev | finding | fix |
|---|---|---|---|
| `F-4` | blocker | delegation table omitted `RFC` — 23 of 24 prefixes routed, while the worked example is `doctrine show RFC-031` | `RFC` joins the governance spine and the `Route` set |
| `F-5` | major | totality test cannot be an integration test (`route()`/`KINDS` are `pub(crate)`) | unit test in `src/commands/show.rs` |
| `F-6` | major | REV sequence not executable (one row per `change add`; `modify` takes a live FK; `revision new` omitted) | `revision new` + one introduce row; prose rides the REV `.md` |
| `F-7` | major | new REQ lands `pending`/`Coherent`, not `Indeterminate`; the lever is the reconcile flip | sec-5 restated |
| `F-8` | major | home adds a `commands → governance` edge; top-level risk is `Unclassified`, not tangle | call per-kind wrappers; R1 corrected |
| `F-9` | major | `spec::run_show` takes no `SpecSubtype` | `Route::Spec` carries no payload |
| `F-10` | minor | `--json` shorthand has four sites, not two | all four named |
| `F-11` | minor | bare-id premise not universal (only backlog/knowledge/spec/review reject) | claim narrowed |
| `F-12` | minor | equivalence reference command needs a per-group map (`REQ` is `spec req show`) | map stated |
| `F-13` | minor | coverage record sample binds no check | explicit check binding + `coverage verify` |
| `F-14` | minor | `REQ-197` misquoted (mandates, not admits-only) | rephrased |
| `F-15` | nit | twelve/memory wording, `parse_ref` detail, fallthrough literal | fixed |

`F-1`, `F-2` (both `control`), `F-5`, `F-13` remain `answered`, to be verified at
`/plan` when their criterion lands on a phase. `RV-384` is `STALE` (it covers the
pre-integration section set); the human section attestations at lock cover the
final content.

### Repair pass and lock (2026-09-25)

`F-16`–`F-22`, `F-24` verified; `F-20`→`F-25`, `F-23`→`F-26` rework, `F-27` new,
`F-28` new. Repaired across `sec-2`/`sec-7`/`sec-8`, `DEC-295`/`DEC-296`/`DEC-298`
and `slice-265.md`: the router ASCII-uppercases the prefix unconditionally, and
the equivalence is scoped to **prefixed** refs (a bare id resolves across kinds
and refuses as ambiguous). `RV-384` concluded with `F-1`, `F-2`, `F-5`, `F-13`
left `answered` for `/plan` (instrument-routed `route:control`); the eight
sections attested and `design-accepted`. Design run **locked at revision 36**.

## Plan (2026-09-25)

Ready. Four phases authored in `plan.toml`, rationale and sequencing in
`plan.md`, runtime sheets materialised (`doctrine slice phases 265`).

1. `PHASE-01` — the verb: `src/commands/show.rs` router (`Route`, `route()`,
   `run_show()`) + `Command::Show` + `explore` family + `guard.rs` Read + census
   56→57 + `CommonShowArgs::format()`.
2. `PHASE-02` — fidelity: `tests/e2e_show_equivalence.rs` (one fixture of every
   numbered prefix, table + json) and `tests/e2e_show_refusals.rs`.
3. `PHASE-03` — guidance: `install/using-doctrine.md`,
   `install/routing-process.md`, `doctrine boot`, and the `guardrails_paragraph`
   test sibling.
4. `PHASE-04` — governance: `REV` (introduce `FR-006` on `SPEC-013`),
   `spec req add`, check-bound `coverage record` then `coverage verify 265`,
   hand-landed spec prose, `revision status done`.

Plan decisions:

- Selectors extended with `install/using-doctrine.md` and
  `install/routing-process.md` — design-targets the locked design's code-impact
  table touches but the design-time selector set did not carry.
- The 24 equivalence fixtures were each confirmed present (`doctrine inspect`)
  before being named in `PHASE-02` EX-2.
- Re-grep at plan time: every design path/symbol resolves; line numbers drifted
  by at most two. No stale premise.
- `RV-384`'s four instrument-routed findings (`F-1`, `F-2`, `F-5`, `F-13`)
  transcribed onto phases and verified; the review reads `done` (28/28).
  `F-6`/`F-17` hold the `SPEC-013` prose off the typed rows — it rides the
  `revision-NNN.md` companion and is hand-landed.
- The tracked `.doctrine/using-doctrine.md` is a stale projected copy (ADR-019);
  `install/using-doctrine.md` is the source, and the legacy copy is out of scope.

## Execution (2026-09-25) — PHASE-01 landed

Driven through the dispatch funnel: coord `dispatch/265` off `B=2464bbfd6`,
confined `pi` worker fork `dispatch/sl265-p01` (reaped). Landed as **two**
commits on `dispatch/265`:

- `d820791ca` `chore(SL-265): declare src/commands/mod.rs a design-target selector`
- `1ca75a00a` `feat(SL-265): kind-blind show router, Command::Show, guard, census (PHASE-01)`

Verified on the coord tree: `every_kinds_row_routes`, `show_is_read`,
`families_partition_the_visible_command_tree` green; the five `show` goldens
(`adr`/`standard`/`knowledge`/`help_families`/`boot_map`) pass unchanged;
`doctrine show SL-001` is byte-identical to `doctrine slice show SL-001`;
`[explore]` carries `show` in `--help` and `--help --boot-map`. `record-delta`
spans `B..S` (the `--start/--end` escape hatch — the phase legitimately spans the
selector commit).

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-25 · PHASE-03 landed (8868d76a0) · 3/4 phases

### Produced

- `SL-265` plan authored (`plan.toml` + `plan.md`) and its runtime sheets
  materialised; `RV-384` closed `done`.
- `PHASE-01` (`1ca75a00a`): the `show` router (`src/commands/show.rs`),
  `Command::Show`, the `explore` family row, `guard.rs` `Read`, census 56→57,
  `CommonShowArgs::format()` (four inline `--json` sites collapsed), and the
  declared `src/commands/mod.rs` selector (`d820791ca`).
- `PHASE-02` (`99f3ad4fd`): `tests/e2e_show_equivalence.rs` (24 prefixes ×
  `--format table|json`, both sides the same binary) and
  `tests/e2e_show_refusals.rs` (unknown prefix / dangling ref / ambiguous bare id).
- `PHASE-03` (`8868d76a0`): `install/using-doctrine.md` (verb table + read-entity
  paragraph) and `install/routing-process.md` (Guardrails) name `doctrine show
  <REF>`; `tests/e2e_claude_install.rs` gains `guardrails_paragraph` + the
  anti-vacuity-guarded execution test. Declared `tests/e2e_claude_install.rs` a
  selector (`6bc730dd8`).

### Learned

- The per-kind ref parsers do not share a prefix case rule (`listing::parse_ref`
  two literal cases; `knowledge`/`backlog` uppercase; `spec` case-sensitive) —
  `RV-384` `F-25`.
- **The import scope belt is the selector set, not the design's prose.** A file a
  phase edits must be a declared selector *before* the spawn (PHASE-01 refused
  `src/commands/mod.rs` `undeclared-scope`; remedy `doctrine slice selector add`).
- **`doctrine check regression diff` false-halts a persisted env failure.** The
  signature normalises `src/<file>.rs:LINE:COL` but keeps the panic's **thread
  id**, so the same pre-existing failure (`reserve::tests::vt3_auto_degradation…`,
  ambient `DOCTRINE_RESERVATION_FALLBACK=1`) reads `changed` (halt) instead of
  `persistent` (tolerated). Reproduced identically at `B` in the untouched
  primary tree. Observation `01a0d82d-21cd-7b50-9b95-a3fe859ab25a`.
- **`worktree gc` certifies TRUNK landing, not coord-branch landing.** A fork
  whose delta landed only on `dispatch/265` refuses a plain reap (`not-landed`);
  reap with `--superseded-head <B>` (asserts the branch spent at its exact head),
  never a blind `--force`.
- All 24 numbered prefixes are **byte-identical** to their kind's own `show` in
  both formats — the PHASE-01 delegation table is faithful; PHASE-02 needed no
  router repair.
- **A rust-embed asset edit does not move the primary's derived snapshot until
  the primary's binary carries it.** `cargo` re-bakes the `install/` embed, but
  the primary binary is built from `edge`, which lacks the not-yet-integrated
  PHASE-01 code — regenerating the primary's `boot.md` there would advertise
  `doctrine show` for a verb that binary refuses. EX-4 is evidenced on the coord
  tree; the primary snapshot refreshes at integrate/close.
- **PHASE-04 is an orchestrator phase, not a dispatch phase.** Its whole
  deliverable (a `REV`, `SPEC-013` prose, a coverage cell) is authored
  `.doctrine/` state — which a confined worker cannot write and the import belt
  rejects. It must be driven directly in a writable tree (the coord tree has the
  PHASE-02 test to bind the coverage check).

### Open

- `PHASE-04` (governance) — the one phase left. Orchestrator-driven, in the
  **coord tree** (`dispatch/265` carries the PHASE-01 code + the PHASE-02 test the
  coverage check binds): `revision new` → one `introduce FR-006 on SPEC-013` row
  → `revision approve`/`apply` → `spec req add` → check-bound `coverage record`
  then `coverage verify 265` (exit on the `Verified` cell) → hand-land
  `spec-013.md` § *Uniform command grammar* + § *Responsibilities* and
  `spec-013.toml`'s `responsibilities` → `revision status … done`. Then
  `dispatch sync --prepare-review` and `/audit`.
