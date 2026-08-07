# SL-249 — design history

Working notes from the design pass, kept out of `design.md` deliberately.

`RV-349` is authoritative for review history, findings and dispositions. This
file holds only what neither the ledger nor `design.md` can: the evolution of a
rule that took six findings to get right, the clearances an adversarial pass
produced by *not* raising anything, one ruling worth not re-deriving, and the
failure shapes the pass exposed.

Nothing here is attested, fingerprinted, or load-bearing on the design. It exists
because `RV-349` `F-34` cut it out of § 10 — where it was generating a fresh
finding every round — and because git history alone would not surface it to
whoever next designs at this length.

Written at design run revision 83.

---

## The rule that took six findings

§ 10 used to mirror the ledger. `F-17` is why it stopped: a hand-maintained copy
of ledger history, living inside the artefact the ledger reviews, goes stale
every round by construction, and had become the repeated subject of its own
findings for miscounting itself. The fix was not more care. It was to stop
keeping the copy.

What followed is a rule being wrong four more times, each time in a way the
previous correction had not anticipated.

- **`F-18`** — the tables went, and the *arithmetic stayed in the prose*:
  "twice", "four rounds", "in any round". The same mutable fact in a form that
  reads as description rather than as a tally. It was already stale when written.
- **`F-19`** — the rule written to prevent that, *shapes and consequences, never
  quantities*, was wrong in both directions at once: it banned design-stable
  numbers the section needed ("the table's three pins", "seven templates") and
  permitted claims that decay without being numeric.
- **`F-20`** — its replacement, *monotone or nothing*, was wrong a third way: as
  a rule over the whole section it was **false of that section**. The press list
  is deliberately a list of current states — *largest thing still undefined*, *no
  home yet*, *remaining unverified* — and a rule banning those deletes the
  section's purpose. It was also, being a totality asserted about content nobody
  had checked it against, the pass's own signature failure committed inside the
  rule written to stop it.
- **`F-23`** — the boundary that replaced it was right, and the section then
  cited, as an example of what the boundary *permits*, a claim that falls
  squarely on the forbidden side of it.
- **`F-26`** — the surviving broad sentence, *nothing below tallies the review*,
  was stricter than the boundary it introduced and false of the section: five
  monotone ordinals stood below it, all permitted by the boundary. `F-20`'s
  defect, a third time, and defended in a disposition before the evidence
  arrived.

### The boundary that holds

A distinction between two kinds of claim, not a ban:

- **Claims about the review's own history** — what the ledger owns and revises as
  it moves — must be monotone. An ordinal that fixes an event in the order it
  happened stays true; a running total does not. *Twice*, *four rounds*, *in any
  round* were all of the second kind, and every one was stale on arrival.
- **Claims about the design's current state** carry no mechanical guarantee.
  `F-21` is the finding that established this: § 10 had claimed its own
  fingerprint bound its claims about *other* sections, and it does not.
  `missing_lanes` (`src/design_run/snapshot.rs`) matches a held attestation on
  its subject **and on that subject's own fingerprint**, so an amendment to § 5.3
  stales sec-8's attestation and leaves sec-14's untouched and live. What stands
  in for the guarantee is discipline: *point, don't restate* — name the section
  or decision that holds the claim, so a design that moves under the pointer
  leaves the pointer true.

The whole-run `ContentCoverage` (`src/design_run/attestation.rs`) does diff every
section, but it invalidates the integrated review and the lock acceptance — not
the attestation covering a given section — and it is spent at the lock.

---

## What the pass cleared

The ledger records findings. It cannot record their absence, which is the one
thing worth keeping here.

Through the external reviewer's last round, raised against revision 72, no
finding had been raised on: `DEC-177`'s tripwire remaining justified for
hand-edits and out-of-band writers; the Phase A/B boundary being coherent; `D4`'s
`body` reuse being carried by objective 3's refusal; `ADR-013` REV routing and
`ADR-004` relation deferral being correctly applied; the seven scaffold templates
seeding exactly their kinds' field sets; and the code claims checked against the
source — `Declaration`'s `deny_unknown_fields`, `set_facet_mixed`'s missing-key
creation, `append_edge → Noop`.

