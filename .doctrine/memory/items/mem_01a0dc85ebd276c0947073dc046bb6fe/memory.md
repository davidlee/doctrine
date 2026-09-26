IMP-490 added `review list --target <ref>` by taking a `target` arg on the
`ReviewCommand::List` variant and post-filtering in `list_rows` on the bare
`[target].ref` — NOT through `CommonListArgs`.

That is deliberate but not free: `CommonListArgs` is documented as "the
mandatory spine of the read surface: a kind cannot quietly grow bespoke list
flags". `--target` here is a structural single-edge filter, true of review only
in its current form.

When RFC-032 slice 2 lands the D5 read-surface remainder (projection, census,
JSON unification, shared filters), decide whether to generalise a `target` axis
into `listing::FilterFields` + `CommonListArgs` (every kind populates it from its
structural edge, absent -> non-match) or keep it review-local. The generalised
form is the DRY outcome if a second kind grows the same need; until then the
local flag avoids widening the shared spine's golden surface.
