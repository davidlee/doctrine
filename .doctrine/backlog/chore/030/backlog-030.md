# CHR-030: Candidate worktree lacks gitignored embed assets (web/map/dist); bin fails to compile until copied in

Surfaced by the SL-172 audit (RV-189). `dispatch candidate create --worktree`
produces a fresh worktree without gitignored build inputs. `map_server::assets`
embeds `web/map/dist/` via RustEmbed `#[folder]`; absent the folder the derive
emits no `get` → `error[E0599]: no associated function 'get' found for struct
Assets` → bin won't compile → audit suite blocked until
`cp -r web/map/dist <cand>/web/map/dist`.

Fix surface (either/or): (a) candidate-create seeds/symlinks gitignored embed
roots the bin requires; (b) the audit skill documents the copy-in as a known
pre-step for slices that don't touch web assets. See RFC-011 case-notes entry
`[audit; SL-172-RV-189-audit]`.
