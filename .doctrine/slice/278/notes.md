# Notes SL-278: Review read surface

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design review passes

- 2026-10-04 · RV-416 (adversarial, GPT) concluded: F-1..F-6 verified, all
  integrated. No further pass recommended before lock. A second pass would
  probe: (a) the census axis split — that `FindingFilter` and the text-only
  `listing::retain` together reproduce the list hide-set exactly; (b)
  `ReviewOutput::Showed { #[serde(flatten)] view }` producing byte-identical
  inner JSON to the CLI's `{"review": view}`. Both are cheaper to settle by the
  sec-8 tests (`census_and_show_agree_on_finding_axes`,
  `cli_json_and_mcp_show_carry_identical_view`) during execution than by
  another reading.

## Execution decisions (lifted from phase sheets at audit)

- PHASE-02 split 2a (table) / 2b (JSON) — the only cut that compiles between.
  2b dropped the false `id`/`body`/`view=summary` sentences from the
  `review_show` description early so the interim commit told the truth.
- PHASE-03: retired MCP `view`/`format` args are ignored, not refused (design
  sec-4; e2e pins an old caller still gets the index). Caller-input refusals
  carry the `invalid arguments:` prefix → -32602. Unknown finding id stays
  plain `anyhow` → -32603 (ISS-518); wrapping `run_show` would break the
  `NotFound` downcast.
- PHASE-04 split 04a (list on `scan_reviews`, `--facet`, vocab public) / 04b
  (census), list first so the census only adds code onto list's target/facet
  predicate and unreadable plumbing.
- 04a: `Listing` is a named struct, not a tuple; `with_warnings` generalised
  to `with_array<T>`; `cap_tail<T>` is the shared row cap (unreadable rides
  uncapped). The MCP description splices `FINDING_STATUSES`/`SEVERITIES`
  between plain `&str` segments rather than escaping braces in a `format!`
  literal; a test asserts every status and severity appears.
- 04b: census substring filter (`retain`) matches finding id + title, not the
  review slug; help says so. Census severity cells go through the shared
  `severity_cell` so capitals agree with `show`.

## Token budget (PHASE-05 VA-1)

- 2026-10-05 · `review show RV-012` (five findings, default depth), bytes/4
  estimate (no tokenizer in the jail): table ≈294 tokens (1175 B), CLI JSON
  ≈497 (1989 B pretty; 1467 B compact ≈367). RFC-032 D5 budget ~200: over.
- Finding titles are about half the table; not cuttable without losing
  content. Remaining levers are small (default role-label line, `findings: N`
  line, zero-filled `outstanding` maps in JSON) or CLI-wide (pretty-printed
  `--json` whitespace, ≈26%) and out of this slice's scope.
- **Accepted by the user** (2026-10-05: "accept it, unless there's an obvious
  way to reduce the token count without omitting important information") —
  no obvious lossless reduction found.

## Close-time actions

- After SL-278 is pushed: `doctrine install -s plan -s reconcile`, then
  `grep -n 'review.finding\b\|does not render the route' .agents/skills/plan/SKILL.md`
  must be empty (RV-417 F-2). Install sources skills from the published repo.
- `doctrine check gate` on the landing tree; expect only the known git-guard
  set in the jail (RV-417 Synthesis).

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open
