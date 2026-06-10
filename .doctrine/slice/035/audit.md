# SL-035 Audit — record-time stderr nudge for hidden thread memories

**Mode:** conformance (post-implementation, single phase PHASE-01).
**Reconciled against:** `design.md` (D1–D4), slice scope, ADR-001 (pure/imperative
split), the SL-008 D6 `thread_expiry` gate. Implementation commit `a39154d`.

## Evidence

- **Gate:** `just check` GREEN — `cargo fmt`, plain `cargo clippy` (bins/lib) zero
  warnings, full test suite, `cargo build`. No read-path edit, so the SL-008
  retrieval suites pass unchanged (behaviour-preservation gate held).
- **Unit (pure `thread_hidden_notice`):** 3 tests, all pass —
  `thread_record_advises_the_hidden_until_verified_gate` (VT-1),
  `non_thread_record_gets_no_advisory` (VT-2, all 5 non-thread kinds),
  `thread_advisory_reference_is_the_verify_handle` (VT-3, key + uid).
- **E2E observed** (freshly-built jail binary, real git repo):
  - thread + key → stdout success line unchanged; **stderr** carries the advisory
    naming the key (`… verify mem.thread.audit.x …`).
  - pattern → stdout success line; **stderr empty** (no nudge).
  - thread, no key → stderr advisory names the **uid**.

## Findings

| # | Expected (cite) | Observed | Disposition |
|---|---|---|---|
| A-1 | Thread record emits one stderr advisory naming the verify handle + `verify` (design Target, VT-1) | Advisory emitted; names key or uid; cites SL-008 D6 | **aligned** |
| A-2 | Non-thread record emits no advisory (design, VT-2) | stderr empty for pattern | **aligned** |
| A-3 | Reference = minted key if present else uid (design D-Target, VT-3) | key path + uid path both observed correct | **aligned** |
| A-4 | Stdout machine-readable success line unchanged for all types; nudge stderr-only (EX-3, D1) | stdout identical to prior shape in all 3 cases | **aligned** |
| A-5 | Pure helper + thin shell, no impurity in pure layer (ADR-001, D2) | `thread_hidden_notice` is text-in/text-out; shell does only `writeln!` | **aligned** |
| A-6 | No read-path / `thread_expiry` change; behaviour preserved (scope Non-Goals) | only `src/memory.rs` touched; full suite green unchanged | **aligned** |
| A-7 | Wording fixed per D4, cites SL-008 D6 | message byte-matches the approved D4 text | **aligned** |
| A-8 | Fires for `--global` threads too (design Trigger scope) | gate keys on `MemoryType::Thread` only — `--global` unaffected by construction; not separately exercised E2E | **aligned** (covered by VT-1/the kind gate; global path is identical) |

## Disposition summary

All findings **aligned**. No fix-now, no drift, no follow-up slice. Design,
scope, and implementation tell one story.

## Harvest

- **Durable fact** (worth memory): the crate is binary-only — tests run via
  `cargo test --bin doctrine`; a `--lib` filter errors. Candidate for
  `/record-memory` if not already captured. *(Low novelty — `just check` is the
  canonical gate; deferred unless it recurs.)*
- No new risks or follow-up work surfaced. IMP-011 is resolved by this slice —
  close it at `/close`.
