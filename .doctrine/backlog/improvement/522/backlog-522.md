# IMP-522: Doctrine quotient forms

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`pi_agent` delegates in doctrine had no Rust toolchain (the global quotient
forms are satan's, built with satan's packages) and could not write at the repo
root (quotient ISS-001: the server's launch-dir profile layer is a protected
store). Interim fix, done 2026-10-24:

```
flake.nix  quotientForms → packages.doctrine-quotient-{read,write}
           = mkJail makeJailedPi + form-1 options (root bind at $PWD, session
             bind, tmpfs /tmp, chdir, die-with-parent) + CARGO_BUILD_RUSTFLAGS fwd
just quotient-forms  → ~/.local/state/quotient/forms/doctrine-{read,write}
~/.config/quotient/doctrine.policy.json   v2; root ~/dev/doctrine; doctrine forms
~/.claude.json (local scope, this project) quotient entry: QUOTIENT_POLICY set,
           launched from ~/.config/quotient/doctrine/ (outside the repo)
```

- `flake.lock`: `agents` bumped to 269b59ad (satan's pin) for
  `mountLaunchDir` / `workspaceDepsFromEnv`.
- Verified: cargo/rustc/mold/cc on PATH; read form denies root writes; write
  form writes at the repo root; a repo-root `write` call through the new server
  ran `cargo check`; the jail sees its own home's quotient state, not the host
  evidence store.
- Cost: the server's default cwd is no longer an allowed root, so every call
  passes `cwd` (CLAUDE.md § Delegating to pi).
- Known noise: pi warns `EROFS` locking `.pi/settings.json` under a read form;
  bwrap warns about the duplicate `--chdir`.

Retirement: CHR-183.
- Post-restart: the live `pi_agent` `write` call at the repo root ran with doctrine's cargo (2026-10-24).
