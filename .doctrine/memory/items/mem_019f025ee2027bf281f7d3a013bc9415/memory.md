# Stale PATH doctrine binary strips new BoundaryRow fields from the registry

A pre-schema `~/.cargo/bin/doctrine` read-modify-writes
`.doctrine/state/slice/NNN/boundaries.toml` through the OLD `BoundaryRow` struct,
silently dropping a newly-added field (e.g. `provenance`, SL-154) from **every**
row — not just the one being touched. `write_registry` re-serialises the whole
in-memory registry, and the stale struct has no field to serialise.

**Why:** in the jail `which doctrine` is the readonly `~/.cargo/bin` binary, which
predates any in-flight schema change. `#[serde(default)]` makes the missing field
read back as the type default (`Provenance::Unknown`), so the strip is invisible —
no parse error, just silent downgrade. `just rebuild-stale` rebuilds `./target`
but **cannot** replace the readonly PATH binary.

**How to apply:** when a slice adds a `BoundaryRow` (or any registry-row) field,
drive ALL lifecycle ops that touch the registry — `slice phase` (the solo
binding), `record-delta` — with `./target/debug/doctrine`, never the PATH binary.
Symptom: the slice's own runtime registry ends all-`unknown` despite solo/funnel
landing. Harmless for a SOLO audit (`slice conformance` keys on the oids, not
provenance — provenance gates only the dispatch prepare-review D11 guard), but it
silently defeats provenance on a dispatched slice.

See [[mem_019edf8f57d2726281fcddd36d5197b1]] (PATH doctrine stale under shared
CARGO_TARGET_DIR) and [[mem_019ef8c35b407a738b66e1fa5eaaa0f3]] (stale test binary
embeds the old fixture corpus — the same stale-binary class).
