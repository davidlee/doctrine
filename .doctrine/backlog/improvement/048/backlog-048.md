# IMP-048: Wire the relation link/unlink CLI verb so structural relations need not be hand-authored

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

SL-048 PHASE-02 built the `link`/`unlink` writer over generic
`append_edge`/`remove_edge`, gated by `RELATION_RULES` `LinkPolicy`
(`src/relation.rs`) — but it is **not surfaced as a CLI verb**: `doctrine link`
is an unrecognized subcommand. Authoring a structural `[[relation]]` row today
means hand-editing the entity TOML (done for SL-057). That is error-prone
(label/target spelling, legality, dangling) and defeats the write-strict guarantee
the writer was built to enforce.

Wire the writer to the CLI (`doctrine link <SRC> <label> <TARGET>` / `unlink`,
or per-kind), refusing dangling/illegal-kind edges at write time (write-strict;
read-tolerant `validate` stays the safety net). Governed by the relation contract
SPEC-018 (draft) / ADR-010, ADR-004. Surfaced while scoping SL-057.
