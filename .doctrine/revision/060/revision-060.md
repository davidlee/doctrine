# REV REV-060 — Govern SPEC-007 ambient memory pointer surface

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

`SL-205` introduced `memory surface` for Claude. `SL-263` keeps its retrieval,
admission, deduplication and formatting pipeline while adding codex and pi input
codecs and a neutral request. `SPEC-007` owns `src/memory.rs` and
`src/retrieve.rs`, yet currently describes only `find` and full-body `retrieve`.
The shipped pointer surface therefore has no durable technical contract in its
owning spec. `REV-058` settles its product boundary in `PRD-004`; this revision
records the memory-side mechanism. `REV-059` separately records installer wiring
in `SPEC-011`.

### Boundary

The memory engine owns the neutral request, scope resolution, retrieval and
pointer output. A harness adapter owns its tool names and input fields. The
installer owns placement of hook entries and generated extensions. No harness
tool token belongs in the neutral request or the shared retrieval pipeline
(`POL-003`). A pointer is metadata that invites a later full-memory request; it
does not expose the memory body. Existing lifecycle suppression and the
non-bypassable holdback apply before a pointer is formatted (`REQ-151` /
`REQ-152`). The pointer title is recalled, untrusted knowledge. Each pointer
must therefore meet `REQ-018`'s quoted, attributed data rule, including its
identity, trust standing, and context; pointer brevity is no exemption.

### `SPEC-007` responsibilities — before/after

**Before:** the responsibilities list names the scope-aware `find` / `retrieve`
reader and its security render contract, but no pointer surface.

**After:** add a responsibility for the optional memory pointer surface:

> Accept a doctrine-owned neutral request for a path set, command or patch;
> normalise harness wires at the edge; resolve one scope probe; reuse the memory
> reader's lifecycle and trust filters; then admit, deduplicate when a session
> identity exists, cap and format concise pointers in the selected output form.
> Render each pointer as quoted, attributed data with identity, trust standing,
> and context. Empty, unresolvable and failed lookups emit no pointer.

The `responsibilities` list in `spec-007.toml` and the **Overview** and
**Responsibilities** prose paragraphs gain this responsibility. Add a dedicated
*Ambient pointer surface* subsection after the scope-aware reader. It describes
the neutral request, the one-query path-set behavior, the pointer-only output,
the fail-open empty case, and the codec boundary. The security subsection
clarifies that full-body `retrieve` and pointer output both obey the data-only
render contract. The pointer surface omits memory bodies, but its titles are
still untrusted content and must be quoted and delimited from instructions.
No existing requirement is weakened.

### `FR-008` — the surface contract

The `introduce` row pins the optional surface's verifiable memory-side
behavior. One requirement covers the coupled pipeline from neutral request to
pointer output: separating retrieval, admission and formatting into independent
requirements would make this single call path appear complete when only one
step worked. The installer behaviors are separately owned by `REV-059`.

### Reconciliation check

`ISS-480` tracks the shipped formatter's `REQ-018` discrepancy: its unquoted
title appears in a bullet, and the command surface shows severity rather than
trust standing. The existing holdback proves admission, not presentation.
Applying this revision establishes the contract; it does not attest that the
current formatter conforms. Resolve `ISS-480` before claiming implementation
conformance.

## Reconcile narrative (SL-263)

- [RV-381 F-5]: SPEC-007 had no memory-side contract for the pointer surface. Approved by the user at reconcile ("agreed", 2026-09-24). Landed by hand: overview and responsibilities (toml list + prose), a new *The ambient pointer surface* subsection, and a render-contract paragraph binding pointers to REQ-018 with ISS-480 named as the open discrepancy.
- introduce row landed via `spec req add`: FR-008 → REQ-481.
