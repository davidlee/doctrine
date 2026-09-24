# REQ-479: `boot install` idempotently merges doctrine's owned codex hook registry — a `SessionStart` emit hook and a `PreToolUse` memory-surface hook across codex's `Bash` and `apply_patch` matchers — into `.codex/hooks.json`, preserving foreign entries and refreshing an owned entry when its matcher, command, or handler fields differ; only the memory-surface handlers carry `additionalContextLimit` and `timeout`, both beside `command` rather than on the matcher group.

## Statement

<!-- The sister TOML's `description` field is the primary, normative statement.
     Prose here may elaborate, expand upon, or disambiguate it — never
     duplicate it. -->

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
