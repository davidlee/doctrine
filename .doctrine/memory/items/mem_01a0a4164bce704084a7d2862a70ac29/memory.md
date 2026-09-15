`serde_json::Error`'s `Display` prints `at line L column C` **only when
`line != 0`** (`error.rs`, `Display for Error`). `from_str` sets it from the
reader; `from_value` constructs its errors without one. So a two-step parse —
`from_str::<Value>` for a pre-deserialisation check, then
`from_value::<Typed>(document)` — keeps serde's words and **silently drops its
position**.

Deserialise the typed form from the **original string** instead. The cost is one
extra parse of a bounded payload; the `Value` stays for whatever wanted it.

```rust
let document: Value = serde_json::from_str(payload)?;   // for the walk
contract_check::refuse_unknown_keys(&document)?;
let request: Typed = serde_json::from_str(payload)?;    // NOT from_value(document)
```

## The second half: `#[serde(flatten)]` buffers, so positions degrade above it

`#[serde(flatten)]` makes serde collect the whole map into `Content` before
deserialising any field. A fault in a **flattened** key is therefore reported
where the buffer ends — the closing brace of the enclosing object — not at the
key:

    "known_revision": "not-a-number"   on line 3  →  at line 5 column 1

**Below** the flattened surface the position is exact (`declare[0].subject` at
line 7 reports `at line 7 column 19`), which is where a long hand-authored
payload's faults actually live. So the repair is worth making, but a test
pinning it must use a **nested** fault or it pins the degraded answer.

## Where this bit

`SL-259` `PHASE-03` added a contract walk over the parsed `Value` and switched
the typed parse to `from_value` in the same edit. `contract_check` deliberately
declines shape faults (`walk_type`'s `None => Ok(())` arm: "serde reports it a
moment later in its own words") — so the slice handed the whole class to serde
and made serde's answer less locatable in one move. `RV-367` `F-2`; repaired at
`3444c59cd`, pinned by `a_shape_fault_keeps_its_line_and_column`.

Related: [[mem.pattern.review.cited-guard-must-assert-the-claim]] — the same
review, the same distance between a sentence and the code beside it.

See also [[mem_019fd03e13397240b4eb05af218f5cf5]] — the other way
`#[serde(flatten)]` bites this same request type: it forbids
`deny_unknown_fields`, so a retired wire field is absorbed rather than refused.
