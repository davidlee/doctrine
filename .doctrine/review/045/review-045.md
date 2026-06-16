# Review RV-045 — design of SL-079

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This Inquisition holds the design of SL-079 ("Finish the CLI colour story") to
the sanctioned doctrine — the governing ADRs, the existing pure/impure boundary,
the column-model invariants of SL-053, and its own declared decisions D1–D7.

The design claims three items: IMP-038 (column model debug_assert!), IMP-039
(priority colour + status-line point-colour), IMP-040 (--color flag). The
Inquisition presses each on ten lines of interrogation:

1. Does IMP-038's debug_assert! actually catch the defect class it claims to, or
   create a false sense of security from a debug-only check?
2. Are SURVEY_COLS/NEXT_COLS arrays correct — do name/header/cell/paint match
   the existing hand-built grid EXACTLY under color: false?
3. Is status_colored's location in listing.rs coherent with the pure/impure
   boundary, or does it smuggle impurity?
4. Does the --color flag wiring cover every surface that emits colour? Any
   colour-emitting surfaces missed?
5. Does the priority golden byte-identity claim hold under close scrutiny?
6. Is Actionability::token() "actionable" correctly mapped in status_hue?
7. Is clap::ColorChoice truly zero-cost (already in the dep tree)?
8. Does resolve_color precedence match NO_COLOR convention and CLI expectations?
9. Are there writeln! status-bearing surfaces beyond the five named that were
   overlooked?
10. Does the debug_assert! placement in select_columns match current code shape?

## Synthesis — Inquisitorial Verdict

The accused design of SL-079 stands, but not without blemish. Five heresies were
dragged into the light; none are mortal, but four stain the design artifact and
one is a tolerated blemish of the flesh.

The CORE DESIGN IS SOUND. The SURVEY_COLS/NEXT_COLS column arrays faithfully
reproduce the hand-built grid under color:false — every cell closure, every
header, every paint mapping checked against the current code. The status_hue map
covers every token the five targeted writeln! surfaces emit (verified by full
vocabulary cross-reference: ADR, Policy, Standard, Knowledge × 4 kinds,
Revision). The pure/impure boundary is respected — status_colored is pure
(both inputs injected), resolve_color is the sole impure addition in tty.rs. The
--color flag precedence (Never > Always > Auto) is correct per NO_COLOR
convention and clap ColorChoice semantics. No new dependency — clap::ColorChoice
arrives with the existing clap 4.6.1 dep.

FOUR DESIGN-LEVEL CORRECTIONS MANDATED before /plan (F-1, F-2, F-3, F-5):

1. **F-1 (design-wrong):** Add a sentence acknowledging revision run_approve's
   approval-status line exists and was consciously excluded from colour scope
   (distinct status axis — approval, not lifecycle).

2. **F-2 (design-wrong):** Clarify that revision run_status requires TWO
   status_colored calls (one for "from", one for "to"), joined by literal
   " → " (uncoloured).

3. **F-3 (fix-now):** Specify the into_list_args seam: add a `color: bool`
   parameter to `CommonListArgs::into_list_args(self, color: bool)`. Callers
   resolve via `tty::resolve_color(cli.color)` and pass it. The method reads
   `color` from the parameter instead of calling `stdout_color_enabled()`.

4. **F-5 (design-wrong):** Preserve the existing clean injection pattern.
   Keep `run_survey(..., render: RenderOpts)` and `run_next(..., render:
   RenderOpts)` signatures UNCHANGED. Resolve `cli.color` in main.rs,
   construct `RenderOpts` there, pass it through. Do not push clap or tty
   into src/priority/mod.rs.

ONE TOLERATED BLEMISH (F-4): The temporary Vec allocation from
`&cols.iter().collect::<Vec<_>>()` is harmless (7-column stack allocation).
Capture under IMP-044 (RenderOpts migration) for a future seam-uniformity pass.

THE TEN LINES OF INTERROGATION ANSWERED:

- **Lines 1, 10** (debug_assert!): The assertion fires before match requested,
  catches invalid defaults in debug, and the existing pick() error is the
  release backstop. Not a false security — it formalizes the existing
  "curated-valid" comment as an enforceable invariant.
- **Lines 2, 5** (column correctness, golden byte-identity): Verified — cell
  closures are character-by-character identical to the hand-built grid.
  paint_header and paint_cell return raw strings under color:false (the
  `!color` early return). render_table receives the identical grid. Goldens
  stay byte-identical.
- **Line 3** (status_colored purity): Location in listing.rs is correct.
  Both inputs (status: &str, color: bool) are injected — no impurity.
  The hue map (status_hue) is its natural dependency, already in listing.rs.
- **Line 4** (--color coverage): ~13 call sites: CommonListArgs::into_list_args
  (serving all list commands) + 4 priority commands + 5 status-bearing
  writeln! surfaces. All identified and correctly wired. F-1 flags the one
  additional status-bearing line (run_approve) as a conscious exclusion.
- **Line 6** (Actionability::token): "actionable" returns None from status_hue
  → badge cell text is "" → nothing visible to colour. Correct.
- **Line 7** (clap::ColorChoice): clap 4.6.1 — present, no new dep.
- **Line 8** (resolve_color precedence): Never > Always > NO_COLOR > isatty.
  Always overrides NO_COLOR (explicit user override per convention). Correct.
- **Line 9** (overlooked surfaces): One found: run_approve (F-1). All other
  writeln! sites checked — creation confirmations, dispatch, worktree boot —
  are not status-bearing or are explicitly out of scope.

JUDGEMENT: NOT GUILTY OF FUNDAMENTAL HERESY.

The design is coherent, the invariants hold, and the four mandated corrections
are surface amendments to the design artifact — not structural redesigns. Apply
them, then advance to /plan with the Inquisition's blessing. The ledger is
closed; let the design be amended and the implementation proceed.

> **HERESIS URITOR; DOCTRINA MANET**
