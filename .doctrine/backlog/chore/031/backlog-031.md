# CHR-031: Claude plugin manifest versioning, parity, and spec compliance

The three doctrine Claude plugins (`doctrine`, `doctrine-memory`,
`doctrine-partner`) have stale/pinned manifests and the marketplace entry is
out of sync with them.

## Current state

### plugin.json (all three plugins)

```json
{ "name": "doctrine", "version": "0.1.0", "description": "..." }
```

- `version: "0.1.0"` is pinned — never bumped on releases. Claude Code treats
  this as "no update available" since the version string never changes.
- Missing optional metadata: `displayName`, `author`, `homepage`, `repository`,
  `license`, `keywords`

### marketplace.json

```json
{
  "name": "doctrine",
  "owner": { "name": "doctrine" },
  "plugins": [
    { "name": "doctrine", "source": "./plugins/doctrine", "description": "..." },
    { "name": "doctrine-memory", "source": "./plugins/doctrine-memory", "description": "..." },
    { "name": "doctrine-partner", "source": "./plugins/doctrine-partner", "description": "..." }
  ]
}
```

- Plugin descriptions differ from their plugin.json counterparts (e.g. doctrine
  marketplace entry mentions "backlog triage" and "handover" which plugin.json
  doesn't; plugin.json mentions "memory capture/retrieval" which marketplace
  doesn't)
- No `version` field per entry — if added, marketplace entry takes precedence
  over plugin.json (Claude docs: "The same field can appear in a plugin's
  marketplace entry, where it takes precedence over the value in plugin.json")
- No `homepage`, `repository` fields

## Required changes

### 1. Decide versioning strategy ✅ DECIDED

**Decision**: track the same semver as the doctrine crate. Plugin `version`
field mirrors the crate version from `Cargo.toml` (currently `0.9.1`). Every
crate release bumps the plugin version identically.

### 2. Sync version from Cargo.toml to all plugin.json manifests

Set `version` in all three `plugin.json` manifests to match the crate version.
Implement a build step or `just` recipe that syncs `Cargo.toml`'s `version`
into all `plugins/*/plugin.json` manifests so it can't drift again.

### 3. Bring marketplace.json entries into parity with plugin.json

- Unify descriptions: single source of truth (probably marketplace overrides)
- Consider adding `version` to marketplace entries (takes precedence)
- Add optional fields: `homepage`, `repository`, `author`

### 4. Audit manifests against Claude plugin schema

Per `docs/claude/plugins-reference.md` § Plugin manifest schema:

Required: `name` ✅ (all three)
Optional to add: `displayName`, `author`, `homepage`, `repository`, `license`,
`keywords`, `$schema`

### 5. Wire into `just release`

The version-sync step must be part of the `just release` pipeline — bump crate
version, sync into `plugins/*/plugin.json`, commit, tag.

## Dependencies

- Relates to IMP-215: `claude plugin update` during `doctrine install` is
  version-gated — won't pull updates unless version string changes.
