# Review RV-354 — design of SL-253

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**The external adversarial pass.** `SL-253`'s design (`design.md`, 2158 lines,
run revision 57, stage `reviewing`) has had two hostile passes — `RF-1`…`RF-9`
and `RF-10`…`RF-14`, both recorded in § 10. **Both were run by the design's own
author**, and § 10 says the one thing neither could do: disagree with the
design's framing. This review is the unspent external pass, and it is the last
gate before the run locks.

### What this review probes

1. **The seam's completeness.** `DEC-196` cuts the kernel/payload seam at row
   *identity*. Four independent enumerations of *what crosses that seam
   backwards* have been made — `DEC-196`'s, the draft's, and each review pass's
   — every one believed complete, every one wrong. Three location classes found
   so far: parameter list (`RF-3`/`RF-7`/`RF-9`), function body (`RF-9`), and a
   field of a parameter's type (`RF-10`). § 10.1 states the correct prior: **a
   fourth location class exists**. Find it, or demonstrate the class is closed.
2. **Ceremony vs. necessity in § 5.2's types.** `D2`/`D3`/`D7` stack a struct,
   a refusing constructor and a `FloorReading` wrapper over a floor of **one
   member**. `D7` was added under review pressure, which is when ceremony gets
   added unnoticed. Is `floor: Result<Floor, FloorProperty>` sufficient?
3. **`D1`/`I3` — only `Proven` holds the floor.** The one substantive § 5 ruling
   with no banked `DEC-` behind it. Owner-confirmed but the argument is the
   author's; neither prior pass touched it. If it is wrong, `FloorStanding` is
   wrong.
4. **`Qualification::Ran`'s shape.** Four collections plus a floor in one
   variant. Both passes endorsed keeping `axes` beside `assurance`; both
   endorsements came from the same author. § 10.2 names it the § 5.2 boundary
   call the author is least sure of.
5. **`DEC-199`'s preservation bar.** `AGENTS.md`'s behaviour-preservation gate
   requires shared-machinery suites green *unchanged*; three type-shape changes
   make that literally false, so `DEC-199` restates the bar as *the nineteen row
   verdicts*. Is the restatement a legitimate re-levelling or a licence to
   change behaviour undetected? § 9.1 and `EVD-022` carry the bracket — and
   `EVD-022`'s pre half is **a single run on one host**, now unrepeatable at
   that commit.
6. **Governance application, not citation.** § 10.5 tabulates conformance to
   `ADR-020`, `ADR-001`, `STD-001`, `POL-002`, `RFC-025`, `ADR-021`. Check the
   *application*, especially `ADR-001`: the design concedes the layering gate
   proves tier direction only, so `I7`'s neutrality claim rests entirely on a
   compile probe **whose negative control is unspecified** (§ 10.3). A probe
   that cannot fail proves nothing.
7. **Cite integrity.** § 10.4 admits this document's cites have a known failure
   mode — off-by-one against a preceding comment line — and that "verified
   twice" was claimed and then falsified twice. Cites are against `94d0b5603`;
   spot-check aggressively, and treat any cite introduced after `RF-14` as
   carrying no provenance.

### Out of scope — do not spend effort here

§ 10.3 flags these as deliberate, not oversights: provisional type names
(`DEC-198` — argue shapes, not spellings); `OQ-3`'s possible fifth `DEC-189`
row; `OQ-4`'s host-failure reporting; the kernel's unpredicted final size (`P1`
— the owner has ruled line counts are not a criterion); the `Table` type's
construction; the ~67-test figure's crudeness (`R3`, budgeting only).

A pass that finds nothing on a given axis should say so plainly rather than
manufacture a finding — but two author-run passes clearing an axis is weak
evidence, not strong.
