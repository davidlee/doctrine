# Review RV-086 — design of SL-100

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

The Inquisition arraigns the **design aspect** of SL-100 — the memory lifecycle
verbs (`status`, `edit`, `tag`) and skill updates. The slice claims `design`
status, *locked*, with a prior adversarial pass already folded in (findings F1–F4
in `design.md`; commit `99c8e64c`). A design blessed once is no design above
suspicion; the Inquisition presses where a self-review is most apt to flatter
itself — the load-bearing claims the canon makes about the *existing substrate*,
for a design that misreads the ground it stands on will lead the implementer into
the pit.

**Lines of interrogation:**

1. **The `--key` immutability premise (R1/F1).** The canon swears the scaffold
   "writes `memory_key = ""`" and that `run_edit` "checks `memory_key.is_empty()`
   rather than `Option<>`." Held against the code: `memory_key: Option<String>`
   (`memory.rs:379`); the scaffold *omits* the line when no key is given
   (`render_memory_toml`, `None => String::new()`, `memory.rs:785`); the very
   doc-comment above it (`memory.rs:779`) confesses that `memory_key = ""` *would
   fail `validate_key` on read*. Does the named risk-mitigation survive, or is it
   built on a phantom?

2. **The single-transaction vow (D2).** `edit` swears "a single read→mutate→write
   transaction" (L101) yet folds `--status` by "the same … logic as memory status"
   (L128) — which routes through the **IO** `set_authored_status` (`dep_seq.rs:344`,
   per the `knowledge::run_status` precedent), a *second* independent file write
   and a second `updated` stamp. Pure or not? One write or two?

3. **Record/edit parity on `--key` (D2 table).** The canon cites the private
   `validate_key` (`memory.rs:270`) — which *rejects* a bare key — where record
   normalizes through `normalize_key` (`memory.rs:293`), prepending `mem.`. Will
   `edit --key foo` refuse what `record --key foo` accepts?

4. **Truthfulness of the asserted field paths (INV-schema).** The scaffold template
   carries `[review]` with only `verification_state` — **no `review_by`**. The
   design names `[review].review_by` "replace" and `--review-by ""` to clear. On a
   fresh memory the key is absent: insert, not replace; clear is a no-op. Stated?

Doctrine consulted: ADR-001 (layering), ADR-004 (`superseded_by` carve-out),
ADR-010 (Tier-3 memory labels), the pure/imperative split, the
behaviour-preservation gate, and the real schema in
`.doctrine/templates/memory.toml`.
