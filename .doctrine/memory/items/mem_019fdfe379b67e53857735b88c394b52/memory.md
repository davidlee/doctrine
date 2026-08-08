# A review reading `done` is not concluded

`doctrine review list` / `review status` showing **`done`** does NOT mean the
pass is concluded. Those are **derived** from the findings (ADR-007 D-C8 — a
review's status is never stored, because the storage rule forbids derived data
in authored files). Every finding verified ⇒ `done`.

The design run's `review-disposed {conducted: RV-NNN}` act reads a *different*
fact: `review.concluded` in `review-NNN.toml`, set only by the raiser's verb
`doctrine review conclude <ref>`. Nothing else sets it, and it is absent from a
freshly-authored ledger.

Submit the disposition without it and the refusal is:

    the `review-disposed` act does not correspond to its rule: `RV-349` carries
    no concluded-pass marker Doctrine can read, so a conducted disposition over
    it is a claim about a pass that has not finished

## Why they are deliberately different facts

`done` is about the **findings** — are they all dispositioned. `concluded` is
about the **reviewer** — have they stopped looking. A raiser can conclude a pass
with findings still open (the verb says so: *"Open findings are fine: disposing
them is the responder's work afterwards"*), and a responder can dispose every
finding while the raiser is still mid-round. Neither implies the other.

`read_pass_facts` (`src/review.rs`) reads absence as `false`, which is honest for
a pass nobody closed and conservative for one somebody did — every ledger minted
before the marker existed reads unconcluded, and no migration says otherwise.

## What to do

`doctrine review conclude <ref>` before the disposition. Idempotent, and there is
no unset. Then the `reviewing → locked` acts go in.

Hit on SL-249 (2026-08-08), closing a design run whose RV had read `done` for
days. See also [[mem.signpost.doctrine.review]].
