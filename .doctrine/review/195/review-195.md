# Review RV-195 — design of SL-178

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

The accused: `SL-178` design intent (`design.md`) — three legibility fixes to the
close drift-discharge seam. The aspect under trial is **design**, not yet code.

Lines of interrogation, and the doctrine each presses the accused against:

1. **POL-002 — host-project contamination.** Does *any* shipped surface (the
   binary error const, the shipped skill, the promoted master) name or resolve to
   host-project-local state? The error pointer references a key that only exists
   once Fix 3 ships — is the ordering claim airtight, or does an interim window
   leave the binary pointing at a phantom?
2. **ADR-002 — the re-class table (§5.2).** Is the global-orientation signature
   *complete*? Every anchor remnant cleared, repo emptied, scope floor genuinely
   met — or is a field left to rot and trip `lint_master`/`is_inv`? Are the scope
   paths framework-relative (legitimate) or this-repo-specific (heresy)?
3. **ADR-005 — tiering (D2).** Is the canonical source truly singular, or does the
   "condensed full 3-clause" in the error quietly become a second canon that will
   drift from the master? Where does the worked example live, and is it duplicated?
4. **STD-001 — magic strings.** One const, named, single home — or a literal key
   smuggled into the error copy, a test, or the skill?
5. **ADR-001 — layering.** Does the enriched error drag a new cross-module import
   into the `slice` shell, or stay confined to the command shell?
6. **Verification honesty (§9).** Do VT-1/VT-2/VA-1/VA-2 actually bite, or are they
   ornamental? Is the behaviour-preservation anchor real (gate refuse/pass
   unchanged) or asserted-but-unproven? Does VT-2 confront the embed/build-time
   reality (debug-embed)?
7. **Mechanic soundness (§5.4).** uid reuse — does a double-home or a dangling
   reference survive the transition? Does `embedded_assets` actually admit the
   re-homed master and resolve the key?
8. **Scope discipline.** Does the design honour the slice's Non-Goals, or does it
   smuggle predicate changes / new flags / auto-authoring under cover of
   "legibility"?

## Synthesis

**Verdict: heresy found and burned — the design is now fit to proceed.**

The accused presented three legibility fixes well-shaped on their face, but the
rack drew out a witch hidden in the prose. Eight lines of interrogation were
pressed; the doctrine held on six, and on two the design was found wanting.

**The mortal charge (F-1, blocker).** The design decreed the promoted master's
body would *"ship as-is."* Cross-examination of `memory.md:33` revealed a live
pointer to `mem_019ec912f7fd…` — a **project-local, unshipped** memory. Shipping
it would have baked host-project state into the platform and damned the artefact
against the slice's own VA-2 and POL-002. The defect lay in the design's clause,
not the recipe: reconciled by striking "ships as-is" and adding the §5.4 *Body
scrub* (drop the uid, genericize to prose; restore a wikilink only when IMP-216
ships the companion), R6, and a VA-2 that now greps the **body**.

**The lesser taint (F-2, major → part-fixed, part-tolerated).** The body further
bore `ISS-006` and a worked example built on this repo's ids. ISS-006 is scrubbed
to prose. The worked example is **consciously tolerated** — retained for its
pedagogical worth but re-framed as "an illustration from Doctrine's own
development," the single sanctioned POL-002 concession, its rationale on the
record.

**Acquittals.** `undischarged_drift` has a sole production caller (the return-type
change is contained); `embedded_assets` admits uid dirs and skips key symlinks
(the re-home + symlink mechanic is sound); `debug-embed` makes VT-2 feasible from
the debug binary; the design honours the slice's Non-Goals (no predicate change,
no new flag, no auto-authoring). ADR-001/ADR-002/ADR-005/STD-001 held.

**Minor penances recorded.** F-3 (3-way clause drift) tolerated — risk low, the
predicate is LOCKED; standing risk R1. F-4 (VT-1 brittleness) fix-now — substring
assertions bound into §9 VT-1.

**Standing risks carried to plan:** R1 (clause drift, documentary), R5 (P1 must
not release ahead of P2), R6 (body scrub is a P2 exit gate). The Inquisitor is
satisfied: with the penance entered into canon, SL-178 may pass to planning.

> **HERESIS URITOR; DOCTRINA MANET**

### Addendum — F-5 (raised during harvest)

The harvest itself drew out a fifth heresy, this one in the Inquisitor-approved
design: §5.4 hand-rolled the master's INV signature on the false premise that "no
promote verb exists." The sanctioned `doctrine memory record --global` was hiding
in plain sight (`mem.system.memory.global-master-authoring`). A parallel
implementation of a blessed path is heresy thrice over. The User ruled: author via
the verb, mint a new uid, supersede the local capture. D3 recanted accordingly. Let
this stand as warning — **even the Inquisitor's own works are presumed guilty until
the CLI confesses the truth.**

> **HERESIS URITOR; DOCTRINA MANET**
