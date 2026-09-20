# ISS-472: Two doc comments deny cargo tracks embedded assets

Two shipped doc comments assert that an `install/` edit is invisible to an
incremental build, and both draw a false conclusion from it:

- `src/commands/design.rs:4465` — *"`install/` is a `debug-embed` RustEmbed root
  with no rerun-if-changed, so an incremental build over a corpus edit would
  false-green a gate that read the compiled embed."*
- `src/design_run/artifact.rs:385` — *"`install/` is embedded with no
  `rerun-if-changed`, so an incremental build over an `install/` edit serves
  stale compiled bytes and would false-green this."*

The literal half is true — there is no `build.rs` and no `cargo:rerun-if-changed`
directive. The conclusion is false. `rust-embed` is configured with
`features = ["debug-embed"]` (`Cargo.toml:168`), so the macro expands to
`include_bytes!` per file and **cargo tracks each embedded file as a build
dependency by that route**. Verified against the dep-info: `target/debug/doctrine.d`
lists `install/design-prompts/reviewing.md` and
`plugins/doctrine/skills/plan/SKILL.md` among 97 `install/` entries.

The residual — the thing that genuinely is untracked — is a **newly added** file,
which changes no already-tracked input and so triggers no re-expansion. That is a
narrower and different hazard than the one the comments describe.

## Why it matters

Both comments are the stated rationale for a *correct* decision (read from disk,
not from the embed). A false rationale attached to a correct decision is the
durable kind: it survives review, gets copied into the next phase sheet, and
quietly invalidates any experiment built on it. `SL-251` `PHASE-06` already found
this from the other direction — the observation record notes that an
`install/`-only edit demonstrably *does* trigger a rebuild, so the
perturb-the-file experiment could not distinguish a disk read from an embed read.
One site was corrected there; these two were not.

Nothing needs to change about either decision — only the reason attached to it.

## Provenance

Found while critically re-reading the `RV-371` repairs on `SL-260`, whose `R7`
mitigation makes the opposite (correct) claim. `SL-260` needs no change; the
divergence is between the design and these two comments, and the comments are the
wrong side.
