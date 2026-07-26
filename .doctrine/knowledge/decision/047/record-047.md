# DEC-047: Create observation records through a shared atomic no-clobber primitive

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

SL-231 will publish observation records through a shared `fsutil` primitive
that combines complete-content publication with create-new/no-clobber
semantics.

The primitive:

1. creates or validates every parent component with the squat-rejecting
   component walk extracted from the authored-entity machinery;
2. writes the complete record to a uniquely named sibling temporary file;
3. creates the authoritative destination with `std::fs::hard_link`, which
   either publishes the already-complete inode or refuses because the
   destination exists; and
4. removes the temporary name after successful publication.

An existing destination is resolved by the observation store as replay or
identity collision; the filesystem primitive never overwrites it. A crash
before publication may leave only a reserved temporary file. A crash after
publication may leave both names for the same complete inode. Observation
loading ignores the reserved temporary-name pattern, and stale temporary names
may be removed operationally without affecting a published record.

This gives the supported macOS and Linux targets one implementation through
Rust's stable standard-library hard-link API. It is preferred to separate
platform-specific exclusive-rename implementations (`renameat2` with
`RENAME_NOREPLACE` on Linux and `renamex_np` with `RENAME_EXCL` on macOS).

The guarantee is deliberately bounded: an interrupted writer cannot expose a
partial authoritative record, concurrent writers cannot clobber an existing
destination, and symlink or non-directory parent squatters encountered during
the component walk are refused. It is not a defence against a malicious local
actor continuously replacing directory components during the operation.
