# cordage NodeId/OverlayId are opaque — capture seed ids from GraphBuilder, never write NodeId(0)

`cordage::NodeId(u32)` and `OverlayId(u16)` are tuple structs with **private**
fields and no public constructor (`crates/cordage/src/lib.rs:20` — "no public
constructor and no accessor for the inner ordinal"; `tests/construction.rs:3` —
"never mint a NodeId/OverlayId directly"). So `NodeId(0)` does **not** compile from
an integration test or an example (both are external crates). The only way to obtain
an id is the value returned by `GraphBuilder::node` / `GraphBuilder::overlay`.

To seed `evaluate`/`reachable` at a specific node, the builder helper that constructs
the graph must **return** the relevant id(s). The SL-038 harness `deep_chain` returns
`(Graph, OverlayId, NodeId)` — the head node appended precisely so the evaluate seed
(`Flag(true)` at the spine head) can be expressed; existing `tests/channels.rs`
captures seed ids the same way.

**Why:** opacity is deliberate (ids are tokens; an adapter maps doctrine ids ↔ these).
**How to apply:** any test/example needing a specific node/overlay id threads it out of
the builder; the SL-038 design §6.4 red snippet's literal `NodeId(0)` is wrong and
PHASE-02 must capture the head instead. See [[mem.pattern.testing.black-box-cli-golden]].
