# DEC-049: Bound automatic observation enrichment to adapter-known context

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

SL-231 will automatically enrich an observation only from bounded facts the
active capture adapter already knows. The v1 allowlist is:

- the CLI adapter's constants for interface, product surface, and command
  (`cli`, Doctrine CLI, `observation record`);
- the MCP adapter's constants for interface, product surface, and tool
  (`mcp`, Doctrine MCP, `observation_record`);
- primary-tree versus worker context from the already-established
  worker-marker/server destination-resolution seam; and
- an opaque agent identifier only when it is supplied through the existing
  capture context.

These sources and their field mappings are named once in code. There is no
environment enumeration, prompt or repository-content inspection, arbitrary
process metadata capture, or inference from incidental strings. Explicit
caller values take precedence over automatic values field by field.

Harness, model, role, dispatch arm, lifecycle stage, skill, and run/session
correlation remain absent unless the caller supplies them explicitly or a
trusted adapter supplies them from its own established context.

Reliable harness detection from named environment markers is a useful
follow-up rather than part of SL-231. IDE-005 already owns that work and should
also consider observation enrichment as a consumer. Any later environment
source must be individually named and validated; it does not relax this
decision into general environment capture.
