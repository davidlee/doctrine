# DEC-228: Worked example kept, corrected, and parse-pinned

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Two jobs, not one

Once a total contract exists, the obvious move is to delete the partial example
it supersedes. That reading is wrong, because the two do different work.

`DECLARATION_EXAMPLE` (`render/envelope.rs:75`) is **copyable at zero cost**. It
rides the turn envelope in the no-drop set, so a caller assembling a mutation has
a working payload in front of it without invoking anything. The contract is
**total**, and totality is what you want when you have a specific question — but
reaching it costs a call.

`DEC-225` decided the same trade one level up: push the address, pull the body.
Keeping a copyable example beside a pointer to the full contract is that decision
applied consistently, not an exception to it.

`IMP-402` already argued against trimming this constant as a byte saving, and the
comment on the constant says so directly: *"Do not trim `needs` back out as a byte
saving: the bound is 1024 and this is under 320."* The arrival of a contract
elsewhere does not change the arithmetic.

## But it is wrong, and now demonstrably so

The example shows a `traversal` carrying `pin`, `posture` and `authority`, and
omits `cursor` — the one key a resuming agent must set. The slice scope records
what that omission cost: fifteen reads of `submission.rs`.

An illustration with a gap is tolerable while nothing contradicts it. Once a total
contract ships, the gap becomes a contradiction between two surfaces that are
supposed to agree, and the example is the one that is wrong. Adding `cursor` costs
roughly fifteen bytes against a 1024-byte bound.

## The drift risk, and a cheap pin

A hand-written example sitting beside a generated contract is exactly the `STD-001`
hazard: two descriptions of one thing, free to disagree.

Generating a worked example from the contract is the hard version of the fix and is
not worth attempting — a good example makes editorial choices about *which* acts to
show, and generation has no basis for those.

The cheap version is a test asserting the constant deserialises as a valid
`ApplyRequest`. That cannot catch a *stale* example, but it catches the failure
mode that matters: an example illustrating keys the payload no longer accepts. It
sits beside the existing compile-time length assertion
(`const _: () = assert!(DECLARATION_EXAMPLE.len() <= ENVELOPE_DECLARATION_EXAMPLE_BYTES)`),
though it must be a test rather than a `const` assertion, since deserialisation is
not const-evaluable.

## Bytes

Roughly 15 for the `cursor` correction, on top of `DEC-225`'s roughly 60 for the
address pointer, against a 1024-byte example bound and a 24576-byte normal
envelope budget. The example still shows two acts of ten; `DEC-225`'s adjacent
pointer covers the rest, which is the division of labour the pair exists for.
