# A recorded thread memory is hidden from find/retrieve until verified (SL-008 D6)

A `thread`-type memory does **not** surface in `doctrine memory find` /
`retrieve` until it is **verified AND `reviewed` within 14 days** of today. It
stays visible in `memory list` / `memory show` the whole time, so it looks
recorded-and-fine while being invisible to scope ranking. Every other type
(`pattern`/`fact`/`concept`/`system`/`signpost`) always passes — the gate is
thread-only.

## Why (working as designed)

- Gate: `thread_expiry` in `src/retrieve.rs` (`if m.kind != Thread { return
  true }` else require `verification_state == "verified"` and `reviewed` ≤
  `THREAD_FRESH_DAYS` = 14). Runs in the query pipeline `base_filter →
  match_scope → thread_expiry → staleness → rank`.
- Decided in SL-008 design **D6** ("thread expiry requires verified + recent",
  review #7). The alternative — surface unverified threads — was explicitly
  **rejected** (would surface unverified stale threads). Do NOT "fix" this by
  loosening `thread_expiry`; it is reviewed canon + behaviour-preservation gate.

## The trap

`doctrine memory record` always scaffolds `verification_state = "unverified"`,
`reviewed = ""`. So a **freshly-recorded thread is always invisible** to
`find`/`retrieve`. Compounding: `verify` **refuses a dirty tree**, so if you
recorded the thread mid-work (dirty tree), you cannot attest it until you reach
a clean tree.

## How to apply

- Recording an open working loop you want to re-find by scope? Either record it
  as the durable type it really is (`pattern`/`system`/`concept`), or record the
  `thread` and immediately `doctrine memory verify <key>` **on a clean tree**.
- A `thread` is for genuinely short-lived working state that SHOULD decay —
  accept that it won't rank until attested.
- Surfaceability sanity-check (skill §7) is misleading for threads: `find` will
  show nothing even when the record is correct. Check `memory show` instead.

Diagnosed from the SL-032 handover "blind spot" (the deferred dedup thread): not
a staleness/indexing/matcher bug — the thread-expiry gate firing. See
[[mem.thread.sl-031.kind-registry-dedup]] (the invisible thread) and its
surfacing sibling [[mem.pattern.entity.numbered-kind-identity-table]] (a
`pattern`, always passes).
