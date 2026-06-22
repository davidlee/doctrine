# Review RV-143 — design of SL-142

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Aspect under trial:** the DESIGN of SL-142 (slice in `design` state; no plan,
no implementation yet). The Inquisition arraigns design intent against canon
(ADR-015) and the authored slice scope (slice-142.md).

**Sanctioned doctrine.** ADR-015 §1 is canon for the priority formula:

> `value_dim = (coefficients.value × value × kind_weight × Σ tag_coefficients) / estimate_midpoint`
> "absent tags / tag storage not yet shipped ⇒ tag contribution is the **identity**."

Identity for a multiplicative term is **×1.0**. The literal term is **Σ** (sum)
of `tag_coefficients`. The identity floor is granted to **absent** tags only —
ADR-015 does not authorise flooring **present** tags whose coefficients sum below
unity.

**Ground-truth code, confessed under cross-examination:**
- `src/priority/config.rs:97` — `tag_coeff` default is **1.0** (`unwrap_or(1.0)`),
  NOT 0.0. Present-but-unconfigured tags already resolve to per-tag identity.
- `src/priority/graph.rs:72` `base_score` — no tag term yet (design not landed).
- `src/facet.rs:19-21` `EntityFacets` — no `tags` field; bears the comment
  "Extended in later slices: tags (SL-136)".
- `src/catalog/scan.rs:260` `read_facets` — present, reads the raw TOML table.

**Lines of interrogation (charges raised below as findings):**
- **C1** — Identity heresy in the SCOPE: slice-142.md Verification preaches
  "zero with empty tags" (×0.0); ADR-015 commands identity (×1.0). Scope
  contradicts canon, and would zero `value_dim` for the untagged majority.
- **C2** — Over-floor clamp in the DESIGN: `tag_sum.max(1.0)` floors present
  tags whose Σ<1.0 up to identity, silently nullifying operator demotion
  coefficients. Correct guard, given `tag_coeff` default 1.0, is
  `if tags.is_empty() { 1.0 } else { Σ }`.
- **C3** — Test arithmetic: design test `["a","b"]` with coeffs 1.5+2.0 expects
  `value_dim ×3.0`, but the design's own `.sum()` yields 3.5. Sum-vs-product
  confusion baked into the verification table.
