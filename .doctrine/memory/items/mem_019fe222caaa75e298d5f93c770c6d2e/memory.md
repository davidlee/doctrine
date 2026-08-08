`doctrine knowledge edit <kind>`'s list-valued facet flags — `--alternatives`
and `--consequences` — are **comma-separated**. Their help says so
("comma-separated; bare flag clears"), and it is easy to read that as *"you may
pass several"* rather than *"your prose will be split"*.

Write a sentence containing a comma and it is stored as **two list elements**.

```bash
doctrine knowledge edit decision DEC-168 \
  --alternatives "Rejected on coupling — see rationale. (Withdrawn: SL-249 D8a, REV-050.)"
```

stores

```toml
alternatives = [
  "Rejected on coupling — see rationale. (Withdrawn: SL-249 D8a",
  " REV-050.)",
]
```

## Why it bites

The failure is **silent and its symptom is misleading**. `knowledge inspect`
renders the list joined with `", "`, so the round-trip looks almost right — the
only visible artefact is a **doubled space** where the leading space of the
second element meets the join:

```
alternatives: … SL-249 D8a,  REV-050.)
```

That reads as a typo, not as a structural error. Nothing refuses, nothing warns,
and the record is now malformed in a way that survives review.

## How to apply

- **Don't put commas in prose passed to a list-valued facet flag.** Use ` — `,
  ` / `, or a semicolon.
- **Verify the shape, not the render**, when it matters — parse the TOML rather
  than reading `inspect`:
  ```bash
  python3 -c "import tomllib;print(tomllib.load(open('<record>.toml','rb'))['facet']['alternatives'])"
  ```
  A doubled space in `inspect` output is the tell that this already happened.
- Genuinely multi-valued content *should* use the commas — that is the flag
  working as designed. The hazard is only single-sentence prose.

Related: [[mem.pattern.doctrine.compose-two-write-cores-bind-both-legs]] — the
other `[facet]`-write footgun whose symptom is also a clean exit code.
