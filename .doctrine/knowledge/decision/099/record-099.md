# DEC-099: Interpretation-surface ownership

**Decision.** Doctrine owns the [[interpretation-surface]] taxonomy and enforces
the *universal* trigger classes (3g git-level auto-load, 4 path-shaped data,
5 resource shape). The **client project declares** its instances of the
language-bound classes (1 explicit execution, 2 build-system evaluation,
3 toolchain auto-load). A project with no declaration is **refused**, never
defaulted.

## Why

POL-002: anything the shipped product enforces must rest on contracts doctrine
owns, never on a host project's conventions. Classes 3g/4/5 are git-level and
language-independent — doctrine genuinely owns them. Classes 1–3 are not
knowable from doctrine's side: `build.rs` and `flake.nix` mean nothing to a
TypeScript project, whose triggers are `postinstall`, `.npmrc`, and
config-as-JS.

**Fail-closed on absence is the load-bearing half.** A shipped default list is
coupled to whichever project authored it, and — worse — passes *silently* for a
project whose triggers it does not know. That is precisely POL-002's
invisible-until-the-second-client failure. Strict-and-owned beats
lenient-and-coupled.

## Shape

The declaration is a **default-deny manifest**, the dual of `.worktreeinclude`:
one says what may enter a capsule, the other what may never be interpreted
outside one. Riding that existing seam rather than inventing a second manifest
idiom.

## Status of the shipped form

Where the declaration lives — a `doctrine.toml` block, a dedicated manifest, or
a field on the work contract — is **not decided here**; see
[[interpretation-surface-declaration-home]]. SL-241 implements it as a
rig-local per-fixture file. The shipped form is post-spike REV work, fenced out
by the slice's non-goals.

## Related

- [[interpretation-surface]] — the taxonomy and the five classes.
- [[two-spike-fixtures]] — the Rust/TypeScript pair that tests this split.
- POL-002 — the governing policy.
