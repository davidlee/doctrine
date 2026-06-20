# Design SL-114: Consolidate per-kind canonical_id onto shared listing helper

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

The id-format `format!("{prefix}-{id:03}")` is reimplemented inline in several
per-kind modules instead of routing through the single id-form authority
`listing::canonical_id` (`listing.rs:36`). Each copy is a place the format can
drift if padding width or separator ever changes. Collapse the remaining copies
onto the shared helper so the format has exactly one home — behaviour-preserving.

## 2. Current State

The slice scope (2026-06-19 audit) named eight sites. By 2026-06-20 **five of those
eight** already delegate (`rec`, `slice`, `revision`, `review`, and the spec **free
fn** at `spec.rs:1164` — the audit's `spec.rs:1160`). Three of the original eight
remain raw (`requirement`, `knowledge`, `backlog`); design review additionally found
a fourth raw site **not** in the audit list — the spec **method** at `spec.rs:106`.
**Four raw `format!` sites total:**

| Site | Form | Body |
|---|---|---|
| `requirement.rs:244` | free fn `canonical_id(id)` | `format!("{}-{id:03}", REQUIREMENT_KIND.prefix)` |
| `knowledge.rs:125` | method `RecordKind::canonical_id(self,id)` | `format!("{}-{id:03}", self.prefix())` |
| `backlog.rs:156` | method `ItemKind::canonical_id(self,id)` | `format!("{}-{id:03}", self.prefix())` |
| `spec.rs:106` | method `SpecSubtype::canonical_id(self,id)` | `format!("{}-{id:03}", self.kind().prefix)` |

Additional finding in `spec.rs`: the file carries **two** wrappers with identical
output — the raw method (`:106`, 5 callers) and a delegating free fn
`canonical_id(subtype, id)` (`:1164`, 4 callers). Same string, two names, one
file: residual duplication the slice's own language ("remove the per-kind
wrapper entirely") invites resolving.

`requirement.rs` does **not** yet reference `listing`; the other three already do.

## 3. Forces & Constraints

- **ADR-001 (module layering: leaf ← engine ← command, no cycles).** `listing`
  is a leaf (it imports none of `requirement`/`knowledge`/`backlog`/`spec` —
  verified). Engine kinds delegating into it is the established, already-shipped
  direction (`slice.rs:761` etc). No new cycle; `requirement → listing` is the
  same edge the siblings already have.
- **Behaviour-preservation gate.** Output strings are unchanged. The existing
  id-format tests are the proof and must stay green **unchanged**:
  `requirement.rs:631` (`-> "REQ-001"`), `knowledge.rs:1418`
  (`canonical_id_uses_the_kind_prefix`), `backlog.rs:2462` (`-> "ISS-001"`),
  `listing.rs:783`.
- Non-goals (slice §Non-Goals): the **parse** side (`strip_prefix`/`id_from_fk`),
  any format/prefix/padding change, the SL-113 mutation seam.

## 4. Guiding Principles

One home for the format. Each wrapper keeps exactly one kind-specific input — its
prefix — and delegates the formatting. Minimum churn consistent with actually
removing duplication, not papering over it.

## 5. Proposed Design

### 5.1 System Model

Four leaf-bound delegations plus one in-file wrapper collapse. No new types, no
signature changes to surviving wrappers, no data-flow change.

### 5.2 Interfaces & Contracts

Surviving public/crate signatures are untouched; only bodies change:

- `requirement.rs:244` →
  `pub(crate) fn canonical_id(id: u32) -> String { crate::listing::canonical_id(REQUIREMENT_KIND.prefix, id) }`
  (fully-qualified call — matches requirement's inline `crate::` style; no new
  `use` line).
- `knowledge.rs:125` (method body) → `listing::canonical_id(self.prefix(), id)`.
- `backlog.rs:156` (method body) → `listing::canonical_id(self.prefix(), id)`.
- **spec.rs (Decision D1, option B):**
  - `SpecSubtype::canonical_id` (`:106`) body → `listing::canonical_id(self.kind().prefix, id)`.
  - **Delete** the free fn `canonical_id(subtype, id)` (`:1164`).
  - Repoint its 4 callers to the method form:
    `:1014` `canonical_id(spec.kind, spec.id)` → `spec.kind.canonical_id(spec.id)`;
    `:1179`, `:1291`, `:1346` `canonical_id(subtype, m.id)` → `subtype.canonical_id(m.id)`.

The unrelated `requirement::canonical_id(...)` calls in `spec.rs` are a different
function and stay as-is.

### 5.3 Data, State & Ownership

None. Pure string formatting; no clock, disk, or git (`listing::canonical_id` is
already pure). The prefix constants remain the per-kind source of truth.

### 5.4 Lifecycle, Operations & Dynamics

n/a — no runtime/lifecycle surface.

### 5.5 Invariants, Assumptions & Edge Cases

- Invariant: every produced id is `PREFIX-NNN` (≥3 digits, zero-padded). Held by
  `listing::canonical_id`; the delegations inherit it verbatim.
- Edge: `id` ≥ 1000 widens past three digits identically in old and new code
  (`{:03}` is a minimum, not a truncation) — unchanged.

## 6. Open Questions & Unknowns

None open. OQ-1 (resolved): a shared id-**parse** helper — out of scope per slice
§Non-Goals, carried as a follow-up, not designed here.

## 7. Decisions, Rationale & Alternatives

**D1 — spec.rs dual wrapper: collapse (B), keep the method, delete the free fn.**
The slice goal is one home for the format; leaving two same-output wrappers in one
file half-completes it. The method form `subtype.canonical_id(id)` reads better
than `canonical_id(subtype, id)`, so the method survives and the free fn dies.
Churn is 4 call sites, all behaviour-preserving.
- Alt A (minimal): make the method delegate, leave both wrappers, note follow-up.
  Rejected — leaves the in-file duplicate the slice explicitly invites removing.

**D2 — requirement: keep the free-fn wrapper, delegate its body.** It encapsulates
`REQUIREMENT_KIND.prefix` and is called cross-module (`spec.rs`) and in tests;
inlining at call sites would re-spread the prefix. Thin delegate is the right unit.

## 8. Risks & Mitigations

- R1: a missed `spec.rs` caller of the deleted free fn → compile error. Mitigation:
  the compiler is the gate; `just check` must pass. Low risk, fully mechanical.
- R2: stale slice scope misreads as "8 sites still open." Mitigation: §2 records
  the drift; slice-114.md reconciled with a drift note before planning.

## 9. Quality Engineering & Validation

- No new tests — this is behaviour-preserving. The three existing id-format tests
  (§3) are the regression proof and stay **green unchanged**.
- Closure evidence (slice §Closure intent): for each `fn canonical_id` in `src/`
  (enumerate via `grep -n 'fn canonical_id' src/`), the body routes through
  `listing::canonical_id` — none reimplements the `"{prefix}-{id:03}"` form. The
  spec free fn `canonical_id(subtype, id)` no longer appears.
- Gate: `just check` (clippy zero-warnings + tests) green.

## 10. Review Notes

Internal adversarial pass (2026-06-20):
- **Closure wording (§9) was loose** — "no module containing `format!`" over-claims;
  many modules use `format!` for other strings. Re-scoped to: each `fn canonical_id`
  body delegates. Fixed.
- **Free-fn delete safety** — confirmed exactly 4 callers (`spec.rs:1014/1179/1291/1346`),
  all in-file; the fn is module-private, so no cross-module breakage. Cross-module
  `requirement::canonical_id` is a different fn, untouched.
- **Type/import check** — `self.prefix()` / `self.kind().prefix` are `&str`;
  `listing::canonical_id(&str, u32)` matches. `listing` already imported in
  knowledge/backlog/spec; requirement uses a fully-qualified `crate::listing::` call,
  no new `use`.
- **Layering (ADR-001)** — `requirement → listing` adds no cycle (`listing` imports
  none of the kind modules). Verdict: sound, no open governance conflict.

External codex pass (GPT-5.5, 2026-06-20): all six central claims verified against
source — no blockers. Two doc nits fixed: §2 delegate count (five of eight, not
"four"); §3 test list now includes `backlog.rs:2462` (`ISS-001`). Codex confirmed
D1 (keep method / delete free fn) is the right call — 4 repoints vs 5 and the better
API shape. Noted `spec.rs:763` (`format!("{prefix}-{:03}", max+1)`) is a member-label
format, NOT a `canonical_id` wrapper — correctly out of scope.
