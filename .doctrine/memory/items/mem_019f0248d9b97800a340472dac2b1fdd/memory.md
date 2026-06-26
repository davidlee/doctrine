# nix is absent inside the bubblewrap jail — flake edits can't be eval-checked in-session

Jail built from flake.nix externally; no nix/nix-instantiate inside. A flake edit can only be validated by relaunching (broken flake fails to launch) — defer flake-eval verification to post-relaunch.
