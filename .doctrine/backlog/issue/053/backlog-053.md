# ISS-053: Stale PATH doctrine binary silently drops new BoundaryRow fields from conformance registry

Captured from [[mem.pattern.jail.stale-binary-strips-registry-field]].

A pre-schema `~/.cargo/bin/doctrine` read-modify-writes `boundaries.toml`
through the old struct, silently dropping newly-added `BoundaryRow` fields
(e.g. `provenance`) from every row. No schema-version guard exists.

Needs investigation into a schema-version sentinel checked at boundary-start;
whether the best path is a binary-build-date check, a TOML field sentinel, or
something else is unverified.
