# DEC-050: Bound observation content without silent truncation

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

SL-231 will keep friction capture permissive while enforcing generous,
deterministic UTF-8 byte limits:

- summary: 1 KiB;
- detail: 32 KiB;
- each facet string value: 512 bytes; and
- the complete serialized observation record: 64 KiB.

The writer refuses an over-limit request with a field-specific diagnostic. It
does not silently truncate, partially store, or reinterpret the content. The
writer rejects NUL, but otherwise permits ordinary Unicode, newlines, tabs, and
content that must be escaped at later boundaries.

Storage safety, presentation safety, and instruction trust are separate
contracts:

- TOML and JSON are emitted through structured serializers rather than string
  interpolation;
- terminal rendering escapes control characters and escape sequences so stored
  text cannot affect the terminal; and
- any observation content supplied to an agent is explicitly framed as
  untrusted data, never concatenated into trusted instructions.

The same validation applies to CLI, MCP, and registered machine-source
adapters. Limits are checked before the atomic publication primitive is
invoked, so a rejected request creates neither an authoritative record nor a
temporary publication file.
