You are operating inside the Cursor harness. Use the tools available via the
Cursor tool-use protocol: `Read`/`Write`/`StrReplace` for file edits, `Grep`/
`Glob` for search, `Shell` for commands. Keep tool outputs concise.

This project's build tooling (`cargo`, `rustfmt`, `clippy`) is provided by a
Nix devshell (`flake.nix`), not the host `PATH`. The Claude/Codex/Pi harnesses
launch inside a `nix develop`-backed bwrap jail automatically; that wiring is
not yet set up for Cursor. Until it is, prefix build/lint/test commands with
`nix develop -c` — e.g. `nix develop -c doctrine check quick`,
`nix develop -c cargo test`. A bare `cargo fmt` / `cargo clippy` will fail
with "no such command" outside the devshell.
