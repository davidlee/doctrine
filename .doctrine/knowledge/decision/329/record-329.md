Install writes `[mcp_servers.doctrine]` and NOTHING else in `.codex/config.toml`. It does not set `[features] hooks = true` — doctrine would be enabling a client tool's feature switch, a different capability from server registration, resting on a version-varying harness seam (POL-003 facet 2).

Instead the codex arm reads the project file for the hooks answer: `[features] hooks` from the same `.codex/config.toml` the MCP leg reads and writes. The disclosure prints step 1 unless the project file says `true`; an absent key, an absent file, a non-boolean value, or an unparseable document prints the instruction, because the fallback is the instruction and a degraded read must never be read as "enabled".

This reverses the original decision at reconcile. RV-403 F-1 falsified the probe's premise: a pre-trust `codex features list` answers for the user layer, not the project the notice names, so it could suppress step 1 for a project that disables hooks — the exact silent-hooks failure the disclosure exists to prevent. The project file is the honest scope: the file the notice names and the operator can edit. A user-layer override remains the operator's.

The pure/imperative split holds: parsing the file's `[features] hooks` is a pure function; reading it is a fail-soft shell seam. No codex subprocess runs on the install path (POL-002 facet 3: the only declared host dependency is `sh`).

The trust caveat also lands here (POL-003 facet 3): codex loads project-scoped config only for trusted projects, and the skip is silent under a non-interactive run, so the output must not imply the entry is live.
