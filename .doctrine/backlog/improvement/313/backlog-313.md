# IMP-313: library tree drops prefix-colliding published addresses

Surfaced by the SL-227 post-implementation audit (RV-302 F-9, minor; external
adversarial pass by codex/GPT-5.5). **Latent** — not currently triggered.

## Defect

`src/commands/library.rs` `TreeNode` is `Branch(BTreeMap) | Leaf(EntryView)` and
cannot hold an entry AND children at one node. `TreeNode::insert` therefore
silently drops one entry whenever a published address is a prefix of another
(e.g. both `a` and `a/b` declared):

- inserting the `[last]` segment `a` does `children.insert(a, Leaf)`, overwriting
  any existing `Branch` at `a` (drops `a/b`'s subtree); and
- inserting `a/b` after `a` descends via `or_insert_with` into the existing
  `Leaf` and hits the `TreeNode::Leaf(_) => {}` no-op arm (drops `a/b`).

Either order silently loses a valid, admitted, prefix-filtered entry from
`library tree` output. `library list` (flat) is unaffected.

## Why latent

All 73 current publication addresses are files; none is a parent-prefix of
another, so no collision fires today. A future manifest that publishes both a
container address and a child under it would trigger it.

## Fix

Let a tree node carry both an optional leaf payload and children, or detect a
prefix collision at insert and surface it (never silently drop). Add a regression
test with a colliding address pair.

## Provenance
Audit RV-302 (SL-227) F-9. See `.doctrine/review/302/`.
