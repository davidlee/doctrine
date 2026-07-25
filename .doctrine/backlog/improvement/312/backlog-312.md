# IMP-312: SL-227 library and install test-fidelity hardening

Surfaced by the SL-227 post-implementation audit (RV-302). One residual
test-fidelity gap — the runtime behaviour is verified-correct, but the test that
guards it is weaker than the invariant it claims. Does not block SL-227 close;
carried here as hardening.

> **Trimmed 2026-07-25 (post-audit fix-now).** The operator elected to fix
> **F-6/F-7/F-8 now**, before SL-227 close, on candidate `cand-227-fix-001`
> `30e538be` (`fix(SL-227): harden reachability gates + VT-6 no-write`). Those three
> items are removed below; only **F-10** remains deferred here. See the RV-302
> post-verification amendment (`.doctrine/review/302/`). F-9 → IMP-313 (untouched).

## Items

- **RV-302 F-10 (minor) — VT-5 proves detection, not adapter installation.**
  `src/install.rs` `detect_agents_is_independent_of_the_base_projection_flip`
  asserts only that `detect_agents()` returns `["claude"]` after a base install;
  it never fires the forward per-agent adapter-install leg or checks an adapter
  lands. Extend it to run the forward leg and assert an adapter file lands
  post-flip (the actual NF-004 claim).

- **RV-302 F-6 (nit) — duplicated base-backing literal.**
  `src/publication.rs` `published_set_covers_the_full_projection_complement`
  hardcodes `const BASE_BACKINGS: [&str; 3]`, duplicating the authoritative
  `install/manifest.toml [base].backings` (STD-001). Derive the base from the
  manifest like the crux gate does (fails safe today — false-red on a 4th base
  backing — but drifts from single-source).

## Provenance
Audit RV-302 (SL-227), external adversarial pass by codex/GPT-5.5 (F-10 here;
F-6/F-7/F-8 fixed-in-candidate — see trim note above). See `.doctrine/review/302/`.
