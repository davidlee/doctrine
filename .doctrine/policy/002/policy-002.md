# POL-002: Platform independence from host-project conventions and state

## Statement

Doctrine is the product; the repository it runs in is merely a client. Anything
the shipped product **enforces, computes, or depends on** must rest on contracts
doctrine itself owns — never on a host project's conventions or its transient
local state. Three prohibitions follow:

1. **No load-bearing on host conventions.** A platform mechanism must not depend
   on a host's commit-message style, branch names, directory layout, tagging
   habits, or any other local convention. Conventions may *inform* a default;
   they must never *carry* correctness.
2. **No leniency baked in for transient local state.** When a one-time, project-
   local data or operational condition tempts permanent leniency or complexity
   in the shipped library, refuse it. Keep the durable code strict and clean; fix
   the transient local state out-of-band.
3. **No undeclared dependency on a host tool.** Doctrine may depend on
   capabilities the host provides — a binary like `git` or `dot`, a version
   floor, a terminal protocol. These are host *capabilities*, not host
   conventions, and facet (1) does not reach them. What is forbidden is
   depending on one **silently**. Every such dependency is declared at the
   altitude matching its reach:
   - a **baseline requirement**, needed for doctrine to function at all, is
     named in the project's stated requirements — README and install
     documentation — including any version floor;
   - a **feature-scoped capability** is opt-in, so no default path acquires the
     dependency, and when it is absent the feature fails with a message naming
     what was missing and what would satisfy it.

   A capability assumed silently is a host coupling by another name.

## Rationale

Doctrine ships to projects with arbitrary commit styles, branch topologies, and
pre-existing data shapes. A mechanism that greps for `(SL-NNN)` commit scoping,
or relaxes an invariant to accommodate entities seeded before a schema grew,
silently couples the product to one client and breaks for every other. The
coupling is invisible until a second client adopts doctrine — the most expensive
time to discover it. Strict-and-owned beats lenient-and-coupled: it fails loudly
at home, and it is the only thing that ports.

## Scope

Applies to all shipped doctrine behaviour — CLI, engine, MCP surface, gates, and
any computed/enforced invariant. It does **not** constrain host projects' own
choices: this repo may keep `(SL-NNN)` conventional commits, `just gate`, the
edge/main split, and so on — those are client habits, and the policy only forbids
the *product* from load-bearing on them. Out-of-band fixes to local state
(one-time corpus rewrites, data-only backfill diffs) are the sanctioned escape
hatch, not a `migrate` verb or create-on-absent leniency in the library.

## Verification

VH — by design review and audit. A reviewer challenges any new mechanism with:
"does this depend on a convention or local-state shape a different host would not
share?" If yes, it must be re-grounded on an owned contract or moved out-of-band.
The conformance work in RFC-004 (record source-delta SHAs rather than grep
`(SL-NNN)`) is the originating worked example.

For facet (3) the challenge has a second limb: "does this acquire a host
capability, and if so is it declared — in the stated requirements if baseline,
behind an opt-in with a descriptive absence path if feature-scoped?"

## References

- RFC-004 — path-intent selector; surfaced facet (1) while resolving slice-delta
  computation (OQ-11a): recorded SHAs over `(SL-NNN)` grep.
- SL-048 — "the cut": tier-1 storage migrated by a throwaway one-time corpus
  rewrite, zero permanent migration surface. Precedent for facet (2).
- SL-060 (C6) — origin of "keep shipped doctrine strict/clean; fix transient
  project-local state out-of-band."
- `mem.pattern.design.product-not-compromised-by-project-local-ops` — the memory
  this policy promotes and unifies with facet (1).
- IDE-046 / SL-245 — inline terminal diagram rendering; the `graphviz`
  dependency that surfaced facet (3). Amended by REV-047.
