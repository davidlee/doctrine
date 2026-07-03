# ISS-207: Over-broad .doctrine/dispatch/ gitignore shadows committed ledger

## Symptom

`doctrine check gate` fails on `edge`:

```
worktree::tests::every_runtime_gitignore_glob_is_classified — src/worktree/mod.rs:825
unclassified runtime gitignore glob `.doctrine/dispatch/` (rep `.doctrine/dispatch/f`)
  — add it to WITHHELD or DERIVED_RUNTIME
```

## Root cause

`61eae2ce chore: dogfood .gitignore` added a blanket `.doctrine/dispatch/` line
(`.gitignore:69`). The parity test (`every_runtime_gitignore_glob_is_classified`)
then requires every `.doctrine/`-prefixed runtime glob to be classified in
`WITHHELD` or `DERIVED_RUNTIME` (`src/worktree/allowlist.rs`) — this one is not.

## Why "just classify it as WITHHELD" is wrong

`.doctrine/dispatch/<slice>/` is NOT purely fork-withheld runtime — it holds
**committed** dispatch-ledger evidence:

- `.doctrine/dispatch/{072,079,093,095}/journal.toml` are git-tracked.
- `boundaries.toml` is committed in history (`039f365f`, `909263cc`, `32b7e1fa`).
- `src/dispatch.rs:739` names it "the **committed** claude-arm ledger".

A blanket `.doctrine/dispatch/` ignore shadows this committed evidence — new
`journal.toml`/`boundaries.toml` under the dir would silently fail to be added.
Classifying it WITHHELD would encode a false "fork-withheld runtime" claim.

## Fix direction (dispatch-domain — needs ADR-012 topology call)

Likely REMOVE or NARROW the `.gitignore` line rather than classify it. The
ledger may be committed on the coordination branch and runtime on `edge`
(ADR-012 isolated-coordination topology) — determine the intended tier, then
either drop the line or scope it (e.g. keep coordination scratch ignored but
`!`-negate the committed `journal.toml`/`boundaries.toml`), and add the matching
`WITHHELD`/`DERIVED_RUNTIME` entry only for the genuinely-runtime remainder.

## Provenance

Surfaced during the SL-192 audit (RV-238), while running the gate on `edge`
after landing the SL-192 fork. Pre-existing on `edge` (pre-merge HEAD `68cbc378`
already carried the line); unrelated to SL-192, which touches neither `.gitignore`
nor `src/worktree/`.
