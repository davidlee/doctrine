# Review RV-375 — code-review of SL-261

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Scope: the committed `PHASE-01` delta (`da9817e6c`) — the retired wire-key roster.
Attempted ceremony: **moderate** (one phase, 424 lines across 4 files). The
phase sheet, `DEC-278`, `DEC-243`, and design `sec-4` are the reference frame;
the whole `design_run` suite and the published-contract golden were re-run as
the behaviour-preservation gate.

Lines of attack:

1. **EX-1..EX-5 conformance** — static `PAYLOAD`, `&'static TypeContract` walk,
   identity match, refusal variant, rendering order, pins, untouched suites.
2. **The identity claim.** `std::ptr::eq` is only sound if every edge supplies
   one address per contract. Probe `const`/`static` leakage and the
   `closure_types` name-dedup hazard the roster exists to sidestep.
3. **Refusal semantics.** A retired key must fire *before* `UnknownPayloadKey`,
   for its owner only, with the remedy attached; a never-known key must keep
   the old text byte-for-byte.
4. **Rendering.** Retired rows after live rows in prompt/document; JSON member
   only when non-empty so the sealed published contract stays byte-identical.
5. **The pins as invariants.** Are the three claimed invariants the ones the
   code actually checks — including for owners whose key surface is not a
   struct's `keys`?
6. **Contradiction between mechanism and guard.** Where a pin forbids what the
   walk supports, the roster's real domain differs from its documented one.

## Synthesis

**Overall**: acceptable.

**Synopsis**: `PHASE-01` implements `DEC-278`'s mechanism exactly as design
`sec-4` specifies, and the small design deviations it does take are improvements
or are honestly disclosed. The identity claim holds: `PAYLOAD` is `static`, every
walk edge carries `&'static TypeContract`, and `retired_from` matches with
`std::ptr::eq`; the two pins that exist to prove identity (a same-name twin is
unreachable; the same key name under another owner is merely unknown) are
discriminating tests, not theatre — a name-matching implementation fails both.
The refusal fires in the right hole (`walk_keys`, for the owner only) and leaves
`UnknownPayloadKey` byte-identical, which `unknown_key_refusal_is_unchanged_for_
never_known_keys` proves against the new seam. Rendering is disciplined: the
`retired` JSON member is emitted only when non-empty, so the sealed published
contract's golden (`the_published_payload_contract_matches_the_renderer`) stays
green while `RETIRED_KEYS` is empty.

Two `minor` findings were raised, neither a blocker. `F-1` was a real defect in
the artefact — `RetiredKey` had been inserted between `PAYLOAD`'s doc comment and
`PAYLOAD`, so `///`-binding stole the root contract's opening line and gave the
new struct a nonsense first line; fixed in place while the phase is hot, verified
green. `F-2` is a guard/mechanism divergence: the never-live pin reports every
*enum* owner as a fault and justifies it with a false premise, while `walk_keys`
does support enum owners — the roster's checked domain is silently narrower than
its supported one. Disposed `follow-up` to `ISS-478`; it cannot bite this slice,
whose only planned row targets the `ApplyRequest` struct.

Consciously accepted, and explicitly **not** raised: the three pin helpers
(`live_retirements` / `unreachable_owners` / `empty_remedies`) are near-identical
filter/map/collects, each named for its invariant; unifying them behind a
predicate-taking helper would save a handful of lines and lose three useful doc
comments, so the duplication earns its keep — considered, not raised. The walk threading the roster through six functions is noisier than the
phase sheet's `Walk { retired }` alternative would be, but keeps
`refuse_unknown_keys(&Value)`'s signature (`DEC-244`) and adds no state object —
the right trade at this size.

**Haiku**:

A retired key speaks —
identity, not the name, and
the remedy it needs.

