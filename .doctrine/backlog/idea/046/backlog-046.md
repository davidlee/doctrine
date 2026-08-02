# IDE-046: Emit rendered diagrams inline via the terminal graphics protocol

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

Today a diagram is a two-step ritual the user performs by hand:

```
doctrine graph SL-243 --depth 1 | dot -T png | viu -
```

Doctrine emits DOT and stops. It could instead render to an image and write it
straight to the terminal using the kitty graphics protocol, which ghostty and
kitty both speak — one command, picture appears.

## Why it is interesting beyond convenience

It changes what the graph verbs are *for*. A DOT stream is something you pipe
somewhere and look at later; an inline image is something you glance at
mid-conversation, which is the altitude these verbs are actually used at. It
also makes DOT the more valuable emitter of the available formats, since it is
the one with a mature rasteriser behind it — which is why SL-243 chose DOT over
d2 for its own deferred rendering (IMP-385).

## Surface

Cross-cutting, not specific to one verb: `graph`, `concept-map export`, and the
anchor report when IMP-385 lands. Whatever shape this takes should be one
mechanism those verbs opt into, not three copies.

## Unknowns worth naming before it is scoped

- **Detection.** Whether the terminal speaks the protocol is a runtime question,
  and the fallback when it does not — emit DOT, say why — has to be as good as
  the current behaviour or this is a regression for everyone else.
- **The `graphviz` dependency.** Rendering needs `dot` present. Shelling out to
  it is the honest option and makes the capability conditional on a host tool;
  linking a renderer is a much larger commitment. In-jail availability is a
  known-answerable question, not an assumption.
- **Piped output.** The escape sequence must not reach a pipe or a file. Doing
  this wrong corrupts every redirected invocation.
- **POL-002.** The protocol is a terminal capability, not a host-project
  convention, so this looks clean — worth confirming rather than assuming.

## Related

- IMP-385 — the anchor report's deferred DOT rendering; the first consumer that
  would benefit.
- SPEC-027 — the graph projection and its DOT emitter.
