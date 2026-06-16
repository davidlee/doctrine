# clap ColorChoice parser case sensitivity

`clap::ColorChoice` implements `ValueEnum`, but derived argument parsing is
case-sensitive unless the `Arg` enables `ignore_case`.

Evidence checked during `RV-046`:

- `clap_builder-4.6.0/src/util/color.rs`: `ColorChoice` possible values are
  `auto`, `always`, and `never`.
- `clap_builder-4.6.0/src/builder/value_parser.rs`: enum parsing uses the
  argument's `ignore_case` setting, which defaults to false.

When adding a `--color` flag as `clap::ColorChoice`, either:

- document and test lowercase-only `auto|always|never`, or
- add `#[arg(ignore_case = true)]` and test mixed/uppercase values.
