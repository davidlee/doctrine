Correcting prose in a **locked** design run costs one regression and one
adoption — not a re-emission of the sections you are editing, and not a
recovery cycle.

The trap is that `declare` with a `body` takes the **whole section**, beginning
with its heading. For a 5000-line design with corrections spread over six
sections that means re-emitting thousands of lines of prose you did not intend
to touch, at real token cost and with real risk of silent drift in the exact
text you are trying to correct precisely. `doctrine design adopt` is the
sanctioned alternative — the sole lawful crossing of an authored watermark
divergence (`DEC-279`, `SL-261`) — and it is a **protocol, not a bypass**: the
warning against "hand-editing a locked run" is about editing *without*
adopting.

The loop — **in this order**:

1. **Regress the stage first.** `{"stage":{"to":"reviewing","reason":"…"}}`
   via `design apply`. `gate::regress` accepts any backward move with a
   non-blank reason. Go to `reviewing`, not `drafting` — the forward conditions
   are cumulative and re-evaluated either way, so the only difference is how
   many edge-local runbooks you must discharge again.
2. **Hand-edit `design.md`** surgically with an exact-match editor.
3. **Review, then adopt.** `doctrine design adopt SL-N --dry-run --diff` shows
   each changed section's hunk and the invalidations the crossing will cause,
   and prints the full document fingerprint. Then
   `doctrine design adopt SL-N --expect <that fingerprint>`. The engine derives
   the section map from the document; nothing is hand-computed.

**Why the order is load-bearing.** Edit first and you are stuck: once the
document diverges, the regress payload is an ordinary mutation and the
watermark refuses it, and `adopt` refuses a diverged document on a **locked**
run before parsing it (`AdoptionLocked`). The way out is to revert the edit,
regress, and edit again.

Then `doctrine design materialise` and diff: it must reproduce `design.md`
**byte-identically** (bar a whitespace-only head before the first marker, which
the adopt report discloses and materialise drops).

**What the edit costs you at the gate**, all of it correct and none of it
skippable: each changed section's attestation is invalidated, and so is the
run-level `design-accepted` act — the adopt report lists each one. Re-locking
therefore needs a fresh attestation per changed section — **in the lane
`review_policy` names**, which is `adversarial-only` on a run that used it —
plus **two user acts** you must not author on your own initiative: the
review-pass disposition (`Conducted` over a review that never saw the new bytes
is a false claim; `Waived` with a stated reason is the honest arm) and the
design acceptance, whose `AcceptanceDeclaration` deliberately has no
`authority` field precisely so a payload cannot claim one.

Worked end to end on SL-248 (design run `dr-019fd432`, revisions 85→93), under
the retired `adopt_authored` payload key: eight corrections across six
sections, two adopt cycles, materialise byte-identical both times. The key is
now refused as retired, naming the verb.

At reconcile, do not run this loop — see
[[mem.pattern.reconcile.edit-design-out-of-band]].
