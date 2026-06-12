# Each list surface must call listing::validate_statuses itself

`listing::validate_statuses(given, known)` is the `--status` known-set guard, but
it is an **opt-in** call — `listing::build` and `listing::retain` do NOT call it.
`build` only parses args into a `Filter`; `retain` silently drops any row whose
status is not in `f.status`. So a list surface that flattens `CommonListArgs` and
rides the spine inherits `--status` *filtering* "for free" but NOT *validation*: an
unknown `--status bogus` silently returns an empty result instead of erroring
`unknown status 'bogus' (known: …)`.

Every list surface must therefore call `validate_statuses(&args.status, <KIND>_STATUSES)?`
itself, before `listing::build`, against its kind's known-set const. The established
call sites: `spec list` (SPEC_STATUSES), governance, review, memory, slice, backlog,
rec. The const must stay in lockstep with the status enum's kebab serde — pin it
with a `*_statuses_matches_the_variants` drift canary (mirrors spec's).

This is exactly the trap SL-045's `spec req list` fell into (RV-005 F-1): it rode
`retain` for filtering but skipped the validation call, so `--status bogus` emptied
the roster silently — a F4/SL-025 uniform-list-contract breach. Fixed by adding
`requirement::REQ_STATUSES` + the `validate_statuses` call in `req_list_rows`.

When adding a new list surface, the checklist is: flatten `CommonListArgs` → add a
`<KIND>_STATUSES` const + drift canary → call `validate_statuses` before `build`.

See [[mem.pattern.listing.column-model-extension]] for the sibling column-model
extension pattern on the same spine.
