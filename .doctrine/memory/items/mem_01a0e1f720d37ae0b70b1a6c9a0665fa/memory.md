Verified against codex-cli 0.155.1 (2026-09-27), in the jail.

codex merges config in layers: the user layer ($CODEX_HOME/config.toml) always,
and a project layer (.codex/config.toml under the project root derived from cwd)
**only when that project is trusted**. Consequence for any probe that asks codex a
question with `cwd` bound to a project that has not yet been trusted: the answer
is the *user* layer's, and the project's own file is not consulted at all.

The 2x2, cwd = a project whose .codex/config.toml is exactly
`[features]\nhooks = false`:

  CODEX_HOME config with no [projects] entry                -> hosts  stable  true
  CODEX_HOME config + [projects."<dir>"] trust_level=trusted -> hooks  stable  false

Same cwd, same project file, same binary; only the trust entry moved the answer.
The same gate applies to whether codex uses trusted hook-file hashes
([hooks.state] trusted_hash entries).

Why it matters: `doctrine boot install --agent codex` runs `codex features list`
with cwd = install root to decide whether to print the "ensure [features] hooks =
true in .codex/config.toml" step (SL-271 PHASE-03, DEC-329). At install time the
project is by definition untrusted - the notice's own next step is "accept the
project trust prompt" - so the probe reports the user layer and can omit the
instruction for a project that disables hooks. Raised as SL-271's RV-402 F-1
(blocker, design-wrong).

Corollaries:
- To learn what a *project* file says, read the file; do not ask codex before
  trust.
- Any probe whose answer is a layer-merged value must state which layer it read,
  or the disclosure it feeds can be wrong in the case it exists for.
- `codex features list` DOES read config (a CODEX_HOME config with
  `[features] hooks = false` reports false), so the failure is the trust gate, not
  a failure to read config.
- A probe also mutates user-global codex state: running `boot install --agent
  codex` in a scratch dir advanced the mtime of ~/.codex/tmp/arg0.


## Related

- [[mem.fact.codex.local-mcp-no-trust-step]] — the same trust gate seen from the
  MCP-config side (it records that a fresh untrusted project ignores
  `.codex/config.toml` entirely; this record adds that `codex features list`
  answers with the *user* layer in that state, which is what makes the probe
  misleading rather than merely empty).
- [[mem_019fe687859a7e73a06fc1b1881ff80b]] — an absence-asserting probe must
  convict a surface it could not read: the same class of defect, one layer up
  (here the surface *was* read, but not the layer the question was about).
