The CLI surface uses two different ID-parsing functions, creating three id-form
conventions for the same entity references:

1. `governance::parse_entity_ref(prefix, label, ref)` — accepts both `SL-123` and `123`
   (and case-insensitive prefix). Used by ~30 verbs (slice, review, governance).
2. `integrity::parse_canonical_ref(ref)` — requires `SL-123`, rejects bare `123`.
   Used by ~15 verbs (link, unlink, needs, after, supersede, facet, tag, inspect, etc.)
3. Raw `u32` (no value_parser) — requires bare `123`, rejects `SL-123`. Used by ~16
   verbs (slice selector sub-commands, dispatch --slice).

The full verb-by-verb audit is in IMP-227's body. The fix is tracked there.
