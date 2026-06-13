# SL-059 — implementation & closeout notes

Durable notes harvested at closeout. Phase-by-phase runtime detail lived in the
gitignored phase sheets; this is what survives them.

## Shape (as shipped)

Four `RecordKind`s (assumption/decision/question/constraint) ride four
`entity::Kind`s over the kind-blind engine — each its own tree + reservation
namespace (`ASM-001` and `DEC-001` coexist). The knowledge-specific surface in
`src/knowledge.rs`: the four kinds, per-kind status vocabularies (data, not an
enum), the typed facet enum-of-structs + shared `Evidence`, the three-layer
tolerant parse (`RawRecordToml` + kind-blind superset `RawFacet` → `validate`
dispatches on `record_kind` → typed `RecordFacet`, with the `""`/`[]`→absent
seam), and per-kind scaffold templates. Structural twin of `backlog.rs`.

Production writes go via `render_record_toml_seed` (template token-substitution)
+ `set_record_status` (toml_edit, edit-preserving). The hand-emit
`render_record_toml` subtree is **test-only** — it backs VT-1's byte-stable
round-trip proof and has no production caller.

## Code-review remediation (post-phases, this session → RV-015)

`/code-review` produced C1–C6; remediation landed before close:

- **C1/C5 (cd0e3c9)** — the test-only render subtree was masked by a module-level
  blanket `cfg_attr(not(test), expect(dead_code))`. Gated each fn `#[cfg(test)]`,
  deleted the blanket expect. Removing it unmasked four more genuinely test-only
  symbols (`default_status` + three facet-enum `KNOWN` sets) — each now
  per-symbol `#[cfg(test)]`. `opt_enum_line` collapsed into
  `opt_text_line(key, kind.map(Enum::as_str))` (byte-identical output). The
  general lesson is recorded as memory
  `mem.pattern.lint.blanket-dead-code-suppression-masks-siblings`.
- **C2 (f11eada)** — added the `set_record_status` malformed-refuse golden
  (parity with adr/standard) — strip seeded `status`/`updated`, assert exit-fail
  + exact error + file byte-untouched.
- **C3 (cd0e3c9)** — `list_rows` reproduces `listing::retain`'s status-keyed
  reveal because the kind-aware `is_hidden` can't ride retain's status-keyed
  closure. Left an in-code DRIFT comment; structural fix deferred.
- Stage-0 hygiene: removed a phantom untracked `src/lib.rs` (1-byte auto-detected
  lib target, unowned — not SL-056's; that slice's "lib unit tests" are in-module
  tests).

## Standing risk / deferred work → IDE-009

The hand-edited knowledge tier accepts **typo'd/foreign facet keys** (C4) and
**out-of-vocab record status on read** (C6) silently — the accepted cost of the
R2 tolerant read. Mitigation is a read-only `knowledge lint` verb, NOT tightening
the read. The C3 structural fix (a kind-aware `retain` closure so the per-item
hide-set is expressed once) is folded into IDE-009 too.

## Audit

RV-015 (reconciliation, → SL-059): 5 findings, all terminal/verified, no
`blocker`. NF-001 behaviour-preservation held — the only existing-test touch was
an addition (C2 golden); no assertion edited to pass. Full `--workspace` gate
green, clippy + fmt clean.
