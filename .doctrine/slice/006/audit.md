# Audit — SL-006 PHASE-04 (`adr status` verb)

Hand-authored review record (no `slice audit` scaffold yet). Reviewer pass over
commit `206794b` against plan.toml PHASE-04 (EX/VT), design §7 (D3, Q1/Q2),
§5.5 (I3/I5), and CLAUDE.md (storage rule, pure/imperative split,
behaviour-preservation, no-parallel-implementation). `/code-review` driven.

**Verdict: solid.** Conforms to the contract. No blocking or important findings.
Gate re-verified at audit time: `just lint` clean, `just test` 157 green.
Findings below are minor/optional — none gate close-out; logged for PHASE-05
harvest and the F1 follow-up.

## Conformance (the contract holds)

- **EX-1** `AdrStatus` ValueEnum, five variants — ✓ (`adr.rs:44`).
- **EX-2** `set_adr_status(root,id,status,today)`: read → I5 guard → `toml_edit`
  set `status`+`updated` → write; missing id errors — ✓ (`adr.rs:177`).
- **EX-3** `AdrCommand::Status{id,status,path}`, `--status` required (non-Option
  ValueEnum, no default) — ✓ (`main.rs:97`). No accidental default confirmed.
- **EX-4** clippy zero warnings — ✓.
- **VT-1/2/3** round-trip / byte-equality no-op / missing-id+out-of-enum — present
  (`adr.rs:367,392,409,417`). VT-2 is a genuine behaviour assertion (byte-equality
  proves the guard short-circuited). See test caveats below.

- **D3 divergence — justified.** `set_adr_status` is correctly NOT a
  generalisation of `state::set_phase_status`: no `[[progress]]` row, no
  `started`/`completed` stamps, `updated` (not `last_updated`), date (not
  timestamp), and it carries the I5 guard `set_phase_status` lacks. One consumer
  each → the non-shared helper is right per generalise-only-as-forced. The ~6-line
  read→parse→write skeleton is the only true duplication; design names F1
  `adr supersede` as the second consumer that earns the extraction. Not a defect.
- **Q1/Q2 — storage rule honoured.** No in-file transition log; `git` history of
  the committed toml is the audit trail. Authored `status`/`updated` are tool-owned
  structured TOML, not prose. ✓
- **pure/imperative split** — clock read in `run_status`, injected as `today: &str`
  into `set_adr_status`. ✓
- **behaviour-preservation gate** — entity/slice/state suites unchanged and green
  (157). ✓
- **I5 no-op guard** — reads current status off the parsed `DocumentMut` (single
  read, `adr.rs:187`), short-circuits before any write. Cannot be bypassed for
  tool-written values (enum-constrained, exact lowercase). ✓

## Findings

`audit.md:F-1: 🟡 minor`: `set_adr_status` (`adr.rs:191-193`) `insert`s `updated`
unconditionally. For a **tool-created** ADR the key always exists (template emits
it) and `toml_edit` updates it in place — correct. But for a **hand-authored** ADR
that omits `updated`, `insert` on the root table appends the key *after* the
`[relationships]` header, landing `updated = "…"` *inside* that table — silent
structural corruption. Low likelihood (design discourages hand-editing; template
always seeds `updated`) and no VT covers it. Fix if cheap: guard/repair when
`updated` is absent, or assert its presence. Otherwise note the assumption.

`audit.md:F-2: 🟡 minor`: VT-3 missing-id test (`adr.rs:409`) calls
`set_adr_status` against a root where **no ADR tree exists** — it exercises
"file absent", not I3's intent ("a missing id *among existing ADRs* is a hard
error, no implicit create"). Same `not found` error fires either way, so the test
passes, but it doesn't prove the I3 case. Stronger: `run_new` one ADR, then
`set_adr_status(..., 9, ...)`.

`audit.md:F-3: 🟡 minor`: the out-of-enum test (`adr.rs:417`) asserts
`AdrStatus::from_str("bogus")` is `Err` — that exercises clap's derive (library
behaviour), and does **not** assert the design edge's second half ("no file
touched"). Cheap proxy, but closer to theatre than behaviour. The real guarantee
(clap rejects before any dispatch) is structural, not covered here.

`audit.md:F-4: 🔵 optional`: `run_status` (`adr.rs:166`) prints
`ADR NNN: <status>` unconditionally, including on an I5 no-op — the output reads
as "set" when nothing was written. Idempotent and not wrong (state *is* that
value), but it gives no signal that the transition was a no-op. Consider an
"unchanged" hint. No VT requires it.

`audit.md:F-5: 🔵 optional`: `fs::write` (`adr.rs:194`) is truncate-then-write,
not atomic. Consistent with `state::set_phase_status` precedent — but that writes
**gitignored disposable runtime state**, whereas this writes a **committed authored
artifact**, where a torn write is higher-stakes (git recovers, but the working
tree is left corrupt). A write-temp-then-rename would harden it. Shared latent
risk; flag for the F1 extraction where the skeleton consolidates.

`audit.md:F-6: 🔵 optional`: the `not found` context on `read_to_string`
(`adr.rs:181`) conflates "ADR absent" with any read failure (e.g. permission
denied) — a permission error would mis-report as "not found". Inherited from the
`set_phase_status` pattern; anyhow preserves the underlying io error as the source,
so diagnosis is still possible. Low value.

## Deferral upheld

- **Schema-header assertion (`schema = "doctrine.adr"`)** deliberately not asserted
  before mutating — **agree with the deferral.** The path is computed to
  `adr_root/NNN/adr-NNN.toml`, so the mutation is structurally constrained to ADR
  files; no VT requires the assertion; no realistic call reaches a non-ADR toml.
  Logged in phase-04.md Decisions. Not a finding.

## Disposition

Highest-value drift check — code vs design §7 / §5.5 / storage rule — found
**no drift**. Remediation actioned at close-out (commit on top of `206794b`):

- **F-1 — FIXED.** `set_adr_status` now bails on a malformed ADR missing the
  template-seeded `status`/`updated` keys, instead of a tail `insert` that would
  corrupt the trailing `[relationships]` table. New VT
  `set_adr_status_on_an_adr_missing_updated_errors` covers it. Fail-loud chosen over
  in-place repair (refuse-to-clobber ethos; scaffold owns those keys).
- **F-2 — FIXED.** VT-3 rewritten
  (`set_adr_status_on_a_missing_id_among_existing_adrs_errors`): scaffolds one ADR
  then targets id 9, proving I3 ("missing id *among existing ADRs*"), not the weaker
  "file absent" case.
- **F-3 — DROPPED.** The out-of-enum test exercised clap's derive, not our code;
  deleted. VT-1 byte-equality already proves the no-op-write guarantee.
- **F-4 / F-6 — DEFERRED (optional polish).** No correctness teeth; harvest to
  `notes.md` at PHASE-05.
- **F-5 — DEFERRED → F1.** Non-atomic `fs::write` hardens (temp+rename) when the
  read→parse→write skeleton extracts for `adr supersede` — fix once, both consumers.

Gate after remediation: `just lint` clean, `just test` **157 green** (deleted F-3,
added F-1 → net zero).
