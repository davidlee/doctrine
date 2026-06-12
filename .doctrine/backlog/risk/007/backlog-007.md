# RSK-007: inspect inbound sort is lexical, not numeric — misorders at id ≥ 1000

Surfaced by the SL-046 `/code-review` (post-RV-006).

`relation_graph::inspect` builds the inbound source list as `Vec<String>` of canonical
refs and calls `srcs.sort()` — a **lexical** string sort. `listing::canonical_id`
zero-pads to a *minimum* of three digits, not a fixed width (`listing.rs:36`; its own
test asserts `REQ-1234` is not truncated). So once any namespace reaches id ≥ 1000:

```
"SL-1000" < "SL-999"   # lexical: '1' < '9' — wrong
```

The inbound section then renders out of numeric order. The doc comment claimed
"ascending canonical-ref order"; it delivers lexical order (comment corrected in the
SL-046 cleanup commit — this item is the real fix).

**Why low/medium:** no namespace is near 1000 today, so determinism holds and every
suite is green. The trap is that `inbound_render_is_permutation_invariant` only seeds
single-digit ids — it certifies "ascending order" while exercising only the range
where lexical == numeric. False confidence at the cliff.

**Do:** sort by `(prefix, numeric id)` (parse the ref back, or sort `EntityKey` before
rendering), and extend the permutation test past id 999.

Relates: SL-046 (`[[slices]]`).
