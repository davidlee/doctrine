# Doctrine installation signpost

`doctrine install` bootstraps doctrine into your repo. Run it once per project.

What it does:

- Projects the minimal base into `.doctrine/`: `doctrine.toml` (the config
  home) and `project-orientation.md` (the onboarding seed), and appends the
  runtime `.gitignore` entries.
- **Publishes** the shipped reference docs rather than copying them into the
  project — read them on demand with `doctrine library show reference/<name>.md`
  (`using-doctrine.md`, `glossary.md`, `doctrine.toml.example`,
  `routing-process.md`, …). The eager projection base is deliberately minimal.
- Wires the session startup hook so the boot snapshot is `@`-imported into your
  agent harness at session start — the mechanism that keeps the routing table,
  core process, and guardrails current.
- With a harness named (`-a claude`, …), installs that harness's integration
  assets — skills, agents, and hooks. With none named it writes the base alone.

What it does NOT do:

- Create slices, ADRs, or specs — those are authored as you work.
- Materialise the shipped memory corpus — `doctrine memory sync` does that.
- Require re-installation after updates — `doctrine boot` regenerates the
  snapshot in place.

After install, the boot snapshot (`.doctrine/state/boot.md`) is the authority
for every session. See [[mem.concept.doctrine.boot-snapshot]] for what it is and
how to keep it fresh.

Idempotent — safe to re-run. The CLI is the source of truth:
`doctrine install --help`.
