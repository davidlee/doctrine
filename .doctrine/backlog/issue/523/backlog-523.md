# ISS-523: install --only-memory is parsed but unwired

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

`InstallArgs.only_memory` carries `#[expect(dead_code, reason = "wired in
PHASE-03 (--only-memory subset derivation)")]` (`src/install.rs:189-201`);
nothing reads it. `doctrine install --only-memory` runs the full install.

## Fix direction

Either wire the subset (define which legs it runs) or remove the flag.

Surfaced by RV-421 F-13 (SL-282 design review); SL-282 treats the flag as a
normal install and does not re-scope it.
