# Review RV-367 — code-review of SL-259

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

**Surface reviewed.** `edge` at `fd0e10ecd`; implementation range
`f02d6d7a5..89c483080` (17 files, +3106/−388 under `src/` and `tests/`).
Depth: **full** — a subsystem-scale slice at pre-close, over the whole
accumulated delta.

**Why a second pass.** `RV-366` closed the reconciliation facet and said in its
own synthesis that it "did not re-review the code for quality — that is
`/code-review`'s facet, and the slice has not had an implementation-facet
adversarial pass", flagging one as the User's call. This is that pass. It does
not re-litigate `RV-366`'s nine findings; every raise below is new.

### Lines of attack

1. **The slice's own standard, turned on the slice.** SL-259 exists because
   `apply` emitted rows that were not true and swallowed input it did not act
   on. So: does the landed code emit a row for something that did not happen,
   and does it still swallow anything? Both directions, not just the reported
   one.
2. **What the new prose claims about the new code.** Five of `RV-366`'s nine
   findings were the design lagging the tree. This pass reads the *doc comments
   written in this slice* against the code beside them — a pin cited for a claim
   it does not make is the same defect one layer down.
3. **Diagnostic quality of the refusal surface.** Three phases changed what
   `apply` refuses and how it says so. A refusal that names less than its
   predecessor is a regression even when the exit code is right.
4. **Residue.** Machinery superseded mid-slice: unreachable branches, helpers
   promoted for a reason then not used, tracked items the code silently closed.

### Invariants held

- A change row is emitted **iff** the change it names happened, once.
- Every claim a `SL-259` doc comment makes about a test, a pin or a proof is
  true of that test, pin or proof.
- A refusal names at least what the construct it replaced named.
- `STD-001` single-source; `STD-003` disclosed degradation; `ADR-001` layering.

## Synthesis

*Written at disposition rather than at close: `F-2`, `F-3`, `F-4` and `F-5` are
disposed `fix-now` and their edits are batched to a fresh session on context
budget, so they stand `answered` until the repairs land and the raiser verifies
them. `F-1` and `F-6` are terminal.*

*Closed 2026-09-15. All four repairs landed and were verified as raiser — the
typed parse re-reads the original string (`3444c59cd`), `shaped` fuses its guard
with its ordering (`a93bb8243`), the untagged pin asserts what it is cited for
and is renamed to say so (`2ba9d8bf1`), and `UnknownPayloadKey` renders through
`gate::join` (`06e23cc94`). `just gate` exits 0 at each. Six findings terminal,
pass concluded.*

**Overall: solid.**

### Synopsis

`SL-259` set out to make `apply`'s exit signal trustworthy, and the code that
landed does that. Six findings, none blocking, none touching the four legs'
substance: the contract walk reaches what serde structurally cannot, the state
axis refuses three keys that used to be swallowed, the value axis refuses a term
its event does not declare, a retired change-event token degrades one row
instead of the file, and creation-time `needs` and `lifecycle` reach the same
diff every update does. `just gate` exits 0.

What the findings are *about* is the interesting part, and it is the same thing
`RV-366` found one layer up. That audit raised nine, and every one was the
design's prose lagging the tree. This pass read the doc comments **written in
this slice** against the code beside them, and four of six are the identical
defect at the identical distance:

- `invalidation_rows` argues no dedup is owed because the two act-death paths
  are "disjoint". True within a revision; false across them, which is where the
  doubling lives (`F-1`).
- `walk_enum`'s untagged arm cites `every_untagged_variant_is_a_shape` as
  pinning "a `Shape` carrying no keys", and the pin asserts only `Shape(_)` —
  and the comment predicts the wrong failure mode on top of that (`F-4`).
- `gate::join` was widened with a doc saying a second spelling of
  "comma-separated" is the duplication it exists to prevent, and the same phase
  wrote one (`F-5`).
- `ISS-451` says the key axis "was left out deliberately"; `shaped` closes it,
  and the slice's own test cites `ordered`'s `unwrap_or(usize::MAX)` as the
  thing it replaced (`F-3`).

Two reviews, two layers, one failure mode. The code in this slice is unusually
careful — the seams are right, the pins are mostly load-bearing, the tests are
two-sided where two-sidedness is what makes them mean anything (`ISS-450`'s
control against an existing node, `the_same_keys_are_honoured_at_the_state_that
_honours_them`, the per-event non-empty assertion that stops a cell failing by
skipping). It is the **sentences about the code** that drift, consistently, and
that is worth knowing before the next slice in `RFC-031`.

Two findings were established by execution rather than by reading, which is the
standard `sec-8` sets for this slice and the one a reviewer owes it. `F-1`'s
double `ActInvalidated` came from a scratch probe in `run.rs`'s own suite
(reverted; tree clean) — the same `(ActKind, DesignId)` slot dies at revision 3
by coverage and again at revision 4 by displacement. `F-2`'s lost position came
from the live binary against this repo's own run.

### Standing risks

- **`F-2` is the one with a user-visible edge.** `contract_check` deliberately
  hands every shape fault back to serde (`walk_type`'s `None => Ok(())` arm says
  so by name), and the same phase made serde's answer less locatable by
  switching the typed parse from `from_str` to `from_value`. An apply payload is
  hand-authored and long. Until the one-line repair lands, a type error carries
  no path and no position.
- **`F-4` is latent and stays latent only while the closure holds one untagged
  type.** Its cost when it fires is a false refusal of legitimate input — the
  direction this module's own `walk_map` comment calls "the expensive
  direction", and the inverse of the defect the slice exists to fix.
- **`ISS-454` is a design question wearing an issue.** Leg 2 as stated is not
  violated; the second row is spurious, not missing. Whichever way it settles,
  something currently in the tree is untrue — the row or the comment.

### Tradeoffs consciously accepted

- **`F-6` routes to `IMP-450` rather than being closed here.** A payload that
  satisfies the published contract can still be hard-refused, which is a new gap
  this slice opened. Closing it means deciding how `submission::Declaration
  ::WIRE_KEYS` and `payload_contract::DECLARATION` reconcile — two tables about
  the same keys, each pinned to serde and neither to the other. That is a design
  move, and contract legibility is an explicit Non-Goal.
- **`F-5`'s five incumbent spellings stay.** Only the site written *after* the
  helper was made reachable is in scope; converting the rest silently would be
  the scope creep the finding complains about.
- **Two nit-grade observations were not raised at all**, deliberately:
  `ValueKind::ALL` is production-visible but read only by tests (beside
  `read_rows`, which is `#[cfg(test)]` for exactly that reason), and
  `decimal_digits` / `widest_canonical_id` / their `const _` sit wedged between
  `enum MintKind` and `impl MintKind`. Neither changes anything for anyone;
  raising them would bury the four that do.

### Haiku

```text
the log learned to speak —
now every sentence about it
must earn its own proof
```
