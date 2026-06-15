# Doctrine installation signpost

`doctrine install` bootstraps doctrine into your repo. Run it once per project.

What it does:

- Copies shipped reference docs into `.doctrine/`: `using-doctrine.md` (how to
  *operate* doctrine), `glossary.md` (vocabulary and ids), `doctrine.toml.example`
  (configuration template), and the routing digest.
- Wires the session startup hook so the boot snapshot is `@`-imported into your
  agent harness at session start — the mechanism that keeps the routing table,
  core process, and guardrails current.
- Seeds the `.doctrine/` directory tree with templates and gitignores.

What it does NOT do:

- Create slices, ADRs, or specs — those are authored as you work.
- Modify your agent harness beyond the `@`-import directive.
- Require re-installation after updates — `doctrine boot` regenerates the
  snapshot in place.

After install, the boot snapshot (`.doctrine/state/boot.md`) is the authority
for every session. See [[concept.doctrine.boot-snapshot]] for what it is and
how to keep it fresh.

Idempotent — safe to re-run. The CLI is the source of truth:
`doctrine install --help`.
