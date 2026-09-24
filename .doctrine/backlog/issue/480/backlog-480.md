# ISS-480: Memory surface pointers omit REQ-018 rendering attributes

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observed

`src/memory.rs::format_block` renders each admitted memory pointer as a bullet
containing a stored title and UID. The title is not quoted or delimited as data.
The path surface shows trust in the bracket, but the command surface shows
severity instead and omits trust standing. Neither row carries the working
context in its attribution.

`REQ-018` requires recalled knowledge to be presented as quoted, attributed
data bearing identity, trust standing and context. Pointer titles are recalled
knowledge even though the memory body is omitted. `REQ-151` / `REQ-152` govern
suppression and holdback; passing those gates does not satisfy presentation.
`REV-058` and `REV-060` preserve this contract for the optional pointer flow.

## Resolution target

Render pointer titles as quoted, delimited data, with UID, trust standing and
working context visible on both path and command surfaces. Keep severity as a
separate triage field where useful. Verify that a title containing instruction
text remains visibly data in every output codec and harness adapter. Reconcile
the shipped behavior against `REQ-018` before claiming conformance; accepting
the revisions alone does not certify the formatter.
