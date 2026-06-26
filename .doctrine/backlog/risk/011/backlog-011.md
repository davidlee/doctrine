# RSK-011: Flake edits unvalidatable inside bubblewrap jail — nix absent, broken flake only caught at relaunch

Captured from [[mem.fact.jail.nix-absent-no-flake-eval]].

The bubblewrap jail is built from `flake.nix` externally; `nix` and
`nix-instantiate` are absent inside. A flake edit can only be validated by
relaunching — a broken flake fails to launch and the agent cannot pre-flight
it in-session. Whether there is a practical mitigation (dry-run eval wrapper,
post-relaunch validation hook, in-jail nix packaging) is unverified.
