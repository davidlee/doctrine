# Repo clippy bans HashSet/HashMap (disallowed-types) — use BTreeSet/BTreeMap

`cargo clippy` denies `clippy::disallowed-types` for `std::collections::HashSet`
and `HashMap`: "Use BTreeSet unless hash iteration nondeterminism is intended."
Reach for `BTreeSet`/`BTreeMap` by default (deterministic order, and the lint
passes). Only an explicit, justified need for hash semantics warrants suppression
via `#[expect(clippy::disallowed_types, reason = "…")]`.

Sibling of the other repo deny patterns: `print_stdout`,
`let_underscore_must_use`, `unused`/`dead_code` (see mem.pattern.lint.clippy-denies),
and `#[expect(..)]`-not-`#[allow]` (mem.pattern.lint.expect-not-allow). Gate is
plain `cargo clippy` (bins+lib), NOT `--all-targets`.
