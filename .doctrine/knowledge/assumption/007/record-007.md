# ASM-007: Interpretation classes exhaustive

**Assumption carried.** The trigger classes in [[interpretation-surface]] are
exhaustive — every way untrusted content acquires agency on the trusted side
falls into one of them.

**Cardinality, stated unambiguously** (RV-340 F-13): **five numbered classes,
one of which has a git-level sub-class** — 1 explicit execution · 2 build-system
evaluation · 3 toolchain auto-load, **3g** git-level auto-load · 4 path-shaped
data · 5 resource shape. Six rows, five classes. Earlier drafts of this record
and of SL-241's design flattened the nesting and kept the count, so a reader who
counted got six and a reader who cited "classes 3g/4" had nothing to parse
against. Since this taxonomy is what clients declare against
([[interpretation-surface-ownership]]), "is there a sixth?" must not be
answerable *yes, it is in the list* for the wrong reason.

## Why we are carrying it rather than proving it

The taxonomy was derived from one language ecosystem (Rust/nix) plus git, then
cross-checked against a second (TypeScript/npm) on paper. Two ecosystems is
weak evidence for exhaustiveness, and a missed class is a silent hole: the rig
audit only refuses what the taxonomy can name.

## What would falsify it

**An independently-derived enumeration of a second ecosystem's interpretation
triggers, classified against the taxonomy, leaving a residue.** Corrected from an
earlier reading of this record (RV-340 F-8), which named SL-241's TypeScript
light fixture ([[two-spike-fixtures]]) as the falsification vehicle. It is not,
and cannot be: *instantiating* a hostile trigger exercises an
**already-classified** class, and the DQ-4 audit greps only for tokens the
declaration already names. A trigger no class describes is, by construction,
one nobody wrote a row for and nobody listed — invisible to both. Every
confirmatory mechanism in a rig is blind to the thing an exhaustiveness claim
needs.

Falsifying an exhaustiveness claim requires **searching**, so the search is a
distinct step (SL-241 design § 5.4, step 0): enumerate the npm/TypeScript
ecosystem's triggers *without consulting this taxonomy* — lifecycle scripts
(`preinstall`/`postinstall`/`prepare`), `.npmrc`, `.nvmrc`, `node_modules/.bin`
on `PATH`, husky, `tsconfig` `extends`, config-as-JS, `package.json` `type` and
`exports` resolution, workspace-protocol links — then classify each. The residue
is the falsifier.

The fixture still earns its keep: it tests the taxonomy's **portability** (does a
known class have a TypeScript instance), which is what DEC-101/DEC-102 want. That
is a different claim from exhaustiveness.

**Empty residue strengthens; it does not discharge.** An independently-derived
list over two ecosystems is modest evidence. Whatever step 0 returns, this record
is to be updated as *strengthened*, never closed.

Falsification is cheap here and expensive later: a class discovered during the
spike amends a knowledge record; a class discovered after the post-spike REV
amends shipped enforcement.

## Related

- [[interpretation-surface]] — the taxonomy under test.
- [[interpretation-surface-ownership]] — what rests on it.
- [[two-spike-fixtures]] — the portability control (not the falsification
  vehicle; see above).
- SL-241 design § 5.4 step 0 — the enumeration that can falsify this.
- RV-340 F-8, F-13.
