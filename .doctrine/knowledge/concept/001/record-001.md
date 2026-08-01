# CPT-001: Interpretation surface

The set of operations that cause **untrusted content to be interpreted on the
trusted side**. Named while designing the RFC-025 capsule spike rig (SL-241),
where the control plane must ingest results from a hostile execution capsule
without ever giving that capsule's content agency.

## The rule

Danger is not a property of a tool. `cargo test` inside a capsule is the
deliverable's build; `cargo test` on the trusted side over harvested content is
an exploit. Same binary, same command. What differs is the triple:

> **(operation × content provenance × side).**

A forbidden-tool list conflates the tool with the trigger, and can only ever
encode the tools of the project that authored the list. The durable question is
never "is this tool dangerous" but "does this operation interpret content whose
provenance is untrusted, on a side that is trusted".

## The five trigger classes

| # | class | what it is | Rust instance | TypeScript instance |
|---|---|---|---|---|
| 1 | explicit execution | you ran the thing | `cargo test` | `npm test` |
| 2 | build-system evaluation | a "declarative" file that is actually a program | `build.rs`, proc macros, `flake.nix` | `package.json` `scripts`, `postinstall`, config-as-JS (jest/vite/webpack), `tsconfig` `extends` |
| 3 | toolchain auto-load | interpreted by *entering or reading* the tree — nobody decided to run anything | `.envrc` (direnv), `rust-toolchain.toml` | `.npmrc`, `.nvmrc`, `node_modules/.bin` on PATH, husky |
| 3g | git-level auto-load | the same, but git's own | `.git/hooks`, `.gitattributes` filter/diff drivers, `.gitconfig` includes, `core.fsmonitor` | identical |
| 4 | path-shaped data | redirects an otherwise-safe read/write elsewhere | symlinks, `..` in a config field, gitlinks/submodules, absolute manifest paths | identical |
| 5 | resource shape | harms by size/depth, not semantics | oversized blob, deep tree, parser bomb | identical |

Class 3 is the sharpest: nothing was executed on purpose. Classes 1–3 are
language-bound; **3g, 4 and 5 are universal** — git-level and
language-independent.

That universal/language-bound split is what makes the ownership division in
[[interpretation-surface-ownership]] possible without violating POL-002.

## Why it matters here

RFC-025 red-team RT-1 (blocker) is exactly a class-1/class-2 finding: "verify
the candidate on the trusted side" executes worker-authored `build.rs`,
proc-macros, and `flake.nix`. RT-4 is class 4. Probe row H7 is class 5. The
taxonomy is the generalisation those findings were reaching for.

The spike's DQ-4 rig audit reads a per-fixture declaration of classes 1–3
rather than a hardcoded token list, so the same audit runs against a Rust and a
TypeScript project unchanged.

## Related

- [[interpretation-surface-ownership]] — who declares what.
- [[interpretation-classes-exhaustive]] — the assumption this taxonomy carries.
- [[interpretation-surface-declaration-home]] — open: the shipped form.
- RFC-025 `red-team.md` RT-1, RT-4; `probe-specs.md` DQ-4, H6/H7/H9/H11/H12.
- POL-002 — why classes 1–3 cannot ship as a doctrine-authored default.
