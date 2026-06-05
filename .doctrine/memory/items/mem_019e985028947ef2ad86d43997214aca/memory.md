# Adding a flag to a CLI command handler can trip clippy bool/arg ceilings

Adding a bool param to a doctrine CLI handler trips too_many_arguments/fn_params_excessive_bools; fix = args struct (RecordArgs/InstallArgs)

## What

The repo's clippy config (`-D clippy::all`, `-D clippy::pedantic`) denies:

- `clippy::too_many_arguments` — fires at **>7** function parameters.
- `clippy::fn_params_excessive_bools` (pedantic) — fires at **>3** `bool`
  parameters.

A command handler like `skills::run_install` sat just under both ceilings
(7 args, 3 bools). Threading one new flag (`only_memory: bool`) tipped it to
8 args / 4 bools → the build fails the lint gate even though the logic is fine.

## How to apply

Bundle the arguments into a borrow-holding args struct, leaving `path` as a
separate param. This is the established house pattern — `memory::RecordArgs` +
`run_record(path, args: &RecordArgs)` — now mirrored by `skills::InstallArgs` +
`run_install(path, args: &InstallArgs<'_>)`:

```rust
pub(crate) struct InstallArgs<'a> {
    pub(crate) agents: &'a [String],
    pub(crate) skills: &'a [String],
    pub(crate) domains: &'a [String],
    pub(crate) only_memory: bool,
    pub(crate) global: bool,
    pub(crate) dry_run: bool,
    pub(crate) yes: bool,
}
```

The lints count **function params**, not struct fields, so the struct may hold
many bools freely. Reach for this pre-emptively when a handler is near the
ceiling — don't `#[allow]` the lint (see [[mem.pattern.lint.expect-not-allow]]).
Related: [[mem.pattern.lint.clippy-denies]], [[mem.pattern.lint.string-build-no-push-format]].
