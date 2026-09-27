In a `design apply`, a finding whose concern names a section is validated
against the **prior** map, not the batch's own declarations. Declaring the
section and a finding about it in one request is refused. Fixtures (e2e and
unit) must seed the section in an earlier apply, then raise the finding.

Related test-authoring trap from the same slice: a JSON body containing
`"## …"` inside a Rust raw string needs `r###"…"###` — both `"#` and `"##`
close the shorter delimiters.
