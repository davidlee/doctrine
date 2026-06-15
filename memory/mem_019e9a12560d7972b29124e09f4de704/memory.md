# Doctrine memory model

Doctrine memory has two faces. Don't confuse them.

1. **Locally-captured memory** — scoped, anchored items you record in *your* repo
   as you work, stored under `.doctrine/memory/items/nnn/` (`memory.toml` +
   `memory.md`, with a `mem.<key>` symlink alias). These carry scope
   (paths/globs/commands), an optional git anchor, type, trust, and review state.
   This is where durable findings from your own sessions go.
2. **This shipped orientation corpus** — repo-empty, unanchored, evergreen
   masters that ship *with* doctrine to orient an agent driving it. They have no
   git anchor (`anchor_kind = none`) and an empty `repo`, so they surface in any
   repo. The concept/fact/pattern/signpost masters you are reading now are these.

Two habits the model exists to support:

- **Retrieve before you assume.** Before touching an unfamiliar subsystem,
  changing a command pipeline, or answering "what's the right way here?", query
  memory first. Use `/retrieve-memory` (wraps `doctrine memory find` /
  `doctrine memory retrieve`) — scope-aware, ranked, with a trust holdback on
  `retrieve`. Don't rediscover what's already recorded.
- **Capture at wrap-up.** When you confirm a durable fact, constraint, footgun,
  or reusable workflow, record it before it's lost to context — `/record-memory`
  (wraps `doctrine memory record`), which captures scope + git anchor.

Point of truth: `doc/memory-spec.md` (the umbrella — entity shape, capture +
provenance, scope-aware retrieval, reserved seams) and the `doctrine memory
--help` surface. See [[concept.doctrine.routing-gate]] for when retrieval is part
of the gate, [[pattern.doctrine.conventions]] for the "durable knowledge
lives in doctrine's memory, not the model's head" rule, and
[[signpost.doctrine.recording-memories]] for the capture-retrieve cycle.
