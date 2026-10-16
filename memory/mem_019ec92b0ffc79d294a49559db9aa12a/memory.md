# Doctrine installation signpost

`doctrine install` bootstraps doctrine into your repo. Run it once per project.

What it does:

- Prints one overview of everything it will do, then asks `Proceed? [y/N/d]`
  (`d` shows every step). `-y` skips the question; `--dry-run` prints the
  overview and details and writes nothing.
- Projects the minimal base into `.doctrine/`: `doctrine.toml` (the config
  home) and `project-orientation.md` (the onboarding seed), and appends the
  runtime `.gitignore` entries.
- **Publishes** the shipped reference docs rather than copying them into the
  project — read them on demand with `doctrine library show reference/<name>.md`
  (`using-doctrine.md`, `glossary.md`, `doctrine.toml.example`,
  `essentials.md`, …). The eager projection base is deliberately minimal.
- Syncs the shipped memory corpus (`doctrine memory sync` is one of its legs).
- Wires the session startup hook so the boot snapshot is `@`-imported into your
  agent harness at session start — the mechanism that keeps the routing table,
  core process, and guardrails current.
- Installs each harness's integration assets — skills, agents, and hooks. Harnesses
  are auto-detected from their markers (`.claude`, `.codex`, `.pi`, `.agents`);
  name them with `-a <harness>` (repeatable), or it asks once when none is found.
- Records the installed version (`installed-version` under `[install]` in
  `.doctrine/doctrine.toml`, plus a per-checkout marker) and prints a
  path-limited `git add` / `git commit` hint for the files it touched.
- Does not project hymn starters. To edit a framework hymn slot, run
  `doctrine prompt customise <band/label>` after install.

What it does NOT do:

- Create slices, ADRs, or specs — those are authored as you work.

Re-run `doctrine install` after upgrading doctrine: the version advisory flags a
repo installed by an older version. `doctrine boot` alone regenerates only the
boot snapshot.

After install, the boot snapshot (`.doctrine/state/boot.md`) is the authority
for every session. See [[mem.concept.doctrine.boot-snapshot]] for what it is and
how to keep it fresh.

Idempotent — safe to re-run. The CLI is the source of truth:
`doctrine install --help`.
