# One REV narrows criterion 3 to the floor

`SL-253` `inq-7` asked what the REV actually says, scoping it at three payloads.
Reading `REQ-459` against `DEC-189`, `DEC-195` and `DEC-198` makes it four over
two artefacts — and two of the three turn out to be one edit.

## Why the disposition cannot be corrected alone

`REV-051` disposed `REQ-459` criterion 3 as *"discharged structurally — one suite
parameterised by backend; a second backend passing it edits nothing."*

That is a paraphrase of criterion 3's own text: *"No macOS or other backend
claims support until its mechanism passes the same property suite
independently."*

`DEC-189` breaks the premise under both. Row membership does not port — a
hypervisor loses rows 10, 12, 13 and 14 and earns others — so there is no single
suite a second mechanism passes. `DEC-198`'s open assurance key exists precisely
to permit that.

What survives is narrower and better: a second mechanism proves **the same
floor** and publishes **its own profile**. *Edits nothing* stays true; *the same
property suite* does not. Correcting the disposition while leaving the criterion
intact would leave `SPEC-030` asserting the thing the correction was made for.

## The four payloads

1. **`REQ-459` criterion 1 splits.** It enumerates seven property families in one
   undifferentiated list with canonical-authority denial inside it — the exact
   conflation `DEC-195` un-conflates. It becomes two criteria of different
   invariance: an authority floor proven on every mechanism, and an assurance
   profile that varies per mechanism and is published rather than reduced.
   `DEC-191`'s retirement of the single green path falls out of this rather than
   being a separate edit.
2. **`REQ-459` criterion 3's text narrows** — same floor, own profile. An
   explicit widening of the slice's scope, taken by the owner.
3. **`REV-051`'s criterion-3 disposition is corrected** to match.
4. **`IMP-405`'s rename** applies across `SPEC-030` § *Platform backend
   contract*, whose title keys mechanism to platform and whose text names the v0
   backend "Linux/bubblewrap"; and **`CPT-002`'s threat priority** lands in
   § **Concerns**.

## Why Concerns, not Overview

§ Overview establishes what the container *is* and what it descends from, so a
threat-priority statement there reads as scoping. § Concerns already carries
"**Security posture is structural**" in the same declarative form — and that
entry is the one a reader most easily takes to mean *escape is the threat*.
`CPT-002`'s escape/heresy asymmetry does its work sitting beside it.

## One REV, not two

Separating the spec edit from the `REV-051` disposition correction buys cleaner
provenance — correcting a `done` revision is a different act from revising a
requirement — at the cost of a window in which the two are not both true. The
four payloads are consequences of one un-conflation plus a wording fix to the
section they all live in, so they land together, with the code.
