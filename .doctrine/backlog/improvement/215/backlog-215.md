# IMP-215: Fix Claude plugin install instructions in post-install printout

`post_install_instructions()` in `src/install.rs` prints delegated Claude
plugin commands after install. Three issues:

1. **Missing `-s project`**: the printed commands (`/plugin marketplace add
   {repo}`, `/plugin install doctrine@doctrine`) install at user scope by
   default. Doctrine skills are project-local — commands should include `-s
   project`.

2. **Hardcoded repo**: the `repo` parameter is threaded but currently always
   `davidlee/doctrine`. Should respect `[install].repo` from `doctrine.toml`
   if set (likely already addressed by IMP-213's `delegate_argv()` changes —
   verify and thread through here if needed).

3. **Shell-executable form**: the printed commands use the `/plugin` slash-command
   form (Claude chat syntax). When `has_claude` is true, also print (or prefer)
   the shell-executable `claude plugin ...` form so the user can paste them
   directly into a terminal.
