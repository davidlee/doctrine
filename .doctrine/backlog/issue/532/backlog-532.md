# ISS-532: boot_harnesses silently drops boot wiring when harness resolution fails

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`boot_harnesses` (src/install.rs ~754) calls
`crate::boot::resolve_harnesses(&names, root).unwrap_or_default()`. If a
registry `boot` value fails to resolve, the boot-wiring leg wires nothing and
says nothing — a silent skip (STD-003). Pre-existing helper shared by
`install` and `install::regenerate`; since SL-285 every provisioned fork
depends on it (DEC-393).

Fix: propagate the error (or disclose it as a leg failure/refusal) and pin
it with a test.

Origin: RV-426 F-7 (SL-285 audit).