- **C4** — Normalisation contradiction: design §2 calls `tag::normalize_tag` in
  `read_facets`; slice-142.md Out-of-scope explicitly forbids it
  ("`tag::normalize_tag` is NOT called in the scoring path … passes them through
  unmodified").
- **C5** — Scope/design unreconciled: the design silently corrected the formula
  (×1.0) while leaving the slice scope preaching the heresy (×0.0). Two authored
  artifacts now disagree; neither was reconciled.

## Synthesis — the verdict

**Judgement: HERESY, and of a recursive kind.** SL-142 sets out to wire a single
multiplicative term into `value_dim`, yet across two authored tiers it cannot
agree with itself, let alone with canon. The slice scope (slice-142.md) preaches
that empty tags **collapse `value_dim` to zero** — a doctrine that, were it
implemented, would strike `value_dim = 0` into the overwhelming, untagged majority
of the corpus and lay waste to every priority surface that descends from it. Yet
ADR-015 §1, the sanctioned law, decrees the opposite under oath: *absent tags ⇒
the **identity***. The scope does not merely err; it brands its error "the
identity/floor" while describing 0.0 — calling darkness light. **Let it burn.**

The design.md confessed it knew better — it cites "ADR-015 §1: empty tags ⇒
identity (×1.0)" — and yet committed two fresh heresies in the act of correction:

1. **The unsanctioned floor (F-2, blocker).** `tag_sum.max(1.0)` does not merely
   spare the absent; it **gags the present**. An operator who configures a tag to
   *demote* (Σ < 1.0) is silently overruled — the clamp invents a unity floor that
   ADR-015 never granted. Given `tag_coeff` already defaults to 1.0
   (`config.rs:97`), the righteous guard is `if f.tags.is_empty() { 1.0 } else { Σ }`
   — identity for the absent, the literal Σ for the present.

2. **The forbidden hand of normalisation (F-4, blocker).** design §2 lays
   `tag::normalize_tag` upon the read path that slice-142.md Out-of-scope
   **expressly forbids it to touch**. SL-136 already sanctifies tags at rest; the
   design reaches in and re-consecrates them anyway, in open defiance of its own
   scope.

And the verification table bears false witness (F-3, major): `["a","b"]` at
`1.5 + 2.0` cannot scale by `3.0` under a `Σ` model — the sum is **3.5**. The
number `3.0` is the fingerprint of a *product* that the formula never computes.

The root of all of it (F-5, major) is a **scope left unreconciled**: the design
quietly corrected the formula and walked away, leaving the slice scope still
preaching the canon-violating ×0.0. Two committed authored artifacts now testify
against each other. This is the heresy from which the others descend.

### Ordered penance

1. **Re-enter `/design` for SL-142** (closes F-5, the root). In one reconciliation
   pass make slice-142.md and design.md agree with ADR-015: **empty tags ⇒
   identity (×1.0)** in every clause.
2. **slice-142.md** (F-1): §Risks and §Verification — strike "collapses to zero"
   and "zero with empty tags"; replace with "value_dim **unchanged** with empty
   tags (identity ×1.0)".
3. **design.md §5** (F-2): replace `tag_sum.max(1.0)` with
   `if f.tags.is_empty() { 1.0 } else { f.tags.iter().map(|t| cfg.tag_coeff(t)).sum() }`.
4. **design.md §2** (F-4): drop the `crate::tag::normalize_tag` call — read raw,
   pass through unmodified per scope; *or* reopen scope and admit normalisation
   with rationale. No silent contradiction survives.
5. **design.md Verification** (F-3): correct `base_score_multiple_tags` expected
   scale to **×3.5** (or restate coefficients whose Σ is the intended factor).

### Verification of penance

- A `base_score` test proving **empty tags leave `value_dim` at the no-tag
  baseline** (identity), distinct from the tag-bearing doubling test.
- A `base_score` test proving a **demoting coefficient (Σ < 1.0) lowers
  `value_dim`** — the guard against the `max(1.0)` floor relapsing.
- `grep` proves `normalize_tag` absent from the `read_facets` path (F-4).
- `just gate` green, zero warnings.

### Standing blockers (the close-gate teeth)

F-1, F-2, F-4 remain **open blockers** by design of this trial — they bar SL-142
from advancing while the authored artifacts still preach heresy. They are
disposed (`design-wrong`, route recorded) but **deliberately unverified**: the
heresy is named and sentenced, not yet expunged. Verification awaits the `/design`
reconciliation. **No tolerated taint.** The slice does not pass to `/plan` until
canon and scope speak with one voice.

> **HERESIS URITOR; DOCTRINA MANET**

## Synthesis — second round (post-revision verdict)

The penitent returned with both authored tiers amended. **Five heresies expunged,
verified terminal:**

- **F-1, F-5** — slice-142.md no longer preaches ×0.0; scope and design now speak
  with one voice: empty tags ⇒ identity (×1.0).
- **F-2** — the unsanctioned `max(1.0)` floor is dead; the new
  `tag_term = 1.0 + Σ(coeff - 1.0)` lets a demoting coefficient (Σ<1.0) actually
  lower `value_dim`. A `base_score_demoting_tag` test stands witness.
- **F-3** — the verification table is now self-consistent (`["a","b"]` ⇒ 2.5).
- **F-4** — `tag::normalize_tag` is banished from `read_facets`; raw pass-through
  as scope commands.

**But the correction birthed a fresh heresy (F-6, blocker).** In choosing the
delta-form `1.0 + Σ(coeff - 1.0)`, the slice has **silently superseded its
governing canon**. ADR-015 §1 decrees the literal `× Σ tag_coefficients`; the two
formulas agree only for a single tag and diverge for two or more (two default-1.0
tags: ADR ⇒ ×2.0 inflation, slice ⇒ ×1.0 identity). The slice's form is the
*better* law — the literal Σ's default-tag inflation is a latent defect in the ADR
— **but a slice may not redraft an accepted ADR's formula by fiat.** Per ADR-013,
that governance change routes through a **Revision**. The cure is to AMEND CANON,
not shrink the slice: open a REV against ADR-015 §1 restating the tag multiplier as
`1.0 + Σ(tag_coeff − 1.0)`. **This blocker gates `/plan`.**

A lesser taint also surfaced (F-7, minor): `tag_term` is unguarded against going
negative — two coeff-0 tags yield `1.0 + (−1.0) + (−1.0) = −1.0`, flipping
`value_dim` negative, which is neither identity nor floor. Floor the multiplier:
`(1.0 + Σ(coeff−1.0)).max(0.0)`, with a ≥2-demoting-tag test.

### Standing penance (open, gating)

- **F-6 (blocker, open):** REV against ADR-015 §1 adopting the delta-form — or
  conform the slice to literal Σ (rejected as inferior). Bars `/plan` until canon
  and slice agree.
- **F-7 (minor, open):** `.max(0.0)` on the multiplier + negative-guard test.

The slice is far cleaner than at first arraignment — but it does not pass to
`/plan` while its formula contradicts the very ADR that governs it.

> **HERESIS URITOR; DOCTRINA MANET**

## Synthesis — third round (canon reconciled, trial closed)

The root governance heresy (F-6) is expunged not by shrinking the slice but by
**amending canon to the truer law**. REV-009 (ADR-013) rewrites ADR-015 §1: the
tag term is now `tag_multiplier = max(0.0, 1.0 + Σ (tag_coefficient − 1.0))` —
delta-from-default, identity for absent/all-default tags, demotion honoured, sign
floored. The slice formula and its governing ADR now speak with one voice; F-7's
floor is folded into canon (and the duplicate F-8 withdrawn). slice-142.md Context
and In-scope were re-cited to the REV-009 form so no stale `Σ tag_coefficients`
reference survives.

**All eight charges terminal.** RV-143 `done · await=none`. No standing blockers.
SL-142 is released from the gate — design and canon are coherent; the slice may
proceed to `/plan`.

> **HERESIS URITOR; DOCTRINA MANET**
