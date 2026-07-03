# IMP-242: Reverse dead-hymn coverage lint

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Split off from **SL-191** design (external adversarial review C4, codex/GPT-5.5).

## What

The **corpus→def** direction of the required-trait coverage lint: warn when an
embedded `model/**` hymn is matchable by **no** embedded worker def's declared
`traits:` — i.e. shipped worker-contract prose that reaches no worker (dead hymn).

Complements SL-191's locked **def→corpus** hard error (D4: every trait a def
declares must match ≥1 hymn). C4 is the mirror: every hymn must be matchable by
≥1 def.

## Why deferred (not in SL-191)

- **No live trigger in SL-191** — its only model hymn (`model/adherence/low.md`)
  is wired to the pi def; nothing is dead.
- The only class C4 catches that def→corpus does **not** is "a hymn targeting a
  trait no def declares." That splits into (a) intentional extension-point
  (`capability/*`, deliberately deferred by SL-191 D3 — a false positive needing
  an annotation escape) and (b) forgotten def-wiring (the genuine catch). Neither
  is exercised until a second trait root or a capability hymn actually lands.
- The annotation mechanism to silence (a) is pure future-proofing for roots that
  don't exist yet.
- SL-191 already covers the human/author-layer drift C4 worries about via the
  full-context `prompt check` (declared→delivered), the hymn README rewrite, and
  the shipped hymns authoring memory.

## Trigger

Implement when the trait corpus grows enough for orphan hymns to be a real risk —
concretely, when a **second** trait root (`capability/*`, …) is populated, or a
model hymn is authored ahead of any declaring def.

## Shape (when built)

- Reuse SL-191's shared `traits_covered`-style engine predicate, run in reverse
  over the corpus's `model/**` selectors against the union of embedded defs' trait
  sets.
- Severity **warning**, not error (a dead hymn is governance-wasteful, not
  runtime-harmful).
- An explicit "extension-point" annotation (sidecar or frontmatter) suppresses the
  warning for intentionally-ahead-of-consumer roots.

Home surfaces: `src/hymns.rs` (predicate), `src/commands/prompt.rs` (`prompt check`).
Relates to SPEC-023, SL-191, SL-192.
