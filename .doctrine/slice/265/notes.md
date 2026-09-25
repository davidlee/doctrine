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

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-25 · locked (revision 36) · 73895ec9d

### Produced

- `SL-265` design locked; slice advanced to `plan`.

### Learned

- The per-kind ref parsers do not share a prefix case rule (`listing::parse_ref`
  two literal cases; `knowledge`/`backlog` uppercase; `spec` case-sensitive) —
  `RV-384` `F-25`.

### Open

- `/plan`: transcribe the `route:control` criteria (`F-1`, `F-2`, `F-5`, `F-13`)
  onto phases; then implement.
