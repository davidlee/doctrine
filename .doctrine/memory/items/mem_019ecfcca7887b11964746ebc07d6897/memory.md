# Concept Map diagnostic serialisation shape for JS consumption

Rust `#[derive(Serialize)]` on `ConceptMapDiagnostic` produces tagged-enum JSON
objects in the GET response — each diagnostic is an object with a single key
(the variant name) and a nested object of fields:

```json
{"CanonicalNodeCollision": {"key":"foo","first_label":"FooBar","first_line":1,"label":"Foo","line":3}}
{"SelfEdge": {"line":2,"node_key":"alpha"}}
{"SimilarNodeLabel": {"label_a":"Foo","line_a":2,"label_b":"Fooo","line_b":4}}
{"RelationDrift": {"rel_a":"depends","line_a":2,"rel_b":"dependes","line_b":5}}
{"EntityRefLike": {"label":"SL-001","line":3}}
{"MalformedLine": {"line":3,"text":"bad line"}}
{"EmptyLabel": {"line":4,"segment":"Source"}}
{"DuplicateEdge": {"line":5,"existing_line":2,"from_key":"a","rel":"relates","to_key":"b"}}
```

JS dispatch: `Object.keys(d)[0]` extracts the variant name, `d[variant]` accesses
the nested field object. Used in `formatDiagnostic()` in `web/map/app.js`.

Line extraction: most variants have `line`, but `SimilarNodeLabel`/`RelationDrift`
use `line_a`/`line_b` — the `diagnosticLine()` helper extracts the first
available line number for prefix display.
