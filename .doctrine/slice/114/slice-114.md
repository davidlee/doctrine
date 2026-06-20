# Consolidate per-kind canonical_id onto shared listing helper

## Context

`listing.rs:36` already provides the generic
`canonical_id(prefix: &str, id: u32) -> String` — prefix + 3-digit zero-pad, the
project's universal id format. The 2026-06-19 architecture audit found at least
eight kinds redefine their own `canonical_id` instead of delegating to it:

- `knowledge.rs:125`, `requirement.rs:244`, `backlog.rs:155`, `spec.rs:1160`,
  `review.rs:866`, `rec.rs:376`, `revision.rs:476`, `slice.rs:759`.

**Drift note (2026-06-20, design):** by design time, four of the eight already
delegate (`rec`, `slice`, `revision`, `review`, and the spec **free fn**) — prior
work consolidated them. Four raw `format!` sites remain: `requirement.rs:244`,
`knowledge.rs:125` (method), `backlog.rs:156` (method), `spec.rs:106` (method).
`spec.rs` additionally carries a same-output duplicate — a method (`:106`) and a
delegating free fn (`:1164`); the design collapses these to the method (D1).

Pure copy-paste of the same `format!("{:03}")` logic. Low-severity but it spreads:
each new kind copies the pattern, and any change to the id format (padding width,
separator) means editing N sites. This is the cheapest DRY win in the audit.

## Scope & Objectives

- Make each per-kind `canonical_id` a thin delegation to
  `listing::canonical_id` with the kind's prefix constant (single source of the
  format), or remove the per-kind wrapper entirely where call sites can use the
  shared helper directly.
- Keep each kind's prefix as the one kind-specific input; the formatting lives in
  one place.
- Preserve every existing `canonical_id_*` unit test's behaviour — output strings
  are unchanged.

Closure intent: `grep 'fn canonical_id'` shows no module re-implementing the
`format!("{:03}")` body; all produce ids via `listing::canonical_id`; existing
id-format tests stay green unchanged.

## Non-Goals

- The id **parse** side (`strip_prefix` + digit parse, `id_from_fk`) — a separate
  potential consolidation; flag as follow-up, do not bundle.
- Changing any id format, prefix, or padding width.
- The entity mutation seam → SL-113.

## Summary

Collapse 8+ copy-pasted `canonical_id` implementations onto the existing
`listing::canonical_id` so the id format has one home. Trivial, mechanical,
behaviour-preserving.

## Follow-Ups

- Consider a matching shared id-**parse** helper (the `strip_prefix`/`id_from_fk`
  pattern is similarly scattered) — its own slice or fold into this one at design
  time if the surface is small.
