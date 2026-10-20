install/manifest.toml declares ownership once: a [framework] roots list (framework-owned canon roots: .doctrine/agents, .doctrine/skills, .doctrine/workflows — refreshed, pruned, gitignored) and a [harness.<name>] table per harness naming its link dirs (agents, skills) and, for delegated skills, the single npx target (pi -> universal). Each harness's agent-def set is enumerated from the embed dir install/agents/<harness>/ via embedded_agent_defs, never listed. User-owned is the default (base backings, hymn starters, authored entities) and keeps write-if-absent. The registry replaces DISPATCH_WORKER_AGENT_ASSET{,_PI} and both `_ => pi` wildcards; an unregistered harness installs no defs, disclosed. Rust vocabulary reuses hymns::Provenance{Framework,User}. Applies to --global installs too: prune there uses the same ownership rule. Rejected: Rust const registry (drifts from the manifest gitignore contract, the CHR-043 failure).



Amended by DEC-377: global installs do not prune (they report orphans); unregistered harness names keep their npx skills route; the registry also carries `detect` and `boot`.


Amended by DEC-391 (SL-284), which supersedes DEC-377: there are no delegated skills and no npx target. Every registered harness carries exactly one `skills` link dir (pi, codex and universal share `.agents/skills`) linking the embedded canon. An explicit unregistered `--agent` is refused before any write, not disclosed. The registry's `boot` maps pi → pi (DEC-389). Global installs still do not prune.
