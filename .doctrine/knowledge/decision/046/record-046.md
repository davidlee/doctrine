# DEC-046: Activate observation capture by execution capability

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

RFC-011 dogfood activation is conditional on the caller's available capture
capability rather than expressed as one universal CLI instruction.

- Trusted agents operating in the primary tree use the observation CLI.
- Confined Claude workers use the bounded `observation_record` MCP tool.
- A worker without a brokered primary-tree capture path must not invoke the CLI
  in its fork; its orchestrator may record reported friction on its behalf.
- IMP-319 owns removal of the temporary subprocess-worker exception.

This preserves the dispatch forbidden-zone invariant: observation capture must
not introduce `.doctrine/**` into a worker delta or cause an otherwise valid
phase import or worker commit to be refused.
