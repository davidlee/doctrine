# toml error classification is version-fragile — pin shapes with canary tests

The `toml` crate (0.8.x) exposes **no stable error-kind enum** — to classify a
parse failure (e.g. "is this a duplicate `parent` key?") you must match on the
error's **span enclosing-line key** AND its **message text**. Both are
version-fragile: a toml bump can shift message wording or span attribution.

Observed shapes pinned in SL-022 (`is_second_parent`, src/spec.rs):
- duplicate key → ``duplicate key `parent` in document root``
- array value → `invalid type: sequence, expected a string` (note: the array
  message does **not** name the key — span attribution is REQUIRED; message
  alone false-hits `slug = []`).

Mitigations (the durable pattern):
- **Span attribution, not message-only.** Confirm the offending source line's
  key before trusting the message. Parser already ignores comments, so a
  scaffold `# parent = …` can never be the span.
- **Canary tests** that assert the exact observed message/shape — they fail loud
  on a toml bump rather than silently misclassifying.
- **Degrade to non-zero, never silent.** On a classifier miss, fall through to a
  generic `Failed to parse` (non-zero exit) — never a silent pass.

This is irreducible fragility (no stable taxonomy upstream), a conscious tradeoff
made for a named diagnostic. Logged as SL-022 audit F-3 / notes R2 (tolerated
drift). Related: [[mem.pattern.render.toml-splice-escape-user-values]].
