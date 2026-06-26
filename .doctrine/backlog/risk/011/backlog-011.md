# RSK-011: Flake edits unvalidatable inside bubblewrap jail — nix absent, broken flake only caught at relaunch

Captured from [[mem.fact.jail.nix-absent-no-flake-eval]].

The bubblewrap jail is built from `flake.nix` externally; `nix` and
`nix-instantiate` are absent inside. A flake edit can only be validated by
relaunching — a broken flake fails to launch and the agent cannot pre-flight
it in-session. Whether there is a practical mitigation (dry-run eval wrapper,
post-relaunch validation hook, in-jail nix packaging) is unverified.

→ RFC-005 (indirect): the jail is the dispatch worker confinement layer
  (ADR-008); this risk is project-local (like Tension 5) but sits at the
  flake layer rather than the cargo build layer. Next revision of RFC-005
  may want to note it alongside OQ-6 as a sibling jail-hygiene concern.
