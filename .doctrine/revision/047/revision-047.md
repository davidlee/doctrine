# REV REV-047 — Declare host tool dependencies in POL-002

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

POL-002 governs what shipped doctrine may depend on. It names two prohibitions —
no load-bearing on a host project's **conventions**, no leniency baked in for
**transient local state** — and is silent on host **tools**. The silence was
surfaced while scoping SL-245 (inline terminal diagram rendering, from IDE-046),
which needs the `graphviz` `dot` binary: the policy neither forbids the
dependency nor sanctions it, so the question would be re-argued from scratch by
every feature that wants a host binary.

The gap is real rather than pedantic, because doctrine already has undeclared
host-tool dependencies of both shapes. `git` is a baseline requirement invoked
on default paths throughout the codebase, and a version floor on it is plausibly
coming. `dot` would be the first feature-scoped one.

The rule that covers both is **declaration**, not opt-in. A baseline requirement
cannot be opt-in — doctrine does not function without `git` — so the obligation
it carries is to be *named in the project's stated requirements*. A
feature-scoped capability can be opt-in, and owes that plus a descriptive
failure when absent. Both are the same prohibition seen at two altitudes: never
depend on a host capability silently.

This preserves the policy's existing posture. Facet 1's concern is conventions —
commit style, branch names, layout — which a different host would not share. A
`dot` binary or a terminal protocol is not a convention; it is a capability
either present or absent, and absence is detectable and reportable. Extending
the policy to name that distinction makes explicit what facet 1 already implies,
rather than relaxing it.

### Before — § Statement, opening

> Doctrine is the product; the repository it runs in is merely a client. Anything
> the shipped product **enforces, computes, or depends on** must rest on contracts
> doctrine itself owns — never on a host project's conventions or its transient
> local state. Two prohibitions follow:

### After — § Statement, opening

> Doctrine is the product; the repository it runs in is merely a client. Anything
> the shipped product **enforces, computes, or depends on** must rest on contracts
> doctrine itself owns — never on a host project's conventions or its transient
> local state. Three prohibitions follow:

### Before — § Statement, after prohibition 2

*(nothing — the section ends at prohibition 2)*

### After — § Statement, new prohibition 3

> 3. **No undeclared dependency on a host tool.** Doctrine may depend on
>    capabilities the host provides — a binary like `git` or `dot`, a version
>    floor, a terminal protocol. These are host *capabilities*, not host
>    conventions, and facet 1 does not reach them. What is forbidden is depending
>    on one **silently**. Every such dependency is declared at the altitude
>    matching its reach:
>    - a **baseline requirement**, needed for doctrine to function at all, is
>      named in the project's stated requirements — README and install
>      documentation — including any version floor;
>    - a **feature-scoped capability** is opt-in, so no default path acquires the
>      dependency, and when it is absent the feature fails with a message naming
>      what was missing and what would satisfy it.
>
>    A capability assumed silently is a host coupling by another name.

### After — § Verification, appended challenge

The reviewer's challenge gains a second limb: "does this acquire a host
capability, and if so is it declared — in the stated requirements if baseline,
behind an opt-in with a descriptive absence path if feature-scoped?"

### After — § References, appended

> - IDE-046 / SL-245 — inline terminal diagram rendering; the `graphviz`
>   dependency that surfaced facet (3).
