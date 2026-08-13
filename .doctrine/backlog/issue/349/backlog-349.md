# ISS-349: Prose facet flags split on commas with no escape

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctrine knowledge edit <kind>`'s list-valued facet flags — `--alternatives`,
`--consequences`, and their siblings on the other kinds — are comma-separated
with **no escape mechanism**. The values they carry are English prose, and
English prose contains commas.

```bash
doctrine knowledge edit decision DEC-168 \
  --alternatives "Rejected on coupling — see rationale. (Withdrawn: SL-249 D8a.)"
```

stores two elements, splitting mid-sentence. Nothing warns.

## Why it is an issue and not a preference

The flag's help says "comma-separated; bare flag clears", which is true and
still misleads: it reads as *you may pass several* rather than *your prose will
be split*. The actual multi-value route — repeating the flag, one occurrence per
element — is not stated anywhere and was established by experiment.

So the documented affordance produces silently mangled data and the working
affordance is undocumented. The corruption is invisible at the call site: the
command succeeds and reports the field names it wrote.

## Sightings

- `mem.pattern.doctrine.comma-separated-facet-flags-split-prose` — the standing
  memory, medium trust, scoped to `src/knowledge.rs` and `doctrine knowledge edit`.
- `SL-254` design, 2026-08-13 — `DEC-214`'s first population shredded five prose
  elements into eleven.
- `SL-254` design, 2026-08-13 — again, and this time as *forced* work rather
  than a slip: `ISS-328` (an unknown key inside `CreateRecord` is dropped)
  landed `DEC-215`/`DEC-216` with empty facets, and back-filling them through
  `knowledge edit` meant re-authoring all eight `alternatives`/`consequences`
  elements comma-free. Observation `019ff90f-770a-7da0-b8fb-ca5a0a382c5c`.

That second one is the argument for fixing it: the footgun compounds. It taxes
every recovery path that routes through `knowledge edit`, which is the path
anything reaching a facet by a route other than its own verb has to take.

## Shape of a fix

Not costed; roughly in increasing order of effort.

1. **Fix the help text.** State that the value is split on commas and that
   repeating the flag appends one element per occurrence. Cheapest, and closes
   the "documented affordance is the broken one" half on its own.
2. **Drop the delimiter, keep repetition.** One occurrence, one element — no
   splitting. This is the shape the working route already has. It is a
   behaviour change for any caller currently relying on the split, which is
   discoverable from the corpus rather than assumed.
3. **A structured input route** — `--input` / stdin carrying the facet map as
   JSON, the way `design apply` already takes its payload. Escapes the shell
   quoting question entirely and would have made both `SL-254` sightings
   impossible. `IMP-332` did the analogous thing for observation records
   (different surface, same reasoning) — worth reading before designing this.

(1) and (2) are close to independent; (1) is worth doing whatever happens to the
rest.

## Related

- `ISS-328` — an unknown key inside `CreateRecord` is dropped rather than
  refused. The two compose: that one forces the detour through
  `knowledge edit`, this one taxes it.
- `IMP-332` — the structured-request precedent on the observation surface.
- `STD-001` — if the delimiter stays, it should be a named constant rather than
  a per-flag literal.
