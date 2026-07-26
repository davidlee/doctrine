# IMP-320: Opt-in agent friction capture guidance from doctrine.toml

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

Once the SL-231 friction observation path exists, projects should be able to opt
agents into using it without permanently baking the instruction into universal
governance. A project-local `doctrine.toml` setting can express that policy and
boot prompt composition can include a focused instruction fragment only when it
is enabled.

## Desired outcome

- Define a stable, default-off `doctrine.toml` option meaning “agents should
  record friction observations.”
- Add an owned prompt fragment that tells agents when and how to invoke the
  observation capture interface.
- Include the fragment in generated boot context only when the option is true
  and the installed binary supports the required observation interface.
- Keep the fragment concise and avoid duplicating the observation schema or
  broader governance prose.
- Verify enabled, disabled, absent, and unsupported-version behavior.

## Boundaries

This is opt-in agent guidance and boot composition, not the observation ledger,
capture implementation, reporting, or automated friction detection. It follows
SL-231 and should consume its final public command/tool contract rather than
anticipating it.
