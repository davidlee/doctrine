# IMP-520: Harden write_atomic's mode preservation

Raised by the SL-284 audit (RV-424 F-4). SL-284 PHASE-04 hardened
`fsutil::write_atomic` to preserve an existing target's mode, closing a real
disclosure: rewriting a private `0600` settings file came back at the umask
default.

Two residual gaps in that fix, both in `src/fsutil.rs`:

1. **The temp is written before the mode is applied.** `fs::write(&tmp, bytes)`
   runs first, `preserve_mode` second, `rename` third. For the duration of the
   write the temp carries the umask default, so the same content the fix exists
   to protect is briefly world-readable in the target's directory. If
   `set_permissions` itself fails the function returns `Err` and leaves that temp
   on disk. The standard shape is the inverse: derive the mode from the target,
   create the temp with that mode, then write.
2. **`mode & 0o7777` copies the setuid, setgid and sticky bits.** For file
   content replacement, `0o777` is the permission set that matters; replicating
   special bits across a rewrite is not something an editor does and is not what
   the disclosure argument needs.

Only `0600` is tested (`write_atomic_preserves_an_existing_files_mode`).

Note the fix spans every `write_atomic` caller, not just pi settings — treat it
as leaf-module work with the behaviour-preservation gate on the existing suite.
