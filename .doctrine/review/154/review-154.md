# Review RV-154 — reconciliation of SL-150

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-150 (family-grouped help + boot-map projection) against
its locked-then-D9-amended design.

**Review surface.** Dispatched slice — audited the candidate interaction branch
`candidate/150/review-001` (cand-150-review-001, tip `57190cf8`, a no-ff merge of
the `review/150` impl-bundle onto `refs/heads/main`), NOT the raw `dispatch/150`
evidence. Code is a single funnel source-delta (`review(150): impl bundle`).
`web/map/dist/` (a gitignored RustEmbed root, orthogonal to this slice) was
symlinked into the candidate worktree to satisfy the unrelated `map_server`
compile — no slice content touched.

**Lines of attack.**
1. **D9 injection shape** — does the as-built inject `command_map: fn() -> String`
   (boot never naming `cli`), or did it regress to a direct `boot→cli` call?
2. **Layering guard** — Command `tangle_baseline` must stay 123 (the gate is the
   guard against the 123→144 ratchet a back-edge would cause).
3. **Single-renderer (D3)** — one `render_boot_map()` behind both the `--boot-map`
   flag and the boot `CommandMap` section; `--boot-map` precedence over `--commands`.
4. **Verification** — full suite, clippy zero-warn, fmt; separate slice signal from
   pre-existing environmental debt (CHR-025 fmt, reserve env-var fail-closed test).

## Synthesis

Clean conformance audit — **no blockers, no drift, four findings all `aligned`**.

The as-built matches design D9 exactly. `boot.rs` threads `command_map: fn() ->
String` through `dispatch → build_sections → produce` (and `run`/`run_check`/
`run_emit`/`boot_check`/`regenerate`), renders the dense map via the injected
closure (`SourceKind::CommandMap => command_map()`), and **never names `cli`** —
the only references are D9 explanatory comments. `cli.rs:1076` supplies
`render_boot_map` to `boot::dispatch`; `cli.rs:748` is the single renderer behind
both surfaces; `main.rs:194` gives `--boot-map` precedence over `--commands`. The
`CommandMap` section is placed immediately after "Routing & Process", before
Governance (VT-3).

The dependency-inversion guard is real and verified: `tangle_baseline.command`
stays **123** in `adr/001/layering.toml` and `architecture_layering_gate` is green
— a regression to a direct `boot→cli` call would have ratcheted it to 144 and the
gate would refuse. Slice golden suites green (`e2e_boot_map_golden` 8/8 incl.
precedence; `e2e_help_families_golden` 6/6). `cargo clippy --workspace` zero-warn.

**Standing risks / consciously-accepted noise (none slice-attributable):**
- Full suite shows 1 failure — `reserve::tests::vt3_auto_degradation_is_fail_closed_with_explicit_optin`
  — induced purely by running with `DOCTRINE_RESERVATION_FALLBACK=1` (the jail
  write-fallback the audit needs), which is exactly the opt-in the test asserts
  must be absent. `reserve.rs` is not in the slice delta; the module is 18/18 green
  with the env unset.
- `cargo fmt --check` reports 7 diffs (`boot.rs:2879`, `main.rs:795`, `memory.rs`
  ×5) — all pre-existing CHR-025 rustfmt debt, all outside the slice's diff hunks
  (verified hunk-by-hunk). Not introduced here; left untouched per CHR-025 scope.

The mid-flight D9 amendment (consult-approved, committed on `edge` @ `94369d9f`)
landed the design and the code at agreement before audit — there is no design↔code
divergence for reconcile to resolve.

## Reconciliation Brief

A clean conformance audit: no spec/governance finding, no per-slice design edit.
Design and code already agree (the D9 amendment closed the only gap mid-flight).

### Per-slice (direct edit)
- _None._ `design.md` already reflects the as-built (D9 injection, §5.2/§7/§9);
  no prose correction required.

### Governance/spec (REV)
- _None._ ADR-001 `tangle_baseline.command` stays 123 (unchanged); no ADR/spec/REQ
  status move is implied by the implementation.

### Integration note (for /close, not reconcile)
- Code lands on `main` only at close (stage-2 integrate). The D9 design amendment
  is on `edge` @ `94369d9f`; promote `edge→main` (`git fetch . edge:main`) before
  the integrate so the landing zone carries it.
- Harvest at close: the D9 footgun memory (a lower module calling back into the CLI
  layer can close a same-tier import cycle and ratchet the ADR-001 Command
  `tangle_baseline` — prefer injecting the renderer as a fn-pointer from the layer
  that already depends downward).
