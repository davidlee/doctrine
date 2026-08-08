Correcting prose in a **locked** design run costs one regression and one
adoption — not a re-emission of the sections you are editing, and not a
recovery cycle.

The trap is that `declare` with a `body` takes the **whole section**, beginning
with its heading. For a 5000-line design with corrections spread over six
sections that means re-emitting thousands of lines of prose you did not intend
to touch, at real token cost and with real risk of silent drift in the exact
text you are trying to correct precisely. `adopt_authored` is the sanctioned
alternative (DEC-092 rule 2 calls it "the sole lawful crossing" of an authored
watermark divergence) and it is a **protocol, not a bypass** — the warning
against "hand-editing a locked run" is about editing *without* adopting.

The loop:

1. **Regress the stage.** `{"stage":{"to":"reviewing","reason":"…"}}`.
   `gate::regress` accepts any backward move with a non-blank reason. Go to
   `reviewing`, not `drafting` — the forward conditions are cumulative and
   re-evaluated either way, so the only difference is how many edge-local
   runbooks you must discharge again.
2. **Hand-edit `design.md`** surgically with an exact-match editor.
3. **Adopt.** `{"adopt_authored":{"fingerprint":<sha256 of the whole file>,
   "sections":{"sec-N":<sha256 of that section's body>,…}}}` — every section the
   run holds, and no other.

**Computing a section body:** the bytes from the start of the line *after* its
`<!-- doctrine:section sec-N -->` marker, up to the start of the next marker
line, minus the single trailing newline. `unescape_line` only rewrites
marker-shaped lines carrying ≥2 colons, so on an ordinary document it is the
identity and the body is the raw file slice. Verify your computation against
`doctrine design show`'s 12-char fingerprint prefixes **before** relying on it —
that is a free positive control, and it also proves the whole-file hash against
the printed watermark.

Then `doctrine design materialise` and diff: it must reproduce `design.md`
**byte-identically**. If it does, the runtime and authored tiers agree and the
adoption was coherent.

**What the edit costs you at the gate**, all of it correct and none of it
skippable: each changed section's attestation is invalidated, and so is the
run-level `design-accepted` act. Re-locking therefore needs a fresh attestation
per changed section — **in the lane `review_policy` names**, which is
`adversarial-only` on a run that used it — plus **two user acts** you must not
author on your own initiative: the review-pass disposition (`Conducted` over a
review that never saw the new bytes is a false claim; `Waived` with a stated
reason is the honest arm) and the design acceptance, whose `AcceptanceDeclaration`
deliberately has no `authority` field precisely so a payload cannot claim one.

Worked end to end on SL-248 (design run `dr-019fd432`, revisions 85→93): eight
corrections across six sections, two adopt cycles, materialise byte-identical
both times.
