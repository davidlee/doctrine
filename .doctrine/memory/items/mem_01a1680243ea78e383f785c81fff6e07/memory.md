`boot::apply_wire(root, exec, harnesses, dry_run=true)` cannot tell you what it
*would* write: imports are collected only when `!dry_run` (`src/boot.rs:3448`)
and per-harness touches are emptied under dry run (`src/boot.rs:3683`). The test
`apply_wire_touches_nothing_when_current_or_dry_run` (`src/boot.rs:8013-8016`)
pins `dry.touched()` empty.

To predict wiring's write set (e.g. a tracked-file guard), compose the existing
dry-run halves — `ensure_boot_import(.., true)`, `install_refresh(h, root, exec,
true)`, `refresh_touched` — rather than changing `apply_wire`'s contract.
Found by SL-285 design review (RV-425 F-1).
