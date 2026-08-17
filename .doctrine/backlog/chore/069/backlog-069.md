# CHR-069: Repair EVD-019 and EVD-025 for cross-corpus coherence

Two of doctrine's own evidence records are incoherent with the corpus next door.
Both were minted before oubliette (`/workspace/oubliette`, formerly checked out
as `microvm-spike`) had a doctrine corpus of its own; oubliette's `ADR-003`
*Evidence has one home, and it says what it does not prove* now sets the rule and
its clause 3 leaves these repairs explicitly to doctrine. They are named in
oubliette's `CHR-012`, which cannot close them from that side.

## 1. `EVD-019` carries a disputed headline and does not say so

`EVD-019`'s datum leads with **8.31 s to a usable fresh capsule**. Oubliette's
`EVD-002` — *Time-to-interactive is about two minutes, not 8.31 s* — establishes
that the two figures come from different runs and must not be quoted as one
result. `EVD-002` names `EVD-019`; `EVD-019` says nothing back, and `QUE-217`
(*which casual capsule backends should complement hardened microVMs*) reads
doctrine's side.

Owed: one sentence in `EVD-019`'s body naming oubliette's `EVD-002` and scoping
the 8.31 s to boot-plus-provision, not to interactivity. Per oubliette `ADR-003`
clause 3 the record is **not** re-measured or superseded — it stands with its
dispute named.

## 2. `EVD-025` cites out of one working tree into another

`EVD-025`'s body cites `../microvm-spike/docs/eval-macos.md` and
`../microvm-spike/docs/plan-b-other-jails.md`. Both are relative paths that climb
out of doctrine's tree and resolve only from a directory neither repo declares,
and both name oubliette under its retired checkout name.

Owed: repo-relative paths qualified by the repo — oubliette's
`docs/eval-macos.md` and `docs/plan-b-other-jails.md` — per oubliette `ADR-003`
clause 2. `EVD-025` itself stays where it is; clause 2 confirms doctrine is its
correct home, since its subject is doctrine's selection between confinement
shapes, not oubliette's mechanism.

## Boundary

Content repair only. The rule that would have prevented it is `CHR-070`; the lint
that would catch the next one is `IMP-440`; the missing `disputes` edge that
forces item 1 to be prose rather than a relation is `IMP-441`.
