# Review RV-287 — code-review of SL-223

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Second-round adversarial verification of the revised SL-223 publication-seam
design against every verified finding on RV-286 and against SPEC-026, PRD-017,
ADR-019, ADR-001 layering, the live install projection implementation, Cargo/Nix
release packaging, and the actual `just gate`/`nix-build` recipes. Lines of
attack: prove the new embed root cannot project; prove current source bytes and
release-artifact bytes are both gated; check that the one-adapter resolver API
can exercise its claimed alternative adapter; audit the neutral two-root
extraction and all `Assets::iter`/`Assets::get` rewires; test whether the
command-free API is buildable under deny-unused; and reject requirement coverage
whose mapped test cannot compile or observe the claimed behaviour.
