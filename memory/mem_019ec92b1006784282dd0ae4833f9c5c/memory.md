# Recording doctrine memories

Doctrine memory lets you capture durable facts, patterns, gotchas, and
constraints so future agents retrieve them instead of rediscovering them.

## Recording

- `doctrine memory record --type <type> --key <key> <title>` — mint a uid and
  scaffold a new memory under `.doctrine/memory/items/`. Writes `memory.toml`
  (structured fields: type, scope, trust, git anchor) and `memory.md` (prose
  body), plus a `mem.<key>` symlink alias.
- Use `--path-scope`, `--glob`, `--command` to set retrieval scopes.
- `--global` mints a shipped orientation master (`repo=""`,
  `anchor_kind=none`) into the repo-root `memory/` tree — only for doctrine's
  own shipped corpus, not project-local capture.

## Verification

`doctrine memory verify <uid|key>` attests a memory against the current working
tree. It stamps the verification axis — refuses a dirty tree (no false
attestation). Verified memories carry provenance; unverified memories carry a
caveat in retrieval.

## Retrieval and trust

- `doctrine memory find` — ranked rows, holdback-exempt. Risk is visible.
- `doctrine memory retrieve` — data-not-instruction blocks for agent context.
  Applies a **non-bypassable trust holdback**: low-trust, high-severity memories
  are suppressed. Use `find` to inspect what `retrieve` withheld.
- `doctrine memory show <uid|key>` — full body of one memory.

Records decay — a memory attested against a commit from 50 commits ago carries
a staleness penalty in ranking. Verify after substantial scope churn.

See [[concept.doctrine.memory-model]] for the two-faces model,
[[concept.doctrine.storage-model]] for the storage rule,
and [[fact.doctrine.cli-source-of-truth]] for the CLI.
