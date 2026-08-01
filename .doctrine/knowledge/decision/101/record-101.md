# DEC-101: Two spike fixtures

**Decision.** The SL-241 rig runs against two fixtures, and every probe row
records which one produced it.

- **Heavy** — a `git clone --no-hardlinks` of this repo (Rust, nix, cargo,
  `just`, edge/main, `(SL-NNN)` commits). Doctrine-installed by construction.
- **Light** — a small **TypeScript** project the rig builds: `package.json`
  with `build`/`clean`/`test`/`lint`/`format`, a trivial red→green test, and
  conventions deliberately unlike this repo's — its own commit style, its own
  trunk name, its own layout. Doctrine-installed, carrying a scratch slice with
  a plan and phases.

## Two reasons, and the second is the important one

**Cost.** P-C3 is 16 rows × 2 harvest mechanisms = 32 cells. Only H11 and H12
need a real build; the rest assert on refusals, refs, and sentinels, which are
independent of tree contents. Heavy-throughout costs ~2 hours and tens of GB
for realism most cells never consume.

**Portability control.** This is the load-bearing reason. A *convention-free*
fixture would only prove the pipeline has no dependency on this repo's habits —
and it can pass **vacuously** (no build system ⇒ verify suite trivially skips ⇒
green means nothing). A *differently-conventioned* fixture proves the pipeline
is correctly parameterised, which is the actual product requirement. Any stage
that passes heavy and fails light has exposed a host-convention dependency —
the cheapest POL-002 audit available, and the direct test of
[[interpretation-surface-ownership]] and [[interpretation-classes-exhaustive]].

## Consequence

The work contract pins `base = <sha>`. The scratch trunk's *name* is a fixture
detail, never a pipeline input — RT-11 already flagged that "accepted canonical
commit" needs naming per project, and an OID is the POL-002-clean form.

## Related

- [[abstract-probe-rows]] — how the two fixtures turn altitude into a
  measurement.
- [[interpretation-surface-ownership]] — what the pair is testing.
- POL-002 · RFC-025 `probe-specs.md` § P-C3.
