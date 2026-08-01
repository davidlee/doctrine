# ASM-008: Interpretation-surface responsibility split

**Assumption carried.** [[interpretation-surface]]'s classes divide cleanly by
*who owns them*:

| classes | property | owner |
|---|---|---|
| 1 explicit execution · 2 build-system evaluation · 3 toolchain auto-load | **language-bound** — the set differs per ecosystem | the **client project** declares them |
| 3g git-level auto-load · 4 path-shaped data · 5 resource shape | **universal** — git-level and language-independent | **doctrine** ships them |

Replaces [[interpretation-classes-exhaustive]] (ASM-007), which claimed the class
list was *complete* and was invalidated by SL-241 PHASE-04 step 0. This claim is
deliberately a different shape: it is about the taxonomy's **structure**, not its
extent, and a newly discovered class does not threaten it — a new class simply
lands on one side of the split or the other.

## Why this is the claim worth carrying

It is the one the design actually load-bears on. POL-002 forbids the platform
from encoding a host project's conventions, and classes 1–3 are exactly such
conventions: `cargo` / `build.rs` / `flake.nix` for one project, `npm` /
`postinstall` / `tsconfig` for another. Doctrine cannot ship them without
becoming a Rust tool. The split is what makes a single shipped audit possible —
it reads whatever the client declared for 1–3, and enforces 3g/4/5 itself.

Exhaustiveness, by contrast, was load-bearing for one closure sentence in SL-241
design § 9 and nothing else.

Applying the diagnostic ASM-007's body records — *if this were false, what would
we do differently?* — the answer here is: redesign who declares what, and
probably retract the claim that one audit is portable. That is a real
consequence, which is what an assumption should have.

## What would falsify it

- A trigger that is **language-bound but not client-declarable** — doctrine would
  have to hardcode a host-project convention to catch it, violating POL-002.
- A trigger that is **universal but not doctrine-shippable** — enforcing it would
  require knowing the client's toolchain.

Explicitly **not** falsifying: discovering a new class. Placing it on one side of
the split is the split working, not failing.

The live test is SL-241's two fixtures: the same DQ-4 audit must run unchanged
against a Rust project and a TypeScript project, reading only what each declared.

## What already supports it

- **The residue survived it.** All four of ASM-007's falsifiers place cleanly:
  R2 (terminal escapes), R3 (metadata interpolation) and R4 (non-inert parsing)
  are universal; R1 (LLM as interpreter) is language-independent. A taxonomy
  failure passed through the split without moving it.
- **Three unforced cross-ecosystem parallels** from step 0: `binding.gyp` ≡
  `build.rs`, `packageManager` ≡ `rust-toolchain.toml`, `.mise.toml` ≡ `.envrc`.
  Derived from opposite ecosystems, landing in the same class without stretching.
- **Independent arrival at the same shape.** RFC-012 § 2 *Resource taxonomy by
  responsibility* is a Doctrine-duty / Project-duty matrix built for containment
  under parallel fan-out — a different problem, same division (operator ruling,
  2026-08-02). Two independent derivations of a shared-responsibility split is
  better evidence than either alone.

## Caveat carried from the residue

The split says who *declares*, not who can *enforce*. R1 and R2 name interpreters
downstream of the toolchain — a model, a terminal, a human reviewer — where the
owning side is clear (doctrine, since they are universal) but no declaration
mechanism exists on either side. Tracked as
[[downstream-interpreter-classification]] (QUE-203). If that question resolves
badly it constrains what the split can *deliver*, without making the split wrong.

## Related

- [[interpretation-surface]] — the taxonomy whose structure this describes.
- [[interpretation-classes-exhaustive]] — ASM-007, the invalidated predecessor.
- [[interpretation-surface-ownership]] — DEC-099, the ownership decision this
  assumption underwrites.
- [[downstream-interpreter-classification]] — QUE-203, the open caveat.
- POL-002 — platform independence from host-project conventions.
- RFC-012 § 2 — the same split, arrived at independently for containment.
- SL-241 `.doctrine/slice/241/step0-enumeration.md` — the evidence.
