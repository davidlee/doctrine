# RSK-227 coupling map — regenerate

Supporting artifacts for RSK-227. The RSK body is the durable record; these
rebuild the picture from source.

## Files

| file | role |
|---|---|
| `edges.txt` | raw module→module dependency dump (input, captured) |
| `gen.py` | parses edges + `layering.toml` tiers → command-tier same-tier subgraph, Tarjan SCC, emits DOT |
| `cmd_tangle.dot` | generated DOT (nodes sized by degree, coloured by fan-out; solid=in-SCC, dashed=acyclic) |
| `cmd_neato.svg` | rendered map (neato force layout) |
| `coupling-map.html` | self-contained diagnostic page (inlines the SVG) — source of the published artifact |

## Refresh the input

`edges.txt` is the printed edge list of the `#[ignore]` `dump_real_graph`
diagnostic in `tests/architecture_layering.rs`:

```sh
cargo test --test architecture_layering dump_real_graph -- --ignored --nocapture \
  | grep -E '^\s+[A-Za-z_]+ -> [A-Za-z_]+$' > edges.txt
```

## Rebuild

```sh
python3 gen.py                                   # → cmd_tangle.dot + summary
neato -Goverlap=prism -Gsplines=true -Gbgcolor=transparent \
      -Tsvg cmd_tangle.dot -o cmd_neato.svg      # → cmd_neato.svg
```

To refresh `coupling-map.html`, splice the new SVG between the `<div class="graph">`
markers (strip the SVG's xml/doctype preamble first).

## Caveat

Tier assignment comes from `.doctrine/adr/001/layering.toml` (the gate's
authoritative source). The diagnostic's edge set yields ~114 in-SCC edges vs the
gate's authoritative baseline of **123** — same order, not a re-measure of the
gate itself. Out-degree in the summary spans all targets; a true concentration
metric needs same-tier filtering (already applied in `gen.py`'s subgraph).
