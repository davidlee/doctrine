# IMP-124: dispatch: [dispatch] deliver_to config as single source of trunk delivery ref

Fulfils the standing TODO in `plugins/doctrine/skills/close/SKILL.md` step-3a:
*"Once project config (`doctrine.toml [dispatch] deliver_to`) lands…"*. Today the
trunk delivery ref is hardcoded as `refs/heads/main` in close prose
(`--trunk refs/heads/main`), and SL-126's close-integration gate reads it
*self-describing from the journal trunk row* (design OQ-1 option (b)) to avoid a
new config surface. This item lands the real single source of truth: a
`[dispatch] deliver_to` key in `doctrine.toml`, defaulting to `refs/heads/main`,
consumed by both close (the `--trunk` arg) and the SL-126 gate (row selection +
tip resolution), retiring the namespace-elimination heuristic.

After: SL-126 (the gate ships first against the self-describing read).

