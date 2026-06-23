# Review RV-147 — design of SL-146

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This Inquisition probes SL-146's design artifact (`design.md`) for conformance to its
scope, to ADR-015 (multi-dimensional priority scoring), to SPEC-001 (graph-derived
priority engine), and to the existing code terrain it rides.

The design admits nine decisions and eleven edge-case rows. The Inquisitor presses
six lines of interrogation:

1. **Does D7's proposed `load()` refactor match the code that exists?** The design
   prescribes an internal split (`read_priority_table` + `load_from_table`) that the
   actual `src/priority/config.rs` does not follow — it is a refactor proposal, not a
   description of present ground truth. If the design is inaccurate about the code it
   rides, the phase-plan will build on sand.

2. **Is the `parse_config_path` strictness compatible with the scope's extensibility
   promise?** The scope says "The `show`/`set`/`get` arg parser is designed to be
   extensible" — yet D2a rejects unknown static keys with `bail!`, making forward-compat
   impossible for `get` and `set`. A new key added to `[priority.coefficients]` in a
   future version of doctrine would be invisible (and unwritable) to the CLI until a
   code change re-validates it.

3. **Are all ADR-015 coefficient domains faithfully reflected in the edge-case table?**
   ADR-015 declares `ref_coeff` "flat and non-negative" — negative values clamp to 0.0.
   The design's edge-case table covers NaN/Inf but omits negative `ref_coeff` explicitly,
   and the `dep_coeff` clamp policy in D5 says `clamp_dep`, correct but the table row is
   `set dep_coeff ≤ 0 → Clamp to 0.0` — is the `= 0` case correct per ADR-015's "(0,1]"
   domain? Yes (0 is the disable sentinel). But the table doesn't call out that `0.0` is
   a legal ADR-015 value, not a clamp artifact.

4. **`clamp != value` for NaN/Inf detection — is the design's assertion accurate?** D5.3
   and D7.7b state "NaN/Inf naturally differ" post-clamp. For NaN: `clamp_general(NaN, 1.0)`
   returns `1.0`, and `1.0 != NaN` is `true` — correct. For Infinity: `clamp_general(Inf, 1.0)`
   returns `1.0`, and `1.0 != Inf` is `true` — correct. The design statement is technically
   sound but the reasoning is truncated — it doesn't explain *why* the comparison works
   (because the clamp replaces non-finite with a finite fallback, and finite ≠ non-finite
   is always true). This is a clarity defect, not a correctness one.

5. **Does the test plan cover all scope verification criteria?** The scope lists 11
   verification bullets. The design test plan enumerates ~24 test cases. Cross-check:
   - Scope: "`doctrine config show --priority` on a `doctrine.toml` without `[priority]`
     prints defaults with `# default` annotation" → Test: "show with no `[priority]` →
     all defaults annotated" ✓
   - Scope: "`doctrine config set --priority coefficients.value 99e9` clamps to
     `COEFF_MAX` and prints a note about clamping" → No explicit test for `99e9` or
     the clamp note. The tests cover "show with clamped value → `# clamped from N`"
     but set's clamp message is not explicitly tested.
   - Scope: "Existing `survey`/`next`/`explain` output uses the new config without
     restart" → No test for this — it's a behavioural claim about other commands,
     but the design only tests `config` subcommands. This is a scope-level verification
     that the design plan doesn't address.

6. **Surface consistency.** `--json` on `show` and `get` but not `set`/`unset` is an
   asymmetry. `set` returns a plain-text confirmation; should it also support structured
   output? The scope doesn't mention `--json` on `set` but the asymmetry is a design
   smell. Similarly, `--path`/`-p` on every subcommand: is there a use case for
   `show --path /other/project`? The design wires it uniformly but doesn't defend the
   choice.

The Inquisitor will now raise each charge individually on the ledger.

**MANDATVM INQVISITIONIS** — Hæc est inquisitio formalis, et quilibet repertus
hæresis in igne purgabitur. Qui designat sine doctrina, designat in tenebris.

## Synthesis

### Judgement

SL-146's design is **doctrinally sound but procedurally blemished**. The nine
decisions correctly capture the architecture — module structure (D1), CLI surface
(D2), output formats (D3-D4), write semantics (D5-D6), and code changes (D7-D9).
The silent-clamp posture matches ADR-015 and `PriorityConfig::load()`. The
edit-preserving `toml_edit` write path follows SL-136 precedent faithfully. The
path validation, clamp dispatch, and no-op guard are correctly specified.

Yet the design commits one mortal sin — **it misrepresents the code it rides.**
D7 states `load()` calls `read_priority_table` as if describing existing code,
when in fact it prescribes a refactor. A phase-plan built on this inaccuracy
would start with a phantom. This is corrected by F-1.

Three further blemishes mar an otherwise clean design:
- **F-2**: The scope's "extensible" claim contradicts the design's strict path
  validation — a design-scope delta the author must resolve.
