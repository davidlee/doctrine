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
`doctrine design show --format prompt`'s 12-char fingerprint prefixes **before**
relying on it (`--format` since `SL-246`/`DEC-261`; the bare verb now renders the
design document) —
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



## The two refusals that read wrong (SL-246, 2026-09-18)

Both cost a round trip; both are in the adoption completeness check
(`adopt_authored`, `src/design_run/run.rs`), which compares the CALLER's
declared map.

**`sections` values are DIGESTS, and the contract says otherwise.** `doctrine
design contract --format prompt` prints the value type as `text`:

    sections  {id(sec-): text}  optional

which reads as the section's prose. Pass prose and it refuses:

    adopt_authored's marker map is not complete and exact:
    0 missing, 0 unknown, 9 mismatched

Read the three counters as a diagnostic — `missing`/`unknown` compare the id
SET, only `mismatched` compares values. So *everything mismatched, nothing
missing* means the ids are right and the value FORM is wrong, not that the
document drifted.

**`sections` is mandatory, despite the contract marking it `optional`.**
Omitting it refuses `9 missing, 0 unknown, 0 mismatched` — the check requires
the declared map to cover every section the run holds. Optional in the wire
schema, required in practice.

Also worked on SL-246 (design run `dr-019fd1ab`, revision 37→38): eight of nine
sections moved in one adopt cycle while integrating RV-370's findings.

## Footgun: computing the section body by splitting on lines

The rule is a **raw byte slice**, and a line-splitting implementation silently
gets it wrong in a way that looks almost right.

Splitting the file into lines, joining `lines[after_marker:next_marker]`, and
stripping one trailing newline drops **one newline too few** for every section
but the last. A non-final section's slice runs up to the start of the next
marker line, so it ends with the blank separator line: the body legitimately
*keeps* a trailing newline after the rule removes one. The final section's slice
ends at EOF with a single newline, so the same buggy code produces the right
answer there.

The signature of the bug is therefore diagnostic and worth recognising on
sight: **the last section matches, every earlier one does not, and the
whole-file hash matches.** That combination means the extraction rule, not the
file.

Do it on the raw text instead — find marker lines with
`^<!-- doctrine:section (sec-\d+) -->\n` under MULTILINE, slice from the match
end to the next match start (or EOF), then remove exactly one trailing newline.

This is why the positive control above is not optional ceremony: it cost one
cheap run to catch, and an unverified `adopt_authored` would have written eight
wrong section fingerprints into the run.
