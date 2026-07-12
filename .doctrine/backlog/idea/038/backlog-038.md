# IDE-038: MCP tools for compare elicit and record

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Intent

Expose the elicitation loop as MCP tools beside the existing review_*/memory_*
set:

- `compare_elicit` — returns the schema-v1 JSON envelope directly in the MCP
  content block (no shell, no JSON-through-stdout parse).
- `compare_record` — capture leg; structured args kill shell-quoting
  fragility (same failure class as the RV-270 case-note: CLI multi-raise
  apostrophe mangling; MCP review_raise fixed it).

Payoff: /elicit curation skill (IMP-284) drives the loop tool-native;
lower token overhead, no quoting hazards, typed answers. Engine already pure
+ read-only (D18) — thin MCP wrapper only.

Consider after IMP-284 proves the interaction shape.
