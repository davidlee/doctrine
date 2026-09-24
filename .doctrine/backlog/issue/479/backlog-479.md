# ISS-479: Explicit codex boot install also writes pi extensions

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observed

`src/boot.rs::install_refresh(Harness::Codex)` calls all three pi extension
installers after merging the codex hook registry. Thus an explicit codex boot
install writes `.pi/extensions/doctrine/index.ts`, `mcp.ts` and `surface.ts`
even when the operator selected only codex. `SL-263`'s `OQ-4` records that pi
currently rides the codex arm and defers first-class pi boot selection.

## Impact

`POL-003`'s opt-in supplement rule cannot be verified per harness while one
harness selection writes another harness's files. A skipped or foreign pi file
can also be reported during a codex-only install. The current routing may be a
conscious compatibility bridge, but it must be named in the install contract
rather than treated as independent pi activation.

## Resolution target

Give pi its own boot-install selection and autodetection, or define an explicit
combined selection with disclosure. An explicit codex-only selection should
leave pi files untouched. Preserve the existing per-file foreign-skip and
regenerate-on-change behavior. See `SL-263`, `POL-003`, and `REV-059`.
