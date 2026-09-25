A cross-kind ref resolver cannot mirror one "per-kind prefix case rule" — there
isn't one.

- `listing::parse_ref` (the listing kinds: slice, adr, policy, rfc, revision,
  rec, concept-map) strips exactly `PREFIX-` or its all-lowercase spelling,
  deliberately not case-insensitively: `dec-031` resolves, `Dec-031` does not.
- `knowledge::resolve_ref` (src/knowledge.rs) and `backlog::parse_ref`
  (src/backlog.rs) call `.to_uppercase()` on the prefix, so they ignore case
  entirely.
- `spec::resolve_spec_ref` (src/spec.rs) never uppercases — case-sensitive.

`kinds::parse_canonical_ref` looks the prefix up verbatim in `KINDS`, so a
cross-kind router must normalise before resolving. SL-265 chose to ASCII-uppercase
the prefix unconditionally: the router then accepts every prefixed ref any kind's
own `show` accepts, plus a few a case-strict kind refuses (a superset). The
equivalence it guarantees is therefore scoped to prefixed refs only — a bare id
is resolved across every kind and refuses as ambiguous when more than one holds
it (SL-265 `DEC-297`).

Surfaced by RV-384 F-25 during the SL-265 design review.
