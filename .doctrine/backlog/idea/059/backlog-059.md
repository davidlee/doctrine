# IDE-059: Open referenced Doctrine artifacts in the human browser or editor

## Idea

Give an agent a Doctrine CLI or MCP route to open a referenced Doctrine artifact
in the human's browser or editor, with Markdown or HTML rendered where useful.
When discussing a decision, the agent could bring the actual document or
relevant passage into view on the human's behalf instead of leaving them to
copy a path or command from chat. The human would retain control of the viewer
and whether to read further.

This supports RFC-034's goal of making governing sources convenient to inspect.
It is a separate tooling and client-integration thread from that RFC's initial
instruction and prompt trial. IDE-016 concerns efficient **agent-facing** MCP
read surfaces; this idea concerns opening material in the **human's** interface.

## Questions before scoping

- Which client surfaces can reliably open a local or remote document for the
  human, and what authorization or focus behavior do they require?
- Should the command open an entity's assembled `doctrine <kind> show` view, an
  authored file, a rendered page, or a link to a specific passage?
- How should the agent tell whether the document actually opened, and avoid
  stealing focus or opening many windows during a dense discussion?
- What is the smallest cross-harness trial that shows whether opening the
  document improves direct inspection more than printing an exact read command?
