# ISS-272: slice conformance reports a declared .doctrine entity selector as undeclared

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced at the SL-231 close, while hand-deriving the honest cross-match the
audit had to fall back on.

## Observed

`.doctrine/slice/231/slice-231.toml` declares:

```toml
[[selector]]
selector = ".doctrine/adr/001/layering.toml"
intent = "design-target"
```

The file *is* in the slice's delta (PHASE-01 added the `observation = "leaf"`
tier row). Yet `doctrine slice conformance 231` lists
`.doctrine/adr/001/layering.toml` under **undeclared**, not conformant.

## Why it is not the known skew

Two other `.doctrine/`-rooted paths in the same delta, declared with an
identical selector shape, come back **conformant**:

- `.doctrine/governance.md`
- `.doctrine/rfc/011/rfc-011.md`

So this is neither a blanket `.doctrine/**` exclusion nor the ISS-268
boundary-row attribution defect (which mis-attributes *which files* are in the
delta; here the file is correctly in the delta and the selector exists — the
match itself fails). `.gitignore`, also a dotfile-rooted literal selector,
matches fine too.

The distinguishing feature of the failing path is that it is an **entity file**
under a numbered entity directory (`adr/001/…`). Worth checking whether the
matcher routes entity-shaped paths through a different classification before the
selector comparison.

## Cost

`slice conformance` is the mechanical drift signal `/audit` and `/close` lean
on. A declared path reported undeclared is a **false positive on the
conservative side** — it does not hide drift, but it further erodes trust in an
output already made untrustworthy by ISS-268/269/271, pushing every close to
hand-derive `git diff --name-only main...<bundle>` against the registry. That
hand-derivation is exactly what the verb exists to remove.

## Repro

```bash
doctrine slice conformance 231 | grep 'layering'
# → appears under `undeclared (N)`, though slice-231.toml declares it
```

Related: ISS-268 (boundary-row attribution), ISS-269 (linked-worktree phase
status), ISS-271 (`verify-vt` primary-tree skew). Same cluster of
"conformance/verification output cannot be trusted at face value".