- **F-3**: The edge-case table has asymmetrical specificity for `ref_coeff` vs
  `dep_coeff`, and the scope-required `99e9` clamp test is absent.
- **F-9**: The integration golden test is marked "optional" when it is the ONLY
  test that exercises the CLI binary end-to-end.

The remaining findings (F-4 through F-8) are clarity defects, missing
rationales, and a missing unset test — individually minor, collectively a
paper-cut that, once healed, leaves no scar.

### Penance — ordered sequence

1. **[F-1] Fix D7's terrain description.** In D7 7a: "NEW: extract shared parse
   function" — clarify that `read_priority_table` and `load_from_table` are
   EXTRACTIONS from the current inline `load()`, not descriptions of existing
   code. In D7 7c: "`load(root)` will be REFACTORED to chain them." Three
   sentences, no code change.

2. **[F-2] Resolve the extensibility contradiction.** Pick ONE:
   - **A**: Add `ConfigPath::Unknown` — relax `get`/`unset` for unknown static
     keys; `set` bails with a clear message. Update scope to note the relaxed
     get/unset. (Preferred — honest to the extensibility claim.)
   - **B**: Remove "extensible" from scope non-goals. The parser is closed;
     unknown keys require a code change. (Simpler — honest to the design.)

3. **[F-3] Add ref_coeff edge cases and 99e9 test.** Two new rows in the
   edge-case table (`set ref_coeff < 0`, `set ref_coeff > COEFF_MAX`). One new
   test: `set coefficients.value 99e9 → clamped to COEFF_MAX`.

4. **[F-4] Acknowledge inherited verification.** Add one sentence to the test
   plan noting that scope verification 11 (survey/next/explain re-reads config)
   is inherited from `PriorityConfig::load()`'s existing behaviour.

5. **[F-5] Expand the clamp-detection explanation.** In D7 7b, replace
   "(NaN/Inf naturally differ)" with a brief IEEE 754 explanation (~40 words).

6. **[F-6] Document the `--json` asymmetry.** Add rationale to D5/D6: `set`
   and `unset` are imperative verbs; exit code is the script interface; match
   `estimate set`/`tag add` precedent.

7. **[F-7] Justify `--path` uniformity.** Add a note: `-p` is the standard
   doctrine project-root override, wired uniformly across all verbs.

8. **[F-8] Add unset empty-subtable test.** Verify that removing the last key
   from a subsection leaves the empty `[subsection]` header in the TOML output.

9. **[F-9] Remove "optional" from integration test.** The golden `config show`
   test is required — it's the minimum viable integration test for a file-I/O
   CLI verb.

**Total cost**: ~15 lines of design prose, ~3 test additions, 1 design-scope
reconciliation decision (F-2). No code change to the design phase.

### Verification of penance

After the penance is applied:
- [ ] `design.md` D7 accurately separates "existing" from "NEW"
- [ ] F-2 resolution lands in both design.md AND slice-146.md (consistency)
- [ ] Edge-case table has explicit `ref_coeff` rows
- [ ] Test plan has the 99e9 set test and the empty-subtable unset test
- [ ] Integration test is marked required
- [ ] `just check` passes (no code change, but gate ritual)

### Standing risks

1. **The extensibility tension (F-2) is the only design decision still open.**
   Option A (relax get/unset) increases implementation scope slightly (one new
   `ConfigPath` variant + unknown-key read logic). Option B (remove scope claim)
   costs nothing but forecloses forward-compat. The author must choose before
   phase-plan.

2. **The D7 `load()` refactor touches the scoring engine's config reader.**
   `load()` is consumed by the priority scoring pipeline — any regression in the
   refactor silently changes every `survey`/`next` output. The extraction is
   mechanical (move code, don't change it) but the existing test suite for
   `priority/config.rs` is the safety net. All existing tests must stay green
   unchanged.

3. **Integration test dependency.** The golden `config show` test depends on the
   live `doctrine.toml` — if the TOML changes (e.g., a future slice adds
   `kind_weights`), the golden output must be updated. This is normal for
   golden tests but worth noting.

### Tolerated drift

- **`--json` asymmetry (F-6)**: `set`/`unset` do not emit JSON. This is
   consistent with `estimate set`/`value set`/`tag add` precedent; the exit
   code is the script interface.
- **No `--dry-run` flag**: The scope deferred it, the design reaffirmed it.
   Acceptable for v1.
- **No `--all` flag**: Same — deferred.

### Harvest

No durable memories to harvest — the findings are specific to SL-146's design
artifact and will be reconciled by applying the penance above. No new patterns,
footguns, or gotchas discovered beyond what SL-146's notes.md already records.

---

**SIC FERITVR SENTENTIA.** The design is absolved of heresy, but not of
penance. Apply the nine corrections, reconcile F-2, and the design shall
stand clean — fit to serve as the blueprint for a righteous phase-plan.

> **HERESIS URITOR; DOCTRINA MANET**
