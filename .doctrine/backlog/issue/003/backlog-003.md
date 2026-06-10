# ISS-003: cordage explain() foreign node id returns singleton chains, rustdoc says empty paths

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Found in the SL-036 post-close code review (codex GPT-5.5 + Opus, independent
agreement).

**Drift.** `Graph::explain` rustdoc (`lib.rs:847`) documents that a foreign node
id yields *empty paths*. The implementation (`lib.rs:846/853`) calls
`predecessor_paths` unconditionally; `chains_to_root` (`query.rs:109`) keys off
`incoming.keys()` and hits its root-termination branch immediately for an unknown
node — so a foreign id actually returns a `paths` map with one entry **per
overlay**, each containing a `[[foreign_node]]` singleton chain, plus a
`Finite(0)` key.

**Severity:** low. Non-panicking; doc/behaviour mismatch, not a crash or a
wrong-answer for in-graph nodes.

**Fix direction:** guard `node.0 >= self.node_count` in `Graph::explain` (return
genuinely empty), or teach `predecessor_paths` to distinguish "root in graph"
from "foreign node". Either way reconcile the rustdoc with the chosen behaviour.
