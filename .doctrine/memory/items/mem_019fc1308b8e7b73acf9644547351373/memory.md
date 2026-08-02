# Read anchors through the JSON contract; a raw grep over the TOML lies twice

`doctrine spec show <ID> --json` emits the anchors and the whole spine under the
`spec` key — no need to touch `.doctrine/` files at all:

```
spec.source[]      → { language, identifier, module? }
spec.parent, spec.descends_from, spec.c4_level, spec.product_level
members[]          → the REQ peers with their FR-/NF- labels and order
```

`doctrine spec list --json` gives the corpus to iterate. Together they are the
supported read seam for any anchor census — and they honour the read-via-`show`
guardrail mechanically rather than by discipline.

## Why the obvious alternative is wrong

`rg 'identifier = ' .doctrine/spec/tech/*/spec-*.toml` is the natural reach, and
it is wrong in two independent ways that both inflate the count:

1. **Slug symlinks double every anchor.** `.doctrine/spec/tech/002-requirement-
   reconciliation-engine` is a symlink to `002/`, so a glob walks both and each
   `[[source]]` block appears twice. Anchor totals come out exactly 2x.
2. **The scaffold template's commented example reads as a real anchor.** Every
   `spec-NNN.toml` carries a commented block ending
   `#   identifier = "doctrine/cli"`. A grep that does not strip comments reports
   `doctrine/cli` as the most-anchored identifier in the corpus. It is not an
   anchor; it is not even a path.

Both were hit live during the IMP-381 census (2026-08-02) — the second produced
a phantom top-of-table entry that survived into a report before being caught.

## Measured corpus figures (2026-08-02, via the JSON contract)

48 specs, **81 real anchors** — rust 71, markdown 5, toml 2, typescript 1,
json 1, directory 1. **Zero non-resolving**, consistent with the CHR-046 repair.

## Related

[[mem.pattern.spec.source-anchor-liveness-unchecked]] — anchors are never
verified, so a live anchor still needs a `test -e` before it counts as coverage.
That check is cheap and correct over the JSON contract; over a grep it is
checking phantoms.

SL-243 builds the mechanical version of this read.
