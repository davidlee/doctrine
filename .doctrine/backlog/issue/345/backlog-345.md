# ISS-345: just fake-darwin cannot run in the jail, so the cross-platform guard is inoperable where development happens

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`just fake-darwin` (`justfile`) is `cargo check --target aarch64-apple-darwin`.
It is the guard against a Linux-only construct reaching the release matrix's
Apple targets. **It cannot be run where development happens.** Attempted
2026-08-10 in the bubblewrap jail: no Apple target is installed, so the run dies
compiling third-party dependencies (`memchr`, `once_cell`, `typenum`, …) long
before reaching any doctrine crate. A developer sees a wall of unrelated
dependency errors, not the real ones.

This is `mem.fact.tooling.x-bit-is-not-runnability`'s second rule: a tool that
cannot be *invoked* is a defect of the harness, never a verdict about the
subject.

## Provenance and why it is a separate item

Split out of `ISS-343` when that item was resolved (RV-353 `F-4`, 2026-08-11).
`ISS-343` carried two questions:

1. *Does `doctrine-control` build on macOS, or is it excluded from non-Linux
   targets?* — **decided and enacted**: excluded. It left `default-members` and
   the release matrix now names `-p doctrine`. That was the release blocker.
2. *How does the guard become operable?* — **not addressed**, and it is not a
   fact about `doctrine-control` at all. It is a fact about the toolchain
   available in the jail and about a recipe that fails indistinguishably from
   the defect it exists to detect.

Keeping (2) inside a resolved crate-specific issue would have hidden a live
harness defect behind a closed release blocker. The two concerns have different
subjects, different owners and different fixes — separating them is the same
discipline `RV-353` `F-5` holds the capsule programme to.

## What a fix has to decide

Three routes, none obviously dominant:

- **Acquire the target in the jail.** `flake.nix` would carry the Apple std
  component. Cost: jail image weight and a cross-toolchain that is never
  otherwise used.
- **Move the guard to CI, where the target exists.** There is currently no CI
  workflow at all — `.github/workflows/` holds `release.yml` alone — so this
  means minting one, and the signal then arrives at push rather than at edit.
- **Make it announce inability instead of failing as though it had run.** The
  cheapest, and strictly better than today whichever of the above lands: probe
  for the installed target first and exit with a distinct message. This does not
  restore the guard, it only stops the recipe lying about why it failed —
  `ISS-335`/`ISS-336`'s lesson, that a probe which cannot run is otherwise
  indistinguishable from a probe that found nothing.

The third is a strict prerequisite of trusting either of the first two, and can
land alone.

## Blast radius, currently

Low, and lower than when `ISS-343` was raised. `crates/doctrine-control` — the
only Linux-only crate in the workspace — is no longer selected by any default
build, so a bare `cargo build`/`cargo clippy`/`cargo test` and the release matrix
are all portable by construction rather than by this guard holding. The guard
matters again the moment a second platform-sensitive construct enters the root
package, where nothing else would catch it before a release tag.
