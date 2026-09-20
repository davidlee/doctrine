# Design-prompt fragment body renders via `design resume`, not `design show`

Verifying that an edit to a shipped stage fragment (`install/design-prompts/*.md`)
actually reaches an agent means reading it back out of **rendered** output — a
stale embed is silent and no structural validation catches it.

**The renderer is `doctrine design resume <SLICE>`.** `fragment_section` is
called only from `run_resume` (`src/commands/design.rs`); `design show --format
prompt` emits the turn envelope — run header, totals, frontier, sections,
records, declare contract — and **no fragment body at all**. Reaching for `show`
and finding nothing reads like a missing edit rather than the wrong verb.

Two properties make this cheap:

- **`run_resume` is a pure read.** It resolves the root, reads the snapshot,
  projects, and emits; it writes nothing. So you may render against *another*
  slice's live run without disturbing it — confirm with an md5 of
  `.doctrine/state/slice/<n>/*.toml` either side if you want the belt.
- **A locked run emits no fragment.** `Fragment::for_stage(Stage::Locked)` is
  `None`, by design. So a slice cannot render its own reviewing turn after its
  lock — find any slice whose run is at the stage you need
  (`grep '^stage' .doctrine/state/slice/*/design.toml`).

The fragment body is elided when the caller declares a current `name@digest`
receipt via `--known-fragment`; omit the flag and the body always rides.

For a published asset that is *not* a stage fragment, `doctrine library show
<address>` reads the same embed — e.g. `reference/review-ledger.md`. That is the
render check for reference docs; `design resume` is the one for fragments.