One claim stood on that list through that round and did not survive the pass:
`skip_serializing_if` totality, which `F-24` disproved against revision 81 and
which § 3 replaced with an enumeration at revision 82. `F-28` then disproved the
*replacement* — see below.

A dated clearance stays true. It is saying such claims "hold" that makes it
false, which is what `F-28` caught in § 10.

---

## `D9`'s proportionality ruling

Put to the reviewer directly, because a single test asserting a spec says "seven"
and not "four" had been rewritten again and again, and that is the profile of a
fix that has outgrown what it protects.

The ruling was that it has not: the test as it stands is compact, each clause
closes a demonstrated failure, and a simpler one would knowingly surrender
coverage. Recorded so a later reader finds a ruling rather than re-deriving one.

---

## What the pass taught

Two shapes recurred, and neither is carelessness.

### A totality asserted rather than enumerated

*The pins are total together. The retry carries the same payload. Every
occurrence is kind-derived. The allowlist expires. The declared phrase is found.
The fingerprint binds it. The wire key set is the populated field set. These
entries are already pointers. The memory says subtable insertion is unsafe.*

Each was cheap to check, and each was part of an argument that was otherwise
sound — which is the mechanism, not an excuse. A claim of this kind inherits the
credibility of the reasoning around it, and so never attracts the one command
that would settle it.

**The dangerous instances are the ones written *into a correction to an earlier
instance*.** `F-28` replaced `F-24`'s false totality with a second one that reads
as its own enumeration. `F-25` asserted the pointer discipline held in the very
sentence prescribing it. `F-29` cited a memory as authority for the opposite of
what the memory says — it scopes its proof to root keys, and a scope limit is not
a finding of unsafety. `F-31` committed the defect it was written to explain, two
sentences before explaining it.

A correction is read as the careful version, so it is trusted at exactly the
moment it has been written fastest. Proximity is not protection either: in
`F-31`'s case the rule was not forgotten or half-remembered, it was being written
out in full one sentence later. Whatever mechanism catches this class cannot rely
on the author having the rule in mind, because there the author demonstrably did.

### The artefact nobody re-reads

The scope card drifted from the design; a criterion widened past the card's own
non-goals because nothing compared it back; a finding sat contested on the ledger
while its substance was being fixed elsewhere; and § 10 drifted from the ledger
repeatedly. Each of those artefacts was the one updated last, after the
substantive work, by hand.

`F-26` and `F-27` are the compact form: both fixes landed at the site the finding
cited and nowhere else, so the same claim standing two paragraphs away survived.
Fixing the instance rather than the class.

`F-32` is the same failure at the scale of a single sentence. "Where this design
states such a thing, it states the enumeration or the identity beside it" stood
as the conclusion of the paragraph cataloguing this exact pattern, and survived
`F-19`, `F-20`, `F-21`, `F-24` and `F-25` unread — including two rounds in which
the responder was editing the paragraph it closes. Five rounds of attention to a
pattern did not cause anyone to read the sentence claiming the pattern was
handled.

The `review.scope` runbook step is the comparison that should catch this class,
and it fires once at the end of a stage — the wrong cadence for a review that
keeps going. Mechanising it is a note for whoever next designs at this length.
There is no fix inside this slice, and resolving to remember would be the same
failure again.

### Why this file exists

`F-34` is the last instance and the reason for the split. § 10 declared that it
did not mirror the ledger and then narrated it at length, so every response to a
finding rewrote the section, invalidated its attestation, and created fresh
claims to review. Two consecutive adversarial rounds found defects almost
exclusively in prose written to fix the previous round's defects. The reviewer's
judgement was that the corrections were converging locally but the section was
not converging structurally, and that the right move was deletion rather than
another repair.

`F-17` and `F-18` were the same cut, twice before. This is the third.

`R4` says objective 4's completion is easy to assert. This pass is that failure
mode, found repeatedly inside the design that guards against it.
