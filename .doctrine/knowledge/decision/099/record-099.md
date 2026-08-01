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

## Amendment (SL-241 internal adversarial pass, finding A5)

The original wording implied the declaration *enforces* all three language-bound
classes. It does not, and the distinction matters:

| class | how it is actually prevented |
|---|---|
| 1 explicit execution | **enforced by the declaration** — the `exec:` token list drives the DQ-4 audit over trusted-side scripts |
| 2 build-system evaluation | **prevented structurally** — the trusted side never materialises a harvested tree; it handles objects only |
| 3 toolchain auto-load | **prevented structurally** — same reason |

So the declaration's class-2/3 half (`interpret:`) drives **probe
instantiation** — where to plant a hostile payload per ecosystem — not
enforcement. There is nothing for a glob to guard on a side that never checks
out the tree.

This is a *better* outcome than the original framing: structure beats
declaration, because it cannot be misconfigured. But the split must be stated
honestly, or a reader will assume a `interpret:` entry is load-bearing for
safety when it is load-bearing only for test coverage. The ownership decision
above is unchanged; only the enforcement claim narrows.

## Amendment 2 (SL-241 external inquisition, RV-340 finding F-5)

**Fail-closed on absence does not defend against substitution**, and the
original wording made absence do all the work. The declaration is a
trust-bearing input, so its *read side* is part of this decision and was
missing:

> The interpretation-surface declaration is read from the **contracted base
> `B`** — content the control plane itself pinned — never from a harvested
> result `S`, and never from any location the executing party can write.

Without it, a worker that edits the declaration in its own clone can narrow
`exec:` so the DQ-4 audit passes vacuously, or replace `verify:` so the attested
run in the verify capsule attests nothing — escalating its own contract. That is
what the executing *process* can do, not merely what it can say, so it sits
inside the threat model this decision is subject to (CON-005), not outside it.

This amendment is stated **here**, in the record that governs the declaration,
and not only in the slice design that discovered it: the ruling binds every
future consumer of DEC-099, including the shipped form chosen by
[[interpretation-surface-declaration-home]].

## Status of the shipped form

Where the declaration lives — a `doctrine.toml` block, a dedicated manifest, or
a field on the work contract — is **not decided here**; see
[[interpretation-surface-declaration-home]]. SL-241 implements it as a
rig-local per-fixture file. The shipped form is post-spike REV work, fenced out
by the slice's non-goals.

Amendment 2 makes that question *safe to defer*. Two of its three candidates
live inside the repository a capsule clones, so without the read-from-`B`
invariant the choice of home would silently decide a security property; with it,
all three are sound and the choice turns on ergonomics as intended.

## Related

- [[interpretation-surface]] — the taxonomy and the five classes.
- [[two-spike-fixtures]] — the Rust/TypeScript pair that tests this split.
- POL-002 — the governing policy.
