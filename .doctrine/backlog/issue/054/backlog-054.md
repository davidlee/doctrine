# ISS-054: main red: knowledge.rs trips NF-001 facet-symbol allowlist tripwire

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced during the SL-162 audit (RV-171 F-2). The integration suite on `main`
fails exactly one test:

```
tests/e2e_estimate_non_blocking.rs :: no_facet_symbol_outside_allowlist
NF-001 structural non-blocking tripwire FAILED — facet symbol(s) found outside allowlist:
  knowledge.rs
```

`src/knowledge.rs` names one of the guarded facet symbols (`EstimateFacet`,
`ValueFacet`, `EstimationConfig`, `ValueConfig`, `resolve_confidence`,
`crate::estimate`, `crate::value`, `estimate::`, `value::`) but is **not** in the
tripwire's ALLOWLIST. NF-001 exists to flag any new gating path that reads
estimate/value — so either knowledge.rs genuinely grew a new estimate/value
exposure that must be reviewed, or the symbol is incidental and the ALLOWLIST
needs updating.

**Provenance.** Unrelated to SL-162 (a test-path-resolution sweep that touches
neither `knowledge.rs` nor the tripwire test). Inherited red on `main` — the
SL-162 candidate is `main` + a test-only sweep, so the tripwire inputs are
byte-identical to `main`. Likely landed with the SL-158 impl bundle (last commit
touching either file: `40854b80 review(158): impl bundle`).

**Action.** Determine whether knowledge.rs's facet-symbol use is a real new
estimate/value exposure (review it) or incidental (add to the ALLOWLIST in
`tests/e2e_estimate_non_blocking.rs`). Either way, get `main` green.
