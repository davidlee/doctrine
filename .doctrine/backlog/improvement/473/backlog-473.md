# IMP-473: Reasons on pruned and deferred inquiry nodes

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

An inquiry node's `pruned` / `deferred` lifecycle transition carries no reason:
`InquiryNode::transition` (`src/design_run/inquiry.rs`) takes the lifecycle only,
and the change row records `from`/`to` labels. So a reader — including the SL-266
tree view — cannot say *why* a question was made moot or put off. Add an optional
(or required) reason on those transitions, carry it on the node, and render it in
the tree's right-hand column in place of the question. Model + payload-contract
change; sequence after SL-264, which edits the same structs.
