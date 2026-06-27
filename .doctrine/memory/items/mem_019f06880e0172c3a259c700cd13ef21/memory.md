# Shipped skills carry no repo-local couplings

Authored skills under `plugins/` (RustEmbed `#[folder = "plugins/"]`,
`src/skills.rs`) are **the product**: `doctrine claude install` materialises
them into arbitrary client projects. POL-002 therefore binds them — a shipped
skill must never load-bear on a convention or entity that only exists in *this*
repo.

Concrete bans when authoring `plugins/**`:

- **No host task-runner commands.** `just check` / `just gate` / `just build` —
  a client has no `justfile`. Call the owned CLI contract instead (e.g.
  `doctrine check quick|gate`, SL-163), which resolves the real command from
  `doctrine.toml [verification]`. A convention may *inform a default*; it must
  never *carry correctness*.
- **No repo-local memory references.** `[[mem.…]]` wikilinks resolve in a client
  only if the key is in the shipped `memory/` corpus (32 keys) or a per-client
  install seed (e.g. `mem.signpost.project.orientation`, seeded via
  `install/manifest.toml`). A bare uid like `mem_019ec65ecbc7` is doctrine-local
  and **dangles on install** — never cite one in shipped prose. To check:
  `comm -23` the skills' referenced keys against `memory_key`s under `memory/`.
- **No repo-local paths / branch names / layout assumptions** (`edge`/`main`,
  `.dispatch/`, etc.) presented as universal.

Verification reflex (POL-002 is VH): for any edit under `plugins/`, ask "would
this resolve in a fresh client with no justfile and only the shipped corpus?"
If no, re-ground on an owned contract or cut it.

Edit `plugins/` (the source), not `.doctrine/skills/` (gitignored installed
copy) — see [[mem.pattern.distribution.skills-source-vs-installed]],
[[mem.signpost.doctrine.skill-masters]]. This is the shipped-skill instance of
the broader rule [[mem.pattern.design.product-not-compromised-by-project-local-ops]]
and the client-facing-content sibling of
[[mem.system.governance.ship-usage-guidance-to-clients]].
