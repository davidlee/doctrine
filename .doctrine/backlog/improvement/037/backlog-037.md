# IMP-037: spec.rs has no read_spec reader: relation_edges + show + list + scan each re-parse Spec toml inline (4 copies) — extract one reader

Surfaced by the SL-046 `/code-review` (post-RV-006).

`spec.rs` is the only edge-authoring kind with **no single `read_spec` reader**.
Four sites re-parse the spec toml inline with `read_to_string` + `toml::from_str::<Spec>`:

- `relation_edges` (SL-046, ~511-515)
- `format_show` path (~874-876)
- `list` scan (~967-973)
- (the lineage Options read)

SL-046 §5.2 design intent: *"parsing stays put — cohesion; the adapter never
re-parses TOML."* The other five kinds honour this via their `read_*` reader;
`spec::relation_edges` could not, because the reader does not exist — so it forked a
fourth inline parse, and its own doc comment wrongly claims *"no new TOML parse."*
(The comment was corrected in the SL-046 cleanup commit; the structural duplication
is this item.)

**Do:** extract `read_spec(subtype, root, id) -> Result<(Spec, body?)>`, route all
four sites through it. Behaviour-preserving — the existing spec show/list goldens are
the proof (behaviour-preservation gate).

Relates: SL-046 (`[[slices]]`), and the `read_*`-per-kind cohesion pattern the other
modules already follow.
