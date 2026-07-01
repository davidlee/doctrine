# Implementation Plan SL-185: Subprocess-arm Seatbelt confinement (macOS jail parity)

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases port SL-183's Seatbelt write-floor to the subprocess (pi) arm on
macOS. The load-bearing sequencing constraint (design §7 RISK-1 / XR-4): the
subprocess arm has never run a long-lived `pi --mode rpc` process under
`sandbox-exec`; if that fails, the launcher strategy is dead. So the plan front-
loads everything that is **low-regret on Linux** (PHASE-01..03: the factoring,
the Linux `jail-prefix` branch, the shell reader) and **gates the only mac-
dependent code** (PHASE-04) behind the RISK-1 falsification probe. Development is
Linux-first by the slice's premise; enforcement is a mac VH gate.

## Sequencing & Rationale

**PHASE-01 — factor, don't fork.** The reuse-first mandate (no parallel builder)
means the subprocess arm rides SL-183's `resolve_with_policy` core, not a copy.
This phase carves that core out of `resolve_inputs` (`acquire_policy` = the
claude-only disk lookup; `resolve_with_policy` = the shared policy→resolved core)
and extracts `write_seatbelt_profile` from `pretooluse`. Two adversarial findings
shape it: XR-3 (thread ONE `Topology` — the naïve split double-probes git and
opens a read-A/resolve-B window) and the behaviour-preservation gate (the
existing claude suites are the proof — they stay green **unchanged**; a red
existing test means the recomposition changed behaviour). Pure leaf builders are
untouched (ADR-001). Nothing platform-specific lands — this is all compiled and
tested on Linux.

**PHASE-02 — the command, Linux half.** `jail-prefix` is the new command-tier
consumer (D1 altitude A). It is built Linux-first so the WHOLE command shell —
argparse, policy build, emit, fail-closed, the `--out` NUL contract — is
exercised by real tests here (D2, the cfg-rot mitigation), leaving only the
irreducibly-mac Seatbelt branch cfg-gated. The security-critical addition is XR-1:
the inline `--extra-rw` flag is raw orchestrator-shell input, unlike the claude
arm's pre-canonicalized disk policy, so `run_jail_prefix` MUST `env.realpath` each
grant BEFORE `validate_policy` (whose D-canon precondition assumes canonical
paths) — else a `../`/symlink grant widens the sandbox. AR-4: reuse a
bwrap-presence helper factored from `probe_backend`, NOT `probe_backend` itself
(its macOS branch reads the disk policy D3 rejects).

**PHASE-03 — the shell seam, Linux-provable.** The spawn script gets the `uname`
branch. The Linux `*` branch keeps the existing inline bwrap array byte-unchanged
(D1). The reader (`mapfile -d '' < --out` + empty-guard) is the AR-1 fail-closed
fix realised in shell, and it is **Linux-testable** against a real `jail-prefix
--out` file (XR-5) — the coverage `cargo check --target` cannot give. The Darwin
arm is a deliberate **fail-closed stub** ("pending RISK-1") so no `sandbox-exec`
dependency lands before the probe clears (XR-4).

**PHASE-04 — mac wiring, gated.** EN-1 is the RISK-1 probe: a disposable
`sandbox-exec -f <floor> -- pi --mode rpc` on a mac. Only once that passes does
the macOS `run_jail_prefix` branch + the real Darwin arm land, OQ-b gets settled
(does pi write `~/.pi`? — grant `extra_rw` iff yes), and the VH enforcement gate
(writes denied/allowed, child inheritance, live fail-closed) closes the slice.
`cargo check --target aarch64-apple-darwin` is the off-mac bitrot tripwire, not
enforcement proof.

## Notes

- **Open questions carried from design:** OQ-a is RESOLVED (thread one `Topology`,
  PHASE-01). OQ-b (pi's `~/.pi` write behaviour → `extra_rw`?) and OQ-c
  (`main_root` via explicit `--main-root "$ROOT"` — the script hardcodes
  `ROOT=/workspace/doctrine`) settle in PHASE-04 against a real mac.
- **RISK-1 parallelism:** the probe (EN-1 of PHASE-04) needs no doctrine code —
  just a floor `.sb` and `pi`. Run it the moment a mac is available, in parallel
  with PHASE-01..03, so PHASE-04 is unblocked the instant the Linux work lands.
- **Behaviour-preservation gate:** PHASE-01 changes shared machinery (the resolver
  the claude arm depends on). The existing `resolve_inputs` + `pretooluse` suites
  are the contract; they must pass unchanged (AGENTS.md § behaviour-preservation).
